#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""SONDA SIN CREDENCIALES: comprueba que el arnes funciona ANTES de gastar un intento real.

Cargar la pagina de acceso NO es un intento de acceso. Esta sonda responde:
  1. ¿El Chromium real (Playwright + xvfb) arranca en este equipo?
  2. ¿La pagina OFICIAL de acceso carga sin reto de Cloudflare?
  3. ¿Que campos trae el formulario? (para que entrar.py acierte a la primera)
  4. ¿Que cookies planta el borde a un navegador REAL? (nombres, NUNCA valores)
  5. ¿Con que huella TLS/HTTP2 sale ese navegador? (contra tls.peet.ws)

La comparacion que importa: el instrumento de pulso (ureq) recibio `_cfuvid` de
Cloudflare; la libreria portada (curl_cffi chrome) NO. Aqui se ve que recibe un
navegador de verdad.
"""

import json
import os
import time
from pathlib import Path

from curl_cffi import requests as creq
from playwright.sync_api import sync_playwright

DIR = Path.home() / "garmin-lab"
DIR.mkdir(exist_ok=True)

NAVEGADOR = {
    "User-Agent": (
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36"
    )
}


def huella(quien, impersonar=None):
    """Huella TLS/h2 vista por el eco publico tls.peet.ws."""
    try:
        if impersonar:
            r = creq.get("https://tls.peet.ws/api/all", impersonate=impersonar, timeout=30)
        else:
            r = creq.get("https://tls.peet.ws/api/all", headers=NAVEGADOR, timeout=30)
        j = r.json()
        http = j.get("http_version")
        ja4 = (j.get("tls") or {}).get("ja4")
        akamai = (j.get("tls") or {}).get("ja3_hash")
        peet = ((j.get("tls") or {}).get("extensions") or [])
        print(f"  {quien:<28} http={http}  ja4={ja4}")
        return {"quien": quien, "http": http, "ja4": ja4, "ja3_hash": akamai}
    except Exception as e:
        print(f"  {quien:<28} fallo: {e}")
        return {"quien": quien, "error": str(e)}


def main():
    print("=== 1. Huellas (contra el eco publico tls.peet.ws) ===")
    medidas = [
        huella("curl_cffi chrome", "chrome"),
        huella("curl_cffi firefox", "firefox"),
        huella("curl de serie (sin cffi)"),
    ]

    print("\n=== 2. Navegador real: pagina OFICIAL de acceso ===")
    con_pantalla = bool(os.environ.get("DISPLAY"))
    print(f"  DISPLAY={'si' if con_pantalla else 'no'} -> headless={not con_pantalla}")
    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            user_data_dir=str(DIR / "perfil-sonda"),
            headless=not con_pantalla,
            locale="es-CL",
            timezone_id="America/Santiago",
            viewport={"width": 1280, "height": 860},
            args=["--disable-blink-features=AutomationControlled"],
        )
        ctx.add_init_script("Object.defineProperty(navigator,'webdriver',{get:()=>undefined});")
        page = ctx.pages[0] if ctx.pages else ctx.new_page()

        ua = page.evaluate("navigator.userAgent")
        print(f"  userAgent del navegador: {ua}")

        antes = {c["name"] for c in ctx.cookies()}
        page.goto("https://connect.garmin.com/modern/", wait_until="domcontentloaded", timeout=60000)
        time.sleep(4)

        print(f"  URL final: {page.url[:110]}")
        print(f"  titulo: {page.title()!r}")

        campos = page.evaluate(
            """() => Array.from(document.querySelectorAll('input,button'))
                .filter(e => e.offsetParent !== null)
                .map(e => (e.tagName + ' id=' + (e.id||'-') + ' name=' + (e.name||'-') +
                           ' type=' + (e.type||'-') + ' [' + (e.innerText||'').trim().slice(0,26) + ']').trim())"""
        )
        print(f"  campos visibles ({len(campos)}):")
        for c in campos:
            print(f"    {c}")

        texto = page.inner_text("body")[:400].replace("\n", " ")
        print(f"  texto de la pagina: {texto[:220]!r}")
        reto = any(
            m in (page.content() or "").lower()
            for m in ["just a moment", "cf-chl", "checking your browser", "challenge-platform"]
        )
        print(f"  ¿reto de Cloudflare en el HTML?: {'SI' if reto else 'NO'}")

        (DIR / "sonda-pagina.html").write_text(page.content() or "")
        if not con_pantalla:
            page.screenshot(path=str(DIR / "sonda-pantalla.png"), full_page=True)

        print("\n  cookies (NOMBRES, nunca valores):")
        for c in ctx.cookies():
            nuevo = "" if c["name"] in antes else "  <- nueva en esta carga"
            print(f"    {c['domain']:<28} {c['name']}{nuevo}")
        print(f"  ¿hay _cfuvid? {'SI' if any(c['name'] == '_cfuvid' for c in ctx.cookies()) else 'NO'}")

        ctx.close()

    (DIR / "sonda-huellas.json").write_text(json.dumps(medidas, indent=2))
    print(f"\nGuardado: {DIR / 'sonda-huellas.json'}, sonda-pagina.html")
    print("SONDA-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
