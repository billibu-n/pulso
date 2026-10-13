# El encargo del cliente, medido contra sus datos (2026-10-13)

**Objetivo que declara**: *«Optimización de la Eficiencia Metabólica y Biomecánica para el Desafío
Farellones 2026»*. Y una fecha: **29 de noviembre de 2026**.

**Escrito el 2026-10-13, el mismo día que llegó.** Existe porque el encargo llegó **redactado por otra
IA**, con sugerencias, requisitos y supuestos **mezclados**, y porque la mitad de lo que pide **se puede
comprobar contra sus ficheros**. Esto es esa comprobación.

> **La regla de este documento**: cada fila está **medida sobre sus 102 sesiones y sus 1.360 ficheros
> de Garmin**, no supuesta. Lo que no se pudo medir, lo dice.

---

## 1. La fecha, que lo cambia todo

**Hoy es el 2026-10-13. Farellones es el 2026-11-29. Quedan 47 días.**

Eso convierte el encargo en un **plan con fecha límite**, y una fecha límite cambia las prioridades: no
se puede empezar por lo bonito. **Lo que no esté listo antes del 29 de noviembre no sirve para nada**,
porque el evento pasa.

---

## 2. Auditoría: lo que pide, uno por uno, contra sus datos

### 2.1 · Lo que SÍ se puede hacer, y con qué

| lo que pide | medido | con qué dato |
|---|---|---|
| **TSS real** | sí | `Session.training_stress_score`, en **83 de 102** sesiones. Mediana **64**, máximo **222**, y **18 sesiones por encima de 100** |
| **Umbral de potencia (FTP)** | sí, y es **suyo** | `threshold_power` = **213 W** en las 83, y `ftpAutoDetected: true` → **lo detectó Garmin, no es un número inventado** |
| **Potencia** | sí | **424.159 muestras**, media 123 W, máxima **596 W** |
| **Cadencia** | sí | **421.449 muestras**, media y mediana **83 rpm** (él dice 84: confirmado) |
| **Potencia sentado y de pie** | sí | `avg_power_position`: **132,5 W sentado / 222,7 W de pie** (42 sesiones) |
| **Cadencia sentado y de pie** | sí | **83,7 rpm sentado / 77,0 rpm de pie** |
| **Respiración** | **sí, y yo la había dado por perdida** | `enhanced_avg_respiration_rate` en **97 ficheros**, media **28** respiraciones/min |
| **Carga anaeróbica y pico de carga** | sí | `total_anaerobic_training_effect` (100), `training_load_peak` (102) |
| **Estado de entrenamiento y VO2max** | sí | **142 días** con datos |
| **Fase de pedaleo de CADA pierna** | **sí, y es lo mejor que hay** | `avg_left_power_phase` / `avg_right_power_phase`: 4 ángulos por pierna, y **difieren en 80 de 83 sesiones** |
| **Masa libre de grasa 56,6 kg** | sí | De su captura de Huawei (69,45 kg × 18,6 % grasa) |
| **Agua corporal** | sí, **a mano** | Su captura: **55,3 %**. Su umbral de aviso: 54 % |

### 2.2 · Lo que NO se puede, y hay que decirlo

| lo que pide | veredicto | el motivo, medido |
|---|---|---|
| **Reparto de potencia izquierda/derecha (el 47/53)** | **NO ESTÁ EN EL FICHERO** | Ver §3. Es el punto más importante de este documento |
| **Centro de presión del pedal (PCO)** | **no sirve** | `avg_left_pco` y `avg_right_pco` valen **0,00 mm en las 83 sesiones, y nunca difieren**. Un sensor que da 0 siempre **no está midiendo**: está devolviendo su centinela |
| **FC en reposo al despertar** | **no está** | **165 ficheros de pulso, TODOS vacíos.** Ver §4 |
| **Sueño** | **no está** | **165 ficheros de sueño, TODOS vacíos**: sin horas, sin fases, sin puntuación |
| **Estrés y minutos de intensidad** | no está | 165 ficheros de cada, todos vacíos |

---

## 3. EL HALLAZGO IMPORTANTE: el 47/53 no está donde parece

**Esto costó tres intentos y merece quedar escrito, porque el error era fácil y las consecuencias
graves.**

El campo del FIT que se llama **`left_right_balance`** parece exactamente lo que el cliente ve en
Garmin Connect. **No lo es.** La fuente oficial (foro de desarrolladores de Garmin, respuesta de *Ben
FIT*, y el propio `Profile.xlsx`) dice que ese campo es una **máscara de bits**: los 7 bits bajos son el
porcentaje, y el bit alto dice de qué pierna. Y **en sus ficheros ese campo no trae un porcentaje: trae
el número de serie del sensor** (37950, 38087, 38201…).

**Y las dos trampas siguientes, que casi me llevan a acusar al cliente de un desequilibrio inventado:**

1. `avg_power_position` = **[145, 410]**. Parece «izquierda 145 W, derecha 410 W» → un **37/63**, un
   desequilibrio enorme. **Es falso.** El `fitparser` trae la definición oficial del campo:
   *«Average power by position. Data value indexes defined by `rider_position_type`»*. Y ese enum es
   **`Seated = 0, Standing = 1`**. O sea: **145 W sentado y 410 W de pie.** Un 410 W de pie en un puerto
   es normal y no dice nada de ninguna rodilla.
2. `avg_cadence_position` = **[83, 85]** tiene la misma trampa: es **cadencia sentado y de pie**.

**LO QUE SÍ ES POR PIERNA, sin ambigüedad**: `avg_left_power_phase` y `avg_right_power_phase` — **la
fase del pedaleo**, cuatro ángulos por pierna. Y **difieren en 80 de las 83 sesiones**. Ejemplo real:

```
izquierda: [357,2 · 184,2 · 185,6 · 90,0]
derecha:   [353,0 · 181,4 · 187,0 · 87,2]
```

**Consecuencia, y hay que ser claro con el cliente**: el **reparto de potencia** izquierda/derecha
**no viaja en el fichero FIT**. Lo calcula Garmin **en su servidor** y lo enseña en su aplicación; el
fichero que se descarga **no lo lleva**. Para tenerlo hay dos caminos, y **ninguno es trabajo de
programación**:

- **que él lo mire en Garmin Connect** y lo anote, o
- **pedir acceso a la API de Garmin Connect** (que es justo lo que él propone, y tiene su trámite).

**Y MIENTRAS TANTO, el módulo de la rodilla SÍ se puede hacer** — sobre la **fase de pedaleo**, que es
una medida **de mecánica** y por tanto más cercana a lo que le preocupa que un reparto de vatios.

---

## 4. El pulso en reposo: qué se puede saber de verdad

**Decisión del dueño**: *«no importa lo que diga directamente, importa los datos de la medición… 
compruébalo tú y ve de qué sirve este dato, y sepáralo si es necesario si genera mucho conflicto».*

**Medido, y hay que separar DOS cosas que se estaban mezclando:**

| | qué es | valor | ¿sirve? |
|---|---|---|---|
| **Pulso en reposo** | el que el reloj mide **mientras duerme** | **48** (medido en el proyecto) | **sí**, pero **el fichero diario de Garmin viene vacío** (165 de 165) |
| **El «71» del encargo** | una **medición suelta de Huawei**, despierto | 71 | **no es lo mismo**, y por eso el aviso del «+10 %» dispararía en falso |

**Y por qué el fichero viene vacío**: sus 165 ficheros `HEART_RATE` tienen `restingHeartRate: null`. **El
reloj no está sincronizando ese dato**, o no lo tiene activado. **Es un dato que existe en el reloj y no
llega al fichero.**

**MEDIDO, y ahora se entiende el conflicto entero** (`cargo run -q --release --bin pulso-reposo`):

```
SESIONES CON PULSO: 97
PULSO MINIMO DE CADA SESION   minimo 59 · mediana 90 · maximo 116
  sesiones donde el pulso bajo de 60: 1 de 97
PULSO MINIMO EN LOS PRIMEROS 5 MINUTOS   minimo 63 · mediana 92
LOS MAS BAJOS: 60 (23-ago), 67 (28-jun, 05-jul, 19-jul), 68, 70, 71...
```

**ESTE CLIENTE EMPIEZA A ENTRENAR CON EL PULSO YA ALTO.** La mediana del mínimo es **90 ppm**, y solo
**una** de sus 97 sesiones baja de 60. Conclusión que importa: **de un entrenamiento NO se puede sacar su
pulso en reposo.** Los días que bajan a 67-71 son sus días suaves, no su reposo.

**Y eso explica el 71 del encargo**: no es un pulso en reposo mal medido — es **su pulso más bajo
despierto**, y coincide con lo que da la báscula de Huawei. **El 48 es el de Garmin, dormido.** Los dos
son ciertos y **no son el mismo dato**.

### Los TRES datos, separados con nombres distintos

| nombre | qué es | valor | de dónde | ¿sirve? |
|---|---|---|---|---|
| **Pulso en reposo** | dormido, el bueno | **48** | del reloj, **pero su fichero diario viene vacío** (165 de 165) | **sí, cuando llegue.** Hoy **no hay serie** |
| **Pulso mínimo despierto** | el más bajo del día despierto | **~71** | medición suelta (Huawei) | sirve para **una línea base propia**, no para tablas de población |
| **Pulso mínimo en entrenamiento** | lo más bajo visto entrenando | **67-71 en días suaves, 90 de mediana** | **medible de los FIT, y es el único con serie** | **sí: comparado CONSIGO MISMO** |

> **SEPARADOS, y con tres nombres distintos.** Mezclarlos fue el origen del conflicto: el encargo decía
> 71 y el proyecto decía 48, y **los dos tenían razón de algo distinto**.
>
> **Y la consecuencia para la regla del cliente**: su *«si el pulso en reposo sube un 10 %»* **no se
> puede aplicar hoy**, porque **no hay serie de pulso en reposo**. Lo que sí se puede es **comparar cada
> sesión con su propia línea base** — y eso, además, es lo que la norma del proyecto ya exige: el RMSSD
> de este cliente **no se puede juzgar contra tablas de reposo porque está medido entrenando**.

### Y el dato que SÍ tiene serie y dice lo mismo: la DERIVA

Ya medido en el proyecto, sobre sus 102 sesiones: **deriva media 9,7 %, mediana 10,8 %, y 43 sesiones por
encima del 10 %** (el 42 % de sus entrenamientos).

**La deriva cardíaca es el indicador clásico de deshidratación, calor o falta de fondo** — o sea, dice
**lo mismo que la regla A del cliente, pero con datos que existen**. Ese es el dato sobre el que se
construye el aviso de recuperación, **no un pulso en reposo que no hay**.

---

## 4-bis. La composición corporal: HAY camino, y no es teclear

**El dueño lo dijo claro, y tenía razón**: *«la idea es que él mismo pueda hacerlo cuando tenga la
aplicación a mano, pues nosotros solo tenemos acceso a la aplicación; no tiene sentido que yo meta algo
a mano»*. **Y LO TIENE**: si alguien tiene que teclear los datos, **la aplicación está mal hecha**.

**Medido primero lo obvio**: las actividades de Garmin (`ACTIVITIES_LIST`) traen distancia, desnivel y
velocidad — **nada de composición corporal**. Ya se sabía: el peso y la grasa **solo salen de la báscula
de Huawei**, y esa es la razón de que existan los dos ecosistemas.

**Y ENTONCES APARECIÓ LA SALIDA, y es oficial:**

> **Huawei tiene una EXPORTACIÓN DE DATOS DE SALUD.** Se pide desde el Centro de Privacidad de Huawei
> (`https://privacy.consumer.huawei.com/tool`) → *«Request Your Data»* → **Health**. Huawei manda por
> correo un **ZIP con el JSON crudo**, dentro de la carpeta *«Health detail data & description»*.
>
> **Confirmado por un proyecto real que hace exactamente eso**:
> `https://github.com/christianeirich/huawei-health-to-health-connect` — exporta el peso y la grasa
> corporal **desde ese mismo JSON** (licencia MIT). Es el mismo camino del que salió su captura.

**Por qué esto es MEJOR que lo que proponía el encargo** (Health Sync → Garmin, Terra API, Sahha):

| | la exportación oficial | lo que proponía el encargo |
|---|---|---|
| **qué trae** | peso, grasa **y también agua corporal, músculo y masa ósea** | peso y grasa, y nada más |
| **intermediarios** | **ninguno** | Health Sync + Garmin + un agregador de pago |
| **si se cae** | no se cae: es un fichero | tres servicios que pueden cambiar o cerrar |
| **el dato llega a** | **nuestra aplicación** | la nube de Garmin, y de ahí a nosotros |

**LO QUE ESTO CAMBIA EN EL PLAN**: la regla A del cliente (agua corporal < 54 %) **deja de depender de
que nadie teclee nada**. Se lee **de su exportación de Huawei**, igual que sus entrenamientos se leen de
sus FIT. **Un fichero más, en la misma carpeta.** Y él lo pide cuando quiere, desde su teléfono.

**Lo que sigue haciendo falta decidir, y es suyo**: **cada cuánto pide la exportación.** No es
automático — Huawei la prepara y la manda por correo, así que tarda horas o días. Para la regla del agua
corporal eso significa que **el aviso trabaja con el último dato que él haya traído**, y la aplicación
**tiene que decir de qué día es** ese dato. Un aviso de hidratación con un dato de hace tres semanas
**no es un aviso: es un adorno**, y la norma del proyecto prohíbe enseñar un número sin decir de cuándo es.

---

## 5. Lo que NO se va a construir, y por qué

**El encargo pide un «agente de IA que ejecuta un bucle cada mañana y noche».** No hace falta, y hay
tres motivos:

1. **Ninguna de sus tres reglas necesita IA.** Son «si X, entonces avisa de Y», y eso es determinista.
   Un sistema así **se puede comprobar**; uno con un modelo dentro, no.
2. **El bucle no hace falta**: la aplicación **se abre** y calcula. Lo único útil de esa idea es un
   **recordatorio**, y es una función pequeña.
3. **Y el motivo de fondo**: esto decide sobre **el cuerpo de una persona de 62 años**. Un sistema que
   improvisa sobre su rodilla, su alimentación y sus espasmos **es un riesgo**, no una comodidad.

**Tampoco se construye**: Health Sync, Terra API, Sahha. **Duplican lo que ya está en su equipo**, y
añaden cuentas, permisos y dependencias que se caen. Sus datos ya están en local.

**Y los suplementos** (CoQ10, Omega-3, Calcio/D3) y el **caldo de huesos**: la aplicación los puede
**organizar y recordar** como lo que son —**sugerencias de una IA, no indicaciones médicas**— pero **no
va a afirmar que sirven**. No hay ni una fuente que lo sostenga, y la norma de este proyecto es que **lo
que no se puede citar no se calcula**.

**Y una regla de honestidad que hace falta desde ya**: la aplicación **no es un médico**. Cuando dé un
aviso sobre la rodilla, **dirá que es una observación de sus datos**, no un diagnóstico.

---

## 6. Y una corrección que hay que hacer hoy

**La aplicación tiene `edad = 60` escrito en el código.** Su perfil de Garmin dice que nació el
**1964-01-25**, así que **hoy tiene 62 años**. De la edad sale la frecuencia máxima estimada, así que
**las zonas que enseña están corridas**.

**Y no se arregla escribiendo 62**: se arregla **guardando la fecha de nacimiento** y calculando la edad
**en el momento de usarla**. Un 62 escrito a mano **caduca el 25 de enero**, y entonces vuelve el mismo
problema. Es el mismo error que los siete datos escritos a mano: **un dato que caduca y nadie lo nota**.
