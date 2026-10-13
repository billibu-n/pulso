#!/usr/bin/env python3
"""Convierte las paginas de documentacion guardadas en texto legible.

POR QUE EXISTE: se guardaron 32 paginas de la documentacion oficial de Slint, y son HTML de un sitio
con menu, barra lateral y navegacion. **Estudiar eso es leer el 90 % de cosas que no son el contenido.**
Esto saca EL TEXTO DE VERDAD, y guarda el codigo de ejemplo tal cual -- que es la mitad del valor.
"""
import glob
import html
import os
import re
import sys

AQUI = os.path.dirname(os.path.abspath(__file__))


def a_texto(bruto):
    s = bruto
    # El contenido esta en <main>; lo de fuera es menu y navegacion.
    m = re.search(r"<main[^>]*>(.*?)</main>", s, re.S)
    if m:
        s = m.group(1)
    # Los bloques de codigo, antes de quitar etiquetas: hay que conservarlos.
    s = re.sub(r"<pre[^>]*>(.*?)</pre>", lambda x: "\n```\n" + re.sub(r"<[^>]+>", "", x.group(1)) + "\n```\n", s, flags=re.S)
    # Fuera scripts, estilos y la navegacion.
    s = re.sub(r"<(script|style|nav|svg)[^>]*>.*?</\1>", "", s, flags=re.S)
    # Un salto por cada bloque.
    s = re.sub(r"</(p|div|li|h1|h2|h3|h4|tr|section)>", "\n", s)
    s = re.sub(r"<h([1-6])[^>]*>", lambda m: "\n" + "#" * int(m.group(1)) + " ", s)
    s = re.sub(r"<li[^>]*>", "- ", s)
    s = re.sub(r"<br\s*/?>", "\n", s)
    # Y fuera el resto de etiquetas.
    s = re.sub(r"<[^>]+>", "", s)
    s = html.unescape(s)
    # Limpiar el aire de mas, sin comerse la estructura.
    s = re.sub(r"[ \t]+", " ", s)
    s = re.sub(r"\n\s*\n\s*\n+", "\n\n", s)
    return s.strip()


def main():
    carpeta = sys.argv[1] if len(sys.argv) > 1 else os.path.join(AQUI, "..", "docs", "slint-fuentes")
    hechos = 0
    for f in sorted(glob.glob(os.path.join(carpeta, "*.html"))):
        txt = a_texto(open(f, encoding="utf-8", errors="replace").read())
        if len(txt) < 400:
            print(f"  AVISO: {os.path.basename(f)} dio muy poco texto ({len(txt)} car.)")
            continue
        salida = f[:-5] + ".txt"
        with open(salida, "w", encoding="utf-8") as o:
            o.write(txt)
        hechos += 1
        print(f"  {os.path.basename(f):<28} {len(txt):>7} caracteres")
    print(f"\n  convertidas: {hechos}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
