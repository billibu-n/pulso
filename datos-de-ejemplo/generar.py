#!/usr/bin/env python3
"""Datos SINTETICOS para probar el motor sin los datos del cliente.

POR QUE EXISTEN: el cliente no ha dado sus datos todavia, pero el FORMATO esta publicado.
Asi que se fabrica un caso realista y se prueba el motor contra el.

EL PERFIL, calcado del cliente real: 60 anos, ciclista competitivo, buen estado fisico.
Eso NO es casual: a esa edad la formula de la frecuencia maxima tiene un error grande, y
queremos que el motor se enfrente a SU caso, no a uno comodo.

TODO ES INVENTADO. No hay ni un dato real de ninguna persona aqui.
"""
import json

# --- El ciclista: 60 anos, entrenado, compite -------------------------------------
EDAD = 60
PESO_KG = 72.0
# Su frecuencia maxima REAL, inventada pero PLAUSIBLE para un competitivo de 60:
# mas alta que la media de su edad (que la formula estima en 166). Es justo el caso
# que hay que saber detectar.
MAX_REAL = 178
REPOSO = 48  # un ciclista entrenado tiene el pulso de reposo bajo

def sesion(tipo, minutos, pulso_medio, potencia_media, cadencia=88, series=None):
    """Una sesion con sus muestras, segundo a segundo (cada 5 s para no hacer un fichero enorme)."""
    muestras = []
    n = (minutos * 60) // 5
    for i in range(n):
        # Se le da forma: calentamiento, cuerpo, y vuelta a la calma.
        fase = i / max(n - 1, 1)
        if fase < 0.15:      # calentamiento
            f = 0.75
        elif fase > 0.85:    # vuelta a la calma
            f = 0.70
        else:                # el cuerpo de la sesion
            f = 1.0
        pulso = int(pulso_medio * f)
        pot = int(potencia_media * f)
        muestras.append({
            "segundo": i * 5,
            "pulso": pulso,
            "potencia_w": pot,
            "cadencia": cadencia,
        })
    return {
        "tipo": tipo,
        "duracion_s": minutos * 60,
        "muestras": muestras,
        "resumen": {"pulso_medio": pulso_medio, "potencia_media": potencia_media},
    }

# --- El mes de entrenamiento: cuatro semanas -------------------------------------
# Sube de carga, que es lo que hace un competitivo... y en la ultima hay un SALTO GRANDE,
# que es justo lo que el estudio dice que causa las lesiones.
semanas = [
    {"semana": 1, "sesiones": [
        sesion("rodillo suave", 45, 118, 150),
        sesion("fondo", 90, 132, 185),
        sesion("series", 60, 152, 240),
    ]},
    {"semana": 2, "sesiones": [
        sesion("rodillo suave", 50, 120, 155),
        sesion("fondo", 100, 135, 190),
        sesion("series", 65, 155, 250),
    ]},
    {"semana": 3, "sesiones": [
        sesion("rodillo suave", 45, 118, 150),
        sesion("fondo", 110, 138, 195),
        sesion("series", 70, 158, 260),
    ]},
    {"semana": 4, "sesiones": [
        sesion("rodillo suave", 40, 115, 145),
        # EL SALTO: una salida mucho mas larga que la mas larga del mes (110 min -> 210 min).
        # Es el caso que el motor TIENE que detectar.
        sesion("salida larga", 210, 128, 180),
    ]},
]

# --- El sueno: tres semanas buenas y una mala ------------------------------------
# Se cruza con la carga: mas carga Y menos sueno es la combinacion que hay que saber ver.
sueno = []
horas_por_semana = [7.8, 7.6, 7.9, 6.2]  # la ultima semana duerme mal
for idx, sem in enumerate(semanas):
    for dia in range(7):
        sueno.append({
            "semana": sem["semana"],
            "dia": dia + 1,
            "horas": round(horas_por_semana[idx] + (0.3 if dia % 2 else -0.2), 1),
            "hrv_ms": int(62 - idx * 2 + (3 if dia % 3 else -4)),
        })

perfil = {
    "nota": "DATOS INVENTADOS, solo para probar el motor. Ningun dato real de nadie.",
    "persona": {
        "edad": EDAD,
        "peso_kg": PESO_KG,
        "pulso_reposo": REPOSO,
        "pulso_maximo_medido": None,   # NO hay prueba de esfuerzo: es su caso real
        "pulso_maximo_real_inventado": MAX_REAL,  # para saber cuanto se equivoca la formula
        "deporte": "ciclismo competitivo",
    },
    "semanas": semanas,
    "sueno": sueno,
}

with open("caso-60-ciclismo.json", "w", encoding="utf-8") as f:
    json.dump(perfil, f, ensure_ascii=False, indent=1)

print("Generado: caso-60-ciclismo.json")
print()
print("  EL PERFIL (inventado, calcado del cliente):")
print("    edad ................. %d anos" % EDAD)
print("    peso ................. %.0f kg" % PESO_KG)
print("    pulso en reposo ...... %d ppm" % REPOSO)
print("    pulso maximo REAL .... %d ppm  (inventado, plausible para un competitivo)" % MAX_REAL)
print("    pulso maximo por FORMULA (Tanaka) ... %d ppm" % round(208 - 0.7 * EDAD))
print("    ==> la formula se queda CORTA por %d pulsaciones" % (MAX_REAL - round(208 - 0.7 * EDAD)))
print()
print("  LAS CUATRO SEMANAS:")
for s in semanas:
    mins = sum(x["duracion_s"] for x in s["sesiones"]) // 60
    print("    semana %d: %d sesiones, %d minutos" % (s["semana"], len(s["sesiones"]), mins))
print()
print("  EL SALTO DE LA SEMANA 4:")
larga = max(x["duracion_s"] for x in semanas[0]["sesiones"] + semanas[1]["sesiones"] + semanas[2]["sesiones"])
salto = max(x["duracion_s"] for x in semanas[3]["sesiones"])
print("    sesion mas larga de las semanas 1-3: %d minutos" % (larga // 60))
print("    sesion de la semana 4 ............. %d minutos" % (salto // 60))
print("    ==> salto de %.0f %%" % ((salto / larga - 1) * 100))
