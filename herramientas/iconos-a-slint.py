#!/usr/bin/env python3
"""Convierte el sprite de iconos del kit web en componentes de Slint.

POR QUE EXISTE
    El kit de `norma-web` trae **200 iconos** en un sprite SVG, y se usan asi:

        <svg class="icono" aria-hidden="true"><use href="kit/iconos/sprite.svg#casa"></use></svg>

    **Slint no usa HTML ni sprites.** Pero si sabe dibujar un TRAZADO (`Path`), y los iconos de
    Lucide **son** trazados: cada uno es una lista de `d="..."`. Asi que se convierten: un componente
    de Slint por icono, con su trazado dentro.

LO QUE NO SE TOCA
    La LICENCIA. Los iconos son de **Lucide, licencia ISC**: permite usar, copiar, modificar y
    distribuir **incluso comercialmente**, y solo exige **conservar el aviso**. Asi que el aviso va
    escrito en el fichero generado y en el LEEME de los iconos. **No es un detalle: es la condicion.**

COMO SE USA, una vez generado
    import { Icono } from "iconos.slint";
    Icono { nombre: "casa"; ancho: 18px; tono: Claro.fg; }

USO
    python3 iconos-a-slint.py                      # del sprite del kit al .slint
    python3 iconos-a-slint.py --lista              # solo enseña los nombres
"""
import argparse
import io
import os
import re
import sys

# De donde salen los iconos, y a donde van. La RUTA SE DERIVA, no se escribe a mano (regla de la casa:
# nunca la ruta de una maquina dentro de una herramienta).
AQUI = os.path.dirname(os.path.abspath(__file__))
SPRITE = os.path.join(AQUI, "..", "..", "norma-web", "kit", "iconos", "sprite.svg")
SALIDA = os.path.join(AQUI, "..", "motor", "ui", "iconos.slint")


def leer_sprite(ruta):
    """Saca (nombre, trazado, relleno) de cada simbolo del sprite.

    Devuelve tambien el `stroke-width`, que en Lucide es 2 para todos -- pero se lee, no se supone.
    """
    if not os.path.exists(ruta):
        sys.exit(f"No encuentro el sprite en {ruta}")
    s = io.open(ruta, encoding="utf-8").read()

    iconos = []
    for m in re.finditer(r'<symbol\s+id="([^"]+)"([^>]*)>(.*?)</symbol>', s, re.S):
        nombre, atributos, cuerpo = m.group(1), m.group(2), m.group(3)
        trazados = re.findall(r'<path[^>]*\sd="([^"]+)"', cuerpo)
        if not trazados:
            continue
        # `fill="none"` = icono de TRAZO (casi todos). Si no, es relleno y se dibuja distinto.
        relleno = 'fill="none"' not in atributos
        w = re.search(r'stroke-width="([0-9.]+)"', atributos)
        grosor = w.group(1) if w else "2"
        iconos.append((nombre, trazados, relleno, grosor))
    return iconos


def limpiar(trazado):
    """Prepara un trazado para Slint.

    DOS COSAS, y las dos se aprendieron probando:
      1. Slint separa las ordenes con ESPACIOS, y el SVG tambien: vale tal cual.
      2. Pero los numeros pegados a una letra (`v-8`, `.709-1.528`) hay que dejarlos: Slint los
         entiende igual que el SVG. **No se reescriben**, porque reescribirlos es donde se rompen.
    """
    return trazado.strip()


def generar(iconos, salida):
    lineas = []
    a = lineas.append
    a("// LOS ICONOS DEL KIT, convertidos a Slint. **FICHERO GENERADO: no editar a mano.**")
    a("//")
    a("// Sale de `herramientas/iconos-a-slint.py`, leyendo el sprite de `norma-web/kit/iconos`.")
    a("// Para rehacerlo:  python3 herramientas/iconos-a-slint.py")
    a("//")
    a("// ============================================================================")
    a("// LA LICENCIA, Y NO ES UN DETALLE: ES LA CONDICION")
    a("// ============================================================================")
    a("//")
    a("// Los iconos son de **Lucide** (https://lucide.dev), licencia **ISC**.")
    a("//")
    a("//   Que permite .... usar, copiar, modificar y distribuir, INCLUSO COMERCIALMENTE.")
    a("//   Que exige ...... CONSERVAR ESTE AVISO. Nada mas.")
    a("//")
    a("// ISC no tiene copyleft: no contagia al resto del proyecto. Pero **el aviso se queda**.")
    a("// Texto completo de la licencia: `norma-web/kit/iconos/CREDITOS.md`.")
    a("//")
    a("// ============================================================================")
    a("// COMO SE USAN")
    a("// ============================================================================")
    a("//")
    a("//   Icono { nombre: \"casa\"; tamano: 18px; tono: Claro.fg; }")
    a("//")
    a("// El `tono` es el color del trazo. **No se hereda solo**, como en CSS con `currentColor`: en")
    a("// Slint hay que pasarlo. Es lo que tiene no tener herencia de color.")
    a("//")
    a(f"// Iconos: {len(iconos)}")
    a("")
    a("// Cada icono del sprite, por su nombre.")
    a("export global NombresDeIcono {")
    a("    out property <[string]> todos: [")
    for nombre, _, _, _ in iconos:
        a(f'        "{nombre}",')
    a("    ];")
    a("}")
    a("")
    a("// UN icono: el trazado, del color que se le diga, y del tamano que se le diga.")
    a("//")
    a("// Se dibuja con `Path` en un lienzo de 24x24 (el sistema de coordenadas de Lucide), y el")
    a("// tamano final lo pone `width` y `height`. Asi el mismo icono vale a 16 que a 48 sin tocar")
    a("// el trazado.")
    a("export component Icono inherits Path {")
    a("    in property <string> nombre: \"casa\";")
    a("    in property <length> tamano: 18px;")
    a("    in property <color> tono: black;")
    a("")
    a("    width: root.tamano;")
    a("    height: root.tamano;")
    a("    viewbox-width: 24.0;")
    a("    viewbox-height: 24.0;")
    a("    fill: transparent;")
    a("    stroke: root.tono;")
    a("    stroke-width: 2px;")
    a("    clip: true;")
    a("")
    a("    // El trazado, elegido por nombre. Es una cadena condicional: Slint no tiene")
    a("    // diccionarios, asi que esto es lo mas directo que hay.")
    a("    commands: root.nombre == \"\" ? \"\"")
    for i, (nombre, trazados, _, _) in enumerate(iconos):
        union = " ".join(limpiar(t) for t in trazados)
        sep = ":" if i == 0 else ":"
        a(f'        {sep} root.nombre == "{nombre}" ? "{union}"')
    # Y si el nombre no existe, no se dibuja nada: mejor un hueco que un icono equivocado.
    a('        : "";')
    a("}")
    a("")

    os.makedirs(os.path.dirname(salida), exist_ok=True)
    io.open(salida, "w", encoding="utf-8").write("\n".join(lineas))
    return len(lineas)


def main():
    ap = argparse.ArgumentParser(description="Convierte el sprite de iconos del kit a Slint.")
    ap.add_argument("--lista", action="store_true", help="solo enseña los nombres")
    ap.add_argument("--sprite", default=SPRITE, help="de donde leer")
    ap.add_argument("--salida", default=SALIDA, help="donde escribir")
    args = ap.parse_args()

    iconos = leer_sprite(args.sprite)
    if not iconos:
        sys.exit("El sprite no tenia ningun icono legible.")

    if args.lista:
        print(f"  {len(iconos)} iconos:")
        for nombre, _, _, _ in iconos:
            print(f"    {nombre}")
        return 0

    n = generar(iconos, args.salida)
    trazados = sum(len(t) for _, t, _, _ in iconos)
    print(f"  iconos convertidos: {len(iconos)}")
    print(f"  trazados en total:  {trazados}")
    print(f"  escrito:            {os.path.relpath(args.salida)} ({n} lineas)")
    print()
    print("  LICENCIA: Lucide, ISC. El aviso va dentro del fichero generado, y hay que conservarlo.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
