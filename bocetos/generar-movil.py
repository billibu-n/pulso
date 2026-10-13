#!/usr/bin/env python3
"""El boceto del MOVIL en el tema calido que se eligio.

POR QUE APARTE DEL DE ESCRITORIO, y no es un capricho:
  - el telefono del cliente mide 411 dp de ancho, y el escritorio 1440: CUATRO veces menos;
  - EN EL MOVIL NO HAY RIEL QUE REPARTIR. El riel es de escritorio. Aqui la navegacion es CROMO
    (arriba y abajo), y el contenido ocupa TODO el ancho.
  - y las areas tocables tienen que ser de 48 dp como minimo, no de 32: el dedo no es un raton.

Esto esta en la norma del proyecto (reglas/09) y en la universal (AU-01, AU-02, AU-03).
"""
from PIL import Image, ImageDraw, ImageFont
import os

# ---- El telefono, medido: Pixel 10, 1080x2424 px a 420 ppp = 411 x 923 dp ----
DP_A_PX = 2.0                      # se dibuja a 2 px por dp, para que se vea nitido
W_DP, H_DP = 411, 880
W, H = int(W_DP*DP_A_PX), int(H_DP*DP_A_PX)

# ---- LOS DOS TEMAS. El oscuro usa NEGRO PURO: en OLED el pixel se APAGA y no gasta. ----
TEMAS = {
  "calido": dict(
    bg="#faf9f7", sup="#ffffff", fg="#2b2724", muted="#78716c", line="#6b6560",
    ac="#0f766e", ac_txt="#ffffff", ok="#15803d", warn="#b45309",
    pista="#f0ece6", info="#f0fdfa", panel="#ffffff",
    z1="#15803d", z2="#0f766e", z3="#7c3aed", z5="#b45309"),
  "oscuro": dict(
    bg="#000000", sup="#0a0a0a", fg="#f2f0ed", muted="#a8a29b", line="#6b6660",
    ac="#2dd4bf", ac_txt="#000000", ok="#4ade80", warn="#fbbf24",
    pista="#1a1a1a", info="#0a1514", panel="#0a0a0a",
    z1="#4ade80", z2="#2dd4bf", z3="#c4b5fd", z5="#fbbf24"),
}
C = TEMAS["calido"]

CABECERA_DP = 56
NAV_DP = 64          # la barra de abajo, cromo
MARGEN_DP = 16

def px(dp): return int(dp*DP_A_PX)

def fuente(t, dp):
    """Fuentes con glifos latinos completos, por su RUTA REAL (la primera version fallo por esto)."""
    candidatos = [
        "/usr/share/fonts/liberation-sans-fonts/LiberationSans-%s.ttf" % ("Bold" if t=="b" else "Regular"),
        "/usr/share/fonts/abattis-cantarell-fonts/Cantarell-%s.otf" % ("Bold" if t=="b" else "Regular"),
    ]
    for r in candidatos:
        if os.path.exists(r):
            return ImageFont.truetype(r, px(dp))
    return ImageFont.load_default()

def caja(d, x, y, w, h, relleno, borde=None, r=10):
    d.rounded_rectangle([x, y, x+w, y+h], radius=px(r), fill=relleno, outline=borde, width=1)

def texto(d, x, y, t, f, color, anchor="la"):
    d.text((x, y), t, font=f, fill=color, anchor=anchor)

def movil(nombre_archivo, titulo, con_nav=True):
    im = Image.new("RGB", (W, H), C["bg"]); d = ImageDraw.Draw(im)
    f_t = fuente("b", 20); f_s = fuente("n", 14); f_xs = fuente("n", 12); f_num = fuente("b", 34)

    M = px(MARGEN_DP)
    ancho_util = W - 2*M

    # ---- CROMO DE ARRIBA ----
    d.rectangle([0, 0, W, px(CABECERA_DP)], fill=C["sup"])
    d.line([0, px(CABECERA_DP), W, px(CABECERA_DP)], fill=C["line"], width=1)
    texto(d, M, px(CABECERA_DP//2), titulo, f_t, C["fg"], "lm")

    y = px(CABECERA_DP + 20)

    # ---- LAS CIFRAS: en el movil van de DOS en DOS, no de cuatro en cuatro ----
    # Con 411 dp, cuatro tarjetas darian 88 dp cada una: no cabe el numero ni la etiqueta.
    cifras = [("183", "pulso máximo"), ("169", "umbral"),
              ("1,02", "vatios por pulso"), ("13,8", "deriva %")]
    cw = (ancho_util - px(12)) // 2
    for i, (n, e) in enumerate(cifras):
        col = i % 2; fila = i // 2
        x = M + col*(cw + px(12)); yy = y + fila*px(78)
        caja(d, x, yy, cw, px(66), C["sup"], C["line"])
        texto(d, x+px(12), yy+px(10), n, f_num, C["ac"] if i < 2 else C["fg"])
        texto(d, x+px(12), yy+px(46), e, f_xs, C["muted"])

    y += px(78*2 + 8)

    # ---- LAS ZONAS: en el movil, la barra va DEBAJO del nombre ----
    # Al lado no cabe: el nombre solo ya come media pantalla.
    zonas = [("recuperación","hasta 137",0.62,C["z1"]),("resistencia","137 – 152",0.72,C["z2"]),
             ("tempo","152 – 159",0.80,C["z3"]),("umbral","159 – 169",0.88,C["ac"]),
             ("consumo máximo","169 – 203",1.0,C["z5"])]
    alto = px(20 + len(zonas)*48 + 16)
    caja(d, M, y, ancho_util, alto, C["sup"], C["line"])
    texto(d, M+px(14), y+px(12), "Zonas de entrenamiento", f_s, C["fg"])
    zy = y + px(42)
    for nom, rango, prop, col in zonas:
        texto(d, M+px(14), zy, nom, f_xs, C["muted"])
        texto(d, W-M-px(14), zy, rango, f_xs, C["fg"], "ra")
        # la barra, debajo
        caja(d, M+px(14), zy+px(18), ancho_util-px(28), px(10), C["pista"], r=5)
        caja(d, M+px(14), zy+px(18), int((ancho_util-px(28))*prop), px(10), col, r=5)
        zy += px(48)
    y += alto + px(12)

    # ---- LA PRECISION: una sola linea, con su color ----
    caja(d, M, y, ancho_util, px(46), C["info"], r=10)
    caja(d, M+px(12), y+px(13), px(52), px(20), C["ok"], r=10)
    texto(d, M+px(38), y+px(23), "alta", f_xs, C["ac_txt"], "mm")
    texto(d, M+px(76), y+px(23), "El umbral está medido, no estimado.", f_xs, C["fg"], "lm")
    y += px(58)

    # ---- EL VACIO, EN UNA LINEA (AU-45) ----
    caja(d, M, y, ancho_util, px(52), C["sup"], C["line"])
    texto(d, M+px(14), y+px(26), "Sin sesiones todavía", f_s, C["muted"], "lm")

    # ---- CROMO DE ABAJO: la navegacion del movil ----
    if con_nav:
        yn = H - px(NAV_DP)
        d.rectangle([0, yn, W, H], fill=C["sup"])
        d.line([0, yn, W, yn], fill=C["line"], width=1)
        items = [("Resumen", True), ("Zonas", False), ("Sesiones", False), ("Datos", False)]
        ancho = W // len(items)
        for i, (nom, act) in enumerate(items):
            cx = ancho*i + ancho//2
            col = C["ac"] if act else C["muted"]
            # el icono: un cuadrado redondeado. En el boceto no hace falta dibujarlo bien.
            caja(d, cx-px(9), yn+px(12), px(18), px(18), col, r=5)
            texto(d, cx, yn+px(46), nom, f_xs, col, "mm")
            if act:
                d.rectangle([cx-px(16), yn, cx+px(16), yn+px(3)], fill=C["ac"])

    im.save(nombre_archivo)
    return nombre_archivo

# Dos pantallas: la de resumen, y la de zonas con el riel convertido en MENU
for clave in TEMAS:
    C = TEMAS[clave]
    movil(f"movil-1-resumen-{clave}.png", "Resumen", con_nav=(clave=="calido"))
    movil(f"movil-2-zonas-{clave}.png", "Zonas", con_nav=(clave=="calido"))

# Y una tercera: el menu lateral, porque en el movil el riel NO desaparece -- se abre.
im = Image.new("RGB", (W, H), C["bg"]); d = ImageDraw.Draw(im)
f_t = fuente("b", 18); f_s = fuente("n", 14); f_xs = fuente("n", 12)
M = px(MARGEN_DP)
ANCHO_MENU = int(W*0.78)
d.rectangle([0, 0, ANCHO_MENU, H], fill=C["panel"])
d.rectangle([ANCHO_MENU, 0, W, H], fill=(0,0,0))
# el fondo oscurecido, para que se vea que el menu esta POR ENCIMA
capa = Image.new("RGBA", (W-ANCHO_MENU, H), (20, 18, 16, 150))
im.paste(Image.alpha_composite(Image.new("RGBA", capa.size, (0,0,0,0)), capa).convert("RGB"), (ANCHO_MENU, 0))
d.rectangle([ANCHO_MENU, 0, ANCHO_MENU+1, H], fill=C["line"])
texto(d, M, px(34), "PULSO", f_t, C["fg"], "lm")
y = px(80)
for nom, act in [("Resumen", True), ("Zonas", False), ("Sesiones", False), ("Datos", False)]:
    if act:
        caja(d, px(10), y-px(8), ANCHO_MENU-px(20), px(40), C["ac"], r=8)
        texto(d, px(26), y+px(12), nom, f_s, C["ac_txt"], "lm")
    else:
        texto(d, px(26), y+px(12), nom, f_s, C["muted"], "lm")
    y += px(48)
texto(d, M, H-px(72), "ESPACIO", f_xs, C["muted"])
texto(d, M, H-px(52), "Ciclismo", f_s, C["fg"])
im.save(f"movil-3-menu-{list(TEMAS.keys())[list(TEMAS.values()).index(C)]}.png")
import glob
for f in sorted(glob.glob("movil-*.png")): print(f"  {f}")
print()
print(f"  dibujado a {W}x{H} px, que son {W_DP}x{H_DP} dp (un Pixel 10 real)")
