#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""La via que USA la propia web de Garmin: `connect.garmin.com/gc-api/...` con la cookie.

MEDIDO el 2026-10-15 (trafico.jsonl, 456 peticiones del navegador YA DENTRO):
  - `POST https://connect.garmin.com/gc-api/graphql-gateway/graphql`  (27 veces)
  - `GET  https://connect.garmin.com/gc-api/activitylist-service/activities/v2/newsfeed/status`
  - `GET  https://connect.garmin.com/gc-api/userpreference-service/ghs.ecgEnabled`
  - `GET  https://connect.garmin.com/gc-api/web-gateway/snapshot/usageIndicators`
  - CERO peticiones a `connectapi.garmin.com` y CERO con `Authorization: Bearer`.
Por eso `connectapi.garmin.com` con cookies da **403**: ese host quiere el token DI del movil.
La web se autentica **con la cookie sobre `connect.garmin.com`**.

QUE HACE
  1. Abre el perfil (la sesion sigue dentro) y prueba los caminos `/gc-api/` DESDE el navegador.
  2. Repite los mismos caminos POR CODIGO, sin navegador, con las cookies del jarro.
     Esta es la pregunta que decide la arquitectura: ¿puede Rust sincronizar sin navegador?
  3. Apunta los nombres de operacion GraphQL que usa la web (sin sus respuestas).

Uso: cd ~/garmin-lab && DISPLAY=:0 ./venv/bin/python gcapi.py
"""

import json
import os
import time
from pathlib import Path

from curl_cffi import requests as creq
from playwright.sync_api import sync_playwright

DIR = Path.home() / "garmin-lab"
DATOS = DIR / "datos"
PERFIL = DIR / os.environ.get("GARMIN_PERFIL", "perfil-sonda")
GC = "https://connect.garmin.com/gc-api"
AGENTE_WEB = ("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
              "(KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36")

# Los mismos nombres de servicio que usa la web, tal como aparecen en el trafico real.
FIJOS = [
    "/userprofile-service/socialProfile",
    "/web-gateway/snapshot/usageIndicators",
    "/activitylist-service/activities/search/activities?start=0&limit=5",
    "/activitylist-service/activities/v2/newsfeed/status",
]


def decir(m):
    print(m, flush=True)


def main():
    DATOS.mkdir(exist_ok=True)
    cookies_ok, grafo = {}, {}

    def al_pedir(req):
        if "graphql-gateway/graphql" in req.url and req.method == "POST":
            try:
                cuerpo = json.loads(req.post_data or "{}")
                op = cuerpo.get("operationName") or "(sin nombre)"
                grafo[op] = grafo.get(op, 0) + 1
            except Exception:
                pass

    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            str(PERFIL), headless=not os.environ.get("DISPLAY"), locale="es-CL",
            timezone_id="America/Santiago", viewport={"width": 1280, "height": 860},
            args=["--disable-blink-features=AutomationControlled"])
        ctx.on("request", al_pedir)
        page = ctx.pages[0] if ctx.pages else ctx.new_page()

        decir("1) abriendo la app (la sesion deberia seguir dentro)...")
        page.goto("https://connect.garmin.com/app/home", wait_until="domcontentloaded", timeout=90000)
        time.sleep(8)
        dentro = ("/sign-in" not in page.url) and ("sso.garmin.com" not in page.url)
        decir(f"   URL: {page.url[:90]}")
        decir(f"   sesion: {'DENTRO' if dentro else 'FUERA (hay que volver a entrar)'}")
        if not dentro:
            ctx.close()
            return 2

        decir(f"   operaciones GraphQL vistas: {sorted(grafo.items(), key=lambda x: -x[1])[:10]}")

        decir("2) /gc-api/ DESDE EL NAVEGADOR (cookie del propio navegador):")
        dn = None
        for ruta in FIJOS:
            try:
                r = page.request.get(GC + ruta, timeout=40000)
            except Exception as e:
                decir(f"   {ruta[:56]:<56} fallo {e}")
                continue
            decir(f"   {ruta[:56]:<56} HTTP {r.status}  {len(r.text())} B")
            if r.status == 200:
                (DATOS / ("nav-" + ruta.strip("/").split("?")[0].replace("/", "_")[:40] + ".json")).write_text(r.text())
                if "socialProfile" in ruta:
                    try:
                        dn = r.json().get("displayName")
                    except Exception:
                        pass

        decir(f"   displayName: {'obtenido' if dn else 'NO obtenido'}")

        cookies = ctx.cookies()
        ctx.close()

    # El jarro entero, como lo mandaria el navegador a connect.garmin.com
    relevantes = [c for c in cookies if "garmin" in c["domain"]]
    cookie_header = "; ".join(f"{c['name']}={c['value']}" for c in relevantes)
    decir(f"3) cookies para connect.garmin.com: {sorted({c['name'] for c in relevantes})}")

    if not dn:
        decir("   sin displayName: las rutas que lo llevan en la URL se saltan")

    cola = list(FIJOS)
    if dn:
        cola += [
            f"/userprofile-service/socialProfile/{dn}",
            f"/usersummary-service/usersummary/daily/{dn}",
            f"/wellness-service/wellness/dailySleepData/{dn}?date=2026-10-14&nonSleepBufferMinutes=60",
            "/wellness-service/wellness/dailyStress/2026-10-14",
            "/metrics-service/metrics/trainingstatus/aggregated/2026-10-14",
        ]

    decir("4) /gc-api/ POR CODIGO, sin navegador (la pregunta que decide si Rust basta):")
    ganadores = []
    for ruta in cola:
        cab = {
            "User-Agent": AGENTE_WEB,
            "Accept": "application/json, text/plain, */*",
            "Referer": "https://connect.garmin.com/app/home",
            "Origin": "https://connect.garmin.com",
            "Cookie": cookie_header,
        }
        try:
            r = creq.get(GC + ruta, headers=cab, impersonate="chrome", timeout=40)
        except Exception as e:
            decir(f"   {ruta[:56]:<56} fallo {e}")
            continue
        marca = ""
        if r.status_code == 200:
            try:
                j = r.json()
                n = len(j) if isinstance(j, list) else len(j.keys())
                marca = f"  <- {n} claves/elementos"
                (DATOS / ("codigo-" + ruta.strip("/").split("?")[0].replace("/", "_")[:40] + ".json")).write_text(r.text())
                ganadores.append(ruta)
            except Exception:
                marca = "  (no-JSON)"
        decir(f"   {ruta[:56]:<56} HTTP {r.status_code}{marca}")

    decir("")
    decir(f"RESUMEN: por codigo funcionan {len(ganadores)} de {len(cola)} caminos")
    for g in ganadores:
        decir(f"   OK  {g}")
    decir("GCAPI-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
