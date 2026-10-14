# Contrato de `huawei.rs`: la pieza que lee la exportacion de Huawei

**Escrito el 2026-10-14, ANTES del codigo.** Es la regla 1 de
[`docs/ARQUITECTURA-piezas-y-contrato.md`](ARQUITECTURA-piezas-y-contrato.md): *antes de escribir codigo
de una pieza, se escribe su contrato*. Si el contrato no cabe en cinco lineas, la pieza es demasiado
grande.

## Que es, y en que capa vive

Es un **adaptador** (capa 2 de 5). Traduce el formato de fuera (la exportacion de salud de Huawei) al
modelo del motor, y **no decide nada**. El motor **no conoce** este fichero: el dia que Huawei cambie su
formato, se cambia **esta pieza** y no se toca una linea del centro.

## Que recibe

- **La ruta de un ZIP tal cual sale del correo de Huawei, O la de una CARPETA ya descomprimida.**
  La pieza decide sola: si es un directorio, lo recorre; si es un fichero, lo abre como ZIP. Dentro, los
  ficheros `health detail data<epoch>.json` de la carpeta *«Health detail data & description»*, **a
  cualquier profundidad**.
- **Abrir el ZIP es trabajo DE LA APLICACION, no del cliente.** Decision del dueño (2026-10-14):
  *«la idea es minimizar el trabajo del cliente»*. La dependencia `zip` esta **autorizada por el** y
  fijada en `Cargo.toml`; se lee **en memoria**, sin escribir nada en disco. (Antes de esta decision la
  pieza devolvia un error tipado si le llegaba un ZIP; ese error ya no existe.)

## Que devuelve

- `Result<Vec<Composicion>, ErrorHuawei>`: **una medicion por fecha**, ordenada de la mas antigua a la
  mas reciente. Un fichero con 40 pesajes da 40 mediciones.
- `Composicion` es un tipo de la **capa 1** (`datos.rs`), porque es del modelo, no del formato:

  ```
  pub struct Composicion {
      pub fecha:             Dato<Fecha>,
      pub peso_kg:           Dato<f32>,
      pub grasa_corporal_pct: Dato<f32>,
      pub agua_corporal_pct: Dato<f32>,
      pub masa_muscular_kg:  Dato<f32>,
      pub masa_osea_kg:      Dato<f32>,
  }
  ```

- **Un campo que no viene queda en `Dato::Falta`. NUNCA en `Dato::Hay(0.0)`.** «No se midio el agua»
  no es «el agua vale cero». La regla que sostiene todo el proyecto (ver `datos.rs`).
- **Siempre en unidades del SI**: kilogramos y porcentaje. La conversion (si Huawei trae gramos) es
  **de esta pieza**, y se prueba aqui, no se filtra al motor.

## Que NO hace (el limite, dicho claro)

- **No calcula** medias, tendencias ni margenes. No decide avisos.
- **No escribe** nada en disco. **No toca la red** (el ZIP ya esta en el equipo). **No pide permisos.**
- **No conoce** a `Persona`, ni a `Hoy`, ni a la pantalla. Solo entrega `Composicion`.
- **No rellena huecos**: si falta el agua, lo dice con `Falta`; no la estima desde el peso ni desde la
  grasa.

## Invariantes que se comprueban solas

1. **Cero ceros falsos**: ningun campo de composicion devuelve `Hay(0.0)` por un campo ausente.
2. **Un valor fuera de rango fisico** (peso <= 0 o > 500 kg; un porcentaje fuera de 0..100) **no se
   convierte en dato**: se trata como `Falta`, porque un sensor roto devuelve su centinela.
3. **La fecha es de calendario**, sin hora ni zona (`Fecha`).
4. **El orden es determinista**: mismo fichero, mismo resultado.

## Como se comprueba (las pruebas de la pieza)

- Con un **JSON minimo inventado** (sin ningun dato del cliente) se lee y salen los valores.
- **Sin agua** en el JSON: `agua_corporal_pct` queda `Falta` y **el resto sigue leyendose**.
- **Valor centinela 0** en agua: queda `Falta`, no 0.
- **Unidad en gramos**: se convierte a kg y la prueba lo exige.
- **JSON corrupto o de otro formato**: devuelve error tipado, **sin panico**.
- **Un ZIP real** (creado en la prueba con el propio crate `zip`, con un JSON sintetico) **se abre sin
  descomprimirlo a disco**, con el fichero anidado a cualquier profundidad.
- **Un fichero que no es un ZIP** y **un ZIP sin datos de composicion**: error tipado, no panico.

## El mapeo, ya CONFIRMADO (2026-10-14)

Medido en `docs/HUAWEI-exportacion.md` contra el XLSX de descripcion del propio export, tres parsers
independientes y la API de Salud de Huawei.

| campo de `Composicion` | clave JSON | unidad |
|---|---|---|
| `peso_kg` | `bodyWeight` | kg |
| `grasa_corporal_pct` | `bodyFatRate` | % |
| `agua_corporal_pct` | `moistureRate` | % |
| `masa_muscular_kg` | `muscleMass` | kg |
| `masa_osea_kg` | `boneSalt` | kg |
| `fecha` | `startTime` (ms epoch UTC) **+ `timeZone` del registro** | fecha local |

Y la forma del fichero, que tambien hay que respetar:

- El fichero es un **array de registros** (o un registro suelto). Cada registro trae `samplePoints[]`.
- Se filtra por `key == "WEIGHT_BODYFAT_BROAD"`, **NUNCA por `type`** (el mismo dato aparece como
  `type: 8` y `type: 10006`).
- `value` es **un texto que contiene otro JSON** (doble codificacion); puede venir tambien como objeto.
- Se deduplica por `(deviceCode, startTime, key)`.

**Lo que sigue SIN confirmar**: si la bascula del cliente rellena de verdad `moistureRate`,
`muscleMass` y `boneSalt` (las muestras publicas solo traian peso/IMC). El cliente usa una Scale 3, que
si los mide, pero **hay que comprobarlo al abrir SU ZIP**. Y el rango exacto de `boneSalt` (el
diccionario dice [0.5, 5] y el XLSX [1.0, 4.0]).

- **Si hay varias mediciones por dia**: se guardan todas y decide quien las consuma, no el adaptador.
