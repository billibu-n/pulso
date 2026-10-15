#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""LA PREGUNTA QUE DECIDE LA ARQUITECTURA: ¿se puede pedir /gc-api POR CODIGO?

Medido el 2026-10-15 (mirar.py): la app de Garmin pide `/gc-api/...` con estas cabeceras
y le responden **200**:

    connect-csrf-token   <- la clave
    nk: NT
    x-app-ver · x-lang · x-requested-with · referer · user-agent · sec-ch-ua*

y un `fetch` a la misma URL sin esas cabeceras da **403**. O sea: el 403 NO era Cloudflare
ni la huella TLS; eran las CABECERAS.

QUE HACE
  1. Junta lo capturado (cabeceras-gcapi.json, 600) con el jarro de cookies
     (sesion-navegador.json, 600) y pide `/gc-api/` con tres clientes distintos:
        - curl_cffi con huella de Chrome
        - curl_cffi SIN huella (TLS propio)
        - **curl de serie**, que es HTTP/1.1 sin nada especial: igual que `ureq` en Rust
  2. Dice de donde sale el `connect-csrf-token` (cookie, meta o almacenamiento), sin imprimirlo.
  3. Si el curl de serie funciona, Rust sincroniza SIN navegador y SIN tocar la huella.

NUNCA imprime tokens ni cookies: solo codigos HTTP y tamanos.
Uso: cd ~/garmin-lab && ./venv/bin/python replicar.py
"""

import json
import os
import subprocess
import tempfile
from pathlib import Path

from curl_cffi import requests as creq

DIR = Path.home() / "garmin-lab"
DATOS = DIR / "datos"
CABECERAS = DIR / "cabeceras-gcapi.json"
SESION = DIR / "sesion-navegador.json"

RUTAS = [
    "/gc-api/web-gateway/snapshot/usageIndicators",
    "/gc-api/userprofile-service/socialProfile",
    "/gc-api/activitylist-service/activities/search/activities?start=0&limit=5",
    "/gc-api/device-service/deviceregistration/devices",
]

# Las que pone el cliente por su cuenta; no se copian.
NO_COPIAR = {"content-length", "accept-encoding", "cookie", "host", "connection"}


def decir(m):
    print(m, flush=True)


def main():
    if not CABECERAS.exists() or not SESION.exists():
        decir("faltan cabeceras-gcapi.json o sesion-navegador.json: corre mirar.py y sesion.py")
        return 2

    capturadas = json.loads(CABECERAS.read_text())
    sesion = json.loads(SESION.read_text())

    modelo = None
    for u, d in capturadas.items():
        if "connect-csrf-token" in d["cabeceras"]:
            modelo = d["cabeceras"]
            decir(f"modelo de cabeceras tomado de: {u.replace('https://connect.garmin.com', '')}")
            break
    if not modelo:
        decir("ninguna peticion capturada traia connect-csrf-token")
        return 3

    cab = {k: v for k, v in modelo.items() if k not in NO_COPIAR}
    galletas = [c for c in sesion["cookies"] if "garmin" in c["domain"]]
    cookie_header = "; ".join(f"{c['name']}={c['value']}" for c in galletas)
    decir(f"cabeceras que se copian: {sorted(cab.keys())}")
    decir(f"cookies que se mandan ({len(galletas)}): {sorted({c['name'] for c in galletas})}")

    # de donde sale el connect-csrf-token
    csrf = cab.get("connect-csrf-token", "")
    donde = []
    for c in galletas:
        if c["value"] == csrf:
            donde.append(f"cookie {c['name']}")
    for k, v in (sesion.get("almacenamiento") or {}).items():
        if isinstance(v, str) and csrf and csrf in v:
            donde.append(f"almacenamiento {k}")
    decir(f"el connect-csrf-token sale de: {donde or 'NO se encontro en cookies ni almacenamiento'}")

    decir("")
    decir("=== POR CODIGO ===")
    resultados = {"curl_cffi chrome": {}, "curl_cffi sin huella": {}, "curl de serie (HTTP/1.1)": {}}

    for ruta in RUTAS:
        url = "https://connect.garmin.com" + ruta
        h = dict(cab)
        h["Cookie"] = cookie_header

        for nombre, kw in (("curl_cffi chrome", {"impersonate": "chrome"}),
                           ("curl_cffi sin huella", {})):
            try:
                r = creq.get(url, headers=h, timeout=40, **kw)
                resultados[nombre][ruta] = r.status_code
                decir(f"  {nombre:<24} {ruta[:46]:<46} HTTP {r.status_code}  {len(r.text)} B")
                if r.status_code == 200 and nombre == "curl_cffi chrome":
                    (DATOS / ("codigo-" + ruta.strip('/').split('?')[0].replace('/', '_')[:36] + ".json")).write_text(r.text)
            except Exception as e:
                resultados[nombre][ruta] = -1
                decir(f"  {nombre:<24} {ruta[:46]:<46} fallo {e}")

        # curl de serie, con las cabeceras en un fichero de configuracion (600) para que
        # los secretos NO vayan en la linea de ordenes (visibles con `ps`).
        with tempfile.NamedTemporaryFile("w", suffix=".curl", delete=False) as f:
            cfg = f.name
            for k, v in h.items():
                f.write(f'header = "{k}: {v}"\n')
        os.chmod(cfg, 0o600)
        try:
            salida = subprocess.run(
                ["curl", "-s", "-o", "/dev/null", "-w", "%{http_code} %{size_download}",
                 "-K", cfg, url],
                capture_output=True, text=True, timeout=60)
            partes = (salida.stdout or "").strip().split()
            estado = int(partes[0]) if partes and partes[0].isdigit() else -1
            tam = partes[1] if len(partes) > 1 else "?"
            resultados["curl de serie (HTTP/1.1)"][ruta] = estado
            decir(f"  {'curl de serie (HTTP/1.1)':<24} {ruta[:46]:<46} HTTP {estado}  {tam} B")
        except Exception as e:
            decir(f"  {'curl de serie':<24} {ruta[:46]:<46} fallo {e}")
        finally:
            os.unlink(cfg)

    decir("")
    decir("=== RESUMEN (codigos por cliente) ===")
    for nombre, mapa in resultados.items():
        buenos = sum(1 for v in mapa.values() if v == 200)
        decir(f"  {nombre:<26} {buenos} de {len(mapa)} caminos con 200")
    decir("REPLICAR-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
