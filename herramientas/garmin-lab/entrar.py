#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Via del NAVEGADOR para Garmin Connect — el arnes que desbloquea el acceso.

QUE SE MIDIO EL 2026-10-15 (evidencia visual y de red en ~/garmin-lab/)
  1. La pagina de acceso que Garmin sirve HOY es
     `https://sso.garmin.com/portal/sso/en-US/sign-in?clientId=GarminConnect&service=...`
     — el flujo **portal**: una aplicacion JavaScript (index-d10b89ff...js), NO el
     `/sso/signin` (widget/embed) que implementa el instrumento de pulso.
  2. El portal NO tiene `<form action>`, ni `<input type=hidden>`, ni `_csrf`: el acceso lo
     hace el JavaScript por detras. Por eso `curl_cffi` recibe HTTP 200 y 31.860 bytes
     **sin formulario** (el navegador recibe 62.854 con formulario): no hay nada que rellenar
     sin un motor de JavaScript. `curl_cffi` con huella de Firefox recibe 427.
  3. Un navegador REAL tambien recibe `_cfuvid`: la cookie que el instrumento nunca tendra es
     `cf_clearance`, y esa se gana EJECUTANDO el reto, no imitando la huella.
  4. **VISTO EN PANTALLA**: la pagina trae una casilla **«I'm not a robot» (reCAPTCHA)**.
     Consecuencia dura: el acceso necesita **una persona**, no solo un navegador.

CONCLUSION: el acceso necesita un MOTOR DE NAVEGADOR y MANO HUMANA (el reCAPTCHA). Por eso
este arnes NO intenta adivinar selectores: **abre la ventana y el acceso lo haces tu**, y el
arnes solo mira y captura el TICKET. El resto del flujo (ticket -> token -> API) si es HTTP
normal y puede hacerlo Rust.

MODO MANUAL (el recomendado, y el unico que sirve con reCAPTCHA)
  cd ~/garmin-lab && ./venv/bin/python entrar.py
  -> se abre una ventana de Chromium en TU pantalla (puede tardar ~20 s: esperala)
  -> tu escribes correo y contrasena EN LA VENTANA, marcas «I'm not a robot» y pulsas Sign In
  -> el arnes detecta el ticket y sigue solo: canje del token + pedido de la API

MODO AUTOMATICO (solo si la cuenta NO pide reCAPTCHA)
  cd ~/garmin-lab && GARMIN_AUTO=1 ./venv/bin/python entrar.py    # pide correo y clave por teclado

SIN PANTALLA (SSH): `xvfb-run -a ./venv/bin/python GARMIN_AUTO=1 entrar.py`, pero sin pantalla
el reCAPTCHA no se puede resolver: sirve para diagnosticar, no para entrar.

REUTILIZA el perfil `perfil-sonda` (ya tiene el `cf_clearance` ganado). Otro perfil:
GARMIN_PERFIL=perfil-nuevo ./venv/bin/python entrar.py

LAS CREDENCIALES NO PASAN POR AQUI EN MODO MANUAL: se teclean en la pagina de Garmin.
"""

import base64
import getpass
import json
import os
import subprocess
import sys
import time
from pathlib import Path

from curl_cffi import requests as creq
from playwright.sync_api import sync_playwright

DIR = Path.home() / "garmin-lab"
DATOS = DIR / "datos"
PERFIL = DIR / os.environ.get("GARMIN_PERFIL", "perfil-sonda")
TICKET_F = DIR / "ticket.txt"
TOKEN_F = DIR / "token.json"
CAPTURA = DIR / "captura.jsonl"

ENTRADA = "https://connect.garmin.com/modern/"
AUTO = bool(os.environ.get("GARMIN_AUTO"))

SEL_EMAIL = ["#email", "input[name=email]", "input[type=email]"]
SEL_PASS = ["#password", "input[name=password]", "input[type=password]"]
SEL_BOTON = ["#login-btn-signin", "button[type=submit]", "input[type=submit]"]
SEL_MFA = [
    "#mfa-code",
    "input[name=mfa-code]",
    "input[autocomplete=one-time-code]",
    "input[inputmode=numeric]",
    "input[name=code]",
]

CLIENTES = [
    "GARMIN_CONNECT_MOBILE_ANDROID_DI_2025Q2",
    "GARMIN_CONNECT_MOBILE_ANDROID_DI_2024Q4",
    "GARMIN_CONNECT_MOBILE_ANDROID_DI",
    "GARMIN_CONNECT_MOBILE_IOS_DI",
]
TIPO_DE_CONCESION = "https://connectapi.garmin.com/di-oauth2-service/oauth/grant/service_ticket"
URL_TOKEN = "https://diauth.garmin.com/di-oauth2-service/oauth/token"
URL_EMBED = "https://sso.garmin.com/sso/embed"


def decir(m):
    print(m, flush=True)


def visible(page, selectores, espera_ms=20000):
    por_selector = max(1200, espera_ms // max(1, len(selectores)))
    for s in selectores:
        try:
            loc = page.locator(s).first
            loc.wait_for(state="visible", timeout=por_selector)
            return loc, s
        except Exception:
            continue
    return None, None


def diagnostico(page):
    try:
        campos = page.evaluate(
            """() => Array.from(document.querySelectorAll('input,button,a'))
                .filter(e => e.offsetParent !== null)
                .map(e => (e.tagName + ' id=' + (e.id||'-') + ' name=' + (e.name||'-') +
                           ' type=' + (e.type||'-') + ' [' + (e.innerText||'').trim().slice(0,30) + ']'))"""
        )
    except Exception as e:
        campos = [f"(no se pudo leer: {e})"]
    try:
        texto = page.inner_text("body")[:300].replace("\n", " ")
    except Exception:
        texto = ""
    return campos, texto


def canjear(ticket):
    """Ticket -> token DI. El MISMO canje que el instrumento (client.py:1343-1348)."""
    for i, cliente in enumerate(CLIENTES, 1):
        decir(f"  canje {i}/{len(CLIENTES)}: {cliente}")
        basic = base64.b64encode(f"{cliente}:".encode()).decode()
        try:
            r = creq.post(
                URL_TOKEN,
                data={
                    "client_id": cliente,
                    "service_ticket": ticket,
                    "grant_type": TIPO_DE_CONCESION,
                    "service_url": URL_EMBED,
                },
                headers={
                    "Authorization": f"Basic {basic}",
                    "Accept": "application/json,text/html;q=0.9,*/*;q=0.8",
                    "Cache-Control": "no-cache",
                },
                impersonate="chrome",
                timeout=40,
            )
        except Exception as e:
            decir(f"    transporte: {e}")
            continue
        decir(f"    HTTP {r.status_code}")
        if r.status_code == 429:
            decir("    429: ventana de espera contra Garmin. Se para aqui, sin reintentar.")
            return None
        if 200 <= r.status_code < 300:
            try:
                j = r.json()
            except Exception:
                decir(f"    no-JSON: {r.text[:160]!r}")
                continue
            if j.get("access_token"):
                j["_cliente"] = cliente
                TOKEN_F.write_text(json.dumps(j, indent=2))
                os.chmod(TOKEN_F, 0o600)
                # Los NOMBRES de lo que devuelve el token: dice si hay refresh_token
                # (sin refresh, la aplicacion tendria que pedir el acceso otra vez).
                decir(f"    TOKEN OK -> {TOKEN_F} (600); campos: {sorted(j.keys())}")
                for k in ("expires_in", "refresh_token_expires_in", "scope", "token_type"):
                    if k in j:
                        decir(f"      {k} = {j[k]}")
                return j["access_token"]
            decir(f"    sin access_token: {sorted(j.keys())}")
        else:
            decir(f"    cuerpo: {r.text[:160]!r}")
    return None


def main():
    DIR.mkdir(exist_ok=True)
    DATOS.mkdir(exist_ok=True)
    peticiones, ticket, bearer = [], {"v": None}, {"v": None}
    pantalla = bool(os.environ.get("DISPLAY"))

    if AUTO:
        usuario = input("Correo de la cuenta Garmin: ").strip()
        clave = getpass.getpass("Contrasena (no se muestra, no se guarda): ")
        if not usuario or not clave:
            decir("sin credenciales no se sigue")
            return 2
    else:
        usuario = clave = None

    decir(f"Chromium {'con pantalla' if pantalla else 'SIN pantalla'}, perfil {PERFIL.name}")
    if pantalla:
        decir("AVISO: la ventana del navegador tarda unos 20 s en aparecer. Esperala.")
    else:
        decir("AVISO: sin pantalla no se puede resolver el reCAPTCHA «I'm not a robot».")

    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            user_data_dir=str(PERFIL),
            headless=not pantalla,
            locale="es-CL",
            timezone_id="America/Santiago",
            viewport={"width": 1280, "height": 900},
            args=["--disable-blink-features=AutomationControlled"],
        )
        ctx.add_init_script("Object.defineProperty(navigator,'webdriver',{get:()=>undefined});")
        page = ctx.pages[0] if ctx.pages else ctx.new_page()

        def al_pedir(req):
            u = req.url
            if "garmin.com" not in u:
                return
            if "ticket=" in u and not ticket["v"]:
                t = u.split("ticket=", 1)[1].split("&")[0]
                if t.startswith("ST-"):
                    ticket["v"] = t
                    decir("  >>> TICKET capturado del redirect.")
            if "connectapi.garmin.com" in u and not bearer["v"]:
                a = req.headers.get("authorization", "")
                if a.lower().startswith("bearer "):
                    bearer["v"] = a[7:]
                    decir("  >>> plan B: token levantado de una llamada de la propia web.")

        def al_responder(resp):
            if "garmin.com" in resp.url:
                peticiones.append({
                    "metodo": resp.request.method,
                    "ruta": resp.url.split("?")[0],
                    "estado": resp.status,
                })

        ctx.on("request", al_pedir)
        ctx.on("response", al_responder)

        decir("1) abriendo la pagina oficial de acceso...")
        page.goto(ENTRADA, wait_until="domcontentloaded", timeout=90000)
        time.sleep(4)
        decir(f"   en {page.url[:100]}")

        if not AUTO:
            decir("")
            decir("2) HAZLO TU EN LA VENTANA DEL NAVEGADOR:")
            decir("   - escribe el CORREO y la CONTRASENA en la pagina de Garmin")
            decir("   - marca la casilla «I'm not a robot»")
            decir("   - pulsa «Sign In» (y el codigo de dos factores, si lo pide)")
            decir("   El arnes NO toca el teclado: solo mira y espera.")
            decir("")
        else:
            decir("2) formulario (la SPA lo dibuja con JavaScript: puede tardar)...")
            le, sel_e = visible(page, SEL_EMAIL, 30000)
            if not le:
                campos, texto = diagnostico(page)
                decir("   NO hay campo de correo. Campos visibles:")
                for c in campos:
                    decir(f"     {c}")
                decir(f"   texto: {texto[:200]!r}")
                ctx.close()
                return 3
            lp, sel_p = visible(page, SEL_PASS, 8000)
            if lp is not None:
                decir(f"   los dos campos a la vez ({sel_e} / {sel_p}) -> una pulsacion")
                le.fill(usuario)
                lp.fill(clave)
                del clave
            else:
                decir(f"   primero el correo ({sel_e})")
                le.fill(usuario)
                b, _ = visible(page, SEL_BOTON, 8000)
                (b.click() if b else page.keyboard.press("Enter"))
                time.sleep(4)
                lp, sel_p = visible(page, SEL_PASS, 20000)
                if lp is None:
                    campos, texto = diagnostico(page)
                    decir("   NO hay campo de contrasena. Campos visibles:")
                    for c in campos:
                        decir(f"     {c}")
                    decir(f"   texto: {texto[:200]!r}")
                    ctx.close()
                    return 4
                lp.fill(clave)
                del clave
            b, _ = visible(page, SEL_BOTON, 8000)
            decir("3) pulsando «Sign In»...")
            (b.click() if b else page.keyboard.press("Enter"))
            time.sleep(6)

        espera_s = int(os.environ.get("GARMIN_ESPERA_S", "3600"))
        decir(f"4) esperando a que entres (hasta {espera_s // 60} minutos)...")
        mfa_hecho = False
        avisado = False
        for _ in range(max(1, espera_s // 4)):
            if ticket["v"] or bearer["v"]:
                break
            if AUTO and not mfa_hecho:
                lm, _ = visible(page, SEL_MFA, 800)
                if lm is not None:
                    cod = input("   CODIGO de dos factores de Garmin: ").strip()
                    lm.fill(cod)
                    b, _ = visible(page, SEL_BOTON, 6000)
                    (b.click() if b else page.keyboard.press("Enter"))
                    mfa_hecho = True
                    time.sleep(6)
                    continue
            if not avisado and not pantalla:
                avisado = True
                decir("   (sin pantalla: si hay reCAPTCHA, esto no puede seguir)")
            time.sleep(4)

        if not pantalla:
            page.screenshot(path=str(DIR / "ultima-pantalla.png"), full_page=True)
            decir(f"   captura -> {DIR / 'ultima-pantalla.png'}")
        decir(f"   resultado: {'ticket' if ticket['v'] else ('token de la web' if bearer['v'] else 'NADA todavia')}")
        if not ticket["v"] and not bearer["v"]:
            campos, texto = diagnostico(page)
            decir(f"   URL: {page.url[:120]}")
            decir(f"   texto: {texto[:220]!r}")
            decir("   (si la ventana sigue abierta, puedes terminar el acceso a mano ahora)")
        ctx.close()

    with open(CAPTURA, "w") as f:
        for x in peticiones:
            f.write(json.dumps(x, ensure_ascii=False) + "\n")
    decir(f"trafico a *.garmin.com: {len(peticiones)} -> {CAPTURA}")

    if not (ticket["v"] or bearer["v"]):
        return 5

    if bearer["v"] and not ticket["v"]:
        TOKEN_F.write_text(json.dumps({"access_token": bearer["v"], "_via": "web"}))
        os.chmod(TOKEN_F, 0o600)
        decir(f"token de la web -> {TOKEN_F}")

    if ticket["v"]:
        TICKET_F.write_text(ticket["v"])
        os.chmod(TICKET_F, 0o600)
        decir("5) ticket -> token DI, POR CODIGO y sin navegador:")
        if not canjear(ticket["v"]):
            decir("   el canje por codigo no dio token.")

    decir("6) API, POR CODIGO y sin navegador:")
    return subprocess.run([sys.executable, str(DIR / "pedir.py")]).returncode


if __name__ == "__main__":
    sys.exit(main())
