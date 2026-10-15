#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""¿Se puede hacer por CODIGO el acceso del flujo `portal` (el que Garmin sirve hoy)?

Dos preguntas, sin credenciales y sin intentar entrar:
  A) ¿Que pide el formulario del portal? (accion, campos ocultos, csrf) — se lee del HTML
     que YA guardo la sonda de un navegador real. Cero peticiones nuevas.
  B) ¿Un cliente programatico con huella de Chrome (curl_cffi) recibe el MISMO formulario,
     o un reto de Cloudflare? Aqui esta la respuesta a si el navegador es imprescindible.

No envia usuario ni contrasena: no es un intento de acceso y no gasta ventana de espera.
"""

import json
import re
from pathlib import Path

from curl_cffi import requests as creq

DIR = Path.home() / "garmin-lab"
HTML = DIR / "sonda-pagina.html"

PORTAL = ("https://sso.garmin.com/portal/sso/en-US/sign-in"
          "?clientId=GarminConnect&service=https%3A%2F%2Fconnect.garmin.com%2Fmodern")


def inspeccionar(html):
    formas = re.findall(r"<form[^>]*>", html, re.I)
    acciones = re.findall(r'action\s*=\s*"([^"]*)"', html, re.I)
    ocultos = re.findall(
        r'<input[^>]*type\s*=\s*"hidden"[^>]*>', html, re.I)
    nombres_ocultos = []
    for o in ocultos:
        n = re.search(r'name\s*=\s*"([^"]*)"', o)
        v = re.search(r'value\s*=\s*"([^"]*)"', o)
        nombres_ocultos.append((n.group(1) if n else "?", (v.group(1) if v else "")[:60]))
    scripts = re.findall(r'<script[^>]*src\s*=\s*"([^"]*)"', html, re.I)
    return {
        "formas": formas[:4],
        "acciones": sorted(set(acciones))[:8],
        "ocultos": nombres_ocultos,
        "scripts": [s.split("/")[-1] for s in scripts][:12],
        "csrf_por_regex": (re.findall(r'name="_csrf"\s+value="(.+?)"', html) or [None])[0] is not None,
        "tiene_MFA": bool(re.search(r"mfa|one-time|verification code", html, re.I)),
        "menciona_cf": bool(re.search(r"challenge-platform|cf-chl|turnstile", html, re.I)),
        "bytes": len(html),
    }


def main():
    print("=== A. El formulario REAL que sirvio el navegador (HTML ya guardado) ===")
    if not HTML.exists():
        print(f"  falta {HTML}: corre sonda.py primero")
        return 2
    html = HTML.read_text(errors="replace")
    info = inspeccionar(html)
    for k, v in info.items():
        print(f"  {k}: {v}")

    print("\n=== B. ¿Lo mismo por CODIGO (curl_cffi chrome)? ===")
    for quien, kw in [("curl_cffi chrome", {"impersonate": "chrome"}),
                      ("curl_cffi firefox", {"impersonate": "firefox"})]:
        try:
            r = creq.get(PORTAL, timeout=40, **kw)
        except Exception as e:
            print(f"  {quien}: fallo de transporte: {e}")
            continue
        cuerpo = r.text or ""
        trae_form = ("id=\"email\"" in cuerpo) or ("name=\"email\"" in cuerpo)
        reto = bool(re.search(r"just a moment|cf-chl|Verifying you are human|turnstile",
                              cuerpo, re.I))
        cookies = sorted({c.name for c in r.cookies.jar})
        print(f"  {quien}: HTTP {r.status_code}  bytes={len(cuerpo)}  "
              f"formulario={'SI' if trae_form else 'NO'}  reto={'SI' if reto else 'NO'}")
        print(f"    cookies: {cookies}")
        (DIR / f"portal-{quien.split()[-1]}.html").write_text(cuerpo)
        if not trae_form:
            print(f"    primeros 240: {cuerpo[:240]!r}")
    print("\nPORTAL-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
