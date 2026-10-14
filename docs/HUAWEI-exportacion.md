# Exportación de datos de salud de Huawei — campos, unidades y claves JSON

**Fecha:** 2026-10-14 · **Autor:** Emilio Bergez, asistido por IA (DeepSeek Harness)
**Ámbito:** composición corporal (peso, grasa, **agua corporal**, **masa muscular**, **masa ósea**) dentro del ZIP de
"Health detail data & description" que Huawei entrega al pedir una copia de datos.

> **Aviso:** el repositorio de Pulso es **público**. Este documento **no contiene ningún dato personal del
> cliente**: los valores de la columna *ejemplo* son **ilustrativos** (redondeados o inventados para mostrar
> el formato) y no proceden de su exportación. Las muestras públicas que se citan son de terceros y se usan
> **solo para leer nombres de campo y estructura**; en las citas se enmascaran los valores.

---

## 0. Resumen ejecutivo

- El camino es el correcto: **Centro de Privacidad de Huawei → "Obtener una copia de sus datos" → Salud**.
  Se puede pedir desde el móvil o desde el PC; la copia llega (o se descarga) **en unos 7 días laborables**
  y el ZIP incluye la carpeta **"Health detail data & description"** con JSON crudo. [S1][S3]
- El JSON es una **lista de registros**; cada registro trae `samplePoints[]`, y cada muestra tiene
  `key` + `value`, donde **`value` es un string que contiene OTRO JSON** (doble codificación). [S4][S5][S6]
- La composición corporal vive bajo la clave **`WEIGHT_BODYFAT_BROAD`** (tipo 10006; también aparece como
  `type: 8`), y sus campos incluyen **`bodyWeight`**, **`bodyFatRate`**, **`moisture`** (agua, kg),
  **`moistureRate`** (agua, %), **`muscleMass`** (músculo, kg), **`skeletalMusclelMass`** y
  **`boneSalt`** (masa ósea, kg). [S4][S7][S8][S9]
- **Advertencia clave:** los nombres y unidades de `moisture`/`muscleMass`/`boneSalt` están **confirmados
  por la documentación de Huawei** (diccionario interno y el propio fichero de descripción del export), pero
  **no he visto una exportación pública real que rellene esos tres valores** (las muestras públicas solo
  traían peso/IMC de una báscula básica). Es decir: **el esquema está confirmado; la presencia efectiva en
  el ZIP del cliente hay que comprobarla al abrirlo.** Ver §4.

---

## 1. Cómo se pide, paso a paso

**Vía PC (recomendada para el ZIP completo):**

1. Entrar en el **Centro de Privacidad** de Huawei con la cuenta Huawei:
   <https://privacy.consumer.huawei.com/tool> [S2]
2. Pulsar **"获取您的数据副本"** / **"Request Your Data"** (Obtener una copia de sus datos). [S1]
3. **Marcar la casilla de Salud** (`Health`) y, si se quiere, acotar el resto. [S1][S3]
4. Elegir el **tamaño máximo de descarga** y pulsar **"提交需求"** (Enviar solicitud). [S1]
5. Esperar **~7 días laborables**; la copia se consulta y descarga **en el propio Centro de Privacidad**. [S1]
   - La herramienta en línea cubre **~1 año**; para más de un año hay que solicitarlo en la página de
     problemas de privacidad indicando la app y el periodo. [S1]
6. Descargar el **ZIP**. [S1][S3]

**Vía móvil (alternativa):** Ajustes → (tu avatar) → Cuenta Huawei → **Privacidad** → **"Gestione sus
datos"** → **"Obtener una copia de sus datos"**. [S1]

**Nota medida sobre las URLs:** la página oficial [S1] se leyó entera (200). La herramienta [S2] responde
**200 pero es una SPA**: el HTML solo trae un `<iframe>` de login y el contenido lo pinta JavaScript, así
que **no es verificable con un fetch** ni sin sesión. No hubo **ningún 403/404** en las URLs usadas para
este documento; la página de desarrollador de Huawei consultada devuelve un armazón JS vacío (**no se usa
como fuente**).

---

## 2. Estructura del ZIP y de los JSON

### 2.1 Dentro del ZIP

- Una carpeta llamada **`Health detail data & description`**, con:
  - varios **`health detail data<epoch>.json`** (un fichero por lote de datos), y
  - un fichero **`Health detail data-Data Field Description.xlsx`** que es **la descripción oficial de los
    campos** (nombres, significado y **unidades**). [S3][S6][S8]
- El export puede venir además **anidado** bajo una carpeta tipo `HUAWEI_HEALTH_<AAAAMMDDHHMMSS>/`; un
  adaptador robusto debe **buscar la carpeta por nombre en cualquier profundidad**. [S6]
- Otras carpetas del export (fuera del alcance): `Sport per minute merged data & description`, ficheros
  `.xlsx` de perfil/acuerdos, etc. [S6]

### 2.2 Esqueleto de un registro (valores enmascarados; claves literales)

```jsonc
[
  {
    "appType": 1,
    "dataId": 0,
    "deviceCode": <int>,              // id del dispositivo
    "endTime": <ms>,
    "healthDataSource": 2,
    "mergedFieldsSupport": false,
    "mergedFlag": 0,
    "recordId": "<base64>",
    "samplePoints": [
      {
        "endTime": <ms>,
        "key": "WEIGHT_BODYFAT_BROAD",
        "startTime": <ms>,           // epoch en MILISEGUNDOS (UTC)
        "value": "{\"bmi\":<num>,\"bodyWeight\":<num>,\"conflictFlag\":0,\"extendAttribute\":\"0\",\"gender\":0,\"height\":<num>}"
      }
    ],
    "startTime": <ms>,
    "timeZone": "+0300",
    "tr": { "ct": <ms>, "lt": 0, "mt": 0, "mts": 0 },
    "type": 8,                          // o 10006, según origen
    "version": <int>
  }
]
```

Fragmentos **literales** de la muestra real (el resto del número está enmascarado):

- `"key":"WEIGHT_BODYFAT_BROAD","startTime":` — la clave de la muestra y su marca de tiempo. [S10]
- `"value":"{\"bmi\":` — el valor es un **string que contiene JSON** (doble codificación). [S10]
- `"timeZone":"+0300"` — zona horaria del registro. [S10]

### 2.3 Reglas del formato (trianguladas con tres parsers independientes)

- El fichero es **un array** (o un solo objeto) de registros. [S4][S5]
- Cada registro tiene `samplePoints[]`; se recorren **todos** los puntos. [S4][S6]
- **`value` es un string JSON** que hay que **volver a parsear** (`json.loads` / `serde_json::from_str`).
  [S4] dice literalmente `data = json.loads(sp['value'])`; [S5] lo llama *"doppelt kodierter JSON-String"*.
- **No filtrar por `type`**: el mismo dato aparece como **`type: 10006`** y también como **`type: 8`**
  (visto en muestras reales). Filtrar por `key == "WEIGHT_BODYFAT_BROAD"` es lo robusto.
- `startTime`/`endTime` van en **ms epoch UTC**. [S4][S10]
- `extendAttribute` / `subUser` sirven para separar **varios usuarios** en el mismo export. [S4]

---

## 3. Tabla de campos

Fuentes: **[S1]** soporte oficial · **[S2]** Centro de Privacidad · **[S3]** README del repo de
christianeirich · **[S4]** `main.py` del mismo repo · **[S5]** `huawei_import.py` de Soellerhaus ·
**[S6]** `huawei_export_adapter.py` de HealthClaw · **[S7]** `dict_config.txt` (diccionario interno
descompilado) · **[S8]** **XLSX de descripción de campos del propio export** · **[S9]** doc de la API de
Salud de Huawei (espejo) · **[S10]** JSON de export real de muestra · **[S11]** modelo Java descompilado.
Las citas completas, en §6.

### 3.1 Envoltorio y metadatos

| clave JSON | dato | unidad | ejemplo (ilustrativo) | de dónde se sabe |
|---|---|---|---|---|
| `samplePoints` | lista de muestras del registro | — | `[{...}]` | [S4] `for sp in rec.get('samplePoints', [])`; [S6] |
| `key` | nombre del dato dentro de la muestra | texto | `"WEIGHT_BODYFAT_BROAD"` | [S4] `if sp.get('key') != 'WEIGHT_BODYFAT_BROAD'`; [S10] |
| `value` | carga útil; **JSON como string** | texto JSON | `"{\"bodyWeight\":70.0,...}"` | [S4] `json.loads(sp['value'])`; [S5] |
| `startTime` / `endTime` | inicio / fin de la medida | **ms epoch (UTC)** | `1700000000000` | [S4] `ms = sp.get('startTime')`; [S8] *"Unit: millisecond"* |
| `timeZone` | zona horaria del registro | `±HHMM` | `"+0300"` | [S10] |
| `type` | tipo de registro | entero | `10006` (o `8`) | [S7] `"typeId": 10006`; [S10] `"type":8` |
| `deviceCode` | id del dispositivo | — | `<int>` | [S6] `item.get("deviceCode")`; [S10] |
| `extendAttribute`, `subUser` | identificación de usuario | texto | `"0"` | [S4] `data.get('extendAttribute')` |
| `fieldsMetadata` | metadatos opcionales del punto | texto JSON | — | [S6] `self._parse_json_string(point.get("fieldsMetadata"))` |
| `recordId`, `healthDataSource`, `appType`, `version`, `tr` | metadatos del registro | — | — | [S10] |

### 3.2 Composición corporal (dentro de `value` cuando `key == "WEIGHT_BODYFAT_BROAD"`)

| clave JSON | dato | unidad | ejemplo (ilustrativo) | de dónde se sabe |
|---|---|---|---|---|
| `bodyWeight` | **peso** | **kg** | `82.3` | [S4] `data.get('bodyWeight')`; [S8] *"Unit: kilogram"*; [S9] *"单位：kg"* |
| `bodyFatRate` | **% de grasa** | **%** | `22.4` | [S4] `data.get('bodyFatRate')`; [S8] *"Accurate to 0.1%. Unit: percentage."* |
| `bodyFat` | **masa grasa** | **kg** | `18.4` | [S8] *"Body fat quantity... Unit: kilogram"*; [S9] *"体脂量... 单位：kg"* |
| `bmi` | índice de masa corporal | **kg/m²** | `24.7` | [S9] `bmi ... 单位：kg/m²`; [S8] lo define como kg ÷ m² |
| `muscleMass` | **masa muscular (total)** | **kg** | `27.9` | [S7] `"muscleMass >= 0.1 && muscleMass <= 150"`; [S8] *"Muscle mass. Unit: kilogram."*; [S9] *"肌肉量... 单位：kg"* |
| `skeletalMusclelMass` (también `skeletalMuscleMass`) | **músculo esquelético** | **kg** | `26.5` | [S7] `"skeletalMusclelMass >= 1 && ... <= 150"`; [S8] *"Skeletal muscle mass... Unit: kilogram"*; [S9] escribe `skeletalMuscleMass` |
| `moisture` | **agua corporal (masa)** | **kg** | `46.5` | [S7] `"moisture >= 0 && moisture <= 500"`; [S8] *"Water weight... Unit: kilogram. Value range: [0, 500]."*; [S9] *"水分量... 单位：kg"* |
| `moistureRate` | **agua corporal (%)** | **%** | `51.6` | [S7] `"moistureRate >= 0 && moistureRate <= 100"`; [S8] *"Water rate... Unit: percentage."*; [S9] *"水分率... 单位：%"* |
| `boneSalt` | **masa ósea (mineral)** | **kg** | `3.21` | [S7] `"boneSalt >= 0.5 && boneSalt <= 5"`; [S8] *"Bone mineral amount Unit: kilogram"*; [S9] *"骨盐量... 单位：kg"* |
| `proteinRate` | % de proteína | **%** | `17.3` | [S7] `"proteinRate >= 0 && proteinRate <= 100"`; [S8] *"Protein ratio... Unit: percentage."* |
| `protein` | masa proteica | **kg (probable; no declarado)** | `14.2` | [S7] política `"protein >= 0 && protein <= 500"` — **unidad sin declarar**, no usarla a ciegas |
| `basalMetabolism` | metabolismo basal | **kcal/día** | `1487` | [S7] `"name": "basalMetabolism"`; [S8] *"unit: kilo-calories/day"*; [S9] *"单位：kcal/day"* |
| `visceralFatLevel` | grasa visceral (nivel) | adimensional | `8.0` | [S7] `"visceralFatLevel >= 1 && visceralFatLevel <= 59"`; [S8] *"Value range: [1.0, 59.0]"* |
| `bodyScore` | puntuación corporal | (etiquetado %, en la práctica 0–100) | `80` | [S7] `"bodyScore >= 0 && bodyScore <= 100"`; [S8] *"Body score. Value range: [0, 100]."* |
| `bodyAge` | edad corporal | años | `40` | [S7] `"bodyAge >= 5 && bodyAge <= 99"`; [S8] *"Physical age Value range: [18, 99]."* |
| `height` | altura | **cm** | `181` | [S6] `value.get("height")` → `height_cm`; [S8] *"Height. (Unit: centimeter)."* |
| `age` | edad | años | `40` | [S6] `value.get("age")`; [S8] *"Age. Value range: [0, 150]"* |
| `gender` | sexo | `-1/0/1/2` | `1` | [S8] *"−1: Unknown... 0: Female. 1: Male. 2: Confidential."*; [S7] `"gender >= 0"` |
| `impedance` | impedancia | **ohm** | `500.0` | [S7] `"impedance >= 0.1 && impedance <= 100000"`; [S8] *"Unit: ohm"*; [S9] *"单位：欧姆"* |
| `leftArmMuscleMass`, `rightArmMuscleMass`, `leftLegMuscleMass`, `rightLegMuscleMass`, `trunkMuscleMass` | músculo por segmento | **kg** | `3.2` | [S7] políticas `...MuscleMass`; [S8] *"Left arm skeletal muscle mass Range: [0.05, 150.0]"*; [S11] |
| `leftArmFatMass`, `rightArmFatMass`, `leftLegFatMass`, `rightLegFatMass`, `trunkFatMass` | grasa por segmento | **kg** | `0.8` | [S7] `"trunkFatMass >= 0.05 && ... <= 150"`; [S8] *"Torso fat mass Range: [0.05, 150.0]"* |
| `waistHipRatio` | relación cintura-cadera | adimensional | `0.85` | [S7] `"waistHipRatio >= 0.2 && ... <= 1.5"`; [S8] *"Estimated hip-waist ratio"* |
| `healthyWeight` | peso saludable (**calculado por la app**) | **kg** | `68.0` | [S8] *"Ideal weight (not included in the original body fat scale data...)"* |
| `healthyFatRate` | % grasa saludable (**calculado por la app**) | **%** | `20.0` | [S8] *"Ideal body fat rate (not included in the original body fat scale data...)"* |
| `conflictFlag` | marca de conflicto | entero | `0` | [S10] presente en la muestra real |

### 3.3 Otras claves observadas en el mismo export (no composición corporal)

Recuento propio sobre **92 JSON** de una exportación pública de muestra (carpeta "Health detail data &
description"), útil para el adaptador pero **no verificado campo a campo en este documento**:

| `type` | `key` | campos internos vistos |
|---|---|---|
| 10002 / 5 | `BLOODPRESSURE` | `BLOOD_PRESSURE_SYSTOLIC`, `BLOOD_PRESSURE_DIASTOLIC`, `sphygmus` |
| 16 / 400021 | `BLOOD_OXYGEN_SATURATION` | `avgSaturation`, `spo2` |
| 400022 | `LOW_BLOOD_OXYGEN` | `minLowSpO2`, `maxLowSpO2`, `lowSpO2Detail` |
| 7 | `DATA_POINT_DYNAMIC_HEARTRATE`, `DATA_POINT_NEW_REST_HEARTRATE` | — |
| 500023 | `DYNAMIC_HEART_RATE` | `bpm` |
| 500024 | `RESTING_HEART_RATE` | `restBpm` |
| 500018 | `BASAL_METABOLISM` | `basalMetabolism` |
| 500026 | `STRESS` | `stressScore` |
| 11 | `STRESS_DATA` | `score`, `grade`, `feature`, `featureAtts`, `calibFlag`, `measureType`, `ppgFlag` |
| 9 | `PROFESSIONAL_SLEEP_DEEP`, `..._LIGHT`, `..._WAKE`, `..._DREAM`, `..._NOON` | — |
| 500005 | `SLEEP_RECORD` | `bedTime`, `risingTime`, `sleepScore`, `effectiveSleepDuration` |
| 400027 | `USER_HEIGHT` | `userHeight` |
| 200005 | `ACTIVE_HOUR` | `isActive` |

Fuente de este recuento: el árbol del repositorio de muestra y sus JSON [S10]. **Es observación, no
especificación**: tratar como orientativo.

---

## 4. Qué está CONFIRMADO y qué NO

### Confirmado

1. **El procedimiento de petición** y el nombre de la carpeta del ZIP: soporte oficial [S1] y README [S3].
2. **La estructura del JSON**: array de registros → `samplePoints[]` → `{key, value, startTime, endTime}`,
   con `value` **doble-codificado**. Tres implementaciones independientes coinciden [S4][S5][S6] y una
   muestra real lo enseña [S10].
3. **Peso y grasa**: `bodyWeight` (kg) y `bodyFatRate` (%) bajo `WEIGHT_BODYFAT_BROAD` [S4][S5][S3].
4. **Esquema de agua, músculo y hueso**: las claves **`moisture`**, **`moistureRate`**, **`muscleMass`**,
   **`skeletalMusclelMass`** y **`boneSalt`** están en el **diccionario interno de Huawei** [S7], en el
   **XLSX de descripción del propio export** [S8] y en el **modelo de la app** [S11].
5. **Unidades**: kg para `moisture`, `muscleMass`, `boneSalt`; % para `moistureRate`; ohm para
   `impedance`; kcal/día para `basalMetabolism` — confirmadas por [S8] y por la doc de la API [S9].

### NO confirmado (y hay que comprobarlo en el ZIP del cliente)

1. **Que una exportación real traiga `moisture`/`muscleMass`/`boneSalt` rellenos.** Las muestras
   públicas que encontré **solo traían** `bmi, bodyWeight, conflictFlag, extendAttribute, gender, height`
   (báscula básica). Los tres campos existen en el esquema, pero **su presencia depende del dispositivo y
   de si la medición de composición corporal se sincronizó**. El cliente usa una **Huawei Scale 3**, que sí
   los mide, así que *probablemente* aparezcan — pero es una inferencia, no una observación.
2. **La grafía exacta de `skeletalMusclelMass`**: Huawei la escribe con una "l" de más en el diccionario y
   en el export [S7][S8], y **sin** ella en la doc de la API [S9]. **Aceptar ambas.**
3. **El rango de `boneSalt`**: el diccionario y la API dicen **[0.5, 5]** [S7][S9], pero el XLSX del export
   dice **[1.0, 4.0]** [S8]. No usar el rango como validación estricta.
4. **La unidad de `protein`**: su política es `>= 0 && <= 500` [S7]; por analogía sería kg, pero **no está
   declarada**. No usarla sin contrastar.
5. **El valor de `type`**: `10006` en el diccionario [S7] y en los parsers [S4][S5], pero `8` en una
   muestra real [S10]. **No filtrar por `type`.**
6. **La disposición exacta del ZIP** (carpeta `HUAWEI_HEALTH_<ts>/` por encima o no): los repos manejan
   ambos casos [S6]; hay que descubrirla al descomprimir.

---

## 5. Cómo lo leería un adaptador Rust

### 5.1 Qué mirar

1. **Abrir el ZIP** y **buscar la carpeta `Health detail data & description` en cualquier profundidad**
   (no asumir nivel fijo). También aceptar un directorio ya descomprimido.
2. Dentro, recorrer **todos los `health detail data*.json`** (los nombres llevan un epoch).
3. Deserializar cada fichero como **`Vec<Record>`** (y tolerar un objeto suelto).
4. Para **cada registro**, recorrer **`samplePoints`** sin filtrar por `type`.
5. Quedarse con los puntos cuya **`key == "WEIGHT_BODYFAT_BROAD"`**.
6. **Parsear `value` otra vez**: si es `String`, `serde_json::from_str`; si ya es objeto, usarlo tal cual;
   si es vacío/`"null"`/"{}", descartar el punto.

### 5.2 Estructuras de apoyo

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Record {
    #[serde(rename = "type")] kind: Option<i64>,          // no filtrar por esto
    #[serde(rename = "deviceCode")] device_code: Option<i64>,
    #[serde(rename = "startTime")] start_time: Option<i64>,
    #[serde(rename = "endTime")] end_time: Option<i64>,
    #[serde(rename = "timeZone")] time_zone: Option<String>,
    #[serde(default)] sample_points: Vec<SamplePoint>,
}

#[derive(Deserialize)]
struct SamplePoint {
    key: Option<String>,
    startTime: Option<i64>,
    endTime: Option<i64>,
    value: Option<serde_json::Value>,   // string JSON o ya objeto
}

/// Todo opcional: si un campo falta, se devuelve None, NUNCA 0.
#[derive(Deserialize, Default, Debug)]
struct BodyComposition {
    #[serde(rename = "bodyWeight")]  body_weight_kg: Option<f64>,
    #[serde(rename = "bodyFatRate")] body_fat_pct:  Option<f64>,
    #[serde(rename = "bodyFat")]     body_fat_kg:   Option<f64>,
    #[serde(rename = "muscleMass")]  muscle_mass_kg: Option<f64>,
    // ojo con la "l" de más: aceptar las dos grafías
    #[serde(rename = "skeletalMusclelMass", alias = "skeletalMuscleMass")]
    skeletal_muscle_kg: Option<f64>,
    #[serde(rename = "moisture")]     body_water_kg:  Option<f64>,   // agua corporal
    #[serde(rename = "moistureRate")] body_water_pct: Option<f64>,   // agua corporal
    #[serde(rename = "boneSalt")]     bone_mass_kg:   Option<f64>,   // masa ósea
    #[serde(rename = "proteinRate")]  protein_pct:    Option<f64>,
    #[serde(rename = "basalMetabolism")] bmr_kcal_day: Option<f64>,
    #[serde(rename = "bmi")]          bmi: Option<f64>,              // kg/m²
    #[serde(rename = "height")]       height_cm: Option<f64>,        // cm, no m
    #[serde(rename = "visceralFatLevel")] visceral_fat_level: Option<f64>,
    #[serde(rename = "bodyScore")]    body_score: Option<f64>,
    #[serde(rename = "bodyAge")]      body_age: Option<f64>,
    // extensiones: conservar crudas si no se modelan
    #[serde(flatten)] extra: std::collections::BTreeMap<String, serde_json::Value>,
}
```

### 5.3 Reglas de lectura

- **Unidades fijas, sin conversión de fondo:** `bodyWeight`, `bodyFat`, `muscleMass`, `skeletalMusclelMass`,
  `moisture`, `boneSalt` → **kg**; `bodyFatRate`, `moistureRate`, `proteinRate` → **%**; `height` → **cm**
  (¡no metros!); `bmi` → **kg/m²**; `impedance` → **ohm**; `basalMetabolism` → **kcal/día**.
  Si el motor trabaja en kg/%, **no hay que multiplicar ni dividir por 100**; solo convertir g→kg o m→cm si
  el export los trajera en otra escala (no visto).
- **Marca de tiempo:** `startTime`/`endTime` en **ms UTC**. Guardar el **ms crudo** y, si se necesita hora
  local, aplicar `timeZone` (formato `±HHMM`). **No** deducir la zona del reloj de la máquina.
- **Deduplicación:** el mismo dato puede aparecer como `type: 8` y `type: 10006`; deduplicar por
  `(deviceCode, startTime, key)`.
- **Multiusuario:** separar por `extendAttribute` / `subUser` si hay más de uno. [S4]
- **`gender`:** mapear `-1`→desconocido, `0`→mujer, `1`→hombre, `2`→confidencial. [S8]

### 5.4 Qué puede faltar, y qué devolver si falta

| situación | qué puede pasar | qué debe devolver el adaptador |
|---|---|---|
| El campo no está en `value` | p. ej. la báscula no mide agua/hueso, o la medición fue básica | **`None`/`null`**, y **no** inventar 0. Opcional: marcar `body_water_kg: None` como "no medido" |
| `value` es `""`, `"null"` o `"{ }"` | punto sin datos | descartar el punto; si todo el registro está vacío, registrar un aviso |
| `value` no es JSON válido | dato corrupto o formato nuevo | descartar ese punto y **anotar el error**, sin abortar el resto del fichero |
| Falta `moistureRate` pero hay `moisture` (o al revés) | solo uno de los dos | devolver el que exista; **no** calcular el % a partir del kg sin altura/volumen (no es fiable) |
| Falta `boneSalt` | la báscula no lo mide | `None`; **no** usar la masa magra como sustituto |
| No aparece ningún `WEIGHT_BODYFAT_BROAD` | el usuario no midió composición o no se sincronizó | devolver **lista vacía + aviso explícito** ("sin datos de composición en el export") |
| `height`/`age`/`gender` ausentes | la balanza no los envía | devolver `None`; el motor decide si puede calcular el IMC con el perfil que ya tenga |
| `type` es `8` en vez de `10006` | variante observada | **no** descartar: se filtra por `key` |

**Principio:** un campo ausente es **información** ("no se midió"), no un cero. El adaptador debe
**exponer qué campos venían** (p. ej. un `Vec<&str>` con las claves presentes) para que el motor sepa si
la báscula era básica o de composición, y para que el informe pueda decir "agua corporal: sin dato" en vez
de "0 %".

---

## 6. Fuentes (URL + cita textual)

### [S1] Soporte oficial de Huawei — *Cómo obtener una copia de sus datos*
<https://consumer.huawei.com/cn/support/content/zh-cn16044986/> — leída entera (HTTP 200).

> "在您的电脑上以 华为帐号 登录华为 隐私中心（链接：https://privacy.consumer.huawei.com/tool?lang=zh-cn&themeName=blue），然后点击"获取您的数据副本"。"

> "在您提交获取数据副本的申请后，预计等待7 个工作日左右，即可在隐私中心 (https://privacy.consumer.huawei.com/tool)中查阅和下载。"

> "通过在线工具的方式，您可以方便的获取到您近一年的数据。"

> "在华为或荣耀手机或平板上点击"设置"进入设置页面……点击"隐私中心" > "管理您的数据" > "获取您的数据副本"。"

### [S2] Centro de Privacidad (herramienta)
<https://privacy.consumer.huawei.com/tool> — HTTP 200, **SPA**: el HTML solo trae un `<iframe id="loginIframe">` y el contenido lo pinta JavaScript; **no verificable sin sesión**.

### [S3] README de `christianeirich/huawei-health-to-health-connect` (MIT)
<https://github.com/christianeirich/huawei-health-to-health-connect>

> "After some hours or days, download the ZIP from the email"
> "Extract the JSON files from the folder "Health detail data & description" into a folder."

### [S4] `main.py` del mismo repo (código)
<https://raw.githubusercontent.com/christianeirich/huawei-health-to-health-connect/HEAD/main.py>

> `if rec.get('type') != 10006:`
> `if sp.get('key') != 'WEIGHT_BODYFAT_BROAD':`
> `data = json.loads(sp['value'])`
> `weight = data.get('bodyWeight')` · `fat = data.get('bodyFatRate')`

### [S5] `huawei_import.py` de `Soellerhaus/garmin-dashboard` (código)
<https://raw.githubusercontent.com/Soellerhaus/garmin-dashboard/0d386e296cfef1fede9b0e5f4057b1c55bc8edbb/huawei_import.py>

> "Huawei Health speichert Gewichtsdaten mit type=10006."
> "Die Messwerte stecken in samplePoints[].value als doppelt kodierter JSON-String."

### [S6] `huawei_export_adapter.py` de `HC-Guo/HealthClaw` (código)
<https://raw.githubusercontent.com/HC-Guo/HealthClaw/6b01821b66e64607ca11848f4821204cd9d3b749/tools/wearable_adapters/huawei_export_adapter.py>

> `detail_dir = root / "Health detail data & description"`
> `value = self._parse_json_string(point.get("value"))`
> `"height_cm": self._safe_float(value.get("height")), "weight_kg": self._safe_float(value.get("bodyWeight")),`

### [S7] Diccionario interno de Huawei, descompilado — `dict_config.txt` (`Kuberwastaken/Huawhoop`)
<https://raw.githubusercontent.com/Kuberwastaken/Huawhoop/b14531b68bbaa4ae5f22eeaedc4a908fdf121837/Huawei-Health-Decompiled/resources/assets/dict_config.txt>
(2 384 líneas; leído con `curl` porque el fetch web trunca a 100 000 caracteres)

> `"typeId": 10006,` / `"name": "WEIGHT_BODYFAT_BROAD",`
> `"moisture >= 0 && moisture <= 500"` · `"moistureRate >= 0 && moistureRate <= 100"`
> `"boneSalt >= 0.5 && boneSalt <= 5"` · `"muscleMass >= 0.1 && muscleMass <= 150"`
> `"skeletalMusclelMass >= 1 && skeletalMusclelMass <= 150"`
> `"name": "moisture",` · `"name": "boneSalt",` · `"name": "muscleMass",`

### [S8] **Descripción de campos del propio export** — `Health detail data-Data Field Description.xlsx`
<https://github.com/kelesmert/sleep-cycle-tracker/blob/fe111d3fd46363c8b67bd1b8714b12ae9b65bf03/all-folders/Health%20detail%20data%20%26%20description/Health%20detail%20data-Data%20Field%20Description.xlsx>
(binario; texto extraído de `xl/sharedStrings.xml` con `curl | python3`, **sin guardar en disco**)
> "Water weight. One decimal place is reserved. Unit: kilogram. Value range: [0, 500]."
> "Water rate. The value is accurate to 0.1%. Unit: percentage. Value range: [0, 100]."
> "Bone mineral amount Unit: kilogram. One decimal place is reserved. Value range: [1.0, 4.0]."
> "Muscle mass. Unit: kilogram. One decimal place is reserved. Value range: [1, 150]."
> "Skeletal muscle mass. Value range: [1, 150]. Unit: kilogram"
> "Weight. Value range: [1, 500]. Unit: kilogram" · "Body fat rate. Accurate to 0.1%. Unit: percentage."
> "Height. (Unit: centimeter). Value range: [1, 300]." · "Gender: –1: Unknown (default value). 0: Female. 1: Male. 2: Confidential."

### [S9] Documentación de la API de Salud de Huawei (espejo en `liasica/harmonyos-skills`)
<https://raw.githubusercontent.com/liasica/harmonyos-skills/aa52a18f212cc4a5e7ffa026171714ff712cb9c7/harmonyos/references/harmonyos-references/health-api-healthfields.md>

> `| moisture | ... | 水分量，若未填写，默认为空。 单位：kg 取值范围：[0, 500] |`
> `| moistureRate | ... | 水分率…… 单位：% 取值范围：[0, 100] |`
> `| boneSalt | ... | 骨盐量…… 单位：kg 取值范围：[0.5, 5] |`
> `| muscleMass | ... | 肌肉量…… 单位：kg 取值范围：[0.1, 150] |`
> `| skeletalMuscleMass | ... | 骨骼肌量…… 单位：kg 取值范围：[1, 150] |`
> `| bmi | ... | 单位：kg/m² 取值范围：[1,200] |` · `| impedance | ... | 单位：欧姆 …|`

### [S10] Muestra real de exportación (tercero) — JSON y árbol
<https://github.com/kelesmert/sleep-cycle-tracker/blob/fe111d3fd46363c8b67bd1b8714b12ae9b65bf03/all-folders/Health%20detail%20data%20%26%20description/health%20detail%20data1760886530932.json>
(se recorrieron **92 JSON** de esa carpeta para el recuento de §3.3; **ningún valor personal se copia aquí**)

> `"key":"WEIGHT_BODYFAT_BROAD","startTime":` · `"value":"{\"bmi\":` · `"timeZone":"+0300"` · `"type":8,`

### [S11] Modelo Java descompilado de la app
<https://raw.githubusercontent.com/Kuberwastaken/Huawhoop/b14531b68bbaa4ae5f22eeaedc4a908fdf121837/Huawei-Health-Decompiled/sources/com/huawei/hihealthservice/sync/syncdata/HiSyncWeightData.java>

> `private Double moisture;` · `private Double moistureRate;` · `private Double boneSalt;`
> `private Double muscleMass;` · `private Double skeletalMusclelMass;`

---

## 7. Datos de la muestra pública que NO se incluyen

Del ZIP/carpeta pública [S10] **no se transcribe ninguna medición, id de dispositivo, `recordId` ni
marca de tiempo concreta**: solo se han usado las **claves** y el **esqueleto**. Cualquier valor de la
columna *ejemplo* de §3 es ilustrativo. Si en el futuro se abre el ZIP del cliente, ese fichero **no debe
entrar nunca en el repositorio público**.
