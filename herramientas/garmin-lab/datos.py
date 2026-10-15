#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Trae los DATOS del cliente por la via que SI funciona: `fetch` DENTRO del navegador.

Medido el 2026-10-15 (dentro.py):
  - `fetch('/gc-api/...')` dentro de la pagina CON las cabeceras de la app -> 200 (46 KB)
  - el mismo camino por codigo (curl_cffi chrome + todas las cabeceras) -> 403 de CLOUDFLARE
Conclusion: la sincronizacion tiene que correr DENTRO de un motor de navegador (WebView en
Android). Las cabeceras son necesarias; el transporte del navegador, imprescindible.

QUE HACE
  1. Toma EN VIVO las cabeceras de una peticion buena de la propia app (incluido el
     connect-csrf-token y el cuerpo de la consulta GraphQL).
  2. Pide, desde dentro, los caminos de datos: perfil, actividades, sueno, estres,
     training status y pulso en reposo.
  3. Guarda lo que devuelva 200 en datos/ y dice el tamano de cada uno.

No imprime cookies ni tokens. Uso: cd ~/garmin-lab && DISPLAY=:0 ./venv/bin/python datos.py
"""

import json
import os
import re
import time
from datetime import date, timedelta
from pathlib import Path

from playwright.sync_api import sync_playwright

DIR = Path.home() / "garmin-lab"
DATOS = DIR / "datos"
PERFIL = DIR / os.environ.get("GARMIN_PERFIL", "perfil-sonda")

vivo = {"cabeceras": None, "grafo": None, "dn": None}


def decir(m):
    print(m, flush=True)


def main():
    DATOS.mkdir(exist_ok=True)
    hoy = date.today()
    ayer = (hoy - timedelta(days=1)).isoformat()
    desde = (hoy - timedelta(days=7)).isoformat()

    def al_pedir(req):
        u = req.url
        if "connect.garmin.com/gc-api" not in u:
            return
        if vivo["cabeceras"] is None and "connect-csrf-token" in {k.lower() for k in req.headers}:
            vivo["cabeceras"] = {k.lower(): v for k, v in req.headers.items()}
        if vivo["dn"] is None:
            m = re.search(r"/gc-api/userprofile-service/socialProfile/([^/?]+)", u)
            if m:
                vivo["dn"] = m.group(1)
        if vivo["grafo"] is None and "graphql-gateway/graphql" in u and req.method == "POST":
            vivo["grafo"] = req.post_data

    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            str(PERFIL), headless=not os.environ.get("DISPLAY"), locale="es-CL",
            timezone_id="America/Santiago", viewport={"width": 1280, "height": 860},
            args=["--disable-blink-features=AutomationControlled"])
        ctx.on("request", al_pedir)
        page = ctx.pages[0] if ctx.pages else ctx.new_page()

        page.goto("https://connect.garmin.com/app/home", wait_until="domcontentloaded", timeout=90000)
        time.sleep(9)
        page.goto("https://connect.garmin.com/app/activities", wait_until="domcontentloaded", timeout=90000)
        time.sleep(9)
        decir(f"sesion: {'DENTRO' if '/sign-in' not in page.url else 'FUERA'}")
        decir(f"cabeceras en vivo: {'SI' if vivo['cabeceras'] else 'NO'}")
        decir(f"displayName visto en las rutas: {'SI' if vivo['dn'] else 'NO'}")
        decir(f"consulta GraphQL capturada: {'SI' if vivo['grafo'] else 'NO'} "
              f"({len(vivo['grafo'] or '')} B)")

        cab = {k: v for k, v in (vivo["cabeceras"] or {}).items()
               if k not in {"content-length", "accept-encoding", "cookie", "host", "connection"}}
        dn = vivo["dn"] or ""

        caminos = [("perfil", "/gc-api/userprofile-service/socialProfile")]
        if dn:
            caminos += [
                ("sueno", f"/gc-api/wellness-service/wellness/dailySleepData/{dn}?date={ayer}&nonSleepBufferMinutes=60"),
                ("estres", f"/gc-api/wellness-service/wellness/dailyStress/{ayer}"),
                ("training", f"/gc-api/metrics-service/metrics/trainingstatus/aggregated/{ayer}"),
                ("pulso-reposo", f"/gc-api/userstats-service/wellness/daily/{dn}?fromDate={desde}&untilDate={hoy.isoformat()}&metricId=60"),
                ("resumen-dia", f"/gc-api/usersummary-service/usersummary/daily/{dn}?calendarDate={ayer}"),
            ]
        caminos += [
            ("actividades-noticias", "/gc-api/activitylist-service/activities/v2/newsfeed/status"),
            ("dispositivos", "/gc-api/device-service/deviceregistration/devices"),
        ]

        decir("")
        decir("=== DATOS, desde dentro del navegador ===")
        buenos = 0
        for nombre, ruta in caminos:
            try:
                x = page.evaluate(
                    """async ([url, cab]) => {
                        const res = await fetch(url, {credentials:'include', headers: cab});
                        const t = await res.text();
                        return {estado: res.status, largo: t.length, cuerpo: t};
                    }""", ["https://connect.garmin.com" + ruta, cab])
            except Exception as e:
                decir(f"  {nombre:<20} fallo {e}")
                continue
            marca = ""
            if x["estado"] == 200:
                (DATOS / f"{nombre}.json").write_text(x["cuerpo"])
                buenos += 1
                try:
                    j = json.loads(x["cuerpo"])
                    if isinstance(j, list):
                        marca = f"  lista de {len(j)}"
                    else:
                        marca = f"  claves: {sorted(j.keys())[:6]}..."
                except Exception:
                    marca = "  (no-JSON)"
            decir(f"  {nombre:<20} HTTP {x['estado']}  {x['largo']} B{marca}")

        if vivo["grafo"]:
            decir("")
            decir("=== la consulta GraphQL que usa la app (reenviada) ===")
            try:
                g = json.loads(vivo["grafo"])
                decir(f"  operacion: {g.get('operationName')}")
                x = page.evaluate(
                    """async ([url, cab, cuerpo]) => {
                        const res = await fetch(url, {method:'POST', credentials:'include',
                            headers: {...cab, 'content-type':'application/json'}, body: cuerpo});
                        const t = await res.text();
                        return {estado: res.status, largo: t.length, cuerpo: t};
                    }""",
                    ["https://connect.garmin.com/gc-api/graphql-gateway/graphql", cab, vivo["grafo"]])
                decir(f"  HTTP {x['estado']}  {x['largo']} B")
                if x["estado"] == 200:
                    (DATOS / f"grafo-{g.get('operationName', 'x')}.json").write_text(x["cuerpo"])
                    buenos += 1
            except Exception as e:
                decir(f"  no se pudo reenviar: {e}")

        ctx.close()
    decir("")
    decir(f"RESUMEN: {buenos} conjuntos de datos guardados en {DATOS}")
    decir("DATOS-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
