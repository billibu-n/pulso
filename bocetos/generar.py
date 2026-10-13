#!/usr/bin/env python3
"""Bocetos de color, sobre el armazon YA APROBADO. No se inventa la distribucion.

EL ARMAZON NO SE TOCA, y es una decision: ya existe, esta medido y paso una revision de contraste.
Lo que se elige aqui es EL COLOR. Cambiar la distribucion seria repetir un estudio que ya esta hecho.

MEDIDAS REALES, sacadas del prototipo aprobado:
  riel izquierdo ........ 176 px
  panel derecho ......... 320 px
  barra inferior ........ una franja
  el contenido .......... el resto, y es el UNICO que crece
"""
from PIL import Image, ImageDraw, ImageFont
import os

W, H = 1440, 900
RIEL, PANEL, BARRA = 176, 320, 34
CABECERA = 56

PALETAS = {
 "1-clasico": dict(
   nombre="Clásico (el actual)",
   nota="Es el que ya usa el prototipo, y ya pasó la revisión de contraste.",
   bg="#f8fafc", sup="#ffffff", fg="#1e293b", muted="#64748b", line="#cbd5e1",
   ac="#2563eb", ac_txt="#ffffff", ok="#16a34a", warn="#b45309",
   pista="#e2e8f0", info="#eff6ff", panel="#ffffff", riel="#ffffff",
   z1="#16a34a", z2="#0891b2", z3="#7c3aed", z5="#b91c1c"),
 "2-oscuro": dict(
   nombre="Oscuro con acento vivo",
   nota="Fondo oscuro, acento azul claro. Es lo que usan Linear, Vercel y Supabase.",
   bg="#0f1115", sup="#171a21", fg="#e6e9ef", muted="#8b93a7", line="#262b36",
   ac="#4c8dff", ac_txt="#0f1115", ok="#3fb950", warn="#d29922",
   pista="#1e232d", info="#151b26", panel="#171a21", riel="#14171d",
   z1="#3fb950", z2="#2dd4bf", z3="#a78bfa", z5="#f87171"),
 "3-minimo": dict(
   nombre="Mínimo en gris",
   nota="Casi todo gris y un solo color. Es la estética de Vercel y Clerk.",
   bg="#ffffff", sup="#ffffff", fg="#111111", muted="#737373", line="#e5e5e5",
   ac="#111111", ac_txt="#ffffff", ok="#16a34a", warn="#a16207",
   pista="#f5f5f5", info="#fafafa", panel="#fafafa", riel="#fafafa",
   z1="#16a34a", z2="#0e7490", z3="#6d28d9", z5="#b91c1c"),
 "4-calido": dict(
   nombre="Claro cálido",
   nota="Fondo crema con acento verde. Es lo que usan Cal y Polar.",
   bg="#faf9f7", sup="#ffffff", fg="#2b2724", muted="#78716c", line="#e7e2db",
   ac="#0f766e", ac_txt="#ffffff", ok="#15803d", warn="#b45309",
   pista="#f0ece6", info="#f0fdfa", panel="#ffffff", riel="#f5f2ee",
   z1="#15803d", z2="#0f766e", z3="#7c3aed", z5="#b45309"),
}

def fuente(t, n):
    """Fuentes con glifos latinos COMPLETOS.

    POR QUE IMPORTA: la primera version usaba una fuente que no tenia las tildes y los guiones
    largos, y el texto salia como «Ultimas» y «recuperacion». **Un boceto que ensena mal el texto
    no sirve para elegir**: se juzga tambien como se lee.
    """
    candidatos = [
        "/usr/share/fonts/liberation-sans-fonts/LiberationSans-%s.ttf" % ("Bold" if t == "b" else "Regular"),
        "/usr/share/fonts/abattis-cantarell-fonts/Cantarell-%s.otf" % ("Bold" if t == "b" else "Regular"),
        "/usr/share/fonts/redhat/RedHatDisplay-%s.otf" % ("Bold" if t == "b" else "Regular"),
    ]
    for r in candidatos:
        if os.path.exists(r):
            return ImageFont.truetype(r, n)
    return ImageFont.load_default()

def caja(d, x, y, w, h, relleno, borde=None, r=8):
    d.rounded_rectangle([x, y, x+w, y+h], radius=r, fill=relleno, outline=borde, width=1)

def texto(d, x, y, t, f, color, anchor="la"):
    d.text((x, y), t, font=f, fill=color, anchor=anchor)

def boceto(p):
    im = Image.new("RGB", (W, H), p["bg"]); d = ImageDraw.Draw(im)
    f_t = fuente("b", 15); f_s = fuente("n", 12); f_xs = fuente("n", 11)
    f_num = fuente("b", 30)

    # ---- barra superior ----
    d.rectangle([0, 0, W, CABECERA], fill=p["sup"])
    d.line([0, CABECERA, W, CABECERA], fill=p["line"], width=1)
    caja(d, 16, 16, 24, 24, p["ac"], r=6)
    texto(d, 50, CABECERA//2, "PULSO", f_t, p["fg"], "lm")
    texto(d, W-24, CABECERA//2, "Entrenamiento", f_xs, p["muted"], "rm")

    # ---- riel izquierdo: 176 px ----
    d.rectangle([0, CABECERA, RIEL, H], fill=p["riel"])
    d.line([RIEL, CABECERA, RIEL, H], fill=p["line"], width=1)
    secciones = [("Resumen", True), ("Zonas", False), ("Sesiones", False), ("Datos", False)]
    y = CABECERA + 20
    for nombre, activo in secciones:
        if activo:
            caja(d, 10, y-6, RIEL-20, 32, p["ac"], r=7)
            texto(d, 34, y+10, nombre, f_s, p["ac_txt"], "lm")
        else:
            texto(d, 34, y+10, nombre, f_s, p["muted"], "lm")
        caja(d, 18, y+4, 12, 12, p["muted"] if not activo else p["ac_txt"], r=3)
        y += 40
    # el espacio, en el pie del riel
    texto(d, 16, H-58, "ESPACIO", f_xs, p["muted"])
    caja(d, 10, H-42, RIEL-20, 30, p["pista"], r=7)
    texto(d, 24, H-27, "Ciclismo", f_s, p["fg"], "lm")

    # ---- contenido: el que crece ----
    X0 = RIEL + 32; X1 = W - PANEL - 32
    texto(d, X0, CABECERA+24, "Últimas cuatro semanas", f_t, p["fg"])

    # las tarjetas de cifras
    cw = (X1 - X0 - 3*14) // 4; y = CABECERA + 62
    cifras = [("183", "pulso máximo"), ("169", "umbral"), ("1,02", "vatios por pulso"), ("13,8", "deriva %")]
    for i, (n, e) in enumerate(cifras):
        x = X0 + i*(cw+14)
        caja(d, x, y, cw, 84, p["sup"], p["line"])
        texto(d, x+14, y+14, n, f_num, p["ac"] if i in (0,1) else p["fg"])
        texto(d, x+14, y+56, e, f_xs, p["muted"])

    # el bloque grande: las zonas
    y2 = y + 104
    caja(d, X0, y2, X1-X0, 250, p["sup"], p["line"])
    texto(d, X0+18, y2+16, "Zonas de entrenamiento", f_s, p["fg"])
    zonas = [("recuperación","hasta 137",0.62),("resistencia","137 – 152",0.72),
             ("tempo","152 – 159",0.80),("umbral","159 – 169",0.88),("consumo máximo","169 – 203",1.0)]
    zy = y2 + 48
    for nombre, rango, prop in zonas:
        texto(d, X0+18, zy, nombre, f_xs, p["muted"])
        ancho = int((X1-X0-220) * prop)
        caja(d, X0+130, zy-4, X1-X0-160-90, 16, p["pista"], r=4)
        # LA ESCALA DE INTENSIDAD, y tiene sentido: de suave a fuerte. No es decoracion --
        # cada zona se distingue de la de al lado, y el orden se ve sin leer.
        escala = {"recuperación": p.get("z1", p["ok"]), "resistencia": p.get("z2", p["muted"]),
                  "tempo": p.get("z3", p["muted"]), "umbral": p["ac"], "consumo máximo": p.get("z5", p["warn"])}
        col = escala[nombre]
        caja(d, X0+130, zy-4, int(ancho*0.8), 16, col, r=4)
        texto(d, X1-18, zy+4, rango, f_xs, p["fg"], "rm")
        zy += 34

    # la franja de puentes, y una nota de vacio en UNA linea
    y3 = y2 + 266
    caja(d, X0, y3, X1-X0, 46, p["info"], r=8)
    texto(d, X0+16, y3+23, "El bloque académico aporta la banda de la semana.", f_xs, p["fg"], "lm")
    y4 = y3 + 62
    caja(d, X0, y4, X1-X0, 60, p["sup"], p["line"])
    texto(d, X0+18, y4+30, "Sin sesiones todavía", f_s, p["muted"], "lm")
    texto(d, X0+200, y4+30, "Conecta el aparato y aparecerán aquí.", f_xs, p["muted"], "lm")

    # ---- panel derecho: 320 px ----
    XP = W - PANEL
    d.rectangle([XP, CABECERA, W, H], fill=p["panel"])
    d.line([XP, CABECERA, XP, H], fill=p["line"], width=1)
    texto(d, XP+20, CABECERA+24, "Este mes", f_s, p["fg"])
    py = CABECERA + 58
    for k, v in [("Sesiones","21"),("Horas","29"),("Zonas alcanzadas","4")]:
        texto(d, XP+20, py, k, f_xs, p["muted"])
        texto(d, W-20, py, v, f_s, p["fg"], "rm")
        py += 30
    d.line([XP+20, py+6, W-20, py+6], fill=p["line"], width=1)
    texto(d, XP+20, py+22, "Precisión", f_xs, p["muted"])
    caja(d, XP+20, py+42, 90, 22, p["ok"], r=11)
    texto(d, XP+65, py+53, "alta", f_xs, p["ac_txt"], "mm")
    texto(d, XP+20, py+80, "El umbral está medido,", f_xs, p["muted"])
    texto(d, XP+20, py+96, "no estimado.", f_xs, p["muted"])

    # ---- barra inferior ----
    d.rectangle([0, H-BARRA, W, H], fill=p["sup"])
    d.line([0, H-BARRA, W, H-BARRA], fill=p["line"], width=1)
    texto(d, 20, H-BARRA//2, "2 puentes activos", f_xs, p["muted"], "lm")
    texto(d, W-20, H-BARRA//2, "última lectura: hoy", f_xs, p["muted"], "rm")

    # ---- el rotulo del boceto ----
    texto(d, 16, H-14, p["nombre"], f_xs, p["muted"])
    return im

for clave, p in PALETAS.items():
    im = boceto(p)
    im.save(f"boceto-{clave}.png")
    print(f"  boceto-{clave}.png  —  {p['nombre']}")
print()
print("  el armazon NO cambia entre bocetos: solo el color.")
