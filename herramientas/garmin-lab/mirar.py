#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Mirar lo que hace DE VERDAD la app de Garmin en /gc-api, y probar un fetch DE DENTRO.

Por que: `page.request.get` y `curl` dan 403 aunque las cookies sean las buenas. El cliente
HTTP de Playwright NO es el motor del navegador (otra huella), asi que Cloudflare lo ve
distinto. Un `fetch` ejecutado DENTRO de la pagina si usa la pila real del navegador.

QUE HACE
  1. Abre la app y apunta las respuestas de /gc-api: metodo, ruta, ESTADO y tamano.
  2. De la primera que responda 200, guarda TODAS sus cabeceras en un fichero 600 (en Kali)
     e imprime solo los NOMBRES — asi se ve de que se autentica sin soltar el secreto.
  3. Hace `fetch` DENTRO de la pagina a los caminos de datos y dice estado y tamano.
  4. Guarda los cuerpos buenos en datos/.

Uso: cd ~/garmin-lab && DISPLAY=:0 ./venv/bin/python mirar.py
"""

import json
import os
import time
from pathlib import Path

from playwright.sync_api import sync_playwright

DIR = Path.home() / "garmin-lab"
DATOS = DIR / "datos"
PERFIL = DIR / os.environ.get("GARMIN_PERFIL", "perfil-sonda")
CABECERAS = DIR / "cabeceras-gcapi.json"
GC = "/gc-api"

CAMINOS = [
    f"{GC}/userprofile-service/socialProfile",
    f"{GC}/web-gateway/snapshot/usageIndicators",
    f"{GC}/activitylist-service/activities/search/activities?start=0&limit=5",
    f"{GC}/activitylist-service/activities/v2/newsfeed/status",
]


def decir(m):
    print(m, flush=True)


def main():
    DATOS.mkdir(exist_ok=True)
    apuntes, guardadas = [], {}

    def al_responder(resp):
        if GC not in resp.url:
            return
        req = resp.request
        h = {k.lower(): v for k, v in req.headers.items()}
        try:
            n = len(resp.body())
        except Exception:
            n = -1
        apuntes.append({"m": req.method, "u": resp.url.split("?")[0], "estado": resp.status, "bytes": n})
        if resp.status == 200 and resp.url.split("?")[0] not in guardadas:
            guardadas[resp.url.split("?")[0]] = {
                "cabeceras": h,
                "cuerpo_inicio": (req.post_data or "")[:600] if req.method == "POST" else "",
            }

    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            str(PERFIL), headless=not os.environ.get("DISPLAY"), locale="es-CL",
            timezone_id="America/Santiago", viewport={"width": 1280, "height": 860},
            args=["--disable-blink-features=AutomationControlled"])
        ctx.on("response", al_responder)
        page = ctx.pages[0] if ctx.pages else ctx.new_page()

        decir("1) abriendo la app...")
        page.goto("https://connect.garmin.com/app/home", wait_until="domcontentloaded", timeout=90000)
        time.sleep(10)
        page.goto("https://connect.garmin.com/app/activities", wait_until="domcontentloaded", timeout=90000)
        time.sleep(8)
        decir(f"   URL: {page.url[:80]}")

        decir(f"2) respuestas de {GC} vistas ({len(apuntes)}):")
        from collections import Counter
        c = Counter((a["m"], a["u"].replace("https://connect.garmin.com", ""), a["estado"]) for a in apuntes)
        for (m, u, e), n in c.most_common(12):
            decir(f"   {n:>3}x {m:<5} {e}  {u[:64]}")

        decir(f"   rutas que respondieron 200: {len(guardadas)}")
        for u, d in list(guardadas.items())[:4]:
            nombres = sorted(d["cabeceras"].keys())
            decir(f"   {u.replace('https://connect.garmin.com', '')[:60]}")
            decir(f"      cabeceras: {nombres}")
            auth = [k for k in nombres if "auth" in k or "token" in k or k.startswith("x-garmin") or k == "nk"]
            decir(f"      de autenticacion: {auth}")
        CABECERAS.write_text(json.dumps(guardadas, indent=2))
        os.chmod(CABECERAS, 0o600)

        decir("3) `fetch` DE DENTRO de la pagina (pila real del navegador):")
        resultados = page.evaluate(
            """async (rutas) => {
                const salida = [];
                for (const r of rutas) {
                    try {
                        const res = await fetch(r, {credentials:'include',
                            headers:{'Accept':'application/json, text/plain, */*'}});
                        const t = await res.text();
                        salida.push({ruta:r, estado:res.status, largo:t.length, inicio:t.slice(0,200)});
                    } catch (e) { salida.push({ruta:r, estado:-1, largo:0, inicio:String(e)}); }
                }
                return salida; }""", CAMINOS)

        for x in resultados:
            decir(f"   {x['ruta'].replace(GC, '')[:52]:<52} HTTP {x['estado']}  {x['largo']} B")
            if x["estado"] == 200:
                nombre = "dentro-" + x["ruta"].strip("/").split("?")[0].replace("/", "_")[:36] + ".json"
                (DATOS / nombre).write_text(x["inicio"])
                decir(f"      primeros 160: {x['inicio'][:160]!r}")

        ctx.close()
    decir("MIRAR-OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
