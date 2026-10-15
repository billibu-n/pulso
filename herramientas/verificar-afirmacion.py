#!/usr/bin/env python3
"""verificar-afirmacion.py — LA PRUEBA DE UNA AFIRMACION, EN UN COMANDO.

Para que sirve: un agente (o una persona) dice «la app no tiene datos de salud», o
«los diarios vienen con dato». Las dos frases pueden ser CIERTAS y CONTRADICTORIAS a la
vez, porque hablan de CAPAS distintas. Esta herramienta mira la capa que le pidas y
devuelve el VALOR CRUDO, no una opinion.

    CAPA                     QUE ES                              PUEDE ESTAR VACIA MIENTRAS OTRA TIENE DATO
    1. el cuerpo de la API   lo que Garmin CONTESTO hoy          SI  (es una foto de un momento)
    2. el volcado            lo que se PIDIO y se guardo antes    SI  (fotos de cada peticion)
    3. la base de datos      lo que el CLI PROCESO               SI  (es acumulativa: no se vacia sola)

REGLA DE ORO: la ausencia de dato se afirma CON ALCANCE Y FECHA («en la cuenta X, del 1 de
mayo al 15 de octubre de 2026, las 165 respuestas de sueno trajeron `null`»), nunca en
absoluto («no hay datos de salud»). El byte cuenta ESTRUCTURA; el dato lo dice el valor.

USO
    python3 herramientas/verificar-afirmacion.py capas
    python3 herramientas/verificar-afirmacion.py api     [--proyecto RUTA]
    python3 herramientas/verificar-afirmacion.py volcado [--proyecto RUTA] [--clase SUEÑO]
    python3 herramientas/verificar-afirmacion.py base    [--db RUTA]
    python3 herramientas/verificar-afirmacion.py pruebas  [--proyecto RUTA]
    python3 herramientas/verificar-afirmacion.py todo    [--proyecto RUTA]

Siempre imprime, al final, el COMANDO CRUDO con el que se comprobo. Ese comando es la
evidencia: sin el, la cifra no vale.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# ---------------------------------------------------------------- utilidades

def _valor(v) -> bool:
    """Un valor CUENTA si no es None ni cadena vacia. El 0 y el False SI cuentan."""
    return v is not None and v != ""


def perfilar(obj, tope_lista: int = 2):
    """(claves_con_valor, claves_nulas) recorriendo diccionarios y listas."""
    con, nulas = [], []

    def anda(o):
        if isinstance(o, dict):
            for k, v in o.items():
                if isinstance(v, (dict, list)):
                    anda(v)
                elif _valor(v):
                    con.append((k, v))
                else:
                    nulas.append(k)
        elif isinstance(o, list):
            for it in o[:tope_lista]:
                anda(it)

    anda(obj)
    return con, nulas


def _json_crudo(path: Path):
    """Carga sin romper por una codificacion rara."""
    with open(path, "rb") as fh:
        return json.loads(fh.read().decode("utf-8", "replace"))


def _titulo(t: str):
    print(f"\n=== {t} ===")


# ---------------------------------------------------------------- 1. capas

def cmd_capas(args) -> int:
    _titulo("LAS TRES CAPAS (y por que dos personas pueden tener razon a la vez)")
    print("  1. EL CUERPO DE LA API   lo que Garmin CONTESTO en una peticion concreta")
    print("     -> puede traer la clave con `null`: eso es «no lo trajo», NO «no existe»")
    print("  2. EL VOLCADO            los .json que se guardaron (una foto por peticion)")
    print("     -> puede estar VACIO mientras la base tiene dato, y al reves: son capas distintas")
    print("  3. LA BASE DE DATOS      lo que el CLI proceso (acumulativa)")
    print("     -> `garmin info` manda sobre «yo recuerdo que se bajo»")
    print()
    print("  El TAMANO de una respuesta mide ESTRUCTURA (nombres de campo + null).")
    print("  El DATO lo dice el VALOR. Confundirlos es el error que costo este documento:")
    print("  `sueno.json` pesa 652 B y su `sleepTimeSeconds` es `null`.")
    return 0


# ---------------------------------------------------------------- 2. la API

def cmd_api(args) -> int:
    raiz = Path(args.proyecto).resolve()
    dirs = [p for p in (raiz / "datos" / "garmin-gcapi", raiz / "datos" / "garmin-gcapi-crudo") if p.is_dir()]
    if not dirs:
        print(f"[SIN MEDIR] no existe ninguna carpeta de crudos de la API bajo {raiz}/datos/")
        print("  (pedirla: el volcado de /gc-api va a datos/garmin-gcapi/)")
        return 2
    d = dirs[0]
    _titulo(f"CAPA 1 · EL CUERPO DE LA API  ->  {d}")
    print(f"{'fichero':<22}{'bytes':>8}{'con valor':>11}{'null':>7}   que claves traen valor")
    totales = {'con': 0, 'nulas': 0, 'f': 0}
    for f in sorted(d.glob("*.json")):
        try:
            con, nulas = perfilar(_json_crudo(f))
        except Exception as e:  # noqa: BLE001
            print(f"{f.name:<22}{f.stat().st_size:>8}{'ILEGIBLE':>11}   {e}")
            continue
        totales['con'] += len(con); totales['nulas'] += len(nulas); totales['f'] += 1
        nombres = ", ".join(k for k, _ in con[:4])
        print(f"{f.name:<22}{f.stat().st_size:>8}{len(con):>11}{len(nulas):>7}   {nombres[:58]}")
    print(f"\n  {totales['f']} ficheros · {totales['con']} claves CON VALOR · {totales['nulas']} claves en `null`")
    print("  LECTURA: si las claves con valor son solo `calendarDate`/`userProfilePK`, la respuesta")
    print("  NO trae dato: trae la pregunta. Decir «hay dato» mirando los bytes es confabular.")
    print(f"\n  COMANDO CRUDO: ls -la {d} && python3 -c \"import json;d=json.load(open('{d}/" +
          (sorted(d.glob('*.json'))[0].name if list(d.glob('*.json')) else 'X') + "'));print(d)\"")
    return 0


# ---------------------------------------------------------------- 3. el volcado

def cmd_volcado(args) -> int:
    raiz = Path(args.proyecto).resolve()
    base = raiz / "datos" / "garmin_files" / "storage"
    if not base.is_dir():
        print(f"[SIN MEDIR] no existe {base}")
        return 2
    _titulo(f"CAPA 2 · EL VOLCADO  ->  {base}")
    clases: dict[str, list[Path]] = {}
    for f in base.glob("*.json"):
        m = re.match(r"\d+_([A-Z_]+)_(\d{4})-(\d{2})", f.name)
        clases.setdefault(m.group(1) if m else "OTRO", []).append(f)
    filtro = args.clase.upper() if args.clase else None
    if filtro:
        clases = {k: v for k, v in clases.items() if filtro in k}
    print(f"{'clase':<20}{'ficheros':>9}{'bytes':>10}{'con valor':>11}{'null':>7}   meses")
    for k in sorted(clases, key=lambda x: -len(clases[x])):
        fs = clases[k]
        con_t = nul_t = 0
        meses = set()
        for f in fs:
            m = re.search(r"(20\d\d)-(\d\d)", f.name)
            if m:
                meses.add(f"{m.group(1)}-{m.group(2)}")
            if k in ("ACTIVITIES_LIST", "OTRO"):
                continue
            try:
                con, nulas = perfilar(_json_crudo(f), tope_lista=1)
            except Exception:  # noqa: BLE001
                continue
            con_t += len(con); nul_t += len(nulas)
        mb = sum(f.stat().st_size for f in fs)
        rango = f"{min(meses)}..{max(meses)}" if meses else "-"
        print(f"{k:<20}{len(fs):>9}{mb:>10}{con_t:>11}{nul_t:>7}   {rango}")
    print("\n  LECTURA: un fichero por clase y dia. Si `con valor` es 0 en SUEÑO/STRESS/HEART_RATE,")
    print("  la cuenta NO tiene ese dato en ese periodo — y eso se dice CON el periodo y la cuenta.")
    print(f"\n  COMANDO CRUDO: ls {base} | sed -E 's/^[0-9]+_//; s/_[0-9TZ:-]+\\.json$//' | sort | uniq -c")
    return 0


# ---------------------------------------------------------------- 4. la base

def cmd_base(args) -> int:
    db = Path(args.db).resolve()
    if not db.is_file():
        print(f"[SIN MEDIR] no existe la base {db}")
        return 2
    _titulo(f"CAPA 3 · LA BASE DE DATOS  ->  {db}  ({db.stat().st_size/1e6:.1f} MB)")
    if not _hay("sqlite3"):
        print("  [SIN MEDIR] falta el binario sqlite3 en esta maquina")
        return 2
    tablas = _sqlite(db, "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;")
    nombres = [t for t in tablas.split() if t]
    salud = [t for t in nombres if re.search(r"sleep|stress|heart|hrv|respiration|body_battery", t)]
    print(f"  {len(nombres)} tablas · {len(salud)} de salud: {', '.join(salud[:10])}")
    print(f"\n{'tabla':<22}{'filas':>8}{'col. con valor':>16}{'desde':>12}{'hasta':>12}")
    for t in salud:
        n = _sqlite(db, f"SELECT COUNT(*) FROM {t};").strip() or "0"
        cols = _sqlite(db, f"PRAGMA table_info({t});")
        nombres_col = [c.split("|")[1] for c in cols.strip().splitlines() if "|" in c]
        fecha = next((c for c in ("calendar_date", "start_ts", "start_time", "date") if c in nombres_col), None)
        valor = next((c for c in ("sleep_time_seconds", "avg_stress_level", "value", "heart_rate",
                                  "resting_heart_rate", "avg_overnight_hrv") if c in nombres_col), None)
        desde = hasta = "-"
        if fecha:
            r = _sqlite(db, f"SELECT MIN({fecha}), MAX({fecha}) FROM {t};").strip()
            partes = (r.split("|") + ["-", "-"])[:2]
            desde, hasta = [p.strip()[:10] or "-" for p in partes]
        con_valor = _sqlite(db, f"SELECT COUNT({valor}) FROM {t};").strip() if valor else "-"
        print(f"{t:<22}{n:>8}{con_valor:>16}{desde:>12}{hasta:>12}")
    print("  LECTURA: `garmin info --db-path <db>` da el resumen oficial; esto lo desglosa por tabla.")
    print(f"\n  COMANDO CRUDO: garmin info --db-path {db}")
    return 0


def _hay(prog: str) -> bool:
    return subprocess.run(["which", prog], capture_output=True).returncode == 0


def _sqlite(db: Path, sql: str) -> str:
    r = subprocess.run(["sqlite3", str(db), sql], capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else ""


# ---------------------------------------------------------------- 5. pruebas

def cmd_pruebas(args) -> int:
    raiz = Path(args.proyecto).resolve()
    motor = raiz / "motor"
    _titulo(f"LAS PRUEBAS DEL ARBOL  ->  {motor}")
    if not motor.is_dir():
        print(f"[SIN MEDIR] no existe {motor}")
        return 2
    total = 0
    for f in sorted(motor.glob("tests/*.rs")):
        n = len(re.findall(r"#\[test\]", f.read_text(errors="replace")))
        total += n
        print(f"  {f.name:<20}{n:>5} pruebas")
    print(f"  {'TOTAL':<20}{total:>5} pruebas")
    print("\n  LECTURA: este numero cuenta FUNCIONES de prueba, no las corre. El numero que MANDA")
    print("  es el que imprime `cargo test` («N passed»). Si el ESTADO dice otro, gana cargo test.")
    print(f"\n  COMANDO CRUDO: grep -c '#\\[test\\]' {motor}/tests/*.rs")
    print(f"                 cd {motor} && cargo test 2>&1 | tail -20")
    return 0


# ---------------------------------------------------------------- main

def main() -> int:
    ap = argparse.ArgumentParser(description="Comprueba en un comando lo que se afirma de los datos.")
    ap.add_argument("que", choices=["capas", "api", "volcado", "base", "pruebas", "todo"])
    ap.add_argument("--proyecto", default=".", help="raiz del proyecto (por defecto: el directorio actual)")
    ap.add_argument("--db", default=None, help="base de datos sqlite (por defecto: datos/garmin_data.db)")
    ap.add_argument("--clase", default=None, help="filtra una clase del volcado (p.ej. SUEÑO no: SLEEP)")
    args = ap.parse_args()
    if args.db is None:
        args.db = str(Path(args.proyecto).resolve() / "datos" / "garmin_data.db")
    if args.que == "todo":
        return max(cmd_capas(args), cmd_api(args), cmd_volcado(args), cmd_base(args), cmd_pruebas(args))
    return {"capas": cmd_capas, "api": cmd_api, "volcado": cmd_volcado,
            "base": cmd_base, "pruebas": cmd_pruebas}[args.que](args)


if __name__ == "__main__":
    sys.exit(main())
