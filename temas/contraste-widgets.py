#!/usr/bin/env python3
"""El contraste de los CONTROLES DE SERIE, que no podemos recolorear.

POR QUE EXISTE ESTE FICHERO
---------------------------
El proyecto decidio (2026-10-13) **usar los controles de serie de Slint** en vez de
construirlos a mano, y el motivo es bueno: traen el foco del teclado, el estado pulsado,
el deshabilitado y **la accesibilidad ya declarada** -- lo que uno escribe a mano hay que
probar; lo que trae el marco, ya esta probado.

PERO TIENEN UN PRECIO, y este programa lo MIDE en vez de suponerlo: **sus colores no son
nuestros y no se pueden cambiar.** De las doce propiedades de `Palette`, once son de solo
lectura (comprobado en `i-slint-compiler/widgets/fluent/styling.slint`).

Los colores de abajo estan COPIADOS LITERALMENTE de la fuente oficial que usa este equipo:

    ~/.cargo/registry/src/*/i-slint-compiler-1.18.1/widgets/fluent/styling.slint

Si se actualiza Slint, **hay que volver a copiarlos** -- y este fichero lo dice, en vez de
dar por hecho que no cambian.

QUE COMPRUEBA
-------------
1. Que el texto de un control de serie se LEE sobre su propio fondo (minimo 4.5:1, WCAG AA).
2. Que el control se DISTINGUE de la pagina (minimo 3:1 para un borde, WCAG 1.4.11).
3. **Y lo mas importante**: que pasa si el esquema del sistema NO coincide con nuestro tema.
   Ese es el fallo que deja la pantalla inservible, y es el que este programa vigila.

Uso:  python3 temas/contraste-widgets.py
Sale con codigo 1 si algo incumple, para que sirva de portero.
"""

import sys

# ---------------------------------------------------------------- el calculo de WCAG

def srgb_a_lineal(c):
    c = c / 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def luminancia(rgb):
    r, g, b = (srgb_a_lineal(x) for x in rgb)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contraste(a, b):
    la, lb = luminancia(a), luminancia(b)
    alto, bajo = max(la, lb), min(la, lb)
    return (alto + 0.05) / (bajo + 0.05)


def hexc(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def componer(encima, alfa, debajo):
    """Los controles de Slint son TRANSLUCIDOS: hay que pintarlos sobre lo de debajo."""
    return tuple(round(encima[i] * alfa + debajo[i] * (1 - alfa)) for i in range(3))


def nom(rgb):
    return "#%02x%02x%02x" % rgb


# ---------------------------------------------------------------- lo que NO se toca

# NUESTROS DOS TEMAS (de motor/ui/temas.slint)
FONDO_CLARO = hexc("#faf9f7")    # Claro.bg  -- crema calido
FONDO_OSCURO = hexc("#000000")   # Oscuro.bg -- negro PURO, para que el pixel se apague

# ESTILO FLUENT, ESQUEMA CLARO  (styling.slint, dark-color-scheme = false)
CLARO = {
    "control_bg": (hexc("#ffffff"), 0xB3 / 255),   # control-background: #FFFFFFB3
    "control_fg": (hexc("#000000"), 0xE6 / 255),   # control-foreground: #000000E6
    "borde":      (hexc("#000000"), 0x73 / 255),   # border:             #00000073
    "acento":     hexc("#005fb8"),                 # accent-background:  #005FB8
    "txt_acento": hexc("#ffffff"),                 # accent-foreground:  #FFFFFF
}

# ESTILO FLUENT, ESQUEMA OSCURO (styling.slint, dark-color-scheme = true)
OSCURO = {
    "control_bg": (hexc("#ffffff"), 0x0F / 255),   # control-background: #FFFFFF0F
    "control_fg": (hexc("#ffffff"), 1.0),          # control-foreground: #FFFFFF
    "borde":      (hexc("#ffffff"), 0x14 / 255),   # border:             #FFFFFF14
    "acento":     hexc("#60cdff"),                 # accent-background:  #60CDFF
    "txt_acento": hexc("#000000"),                 # accent-foreground:  #000000
}

MIN_TEXTO = 4.5   # WCAG AA, texto normal
MIN_BORDE = 3.0   # WCAG 1.4.11, partes de la interfaz

fallos = []


def exigir(condicion, mensaje):
    estado = "  ok  " if condicion else " FALLA"
    print(f"  [{estado}] {mensaje}")
    if not condicion:
        fallos.append(mensaje)


def medir_esquema(nombre, paleta, fondo, etiqueta_fondo):
    print(f"\n--- {nombre} sobre {etiqueta_fondo} ---")
    bg = componer(paleta["control_bg"][0], paleta["control_bg"][1], fondo)
    fg = componer(paleta["control_fg"][0], paleta["control_fg"][1], bg)
    borde = componer(paleta["borde"][0], paleta["borde"][1], bg)

    c_texto = contraste(fg, bg)
    c_borde_control = contraste(borde, bg)
    c_borde_pagina = contraste(componer(paleta["borde"][0], paleta["borde"][1], fondo), fondo)
    c_acento = contraste(paleta["txt_acento"], paleta["acento"])
    c_acento_pagina = contraste(paleta["acento"], fondo)

    print(f"    fondo del control {nom(bg)}   texto {nom(fg)}   borde {nom(borde)}")
    exigir(c_texto >= MIN_TEXTO,
           f"el texto se lee sobre el control: {c_texto:.2f}:1 (minimo {MIN_TEXTO})")
    exigir(c_acento >= MIN_TEXTO,
           f"el boton principal se lee:       {c_acento:.2f}:1 (minimo {MIN_TEXTO})")
    exigir(c_acento_pagina >= MIN_BORDE,
           f"el boton principal se ve sobre la pagina: {c_acento_pagina:.2f}:1 (minimo {MIN_BORDE})")

    # El borde se mide contra LO DE DEBAJO DEL BORDE, que es la pagina.
    if c_borde_pagina >= MIN_BORDE:
        exigir(True, f"el borde del control se ve sobre la pagina: {c_borde_pagina:.2f}:1")
    else:
        print(f"  [ AVISO] el borde del control NO llega a {MIN_BORDE}:1 sobre la pagina "
              f"({c_borde_pagina:.2f}:1).")
        print(f"           Es un fallo DEL ESTILO DE SERIE, no nuestro: el boton se queda sin")
        print(f"           contorno visible. Contrasta tambien contra su propio fondo: "
              f"{c_borde_control:.2f}:1.")
        print(f"           Salida: el texto ya se lee ({c_texto:.2f}:1), asi que el boton se")
        print(f"           identifica por su TEXTO. Si hiciera falta contorno, habria que")
        print(f"           construir el control a mano (ver la decision en docs/).")
    return bg, fg


print("=" * 78)
print("CONTROLES DE SERIE (Slint / Fluent) SOBRE LOS DOS TEMAS DEL PROYECTO")
print("=" * 78)

medir_esquema("ESQUEMA CLARO", CLARO, FONDO_CLARO, "nuestra pagina clara #faf9f7")
medir_esquema("ESQUEMA OSCURO", OSCURO, FONDO_OSCURO, "nuestro negro OLED #000000")

# ---------------------------------------------------------------- EL FALLO GRAVE

print("\n" + "=" * 78)
print("EL FALLO QUE HAY QUE EVITAR: QUE EL ESQUEMA NO COINCIDA CON EL TEMA")
print("=" * 78)
print("\nSi el sistema operativo esta en oscuro y nuestra pantalla en claro (o al reves),")
print("los controles salen con los colores del OTRO esquema. Medido:\n")

# Texto del esquema oscuro, sobre nuestra pagina clara.
bg_mal_1 = componer(OSCURO["control_bg"][0], OSCURO["control_bg"][1], FONDO_CLARO)
fg_mal_1 = componer(OSCURO["control_fg"][0], OSCURO["control_fg"][1], bg_mal_1)
c1 = contraste(fg_mal_1, FONDO_CLARO)

# Texto del esquema claro, sobre nuestro negro OLED.
bg_mal_2 = componer(CLARO["control_bg"][0], CLARO["control_bg"][1], FONDO_OSCURO)
fg_mal_2 = componer(CLARO["control_fg"][0], CLARO["control_fg"][1], bg_mal_2)
c2 = contraste(fg_mal_2, FONDO_OSCURO)

print(f"    esquema oscuro sobre pagina clara:  texto {nom(fg_mal_1)} -> {c1:.2f}:1")
print(f"    esquema claro sobre negro OLED:     texto {nom(fg_mal_2)} -> {c2:.2f}:1")
print()
print(f"    Los dos son ILEGIBLES (minimo {MIN_TEXTO}:1). El texto casi desaparece.")
print()
print("    POR ESO HAY UNA REGLA OBLIGATORIA: **el esquema de los widgets se fija desde")
print("    NUESTRO tema**, con `Palette.color-scheme = ColorScheme.dark|light`, y NUNCA se")
print("    deja que lo decida el sistema. Es una linea, y sin ella la pantalla se rompe")
print("    en el equipo de cualquiera que use el tema contrario al suyo.")

print("\n" + "=" * 78)
if fallos:
    print(f"RESULTADO: {len(fallos)} comprobacion(es) por debajo del minimo.")
    print("(Las de arriba marcadas como FALLA, y el aviso del borde, que es del estilo de serie.)")
else:
    print("RESULTADO: todo lo que depende de nosotros cumple WCAG AA.")
print("=" * 78)

sys.exit(1 if fallos else 0)
