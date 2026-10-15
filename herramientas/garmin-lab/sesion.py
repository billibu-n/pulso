#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Saca la SESION ya iniciada del navegador y prueba la API — sin volver a entrar.

POR QUE ASI: medido el 2026-10-15. El flujo `portal` NO deja el ticket en la URL
(tras el acceso el navegador queda en `connect.garmin.com/app/home`, sin `?ticket=ST-`),
asi que la captura del redirect no encuentra nada. Pero el perfil del navegador CONSERVA
la sesion: no hace falta volver a entrar ni resolver otro reCAPTCHA.

QUE HACE
  1. Abre el MISMO perfil (la sesion ya esta dentro) y comprueba que sigue dentro.
  2. Guarda el material de sesion en sesion-navegador.json (600) — NUNCA imprime valores.
  3. Apunta TODO el trafico a *.garmin.com en trafico.jsonl: metodo, ruta, si lleva
     `Authorization`, `NK`, `DI-Backend` y los NOMBRES de las cookies.
  4. Pide la API **desde el navegador** (mismo jarro de cookies) y guarda los JSON.
  5. Pide la API **por CODIGO, sin navegador** (curl_cffi con la cookie JWT_WEB), que es
     la pregunta que decide si Rust puede sincronizar sin navegador.

Uso: cd ~/garmin-lab && DISPLAY=:0 ./venv/bin/python sesion.py
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
TRAFICO = DIR / "trafico.jsonl"
SESION = DIR / "sesion-navegador.json"

API = "https://connectapi.garmin.com"
AGENTE = "GCM-Android-5.23"
AGENTE_GARMIN = ("com.garmin.android.apps.connectmobile/5.23; ; "
                 "Google/sdk_gphone64_arm64/google; Android/33; Dalvik/2.1.0")

RUTAS = [
    ("perfil.json", "/userprofile-service/socialProfile"),
    ("actividades.json", "/activitylist-service/activities/search/activities?start=0&limit=10"),
]


def decir(m):
    print(m, flush=True)


def pedir_con_cookie(jwt, ruta, dn=None):
    """La via de RESERVA que ya tiene el instrumento: cookie JWT_WEB + NK + DI-Backend."""
    cab = {
        "User-Agent": AGENTE,
        "X-Garmin-User-Agent": AGENTE_GARMIN,
        "Accept": "application/json,text/html;q=0.9,*/*;q=0.8",
        "NK": "NT",
        "Origin": "https://connect.garmin.com",
        "Referer": "https://connect.garmin.com/modern/",
        "DI-Backend": "connectapi.garmin.com",
        "Cookie": f"JWT_WEB={jwt}",
    }
    try:
        return creq.get(API + ruta, headers=cab, impersonate="chrome", timeout=40), None
    except Exception as e:
        return None, str(e)


def main():
    DATOS.mkdir(exist_ok=True)
    f = open(TRAFICO, "a", encoding="utf-8")

    def al_pedir(req):
        if "garmin" not in req.url:
            return
        h = {k.lower(): v for k, v in req.headers.items()}
        aut = h.get("authorization", "")
        f.write(json.dumps({
            "m": req.method,
            "u": req.url.split("?")[0],
            "auth": "bearer" if aut.lower().startswith("bearer") else ("otra" if aut else ""),
            "nk": h.get("nk", ""),
            "di_backend": h.get("di-backend", ""),
            "cookies": sorted({c.split("=")[0].strip() for c in h.get("cookie", "").split(";") if c.strip()}),
            "cabeceras": sorted(h.keys()),
        }, ensure_ascii=False) + "\n")
        f.flush()

    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            str(PERFIL),
            headless=not os.environ.get("DISPLAY"),
            locale="es-CL",
            timezone_id="America/Santiago",
            viewport={"width": 1280, "height": 860},
            args=["--disable-blink-features=AutomationControlled"],
        )
        ctx.on("request", al_pedir)
        page = ctx.pages[0] if ctx.pages else ctx.new_page()

        decir("1) ¿sigue la sesion dentro?")
        page.goto("https://connect.garmin.com/app/home", wait_until="domcontentloaded", timeout=90000)
        time.sleep(7)
        url = page.url
        dentro = ("/sign-in" not in url) and ("sso.garmin.com" not in url)
        decir(f"   URL: {url[:95]}")
        decir(f"   sesion iniciada: {'SI' if dentro else 'NO (habria que volver a entrar)'}")

        cookies = ctx.cookies()
        almacen = page.evaluate(
            """() => { const o = {};
                for (const s of [localStorage, sessionStorage]) {
                    for (let i = 0; i < s.length; i++) { const k = s.key(i); o[k] = s.getItem(k); }
                } return o; }"""
        )
        SESION.write_text(json.dumps(
            {"url": url,
             "cookies": [{"name": c["name"], "domain": c["domain"], "value": c["value"]} for c in cookies],
             "almacenamiento": almacen}, indent=2))
        os.chmod(SESION, 0o600)
        decir(f"2) material de sesion -> {SESION.name} (600)")
        decir(f"   cookies (NOMBRES): {sorted({c['name'] for c in cookies})}")
        decir(f"   claves de almacenamiento: {sorted(almacen.keys())}")

        # ¿hay algo con pinta de token dentro del almacenamiento?
        sospechosas = [k for k in almacen if any(
            t in k.lower() for t in ("token", "oauth", "di_", "jwt", "session", "auth", "ticket"))]
        decir(f"   claves con pinta de credencial: {sospechosas}")

        decir("3) pidiendo la API DESDE el navegador (mismo jarro de cookies):")
        for nombre, ruta in RUTAS:
            try:
                r = page.request.get(API + ruta, timeout=40000)
                decir(f"   {ruta.split('?')[0]}: HTTP {r.status}  {len(r.text())} B")
                if r.status == 200:
                    (DATOS / nombre).write_text(r.text())
            except Exception as e:
                decir(f"   {ruta.split('?')[0]}: fallo {e}")

        jwt = next((c["value"] for c in cookies if c["name"] == "JWT_WEB"), None)
        decir(f"4) ¿hay cookie JWT_WEB?: {'SI' if jwt else 'NO'}")
        ctx.close()
    f.close()

    if jwt:
        decir("5) pidiendo la API POR CODIGO, sin navegador (cookie JWT_WEB):")
        for nombre, ruta in RUTAS:
            r, err = pedir_con_cookie(jwt, ruta)
            if err:
                decir(f"   {ruta.split('?')[0]}: fallo {err}")
                continue
            decir(f"   {ruta.split('?')[0]}: HTTP {r.status}  {len(r.text())} B")
            if r.status == 200:
                (DATOS / ("codigo-" + nombre)).write_text(r.text())
    else:
        decir("5) sin JWT_WEB no se prueba la via por codigo (mirar trafico.jsonl)")

    decir(f"trafico apuntado en {TRAFICO.name}")
    decir("SESION-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
