#!/usr/bin/env python3
"""Mide el CONTRASTE de los dos temas, segun WCAG.

POR QUE EXISTE: un tema no se elige por bonito. El texto tiene que poder leerse, y eso es una
cuenta, no una opinion: la relacion de contraste entre el texto y su fondo.

  minimo 4.5:1  para texto normal   (WCAG AA)
  minimo 3.0:1  para texto grande, bordes e iconos

Y el estudio de Mimo dejo una leccion que se aplica aqui: **el contraste se garantiza REDEFINIENDO
TOKENS**, no retocando piezas. Por eso esto se mide sobre los valores declarados.
"""
def a_rgb(h):
    h = h.lstrip('#')
    return tuple(int(h[i:i+2], 16)/255 for i in (0, 2, 4))

def luminancia(c):
    def f(v): return v/12.92 if v <= 0.03928 else ((v+0.055)/1.055)**2.4
    r, g, b = (f(x) for x in c)
    return 0.2126*r + 0.7152*g + 0.0722*b

def contraste(a, b):
    l1, l2 = luminancia(a_rgb(a)), luminancia(a_rgb(b))
    if l1 < l2: l1, l2 = l2, l1
    return (l1 + 0.05) / (l2 + 0.05)

# ---- TEMA 1: CLARO CÁLIDO (el elegido) ----
CLARO = dict(
    nombre="Claro cálido",
    bg="#faf9f7", sup="#ffffff", fg="#2b2724", muted="#78716c", line="#6b6560",
    ac="#0f766e", ac_txt="#ffffff", ok="#15803d", warn="#b45309",
    z1="#15803d", z2="#0f766e", z3="#7c3aed", z5="#b45309",
)
# ---- TEMA 2: NEGRO OLED (apaga el pixel) ----
OSCURO = dict(
    nombre="Negro OLED",
    bg="#000000",      # NEGRO PURO: en OLED el pixel se APAGA y no gasta
    sup="#0a0a0a",     # las tarjetas, casi negro
    fg="#f2f0ed", muted="#a8a29b", line="#6b6660",
    ac="#2dd4bf", ac_txt="#000000",
    ok="#4ade80", warn="#fbbf24",
    z1="#4ade80", z2="#2dd4bf", z3="#c4b5fd", z5="#fbbf24",
)

MIN_NORMAL = 4.5
MIN_GRANDE = 3.0

def revisar(t):
    print(f"\n{'='*66}")
    print(f"  {t['nombre']}")
    print(f"{'='*66}")
    pares = [
        ("texto sobre fondo",        t['fg'],  t['bg'],  MIN_NORMAL),
        ("texto sobre tarjeta",      t['fg'],  t['sup'], MIN_NORMAL),
        ("texto apagado sobre fondo",t['muted'], t['bg'], MIN_NORMAL),
        ("texto apagado en tarjeta", t['muted'], t['sup'], MIN_NORMAL),
        ("acento sobre fondo",       t['ac'],  t['bg'],  MIN_GRANDE),
        ("texto del acento",         t['ac_txt'], t['ac'], MIN_NORMAL),
        ("borde sobre fondo (frontera)", t['line'], t['bg'],  MIN_GRANDE),
        ("correcto sobre fondo",     t['ok'],  t['bg'],  MIN_GRANDE),
        ("aviso sobre fondo",        t['warn'], t['bg'], MIN_GRANDE),
    ]
    for i in range(1, 6):
        c = t.get(f'z{i}')
        if c: pares.append((f"zona {i} sobre fondo", c, t['bg'], MIN_GRANDE))

    fallos = 0
    for nombre, a, b, minimo in pares:
        r = contraste(a, b)
        ok = r >= minimo
        if not ok: fallos += 1
        marca = "OK " if ok else "FALLA"
        print(f"  {marca} {nombre:<28} {r:5.2f}:1   (minimo {minimo})")
    print(f"\n  ==> {'TODO PASA' if fallos==0 else str(fallos) + ' FALLAN'}")
    return fallos

f1 = revisar(CLARO)
f2 = revisar(OSCURO)

print(f"\n{'='*66}")
print("  LO QUE ESTO DICE DEL TEMA OSCURO")
print(f"{'='*66}")
print("  El fondo es #000000, NEGRO PURO, y no un gris oscuro.")
print("  EN OLED ESO NO ES ESTETICA: el pixel se APAGA y no consume.")
print("  El tema negro que ya existia usaba #1e1e1e: ese pixel SIGUE ENCENDIDO.")
print()
print(f"  Y el contraste se cumple: el texto sobre negro da {contraste(OSCURO['fg'], OSCURO['bg']):.1f}:1")
