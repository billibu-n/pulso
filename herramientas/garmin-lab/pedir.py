#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Pide a la API de Connect los datos que el motor necesita, y los deja crudos.

Lee el token de ~/garmin-lab/token.json (600) y NO lo imprime nunca.
Reutilizable: no repite el acceso, asi que no gasta ventana de espera contra Garmin.

Las rutas y las cabeceras son las que el instrumento de pulso ya tiene medidas
(motor/src/sincronizacion.rs y motor/src/bin/sincronizacion.rs:235-249):

  /userprofile-service/socialProfile                       -> displayName
  /activitylist-service/activities/search/activities       -> actividades (start, limit)
  /userstats-service/wellness/daily/{dn}                   -> pulso en reposo (fromDate, untilDate, metricId=60)
  /wellness-service/wellness/dailySleepData/{dn}           -> sueno (date, nonSleepBufferMinutes=60)
  /wellness-service/wellness/dailyStress/{fecha}           -> estres
  /metrics-service/metrics/trainingstatus/aggregated/{fecha} -> training status
"""

import json
import sys
from datetime import date, timedelta
from pathlib import Path

from curl_cffi import requests as creq

DIR = Path.home() / "garmin-lab"
DATOS = DIR / "datos"
TOKEN_F = DIR / "token.json"

API = "https://connectapi.garmin.com"
AGENTE = "GCM-Android-5.23"
AGENTE_GARMIN = (
    "com.garmin.android.apps.connectmobile/5.23; ; "
    "Google/sdk_gphone64_arm64/google; Android/33; Dalvik/2.1.0"
)


def cabeceras(token):
    return {
        "User-Agent": AGENTE,
        "X-Garmin-User-Agent": AGENTE_GARMIN,
        "Accept": "application/json",
        "Authorization": f"Bearer {token}",
    }


def pedir(token, ruta, params=None, segundos=60):
    try:
        r = creq.get(
            API + ruta,
            params=params or {},
            headers=cabeceras(token),
            impersonate="chrome",
            timeout=segundos,
        )
    except Exception as e:
        return None, f"fallo de transporte: {e}"
    return r, None


def guardar(nombre, texto):
    f = DATOS / nombre
    f.write_text(texto)
    return f


def main():
    DATOS.mkdir(exist_ok=True)
    if not TOKEN_F.exists():
        print(f"No hay {TOKEN_F}: primero entrar.py")
        return 2
    token = json.loads(TOKEN_F.read_text()).get("access_token")
    if not token:
        print("token.json no trae access_token")
        return 2

    hoy = date.today()
    desde = (hoy - timedelta(days=7)).isoformat()
    hasta = hoy.isoformat()
    ayer = (hoy - timedelta(days=1)).isoformat()
    print(f"Ventana de prueba: {desde} .. {hasta}")

    # 1) perfil -> displayName (lo necesitan sueno y pulso en reposo, va en la URL)
    r, err = pedir(token, "/userprofile-service/socialProfile")
    if err:
        print(f"  perfil: {err}")
        return 3
    print(f"  perfil: HTTP {r.status_code}")
    guardar("perfil.json", r.text)
    dn = None
    if r.status_code == 200:
        try:
            dn = r.json().get("displayName")
        except Exception:
            pass
    if not dn:
        print("  sin displayName: se sigue con lo que no lo necesita")
    else:
        print(f"  displayName obtenido (no se imprime)")

    pruebas = [
        ("actividades.json", "/activitylist-service/activities/search/activities",
         {"start": "0", "limit": "20"}),
        ("pulso-reposo.json", f"/userstats-service/wellness/daily/{dn}" if dn else None,
         {"fromDate": desde, "untilDate": hasta, "metricId": "60"}),
        ("sueno.json", f"/wellness-service/wellness/dailySleepData/{dn}" if dn else None,
         {"date": ayer, "nonSleepBufferMinutes": "60"}),
        ("estres.json", f"/wellness-service/wellness/dailyStress/{ayer}", None),
        ("training.json", f"/metrics-service/metrics/trainingstatus/aggregated/{ayer}", None),
    ]

    resumen = []
    for nombre, ruta, params in pruebas:
        if not ruta:
            resumen.append((nombre, "saltado (falta displayName)", 0))
            continue
        r, err = pedir(token, ruta, params)
        if err:
            resumen.append((nombre, err, 0))
            continue
        texto = r.text
        f = guardar(nombre, texto)
        vacio = ""
        if r.status_code == 200 and texto.strip():
            try:
                j = json.loads(texto)
                n = len(j) if isinstance(j, list) else len(j.keys())
                vacio = f"{n} claves/elementos"
            except Exception:
                vacio = f"{len(texto)} B de texto"
        print(f"  {nombre}: HTTP {r.status_code}  {vacio}  -> {f.name}")
        resumen.append((nombre, f"HTTP {r.status_code}", vacio))

    print("\nRESUMEN")
    for nombre, estado, detalle in resumen:
        print(f"  {nombre:<20} {estado:<28} {detalle}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
