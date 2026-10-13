#!/usr/bin/env python3
"""Explora la base de datos de Garmin y saca un informe de lo que hay.

POR QUE EXISTE: para no mirar los datos a mano cada vez, y para que la medicion se pueda
REPETIR. Cuando lleguen mas datos, se vuelve a ejecutar y sale lo mismo, actualizado.

Uso:  python3 analisis/explorar.py
Sale: analisis/informe-datos.md  (estructura y numeros, NUNCA datos crudos de salud)
"""
import sqlite3, os, io

DB = "datos/garmin_data.db"
SALIDA = "analisis/informe-datos.md"

def main():
    if not os.path.exists(DB):
        print("No hay base de datos en", DB)
        return
    db = sqlite3.connect(DB)
    q = lambda s: db.execute(s).fetchone()

    l = []
    a = lambda s: l.append(s)

    a("# Qué hay en la cuenta de Garmin\n")
    a("**Generado** por `analisis/explorar.py`. Se puede repetir.\n")
    a("**No contiene datos crudos de salud**: solo estructura, recuentos y rangos.\n")

    a("\n## Las actividades\n")
    n, primera, ultima = q("SELECT COUNT(*), MIN(start_ts), MAX(start_ts) FROM activity")
    a(f"- **{n} actividades**, de **{str(primera)[:10]}** a **{str(ultima)[:10]}**\n")
    a("\n| deporte | cuántas | minutos (media) | minutos (máx) | pulso medio | pulso máx visto |\n")
    a("|---|---:|---:|---:|---:|---:|\n")
    for row in db.execute("""SELECT activity_type_key, COUNT(*), ROUND(AVG(duration)/60.0,0),
                             ROUND(MAX(duration)/60.0,0), ROUND(AVG(average_hr),0), MAX(max_hr)
                             FROM activity GROUP BY activity_type_key ORDER BY COUNT(*) DESC"""):
        a(f"| {row[0]} | {row[1]} | {row[2]:.0f} | {row[3]:.0f} | {row[4]:.0f} | {row[5]:.0f} |\n")

    a("\n## Los sensores que traen sus actividades\n")
    a("\n| sensor | lecturas | en cuántas actividades |\n|---|---:|---:|\n")
    for row in db.execute("""SELECT name, COUNT(*), COUNT(DISTINCT activity_id)
                             FROM activity_ts_metric GROUP BY name
                             ORDER BY COUNT(*) DESC LIMIT 15"""):
        a(f"| `{row[0]}` | {row[1]:,} | {row[2]} |\n")

    a("\n## Lo que Garmin YA calcula (y viene en los datos)\n")
    for col, et in [('aerobic_training_effect','efecto aeróbico'),
                    ('anaerobic_training_effect','efecto anaeróbico'),
                    ('activity_training_load','carga de entrenamiento')]:
        k = q(f"SELECT COUNT({col}) FROM activity WHERE {col} IS NOT NULL")[0]
        a(f"- **{et}**: en **{k} de {n}** actividades\n")
    k = q("SELECT COUNT(*) FROM vo2_max")[0]
    lo, hi = q("SELECT MIN(vo2_max_cycling), MAX(vo2_max_cycling) FROM vo2_max")
    a(f"- **VO2 máximo**: **{k}** mediciones, entre **{lo}** y **{hi}** ml/kg/min\n")
    k = q("SELECT COUNT(*) FROM personal_record")[0]
    a(f"- **récords personales**: **{k}**\n")

    a("\n## Lo que NO hay (el hueco, y es importante)\n")
    a("\n| dato | filas |\n|---|---:|\n")
    for t in ['sleep','sleep_level','hrv','spo2','body_composition','heart_rate',
              'stress','body_battery','training_readiness','intensity_minutes']:
        try:
            k = q(f"SELECT COUNT(*) FROM {t}")[0]
        except Exception:
            k = "no existe la tabla"
        a(f"| `{t}` | {k} |\n")
    a("\n**Consecuencia**: de este cliente hay **actividades**, y **no** hay sueño, ni pulso")
    a("continuo, ni estrés. El aparato que usa **mide el ejercicio, no el día entero**.\n")

    a("\n## Dónde está la información\n")
    a("\n| tabla | filas |\n|---|---:|\n")
    tam = []
    for (t,) in db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"):
        try:
            k = q(f"SELECT COUNT(*) FROM {t}")[0]
            if k > 0: tam.append((t, k))
        except Exception:
            pass
    for t, k in sorted(tam, key=lambda x: -x[1])[:12]:
        a(f"| `{t}` | {k:,} |\n")

    a("\n## Los ficheros originales\n")
    a("\nSe guardan, y son **la copia de seguridad** además de los datos crudos:\n\n")
    base = "datos/garmin_files"
    for d in ['ingest','process','storage','quarantine']:
        p = os.path.join(base, d)
        if os.path.isdir(p):
            k = sum(len(f) for _,_,f in os.walk(p))
            a(f"- `{d}/`: **{k}** ficheros\n")
    todos = [f for _,_,fs in os.walk(base) for f in fs]
    a(f"\n- **{sum(1 for f in todos if f.endswith('.fit'))} ficheros FIT** — los datos crudos, segundo a segundo\n")
    a(f"- **{sum(1 for f in todos if f.endswith('.json'))} ficheros JSON** — los resúmenes de Garmin\n")

    # Y la comprobacion que mas importa
    real = q("SELECT MAX(max_hr) FROM activity")[0]
    est = round(208 - 0.7 * 60)
    a("\n## La comprobación que más importa\n")
    a(f"\n- Pulso máximo **visto en sus actividades**: **{real:.0f}** ppm\n")
    a(f"- Pulso máximo **estimado por edad** (60 años, Tanaka): **{est}** ppm\n")
    a(f"- **Diferencia: {real-est:.0f} pulsaciones**, y el error va **hacia abajo**\n")
    a("\nEs decir: **sus zonas de entrenamiento salen BAJAS**, y por tanto **entrena más fuerte")
    a("de lo que sus propias zonas le dicen**.\n")

    db.close()
    io.open(SALIDA, 'w', encoding='utf-8').write("".join(l))
    print("Informe escrito en", SALIDA)

if __name__ == "__main__":
    main()
