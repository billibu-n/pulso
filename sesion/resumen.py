#!/usr/bin/env python3
"""resumen.py — EL SELLO DE MANO: lo que un chat NUEVO necesita saber, en un comando.

El problema que resuelve: al abrir un chat nuevo el agente no sabe donde quedo la tarea, y el
fichero de estado (44 KB, 626 lineas) es demasiado grande para leerlo entero y demasiado viejo
para creerlo. Resultado medido el 2026-10-15: el dueno tuvo que re-explicar el objetivo del
cliente DENTRO de la misma conversacion.

Regla de la casa que esto implementa: **la conversacion NO es el portador del estado; el estado
vive en DISCO y se lee**. Aqui lo que manda es `datos/trabajo-ui/TASKS.json` (el encargo) y
`ESTADO.md` (donde quedo), y este comando imprime SOLO lo que hace falta para arrancar.

    python3 sesion/resumen.py            # el sello de mano (texto para pegar en un chat nuevo)
    python3 sesion/resumen.py --json     # lo mismo, para que lo consuma un agente

TOPE DURO: si el sello pasa de 70 lineas, este script FALLA en vez de imprimir. Un estado que
crece es un estado que nadie lee: la casa ya pago ese error (44 KB y contradicciones dentro).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

TOPE_LINEAS = 70
RAIZ = Path(__file__).resolve().parent.parent


def _leer_json(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8", errors="replace"))
    except Exception:  # noqa: BLE001
        return None


def _tareas(tasks: dict):
    """Devuelve (hechas, en_curso, pendientes) aceptando las dos formas del fichero.

    Una entrada cuyo texto empieza por el id, o que arranca con 'Fase N HECHA', 'Hito X',
    es una NOTA de estado, no una tarea: se va a `curso` y no ensucia la lista de pendientes.
    """
    bruto = tasks.get("tareas") or tasks.get("tasks") or []
    if not bruto:
        # El PLAN es la fuente de las tareas cuando el TASKS.json solo lleva el encargo.
        plan = _leer_json(RAIZ / "PLAN.json") or {}
        bruto = plan.get("tasks") or []
    hechas, curso, pend = [], [], []
    for t in bruto:
        if not isinstance(t, dict):
            continue
        estado = str(t.get("estado") or t.get("status") or "").lower()
        que = " ".join(str(t.get("descripcion") or t.get("title") or t.get("que") or "").split())
        item = {"id": t.get("id", "?"), "que": que[:300], "estado": estado}
        es_nota = bool(re.match(r"^(F\d|Hito|[A-Z]\d?F\d)", que)) or " hecha" in que[:26].lower()
        if estado.startswith(("hech", "done", "cerrad", "complet")):
            hechas.append(item)
        elif estado.startswith(("en ", "en_", "curso", "wip", "progreso", "abiert", "in-", "in_")) or es_nota:
            curso.append(item)
        else:
            pend.append(item)
    return hechas, curso, pend


def sello() -> list[str]:
    # EL ORDEN IMPORTA: manda el TASKS.json de la RAIZ (el del andamiaje). El de
    # datos/trabajo-ui/ es el encargo de un chat concreto y puede estar viejo.
    tasks = _leer_json(RAIZ / "TASKS.json") or _leer_json(RAIZ / "datos" / "trabajo-ui" / "TASKS.json")
    if tasks is None:
        return ["[SIN SELLO] no encuentro TASKS.json ni en la raiz ni en datos/trabajo-ui/: el encargo no esta en disco."]

    L: list[str] = []
    L.append("# SELLO DE MANO — pulso (leer esto ANTES de tocar nada)")
    L.append(f"actualizado: {tasks.get('actualizado', '?')}   rama: {tasks.get('rama', '?')}")
    L.append("")
    L.append("## EL OBJETIVO DEL CLIENTE (esto NO se reinterpreta, prima sobre todo lo demas)")
    meta = tasks.get("meta_del_proyecto") or tasks.get("encargo")
    if not meta:
        meta = "(el TASKS.json no declara objetivo: anade `encargo` o `meta_del_proyecto`)"
    for lin in _envolver(str(meta), 96):
        L.append("  " + lin)
    L.append("")

    hechas, curso, pend = _tareas(tasks)
    L.append(f"## ESTADO DE LAS TAREAS — {len(hechas)} hechas · {len(curso)} en curso/notas · {len(pend)} pendientes")
    for t in curso[:5]:
        L.append(_linea("[EN CURSO]", t))
    for t in pend[:8]:
        L.append(_linea("[PENDIENTE]", t))
    L.append("")

    if tasks.get("cierre_del_chat"):
        L.append("## COMO CERRO EL CHAT ANTERIOR")
        for lin in _envolver(str(tasks["cierre_del_chat"]), 96):
            L.append("  " + lin)
        L.append("")

    decisiones = tasks.get("decisiones_pendientes") or tasks.get("decisiones") or []
    if decisiones:
        L.append("## LO QUE ESPERA UNA DECISION DEL DUENO (no lo decidas tu)")
        for d in _decisiones_texto(decisiones)[:6]:
            L.append(f"  ? {d}")
        L.append("")

    L.append("## ANTES DE AFIRMAR NADA: midelo")
    L.append("  python3 herramientas/verificar-afirmacion.py api      # que CONTESTO Garmin")
    L.append("  python3 herramientas/verificar-afirmacion.py volcado  # que se guardo")
    L.append("  python3 herramientas/verificar-afirmacion.py base     # que PROCESO el CLI")
    L.append("")
    L.append("## REGLA DE LA CASA")
    L.append("  1. El objetivo del cliente prima. Si una sugerencia no lo sirve, se descarta.")
    L.append("  2. Ninguna cifra entra aqui sin COMANDO con el que se comprobo. Sin comando, es hipotesis.")
    L.append("  3. Si no lo se, digo «no lo se» y digo con que comando se averigua. No se rellena el hueco.")
    return L


def _linea(etiqueta: str, t: dict, tope: int = 118) -> str:
    """Una tarea, en UNA linea: el sello se lee de un vistazo o no sirve."""
    cuerpo = f"  {etiqueta} {t['id']}: {t['que']}"
    return cuerpo if len(cuerpo) <= tope else cuerpo[: tope - 1].rstrip() + "…"


def _decisiones_texto(d) -> list[str]:
    salida = []
    if isinstance(d, dict):
        d = [f"{k}: {v}" for k, v in d.items()]
    for x in d if isinstance(d, list) else [d]:
        salida.append(str(x).replace("\n", " ")[:200])
    return salida


def _envolver(texto: str, ancho: int) -> list[str]:
    palabras, linea, out = texto.split(), "", []
    for p in palabras:
        if len(linea) + len(p) + 1 > ancho:
            out.append(linea)
            linea = p
        else:
            linea = f"{linea} {p}".strip()
    if linea:
        out.append(linea)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="El sello de mano del proyecto (con tope duro de lineas).")
    ap.add_argument("--json", action="store_true", help="salida para consumo de un agente")
    args = ap.parse_args()
    texto = sello()
    if args.json:
        print(json.dumps({"lineas": len(texto), "sello": texto}, ensure_ascii=False, indent=2))
        return 0
    if len(texto) > TOPE_LINEAS:
        print(f"[FALLA] el sello tiene {len(texto)} lineas y el tope es {TOPE_LINEAS}.")
        print("Un estado que crece no lo lee nadie: mueve el detalle a un tema y deja aqui el resumen.")
        return 1
    print("\n".join(texto))
    return 0


if __name__ == "__main__":
    sys.exit(main())
