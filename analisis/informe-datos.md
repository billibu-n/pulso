# Qué hay en la cuenta de Garmin
**Generado** por `analisis/explorar.py`. Se puede repetir.
**No contiene datos crudos de salud**: solo estructura, recuentos y rangos.

## Las actividades
- **21 actividades**, de **2026-09-13** a **2026-10-11**

| deporte | cuántas | minutos (media) | minutos (máx) | pulso medio | pulso máx visto |
|---|---:|---:|---:|---:|---:|
| indoor_cycling | 18 | 77 | 210 | 125 | 179 |
| mountain_biking | 3 | 250 | 276 | 113 | 161 |

## Los sensores que traen sus actividades

| sensor | lecturas | en cuántas actividades |
|---|---:|---:|
| `temperature` | 112,042 | 21 |
| `distance` | 112,042 | 21 |
| `heart_rate` | 106,305 | 20 |
| `enhanced_respiration_rate` | 105,742 | 20 |
| `accumulated_power` | 83,402 | 18 |
| `power` | 83,381 | 18 |
| `fractional_cadence` | 83,332 | 18 |
| `cadence` | 83,332 | 18 |
| `right_pco` | 83,281 | 18 |
| `left_pco` | 83,281 | 18 |
| `left_right_balance` | 83,247 | 18 |
| `speed` | 40,071 | 6 |
| `position_long` | 40,071 | 6 |
| `position_lat` | 40,071 | 6 |
| `enhanced_speed` | 40,071 | 6 |

## Lo que Garmin YA calcula (y viene en los datos)
- **efecto aeróbico**: en **21 de 21** actividades
- **efecto anaeróbico**: en **21 de 21** actividades
- **carga de entrenamiento**: en **21 de 21** actividades
- **VO2 máximo**: **9** mediciones, entre **54.3** y **56.8** ml/kg/min
- **récords personales**: **12**

## Lo que NO hay (el hueco, y es importante)

| dato | filas |
|---|---:|
| `sleep` | 0 |
| `sleep_level` | 0 |
| `hrv` | 0 |
| `spo2` | 0 |
| `body_composition` | 0 |
| `heart_rate` | 0 |
| `stress` | 0 |
| `body_battery` | 0 |
| `training_readiness` | 0 |
| `intensity_minutes` | 0 |

**Consecuencia**: de este cliente hay **actividades**, y **no** hay sueño, ni pulsocontinuo, ni estrés. El aparato que usa **mide el ejercicio, no el día entero**.

## Dónde está la información

| tabla | filas |
|---|---:|
| `activity_ts_metric` | 1,294,181 |
| `activity_lap_metric` | 10,174 |
| `activity_event` | 2,479 |
| `supplemental_activity_metric` | 755 |
| `training_load` | 26 |
| `activity` | 21 |
| `cycling_agg_metrics` | 21 |
| `activity_hrv` | 20 |
| `acclimation` | 18 |
| `personal_record` | 12 |
| `vo2_max` | 9 |
| `activity_path` | 6 |

## Los ficheros originales

Se guardan, y son **la copia de seguridad** además de los datos crudos:

- `ingest/`: **0** ficheros
- `process/`: **0** ficheros
- `storage/`: **256** ficheros
- `quarantine/`: **0** ficheros

- **21 ficheros FIT** — los datos crudos, segundo a segundo
- **235 ficheros JSON** — los resúmenes de Garmin

## La comprobación que más importa

- Pulso máximo **visto en sus actividades**: **179** ppm
- Pulso máximo **estimado por edad** (60 años, Tanaka): **166** ppm
- **Diferencia: 13 pulsaciones**, y el error va **hacia abajo**

Es decir: **sus zonas de entrenamiento salen BAJAS**, y por tanto **entrena más fuertede lo que sus propias zonas le dicen**.
