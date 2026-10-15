#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Separa las dos causas posibles del 403: CABECERAS o TRANSPORTE.

Hechos medidos (2026-10-15):
  - las peticiones de la app a /gc-api responden 200 y llevan connect-csrf-token + nk: NT...
  - un `fetch` dentro de la pagina SIN esas cabeceras: 403
  - el MISMO camino, por codigo, CON esas cabeceras copiadas: 403 tambien (3 clientes)

Entonces quedan dos explicaciones y esta sonda las separa:
  (a) faltan cabeceras -> `fetch` DENTRO de la pagina CON las cabeceras copiadas dara 200
  (b) es el transporte  -> el `fetch` de dentro dara 200 igual y curl dara 403 aunque copie todo
      (la cookie cf_clearance esta atada a la huella/UA que la gano)

Y ademas enseña QUIEN responde el 403: se imprimen las cabeceras de la respuesta.

Uso: cd ~/garmin-lab && DISPLAY=:0 ./venv/bin/python dentro.py
"""

import json
import os
import time
from pathlib import Path

from curl_cffi import requests as creq
from playwright.sync_api import sync_playwright

DIR = Path.home() / "garmin-lab"
CABECERAS = DIR / "cabeceras-gcapi.json"
SESION = DIR / "sesion-navegador.json"
PERFIL = DIR / os.environ.get("GARMIN_PERFIL", "perfil-sonda")

CAMINO = "/gc-api/web-gateway/snapshot/usageIndicators"
URL = "https://connect.garmin.com" + CAMINO


def decir(m):
    print(m, flush=True)


def main():
    capturadas = json.loads(CABECERAS.read_text())
    modelo = None
    for u, d in capturadas.items():
        if "connect-csrf-token" in d["cabeceras"]:
            modelo = d["cabeceras"]
            break
    if not modelo:
        decir("no hay modelo de cabeceras")
        return 2
    cab = {k: v for k, v in modelo.items()
           if k not in {"content-length", "accept-encoding", "cookie", "host", "connection"}}
    decir(f"cabeceras copiadas: {sorted(cab.keys())}")

    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            str(PERFIL), headless=not os.environ.get("DISPLAY"), locale="es-CL",
            timezone_id="America/Santiago", viewport={"width": 1280, "height": 860},
            args=["--disable-blink-features=AutomationControlled"])
        page = ctx.pages[0] if ctx.pages else ctx.new_page()
        page.goto("https://connect.garmin.com/app/home", wait_until="domcontentloaded", timeout=90000)
        time.sleep(8)
        decir(f"sesion: {'DENTRO' if '/sign-in' not in page.url else 'FUERA'}")

        decir("(a) fetch DENTRO de la pagina, CON las cabeceras copiadas:")
        x = page.evaluate(
            """async ([url, cab]) => {
                try {
                    const res = await fetch(url, {credentials:'include', headers: cab});
                    const t = await res.text();
                    return {estado: res.status, largo: t.length, inicio: t.slice(0, 300)};
                } catch (e) { return {estado: -1, largo: 0, inicio: String(e)}; }
            }""", [URL, cab])
        decir(f"    HTTP {x['estado']}  {x['largo']} B")
        decir(f"    primeros 220: {x['inicio'][:220]!r}")

        decir("(a2) el mismo fetch SIN las cabeceras (control):")
        y = page.evaluate(
            """async (url) => {
                const res = await fetch(url, {credentials:'include'});
                const t = await res.text();
                return {estado: res.status, largo: t.length};
            }""", URL)
        decir(f"    HTTP {y['estado']}  {y['largo']} B")

        galletas = [c for c in ctx.cookies() if "garmin" in c["domain"]]
        ctx.close()

    decir("(b) el mismo camino POR CODIGO, y quien responde:")
    cookie_header = "; ".join(f"{c['name']}={c['value']}" for c in galletas)
    h = dict(cab)
    h["Cookie"] = cookie_header
    r = creq.get(URL, headers=h, impersonate="chrome", timeout=40)
    decir(f"    HTTP {r.status_code}  {len(r.text)} B")
    interesantes = {k: v for k, v in r.headers.items()
                    if k.lower() in ("server", "cf-ray", "cf-mitigated", "www-authenticate",
                                     "content-type", "x-frame-options", "location", "set-cookie",
                                     "report-to", "x-content-type-options")}
    for k, v in interesantes.items():
        mostrar = v if k.lower() != "set-cookie" else "(se omite el valor)"
        decir(f"    respuesta {k}: {mostrar[:110]}")
    decir(f"    cuerpo: {r.text[:220]!r}")
    decir("DENTRO-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
