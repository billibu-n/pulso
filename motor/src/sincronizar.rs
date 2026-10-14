//! **Lo que la cuenta de Garmin trae de nuevo: las formas de la API y la parte PURA de la
//! sincronización.**
//!
//! QUÉ ES Y DE QUÉ CAPA: es un **adaptador** (capa 2 de 5). Traduce el JSON de Garmin al modelo del
//! motor y **decide una sola cosa**: qué de lo que llegó **no estaba ya guardado**. El motor **no
//! conoce** este fichero: el día que Garmin cambie una clave se cambia **esta pieza**, y no se toca
//! una línea del centro.
//!
//! QUÉ RECIBE: **texto JSON tal cual sale de la API** (los cuerpos que pide
//! `motor/src/bin/sincronizacion.rs`) y listas de `Actividad` / `Diario` ya leídas (las **guardadas** y
//! las **traídas**). Nada más: ni rutas, ni sesión, ni red.
//!
//! QUÉ DEVUELVE: `Actividad`, `Diario` (la fecha y **un `Option` por dato**: `None` quiere decir «este
//! cuerpo no lo traía») y `Resumen`, que es lo que la pantalla necesita para decir «3 actividades y 2
//! diarios nuevos» **sin contar nada por su cuenta**.
//!
//! QUÉ NO HACE: **no sale a la red, no pide el login, no escribe nada, no toca el motor de cálculo y
//! no formatea ni convierte unidades** (los metros son metros; los segundos, segundos). Tampoco
//! decide significados: que un `0` quiera decir «no medido» se decide **donde se usa el dato**, no
//! aquí (§DUDAS). Y **no rellena huecos**: un dato que falta queda en `None`, **nunca** en un 0, en
//! una cadena vacía ni en un texto de relleno. Un diario sin sueño es un diario sin sueño.
//!
//! LAS REGLAS DEL PROYECTO QUE CUMPLE: unidades sin formatear · `Option` para «no existe» · funciones
//! **puras**, probadas **sin red** (todas las pruebas de este fichero corren con JSON de ejemplo
//! escrito a mano) · textos en español · y cada campo **citado** a la línea de la librería de la que
//! sale su forma.
//!
//! DE DÓNDE SALE CADA FORMA — de la librería `garminconnect`, leída **solo para las formas** (nombres
//! de campo), nunca para valores:
//!
//! - **Actividades** — `/activitylist-service/activities/search/activities` (`__init__.py:735`).
//!   `get_activities` devuelve **la respuesta tal cual** (`__init__.py:2627-2633`), así que es una
//!   **lista JSON, sin envoltorio**. Cada elemento lo modela `typed.py:396-415` (`activityId:403`,
//!   `activityName:404`, `startTimeLocal:406`, `activityType:409`, `duration:411`, `distance:415`), y
//!   el tipo anidado, `typed.py:387-391` (`typeKey:391`). Todos van con `| None` en la librería: por
//!   eso aquí son `Option` **salvo el `id`** (ver «LA IDENTIDAD»).
//! - **Sueño** — `/wellness-service/wellness/dailySleepData` (`__init__.py:2126`): un cuerpo **por
//!   día** (el parámetro es `date`). El resumen vive en `dailySleepDTO` (`typed.py:238`) y dentro:
//!   `calendarDate:188`, `sleepTimeSeconds:190`, `deepSleepSeconds:196`, `remSleepSeconds:198`
//!   (`typed.py:184-199`).
//! - **Pulso en reposo** — `/userstats-service/wellness/daily` con `metricId=60` (`__init__.py:697`).
//!   El cuerpo de **rango** trae **varios días**: `allMetrics.metricsMap.WELLNESS_RESTING_HEART_RATE`
//!   (`__init__.py:2207-2218`), y cada fila trae `calendarDate` + `value` (`__init__.py:2219-2223`).
//!   La librería **descarta la fila sin valor** (`is not None`, `__init__.py:2222`): una fila así es
//!   «ese día no tiene dato». Aquí se hace exactamente lo mismo, con la misma cita.
//! - **Estrés** — `/wellness-service/wellness/dailyStress` (`__init__.py:2173`): un cuerpo por día. La
//!   forma es la de `DailyStats`: `calendarDate` (`typed.py:100`) y `averageStressLevel`
//!   (`typed.py:134`, entero).
//! - **Training status** — `/metrics-service/metrics/trainingstatus/aggregated` (`__init__.py:715`):
//!   `get_training_status` **devuelve el cuerpo tal cual y `typed.py` no modela esa respuesta**
//!   (`__init__.py:2478-2483`). **No hay ninguna forma que citar**, así que `training_status` y
//!   `vo2max` están declarados en `Diario` pero **ninguna función de este módulo los rellena**: se
//!   rellenarán el día que la primera pasada real diga cuáles son sus claves, y entonces se añade su
//!   lector **con la cita**. No se inventa una forma para llenar dos campos.
//!
//! LA IDENTIDAD, que es lo que decide todo: una actividad es **su `id`** (no su nombre ni su fecha: el
//! mismo entrenamiento se puede renombrar), y un diario es **su `fecha`** (la API los da por día: dos
//! diarios de la misma fecha **son el mismo diario**). Por eso `id` y `fecha` son **obligatorios**: si
//! faltan, el parseo **falla diciendo el campo**, en vez de inventarse una identidad con la que luego
//! se duplicarían datos al guardarlos.
//!
//! LA REGLA DEL ERROR ACOTADO: de serde se toma **la primera línea** del mensaje y se le quitan los
//! **valores** —los tramos entre comillas dobles y los que van entre acentos graves cuando son
//! números—, porque ahí es donde serde copia lo que venía en el cuerpo. Se conservan los **nombres**
//! (los acentos graves de «missing field», que es el campo que falta): un nombre de campo es justo lo
//! que hay que decir. **El cuerpo del JSON no sale nunca en un mensaje de error**, porque puede traer
//! datos de una persona. El texto técnico que queda es el de serde, en inglés, a propósito: es una
//! descripción de la biblioteca y traducirla haría perder la pista de qué comprobación falló.
//!
//! §DUDAS, para el dueño (decisiones que **no** toma este módulo, y que están probadas como están
//! para que cambiarlas sea una línea):
//!
//! 1. **El `0` de una noche.** Aquí el parser **deja el 0 como viene** (`Some(0.0)`): traduce formas,
//!    no significados. El instrumento hermano **sí** trata el 0 de `sleepTimeSeconds` como «sin dato»
//!    (`bin/sincronizacion.rs`, `sueno_tiene_dato`). Las dos cosas no pueden ser verdad a la vez: o el
//!    0 se descarta al LEER o se descarta al USAR. Se decide **con el número delante**, y la prueba
//!    `el_cero_del_sueno_se_queda_como_viene` lo deja escrito.
//! 2. **`stressValuesArray`.** No se promedia: la librería **no lo modela** (no hay forma que citar) y
//!    en los cuerpos de ejemplo del proyecto viene vacío. Se lee `averageStressLevel` (`typed.py:134`).
//! 3. **`fecha_local`** va **tal cual** la manda la API: puede venir con espacio (`2026-10-01
//!    07:30:00`) o con `T` (`2026-10-01T07:30:00`). Este módulo **no la normaliza**; quien agrupe por
//!    día, que recorte (el instrumento hermano lo hace).

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;

// ---------------------------------------------------------------------------------------------
// Las formas de la API
// ---------------------------------------------------------------------------------------------

/// **Una actividad**, con la identidad en su `id`.
///
/// Los nombres de campo son los de la API, y van con `#[serde(rename = "...")]`: el `id` es
/// `activityId`, etc. Los campos que la API **puede omitir** (`typed.py:396-415` los modela todos con
/// `| None`) van con `#[serde(default)]` y son `Option`: **no se rellenan**. Las claves que este tipo
/// no lee **se ignoran** (serde no falla por campo desconocido, y la API manda muchas más).
///
/// El ÚNICO campo obligatorio es el `id`: es la identidad, y sin ella «qué es nuevo» no está definido.
/// Si falta, el parseo falla **nombrando el campo** (`missing field`: `activityId`), y sin el cuerpo.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Actividad {
    /// `activityId` (`typed.py:403`). **La identidad**: no el nombre ni la fecha.
    #[serde(rename = "activityId")]
    pub id: u64,
    /// `activityName` (`typed.py:404`). Puede faltar; entonces es `None`, no `""`.
    #[serde(rename = "activityName", default)]
    pub nombre: Option<String>,
    /// El `typeKey` **de dentro** de `activityType` (`typed.py:391`, anidado en `typed.py:409`).
    /// Puede faltar; entonces es `None`, no `"sin tipo"`.
    #[serde(rename = "activityType", default, deserialize_with = "solo_type_key")]
    pub tipo: Option<String>,
    /// `startTimeLocal` (`typed.py:406`), **tal cual viene**: puede traer un espacio o una `T`. No se
    /// normaliza (§DUDAS 3).
    #[serde(rename = "startTimeLocal", default)]
    pub fecha_local: Option<String>,
    /// `duration` (`typed.py:411`), en **segundos** y sin formatear.
    #[serde(rename = "duration", default)]
    pub duracion_s: Option<f64>,
    /// `distance` (`typed.py:415`), en **metros** y sin formatear: la API da metros.
    #[serde(rename = "distance", default)]
    pub distancia_m: Option<f64>,
}

/// Saca el `typeKey` de dentro del objeto `activityType` (`typed.py:387-391`).
///
/// Existe porque el dato está **anidado** y el tipo de fuera solo tiene el nombre del objeto. Si
/// `activityType` viene nulo, o sin `typeKey`, o con un `typeKey` que no es texto, el resultado es
/// `None`: **una actividad sin tipo legible no tira abajo la lista entera** de las que sí lo traen.
fn solo_type_key<'de, D>(deserializador: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let valor = Option::<Value>::deserialize(deserializador)?;
    Ok(valor.and_then(|v| {
        v.get("typeKey")
            .and_then(Value::as_str)
            .map(str::to_string)
    }))
}

/// **Un diario**: todo lo que se sabe de un día, con la fecha como identidad.
///
/// Esta forma es **la nuestra** (la que se guarda), no la de un endpoint: sale de juntar lo que traen
/// los cuatro cuerpos de la API, y por eso se lee con `parsear_diarios` y **no** con `serde_json`
/// contra un cuerpo de Garmin. Los cuerpos VIVOS se leen con `sueno`, `pulso_reposo` y `estres`, y lo
/// que cada uno trae se junta con `un_diario_por_fecha`.
///
/// `training_status` y `vo2max` están declarados porque los pide el contrato, pero **ninguna función
/// de este módulo los rellena**: la librería no cita sus claves (§DUDAS y cabecera). Van en `None`
/// hasta que haya forma que citar.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Diario {
    /// La fecha del día, `YYYY-MM-DD` (`dailySleepDTO.calendarDate`, `typed.py:188`; el estrés la
    /// trae igual, `typed.py:100`). **Es la identidad** del diario: obligatoria.
    pub fecha: String,
    /// `sleepTimeSeconds` (`typed.py:190`), en **segundos**, sin formatear.
    #[serde(default)]
    pub sueno_total_s: Option<f64>,
    /// `deepSleepSeconds` (`typed.py:196`), en segundos.
    #[serde(default)]
    pub sueno_profundo_s: Option<f64>,
    /// `remSleepSeconds` (`typed.py:198`), en segundos.
    #[serde(default)]
    pub sueno_rem_s: Option<f64>,
    /// El `value` de la fila `WELLNESS_RESTING_HEART_RATE` (`__init__.py:2220`), en pulsaciones por
    /// minuto. Es un entero (`typed.py:117` lo modela `int`).
    #[serde(default)]
    pub pulso_reposo_ppm: Option<u32>,
    /// `averageStressLevel` (`typed.py:134`), entero.
    #[serde(default)]
    pub estres_promedio: Option<u32>,
    /// El training status del día. **Sin fuente citable todavía** (ver cabecera): queda en `None`.
    #[serde(default)]
    pub training_status: Option<String>,
    /// El VO2 máx del día. **Sin fuente citable todavía** (ver cabecera): queda en `None`.
    #[serde(default)]
    pub vo2max: Option<f64>,
}

/// Lo que se ha de contar, y nada más: lo que la pantalla necesita para decir «3 actividades y 2
/// diarios nuevos».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resumen {
    /// Cuántas actividades nuevas.
    pub actividades: usize,
    /// Cuántos diarios nuevos.
    pub diarios: usize,
    /// Las fechas de esos diarios, **en orden de llegada** (una por diario).
    pub fechas_de_diarios: Vec<String>,
}

/// El sueño de un día: `dailySleepData` (`__init__.py:2126-2130`) con su `dailySleepDTO`
/// (`typed.py:238`).
#[derive(Deserialize)]
struct SuenoCrudo {
    #[serde(default, rename = "dailySleepDTO")]
    dto: Option<SuenoDelDto>,
}

/// Dentro del `dailySleepDTO` (`typed.py:184-199`).
#[derive(Deserialize)]
struct SuenoDelDto {
    #[serde(default, rename = "calendarDate")]
    fecha: Option<String>,
    #[serde(default, rename = "sleepTimeSeconds")]
    total_s: Option<f64>,
    #[serde(default, rename = "deepSleepSeconds")]
    profundo_s: Option<f64>,
    #[serde(default, rename = "remSleepSeconds")]
    rem_s: Option<f64>,
}

/// El pulso en reposo de un rango (`__init__.py:2207-2218`).
#[derive(Deserialize)]
struct PulsoCrudo {
    #[serde(default, rename = "allMetrics")]
    todo: Option<TodosLosMetricos>,
}

/// `allMetrics` (`__init__.py:2216`).
#[derive(Deserialize)]
struct TodosLosMetricos {
    #[serde(default, rename = "metricsMap")]
    mapa: Option<MapaDeMetricas>,
}

/// `metricsMap` (`__init__.py:2216-2217`).
#[derive(Deserialize)]
struct MapaDeMetricas {
    #[serde(default, rename = "WELLNESS_RESTING_HEART_RATE")]
    pulso: Vec<FilaDePulso>,
}

/// Una fila de la serie (`__init__.py:2219-2223`).
#[derive(Deserialize)]
struct FilaDePulso {
    #[serde(default, rename = "calendarDate")]
    fecha: Option<String>,
    /// El valor se lee como `Value` a propósito: así una fila rara **se descarta sola** en vez de
    /// tirar abajo el cuerpo entero, que es lo que haría un tipo estricto.
    #[serde(rename = "value", default)]
    valor: Value,
}

/// El estrés de un día (`__init__.py:2173`), con la forma de `DailyStats` (`typed.py:92-138`).
#[derive(Deserialize)]
struct EstresCrudo {
    #[serde(default, rename = "calendarDate")]
    fecha: Option<String>,
    /// Igual que en el pulso: `Value`, para que un valor raro no tumbe el cuerpo.
    #[serde(default, rename = "averageStressLevel")]
    promedio: Value,
}

// ---------------------------------------------------------------------------------------------
// El fallo, tipado y acotado
// ---------------------------------------------------------------------------------------------

/// Lo que puede salir mal al leer un cuerpo: **tipado**, en español, y **sin el cuerpo dentro**.
///
/// `esperaba` dice **qué** se esperaba (en español, porque lo lee una persona); `detalle` es la
/// descripción acotada del fallo, que viene de serde (ver «LA REGLA DEL ERROR ACOTADO» en la
/// cabecera). Nunca lleva el JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FalloDeParseo {
    /// Qué se esperaba leer.
    pub esperaba: String,
    /// Qué pasó, en una línea y sin valores del cuerpo.
    pub detalle: String,
}

impl FalloDeParseo {
    /// El fallo que trae serde, ya acotado.
    fn del_json(esperaba: &str, e: &serde_json::Error) -> Self {
        FalloDeParseo {
            esperaba: esperaba.to_string(),
            detalle: acotado(e),
        }
    }

    /// El fallo de una clave que **falta** y que serde no puede echar de menos (su ausencia es legal:
    /// el objeto se lee igual, pero sin ese dato no hay identidad ni día donde colocarlo).
    fn falta(esperaba: &str, que: &str) -> Self {
        FalloDeParseo {
            esperaba: esperaba.to_string(),
            detalle: format!("falta {que}"),
        }
    }
}

impl fmt::Display for FalloDeParseo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.esperaba, self.detalle)
    }
}

impl std::error::Error for FalloDeParseo {}

/// El mensaje de serde, en **una línea** y **sin los valores** que venían en el cuerpo.
fn acotado(e: &serde_json::Error) -> String {
    let mensaje = e.to_string();
    let primera = mensaje.lines().next().unwrap_or("").trim();
    sin_valores(primera)
}

/// Quita del mensaje lo que serde **copia del cuerpo**.
///
/// Dos sitios: los tramos entre **comillas dobles** (serde escribe ahí el valor tal cual, p. ej.
/// `invalid type: string "…", expected u64`) y los tramos entre **acentos graves que son números**
/// (serde los usa para los valores numéricos, p. ej. `invalid value: integer \`5\``, mientras que los
/// **nombres** —`missing field \`activityId\``— también van entre acentos graves y **se conservan**,
/// porque son exactamente lo que hay que decir).
fn sin_valores(mensaje: &str) -> String {
    let mut salida = String::with_capacity(mensaje.len());
    let mut tramo = String::new();
    let mut entre_comillas = false;
    let mut entre_acentos = false;
    let mut escapado = false;
    for c in mensaje.chars() {
        if entre_comillas {
            if escapado {
                escapado = false;
            } else if c == '\\' {
                escapado = true;
            } else if c == '"' {
                entre_comillas = false;
                salida.push_str(VALOR_OMITIDO);
            }
            continue;
        }
        if entre_acentos {
            if c == '`' {
                entre_acentos = false;
                if es_un_numero(&tramo) {
                    salida.push_str(VALOR_OMITIDO);
                } else {
                    salida.push('`');
                    salida.push_str(&tramo);
                    salida.push('`');
                }
                tramo.clear();
            } else {
                tramo.push(c);
            }
            continue;
        }
        match c {
            '"' => entre_comillas = true,
            '`' => entre_acentos = true,
            _ => salida.push(c),
        }
    }
    // Un tramo que se quedara abierto no se copia: mejor perder texto que copiar un valor.
    salida
}

/// Lo que se escribe en el hueco de un valor: dice que **había algo ahí** y que no se copia.
const VALOR_OMITIDO: &str = "«valor omitido»";

/// ¿Este tramo entre acentos graves es un **valor** (un número) o un **nombre**? Solo se borran los
/// números: los nombres de campo hay que decirlos.
fn es_un_numero(tramo: &str) -> bool {
    !tramo.is_empty()
        && tramo
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == '+' || c == '.')
}

// ---------------------------------------------------------------------------------------------
// Las lecturas (puras: texto -> datos, sin red y sin disco)
// ---------------------------------------------------------------------------------------------

/// Las actividades de la respuesta de `search/activities`, que es una **lista JSON**
/// (`__init__.py:2627-2633`).
///
/// Un objeto envoltorio (`{"activities": [...]}`) **no vale**: la API no lo manda así, y aceptar
/// formas que no existen es la manera de no enterarse de que el endpoint ha cambiado.
pub fn parsear_actividades(json: &str) -> Result<Vec<Actividad>, FalloDeParseo> {
    serde_json::from_str::<Vec<Actividad>>(json)
        .map_err(|e| FalloDeParseo::del_json(ESPERABA_ACTIVIDADES, &e))
}

/// Los diarios **ya con nuestra forma** (la que se guarda), de una lista.
///
/// No es la forma de Garmin: para los cuerpos vivos están `sueno`, `pulso_reposo` y `estres`.
pub fn parsear_diarios(json: &str) -> Result<Vec<Diario>, FalloDeParseo> {
    serde_json::from_str::<Vec<Diario>>(json).map_err(|e| FalloDeParseo::del_json(ESPERABA_DIARIOS, &e))
}

/// El sueño de **un** día, de un cuerpo de `dailySleepData` (`__init__.py:2126-2130`).
///
/// Un cuerpo = un día (el parámetro de la petición es la fecha). Si el cuerpo no trae
/// `dailySleepDTO`, o su `calendarDate`, es que **no es la respuesta del sueño**: falla diciéndolo, en
/// vez de devolver un diario sin fecha.
pub fn sueno(cuerpo: &str) -> Result<Diario, FalloDeParseo> {
    let crudo: SuenoCrudo =
        serde_json::from_str(cuerpo).map_err(|e| FalloDeParseo::del_json(ESPERABA_SUENO, &e))?;
    let Some(dto) = crudo.dto else {
        return Err(FalloDeParseo::falta(ESPERABA_SUENO, "la clave `dailySleepDTO`"));
    };
    let Some(fecha) = dto.fecha else {
        return Err(FalloDeParseo::falta(
            ESPERABA_SUENO,
            "`dailySleepDTO.calendarDate`",
        ));
    };
    let mut diario = Diario::vacio(fecha);
    diario.sueno_total_s = dto.total_s;
    diario.sueno_profundo_s = dto.profundo_s;
    diario.sueno_rem_s = dto.rem_s;
    Ok(diario)
}

/// El pulso en reposo de **uno o varios días**, de un cuerpo de rango (`__init__.py:2207-2223`).
///
/// Devuelve un diario por **día con valor**. Las filas sin `value` se descartan, igual que en la
/// librería (`__init__.py:2222`): ese día no tiene dato, y no se rellena con nada.
pub fn pulso_reposo(cuerpo: &str) -> Result<Vec<Diario>, FalloDeParseo> {
    let crudo: PulsoCrudo =
        serde_json::from_str(cuerpo).map_err(|e| FalloDeParseo::del_json(ESPERABA_PULSO, &e))?;
    let Some(todo) = crudo.todo else {
        return Err(FalloDeParseo::falta(ESPERABA_PULSO, "la clave `allMetrics`"));
    };
    let filas = todo.mapa.map(|m| m.pulso).unwrap_or_default();
    Ok(filas
        .into_iter()
        .filter_map(|fila| {
            // Sin un entero no hay pulso que dar (y sin `calendarDate` no hay día donde ponerlo):
            // ni se redondea ni se inventa una fecha.
            let ppm = u32::try_from(fila.valor.as_u64()?).ok()?;
            let mut diario = Diario::vacio(fila.fecha?);
            diario.pulso_reposo_ppm = Some(ppm);
            Some(diario)
        })
        .collect())
}

/// El estrés de **un** día, de un cuerpo de `dailyStress` (`__init__.py:2173`).
///
/// Un cuerpo = un día. `averageStressLevel` (`typed.py:134`) puede faltar: entonces queda en `None`,
/// **no en 0** (el 0 del estrés es un valor del reloj, no un hueco). Sin `calendarDate` no hay dónde
/// poner el día, y eso sí es un fallo que se dice.
pub fn estres(cuerpo: &str) -> Result<Diario, FalloDeParseo> {
    let crudo: EstresCrudo =
        serde_json::from_str(cuerpo).map_err(|e| FalloDeParseo::del_json(ESPERABA_ESTRES, &e))?;
    let Some(fecha) = crudo.fecha else {
        return Err(FalloDeParseo::falta(ESPERABA_ESTRES, "la clave `calendarDate`"));
    };
    let mut diario = Diario::vacio(fecha);
    diario.estres_promedio = crudo.promedio.as_u64().and_then(|v| u32::try_from(v).ok());
    Ok(diario)
}

impl Diario {
    /// Un diario con **solo la fecha** puesta: el resto en `None`.
    ///
    /// Existe para que las lecturas no repitan siete `None`, y para que se vea que **lo que un cuerpo
    /// no traía no se rellena con nada**: se queda vacío hasta que otro cuerpo lo traiga.
    fn vacio(fecha: String) -> Diario {
        Diario {
            fecha,
            sueno_total_s: None,
            sueno_profundo_s: None,
            sueno_rem_s: None,
            pulso_reposo_ppm: None,
            estres_promedio: None,
            training_status: None,
            vo2max: None,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// «Qué es nuevo» y la fusión por fecha (puras, sin red y sin disco)
// ---------------------------------------------------------------------------------------------

/// **Las actividades TRAÍDAS cuyo `id` no está entre las GUARDADAS.**
///
/// La identidad es el `id` (`typed.py:403`), no el nombre ni la fecha: la misma actividad renombrada
/// **no** es nueva. El orden del resultado es **el de llegada de `traidas`** (no se ordena por id ni
/// por fecha: lo que trajo el reloj se respeta).
///
/// Una misma `id` repetida dentro de la tanda **sale una sola vez, la primera**: si la paginación se
/// solapa, guardar dos veces la misma actividad sería duplicar datos. El conjunto hace las dos cosas
/// de una vez —descartar lo guardado y lo repetido— porque `insert` dice si el id era nuevo.
pub fn que_es_nuevo(guardadas: &[Actividad], traidas: &[Actividad]) -> Vec<Actividad> {
    let mut vistas: BTreeSet<u64> = guardadas.iter().map(|a| a.id).collect();
    let mut nuevas = Vec::new();
    for actividad in traidas {
        if vistas.insert(actividad.id) {
            nuevas.push(actividad.clone());
        }
    }
    nuevas
}

/// **Los diarios TRAÍDOS cuya `fecha` no está entre las GUARDADAS.**
///
/// Un diario de la misma fecha **es el mismo diario** (la API los da por día: `typed.py:188`). Orden:
/// el de llegada de `traidos`, y una fecha repetida en la tanda sale una sola vez (la primera).
///
/// OJO, y es a propósito: esta función responde «qué fechas no están guardadas todavía». Un diario de
/// una fecha ya guardada que llegue **con más datos** (por ejemplo, el sueño que ahora sí se midió) NO
/// es nuevo por esta regla, y este módulo **no decide** si se refresca el guardado o no: eso lo decide
/// quien guarda, que es el único que sabe si su memoria es de solo añadir.
pub fn diarios_nuevos(guardadas: &[Diario], traidos: &[Diario]) -> Vec<Diario> {
    let mut vistas: BTreeSet<String> = guardadas.iter().map(|d| d.fecha.clone()).collect();
    let mut nuevos = Vec::new();
    for diario in traidos {
        if vistas.insert(diario.fecha.clone()) {
            nuevos.push(diario.clone());
        }
    }
    nuevos
}

/// **Un diario por fecha**, juntando lo que trae cada cuerpo.
///
/// Cada lectura (`sueno`, `pulso_reposo`, `estres`) devuelve diarios con **un solo dato** y el resto
/// en `None`; esto los junta por `fecha`. **No rellena huecos**: un campo que ningún cuerpo trajo se
/// queda en `None`, y si dos partes traen el mismo campo manda **la primera que lo trajo** (el orden
/// de las partes es el de la lectura, y el que lee sabe en qué orden las puso).
///
/// El orden del resultado es el de **primera aparición** de la fecha. Con siete días de ventana la
/// lista es diminuta, así que se recorre a pelo: un mapa aquí sería más código para nada.
pub fn un_diario_por_fecha(partes: &[Diario]) -> Vec<Diario> {
    let mut juntos: Vec<Diario> = Vec::new();
    for parte in partes {
        let posicion = juntos.iter().position(|d| d.fecha == parte.fecha);
        match posicion {
            Some(i) => {
                let ya = &mut juntos[i];
                if ya.sueno_total_s.is_none() {
                    ya.sueno_total_s = parte.sueno_total_s;
                }
                if ya.sueno_profundo_s.is_none() {
                    ya.sueno_profundo_s = parte.sueno_profundo_s;
                }
                if ya.sueno_rem_s.is_none() {
                    ya.sueno_rem_s = parte.sueno_rem_s;
                }
                if ya.pulso_reposo_ppm.is_none() {
                    ya.pulso_reposo_ppm = parte.pulso_reposo_ppm;
                }
                if ya.estres_promedio.is_none() {
                    ya.estres_promedio = parte.estres_promedio;
                }
                if ya.training_status.is_none() {
                    ya.training_status = parte.training_status.clone();
                }
                if ya.vo2max.is_none() {
                    ya.vo2max = parte.vo2max;
                }
            }
            None => juntos.push(parte.clone()),
        }
    }
    juntos
}

/// **Lo que se ha de contar**: cuántas actividades nuevas, cuántos diarios nuevos y de qué fechas.
///
/// Existe para que la pantalla diga «3 actividades y 2 diarios nuevos» **sin contar nada ella**: si
/// contara por su lado, habría dos maneras de contar y algún día no dirían lo mismo.
pub fn resumen_de_nuevo(actividades: &[Actividad], diarios: &[Diario]) -> Resumen {
    Resumen {
        actividades: actividades.len(),
        diarios: diarios.len(),
        fechas_de_diarios: diarios.iter().map(|d| d.fecha.clone()).collect(),
    }
}

// ---------------------------------------------------------------------------------------------
// Los textos de «qué se esperaba», en español: los lee una persona
// ---------------------------------------------------------------------------------------------

const ESPERABA_ACTIVIDADES: &str = "las actividades, de la API, son una lista JSON";
const ESPERABA_DIARIOS: &str = "los diarios guardados, con la forma de este módulo, son una lista JSON";
const ESPERABA_SUENO: &str = "el sueño de un día";
const ESPERABA_PULSO: &str = "el pulso en reposo";
const ESPERABA_ESTRES: &str = "el estrés de un día";

// ---------------------------------------------------------------------------------------------
// Pruebas: sin red, con JSON de EJEMPLO (valores inventados y reconocibles, nunca datos de nadie)
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod pruebas {
    use super::*;

    /// La forma que devuelve `search/activities` (`__init__.py:2627-2633`): una **lista**. Tres
    /// actividades inventadas: ids 1, 2 y 3. La 2 viene con `T` en la fecha y **sin** distancia (la API
    /// puede omitirla, `typed.py:415`); la 3 no trae `activityName`.
    const TRES_ACTIVIDADES: &str = r#"[
        {"activityId": 1, "activityName": "Carrera de prueba UNO", "activityType": {"typeId": 1, "typeKey": "running"},
         "startTimeLocal": "2026-10-01 07:30:00", "duration": 3600.0, "distance": 10000.0},
        {"activityId": 2, "activityName": "Fuerza de prueba DOS", "activityType": {"typeKey": "strength_training"},
         "startTimeLocal": "2026-10-02T18:00:00", "duration": 1800.0},
        {"activityId": 3, "activityType": {"typeKey": "walking"},
         "startTimeLocal": "2026-10-03 09:00:00", "duration": 1200.5, "distance": 1500.0}
    ]"#;

    /// El sueño de un día (`dailySleepDTO`, `typed.py:184-199`), con todo.
    const SUENO_DEL_DIA: &str = r#"{"dailySleepDTO": {"userProfilePK": 0, "calendarDate": "2026-10-01",
        "sleepTimeSeconds": 25200, "deepSleepSeconds": 5400, "remSleepSeconds": 7200,
        "awakeSleepSeconds": 600}, "remSleepData": true}"#;

    /// Un rango de pulso en reposo (`__init__.py:2207-2223`): dos días con valor y uno **sin** dato.
    const PULSO_DE_TRES_DIAS: &str = r#"{"allMetrics": {"metricsMap": {"WELLNESS_RESTING_HEART_RATE": [
        {"calendarDate": "2026-10-01", "value": 48},
        {"calendarDate": "2026-10-02", "value": null},
        {"calendarDate": "2026-10-03", "value": 50}]}}}"#;

    /// El estrés de un día (`typed.py:92-138`), con el nombre REAL de la clave del array que no se
    /// promedia (§DUDAS 2): que venga vacío no cambia nada de lo que sí se lee.
    const ESTRES_DEL_DIA: &str =
        r#"{"calendarDate": "2026-10-01", "averageStressLevel": 31, "stressValuesArray": []}"#;

    /// Una actividad guardada, para las pruebas de «qué es nuevo».
    fn guardada(id: u64) -> Actividad {
        Actividad {
            id,
            nombre: Some(format!("Ya la tengo {id}")),
            tipo: Some("running".to_string()),
            fecha_local: Some("2026-09-30 07:00:00".to_string()),
            duracion_s: Some(1800.0),
            distancia_m: Some(5000.0),
        }
    }

    // ---- Actividades: el parseo ----

    #[test]
    fn las_tres_actividades_de_ejemplo_se_leen_enteras() {
        let traidas = parsear_actividades(TRES_ACTIVIDADES).expect("una lista de actividades");
        assert_eq!(traidas.len(), 3, "la lista trae tres");
        assert_eq!(traidas[0].id, 1);
        assert_eq!(traidas[0].nombre.as_deref(), Some("Carrera de prueba UNO"));
        assert_eq!(traidas[0].tipo.as_deref(), Some("running"));
        assert_eq!(
            traidas[0].fecha_local.as_deref(),
            Some("2026-10-01 07:30:00"),
            "la fecha local va TAL CUAL viene, con su espacio"
        );
        assert_eq!(traidas[0].duracion_s, Some(3600.0));
        assert_eq!(traidas[0].distancia_m, Some(10000.0));
        assert_eq!(
            traidas[1].fecha_local.as_deref(),
            Some("2026-10-02T18:00:00"),
            "y tal cual con la `T`: este módulo no la normaliza"
        );
        assert_eq!(
            traidas[0].distancia_m,
            Some(10000.0),
            "los metros son metros: no se formatean"
        );
        assert_eq!(traidas[2].duracion_s, Some(1200.5), "y los segundos, segundos");
    }

    #[test]
    fn el_tipo_sale_de_dentro_de_activity_type() {
        let traidas = parsear_actividades(TRES_ACTIVIDADES).expect("una lista de actividades");
        assert_eq!(
            traidas[1].tipo.as_deref(),
            Some("strength_training"),
            "el `typeKey` está anidado (`typed.py:387-391`): hay que sacarlo de dentro"
        );
    }

    #[test]
    fn lo_que_la_api_omite_queda_en_none_y_no_en_un_relleno() {
        let traidas = parsear_actividades(
            r#"[{"activityId": 7}, {"activityId": 8, "activityType": {}, "activityName": null}]"#,
        )
        .expect("una actividad sin nombre ni fecha ni tipo sigue siendo una actividad");
        assert_eq!(traidas[0].nombre, None, "sin nombre no hay `\"\"` de relleno");
        assert_eq!(traidas[0].tipo, None, "sin tipo no hay `\"sin tipo\"` de relleno");
        assert_eq!(traidas[0].fecha_local, None);
        assert_eq!(traidas[0].duracion_s, None, "sin duración no hay un 0 de relleno");
        assert_eq!(traidas[0].distancia_m, None);
        assert_eq!(traidas[1].nombre, None, "un `null` es lo mismo que un campo ausente");
        assert_eq!(traidas[1].tipo, None, "un `activityType` vacío no da tipo");
    }

    #[test]
    fn un_type_key_que_no_es_texto_no_tira_abajo_la_lista_entera() {
        let traidas = parsear_actividades(
            r#"[{"activityId": 1, "activityType": {"typeKey": 42}},
                {"activityId": 2, "activityType": {"typeKey": "cycling"}}]"#,
        )
        .expect("la lista se lee igual");
        assert_eq!(traidas[0].tipo, None, "un tipo ilegible es «no consta»");
        assert_eq!(traidas[1].tipo.as_deref(), Some("cycling"));
    }

    #[test]
    fn una_actividad_sin_id_no_se_lee_y_el_error_dice_el_campo() {
        let fallo = parsear_actividades(r#"[{"activityName": "sin id", "duration": 10.0}]"#)
            .expect_err("sin `activityId` no hay identidad: no se puede decidir qué es nuevo");
        assert!(
            fallo.detalle.contains("activityId"),
            "el error tiene que decir el CAMPO que falta: {}",
            fallo.detalle
        );
        assert_eq!(fallo.esperaba, ESPERABA_ACTIVIDADES);
    }

    #[test]
    fn el_envoltorio_no_vale_como_lista_de_actividades() {
        let fallo = parsear_actividades(r#"{"activities": []}"#)
            .expect_err("la API manda una lista, no un objeto con `activities`");
        assert_eq!(fallo.esperaba, ESPERABA_ACTIVIDADES);
        assert!(
            fallo.detalle.contains("sequence"),
            "el detalle técnico de serde dice que esperaba una secuencia: {}",
            fallo.detalle
        );
    }

    // ---- El error acotado ----

    #[test]
    fn el_error_de_parseo_no_copia_el_cuerpo_ni_ocupa_dos_lineas() {
        const MARCADOR: &str = "MARCADOR-QUE-NO-DEBE-SALIR-4711";
        let cuerpo = format!(
            r#"[{{"activityId": "{MARCADOR}", "activityName": "{MARCADOR}"}}]"#
        );
        let fallo = parsear_actividades(&cuerpo).expect_err("un texto no es un id");
        let texto = fallo.to_string();
        assert!(
            !texto.contains(MARCADOR),
            "el error copió un valor del cuerpo: {texto}"
        );
        assert!(!texto.contains('\n'), "el error trae más de una línea: {texto}");
        assert!(
            texto.contains("u64"),
            "el error tiene que decir qué se esperaba leer: {texto}"
        );
        assert!(
            texto.contains(VALOR_OMITIDO),
            "el hueco del valor se dice, no se deja en blanco: {texto}"
        );
    }

    #[test]
    fn un_numero_del_cuerpo_fuera_de_rango_tampoco_se_copia() {
        // Un id imposible: si serde lo copiara, saldría en el mensaje.
        let fallo = parsear_actividades(r#"[{"activityId": -98765432109876}]"#)
            .expect_err("un id negativo no es un identificador");
        let texto = fallo.to_string();
        assert!(
            !texto.contains("98765432109876"),
            "el error copió un número del cuerpo: {texto}"
        );
        assert!(!texto.contains('\n'), "una sola línea: {texto}");
    }

    #[test]
    fn los_nombres_de_campo_si_se_dicen() {
        assert_eq!(
            sin_valores("missing field `activityId` at line 1 column 20"),
            "missing field `activityId` at line 1 column 20",
            "un nombre de campo es lo que hay que decir: no se toca"
        );
        assert_eq!(
            sin_valores(r#"invalid type: string "lo que venía", expected u64 at line 1 column 21"#),
            "invalid type: string «valor omitido», expected u64 at line 1 column 21",
            "un valor, no: ese se omite"
        );
    }

    // ---- «Qué es nuevo» ----

    #[test]
    fn con_todo_guardado_no_hay_nada_nuevo() {
        let guardadas = vec![guardada(1), guardada(2), guardada(3)];
        let traidas = parsear_actividades(TRES_ACTIVIDADES).expect("la lista de ejemplo");
        assert!(que_es_nuevo(&guardadas, &traidas).is_empty());
    }

    #[test]
    fn una_sola_nueva() {
        let guardadas = vec![guardada(1), guardada(3)];
        let traidas = parsear_actividades(TRES_ACTIVIDADES).expect("la lista de ejemplo");
        let nuevas = que_es_nuevo(&guardadas, &traidas);
        assert_eq!(nuevas.len(), 1);
        assert_eq!(nuevas[0].id, 2, "la que no estaba es la 2");
    }

    #[test]
    fn todas_nuevas_cuando_no_hay_nada_guardado() {
        let traidas = parsear_actividades(TRES_ACTIVIDADES).expect("la lista de ejemplo");
        let nuevas = que_es_nuevo(&[], &traidas);
        assert_eq!(nuevas.len(), 3, "la primera sincronización trae todo");
    }

    #[test]
    fn la_identidad_es_el_id_no_el_nombre() {
        let guardadas = vec![guardada(1)];
        let renombrada = Actividad {
            nombre: Some("Le he cambiado el nombre".to_string()),
            fecha_local: Some("2027-01-01 00:00:00".to_string()),
            ..guardada(1)
        };
        assert!(
            que_es_nuevo(&guardadas, &[renombrada]).is_empty(),
            "el mismo id con otro nombre NO es una actividad nueva"
        );
    }

    #[test]
    fn el_mismo_id_dos_veces_en_la_tanda_sale_una_vez_y_manda_la_primera() {
        let primera = guardada(5);
        let segunda = Actividad {
            nombre: Some("La misma, otra vez".to_string()),
            ..guardada(5)
        };
        let nuevas = que_es_nuevo(&[], &[primera.clone(), segunda]);
        assert_eq!(nuevas.len(), 1, "una id repetida no se guarda dos veces");
        assert_eq!(
            nuevas[0].nombre.as_deref(),
            Some("Ya la tengo 5"),
            "manda la primera que llegó"
        );
    }

    #[test]
    fn el_orden_de_las_nuevas_es_el_de_llegada() {
        let traidas = parsear_actividades(
            r#"[{"activityId": 9}, {"activityId": 4}, {"activityId": 7}]"#,
        )
        .expect("la lista de ejemplo");
        let nuevas = que_es_nuevo(&[], &traidas);
        let ids: Vec<u64> = nuevas.iter().map(|a| a.id).collect();
        assert_eq!(ids, vec![9, 4, 7], "no se ordena por id ni por fecha");
    }

    // ---- Los cuerpos de la API: sueño, pulso, estrés ----

    #[test]
    fn el_sueno_de_un_dia_se_lee_entero() {
        let diario = sueno(SUENO_DEL_DIA).expect("un cuerpo de sueño");
        assert_eq!(diario.fecha, "2026-10-01");
        assert_eq!(diario.sueno_total_s, Some(25200.0));
        assert_eq!(diario.sueno_profundo_s, Some(5400.0), "`deepSleepSeconds`");
        assert_eq!(diario.sueno_rem_s, Some(7200.0), "`remSleepSeconds`");
        assert_eq!(diario.pulso_reposo_ppm, None, "el sueño no trae el pulso en reposo");
        assert_eq!(diario.estres_promedio, None);
        assert_eq!(diario.training_status, None, "y menos aún el training status");
        assert_eq!(diario.vo2max, None);
    }

    #[test]
    fn el_sueno_sin_daily_sleep_dto_no_es_un_cuerpo_de_sueno() {
        let fallo = sueno(r#"{"remSleepData": true}"#)
            .expect_err("sin `dailySleepDTO` no hay sueño que leer");
        assert_eq!(fallo.esperaba, ESPERABA_SUENO);
        assert!(fallo.detalle.contains("dailySleepDTO"), "{}", fallo.detalle);
    }

    #[test]
    fn el_sueno_sin_calendar_date_no_se_puede_colocar_en_un_dia() {
        let fallo = sueno(r#"{"dailySleepDTO": {"sleepTimeSeconds": 25200}}"#)
            .expect_err("sin fecha no hay día al que apuntar el sueño");
        assert!(fallo.detalle.contains("calendarDate"), "{}", fallo.detalle);
    }

    /// **El 0 de una noche, tal cual viene.** Decisión consciente y reversible: este módulo traduce
    /// FORMAS, no significados. El instrumento hermano sí trata el 0 de `sleepTimeSeconds` como «sin
    /// dato» (`bin/sincronizacion.rs`, `sueno_tiene_dato`); las dos cosas no pueden ser verdad a la
    /// vez, y el sitio de decidirlo es uno solo (§DUDAS 1). Si el dueño dice que el 0 es un hueco,
    /// esta prueba es la que hay que cambiar.
    #[test]
    fn el_cero_del_sueno_se_queda_como_viene() {
        let diario = sueno(r#"{"dailySleepDTO": {"calendarDate": "2026-10-02", "sleepTimeSeconds": 0}}"#)
            .expect("un cuerpo con ceros se lee igual");
        assert_eq!(
            diario.sueno_total_s,
            Some(0.0),
            "el parser no convierte el 0 en «no consta»: eso se decide donde se usa el dato"
        );
        assert_eq!(diario.sueno_profundo_s, None, "pero lo que NO viene es None, no 0");
    }

    #[test]
    fn el_pulso_en_reposo_de_un_rango_da_un_diario_por_dia_con_valor() {
        let diarios = pulso_reposo(PULSO_DE_TRES_DIAS).expect("un cuerpo de pulso en reposo");
        let fechas: Vec<&str> = diarios.iter().map(|d| d.fecha.as_str()).collect();
        assert_eq!(
            fechas,
            vec!["2026-10-01", "2026-10-03"],
            "el día sin valor no aparece: la librería también lo descarta (__init__.py:2222)"
        );
        assert_eq!(diarios[0].pulso_reposo_ppm, Some(48));
        assert_eq!(diarios[1].pulso_reposo_ppm, Some(50));
        assert_eq!(diarios[0].sueno_total_s, None, "el pulso no trae el sueño");
    }

    #[test]
    fn el_pulso_sin_all_metrics_no_es_un_cuerpo_de_pulso() {
        let fallo = pulso_reposo(r#"{"algoMas": 1}"#)
            .expect_err("sin `allMetrics` esa respuesta no es la del pulso");
        assert!(fallo.detalle.contains("allMetrics"), "{}", fallo.detalle);
    }

    #[test]
    fn un_all_metrics_sin_la_serie_da_cero_dias_y_no_un_fallo() {
        let diarios = pulso_reposo(r#"{"allMetrics": {"metricsMap": {}}}"#)
            .expect("una serie vacía es que no hay días con dato, no que el endpoint haya cambiado");
        assert!(diarios.is_empty());
    }

    #[test]
    fn una_fila_de_pulso_sin_fecha_no_se_coloca_en_ningun_dia() {
        let diarios = pulso_reposo(
            r#"{"allMetrics": {"metricsMap": {"WELLNESS_RESTING_HEART_RATE": [{"value": 55}]}}}"#,
        )
        .expect("la fila se descarta, pero el cuerpo es legible");
        assert!(
            diarios.is_empty(),
            "sin `calendarDate` no se inventa una fecha: se descarta la fila"
        );
    }

    #[test]
    fn el_estres_de_un_dia_se_lee_por_calendar_date() {
        let diario = estres(ESTRES_DEL_DIA).expect("un cuerpo de estrés");
        assert_eq!(diario.fecha, "2026-10-01");
        assert_eq!(diario.estres_promedio, Some(31));
    }

    #[test]
    fn el_estres_sin_average_stress_level_queda_en_none_y_no_en_cero() {
        let diario = estres(r#"{"calendarDate": "2026-10-04"}"#)
            .expect("el día sin estrés medido se lee igual");
        assert_eq!(
            diario.estres_promedio, None,
            "un estrés que no viene es None: el 0 sería un dato del reloj, no un hueco"
        );
    }

    #[test]
    fn el_estres_sin_fecha_no_se_puede_colocar_en_un_dia() {
        let fallo = estres(r#"{"averageStressLevel": 31}"#)
            .expect_err("sin `calendarDate` no hay día");
        assert!(fallo.detalle.contains("calendarDate"), "{}", fallo.detalle);
    }

    // ---- La fusión por fecha ----

    #[test]
    fn los_cuerpos_del_mismo_dia_se_juntan_en_un_diario_por_fecha() {
        let mut partes = Vec::new();
        partes.push(sueno(SUENO_DEL_DIA).expect("el sueño del 2026-10-01"));
        partes.push(estres(ESTRES_DEL_DIA).expect("el estrés del 2026-10-01"));
        partes.extend(pulso_reposo(PULSO_DE_TRES_DIAS).expect("el pulso del rango"));
        let juntos = un_diario_por_fecha(&partes);

        let fechas: Vec<&str> = juntos.iter().map(|d| d.fecha.as_str()).collect();
        assert_eq!(
            fechas,
            vec!["2026-10-01", "2026-10-03"],
            "dos días: el 01 lo traen tres cuerpos, el 03 solo el pulso"
        );
        let primero = &juntos[0];
        assert_eq!(primero.sueno_total_s, Some(25200.0));
        assert_eq!(primero.estres_promedio, Some(31));
        assert_eq!(primero.pulso_reposo_ppm, Some(48));
        assert_eq!(primero.training_status, None, "nadie lo trajo: se queda en None");
        let segundo = &juntos[1];
        assert_eq!(segundo.pulso_reposo_ppm, Some(50));
        assert_eq!(
            segundo.sueno_total_s, None,
            "un día que solo está en el pulso sale con lo demás en None, no con ceros"
        );
        assert_eq!(segundo.estres_promedio, None);
    }

    #[test]
    fn el_primero_que_trae_un_dato_manda_al_juntar() {
        let primera = sueno(r#"{"dailySleepDTO": {"calendarDate": "2026-10-01", "sleepTimeSeconds": 25200}}"#)
            .expect("el primer sueño");
        let segunda = sueno(r#"{"dailySleepDTO": {"calendarDate": "2026-10-01", "sleepTimeSeconds": 1}}"#)
            .expect("el segundo sueño");
        let juntos = un_diario_por_fecha(&[primera, segunda]);
        assert_eq!(juntos.len(), 1, "misma fecha: un solo diario");
        assert_eq!(juntos[0].sueno_total_s, Some(25200.0), "manda el primero");
    }

    #[test]
    fn juntar_nada_da_nada() {
        assert!(un_diario_por_fecha(&[]).is_empty());
    }

    // ---- Los diarios guardados: qué es nuevo y el resumen ----

    /// Los diarios **ya con nuestra forma** (la que se guarda), incluidos los dos campos que ninguna
    /// función de este módulo rellena: así se prueba que la forma guardada los conserva.
    const DIARIOS_GUARDADOS: &str = r#"[
        {"fecha": "2026-09-30", "sueno_total_s": 25200.0, "sueno_profundo_s": 5400.0,
         "sueno_rem_s": 7200.0, "pulso_reposo_ppm": 48, "estres_promedio": 31,
         "training_status": "PRODUCTIVE", "vo2max": 51.0},
        {"fecha": "2026-10-01", "pulso_reposo_ppm": 49}
    ]"#;

    #[test]
    fn los_diarios_guardados_se_vuelven_a_leer_con_su_forma() {
        let diarios = parsear_diarios(DIARIOS_GUARDADOS).expect("una lista de diarios");
        assert_eq!(diarios.len(), 2);
        assert_eq!(diarios[0].fecha, "2026-09-30");
        assert_eq!(diarios[0].training_status.as_deref(), Some("PRODUCTIVE"));
        assert_eq!(diarios[0].vo2max, Some(51.0));
        assert_eq!(diarios[1].sueno_total_s, None, "lo que no venía es None");
        assert_eq!(
            diarios[1].estres_promedio, None,
            "y el 49 del pulso no se copia a ningún otro sitio"
        );
    }

    #[test]
    fn ninguna_lectura_de_la_api_rellena_training_status_ni_vo2max() {
        // La librería no cita la forma de ese endpoint (`__init__.py:2478-2483`): hasta que la cite,
        // estos dos campos NO se llenan desde ningún cuerpo, y esta prueba lo deja escrito.
        let del_sueno = sueno(SUENO_DEL_DIA).expect("el sueño");
        let del_estres = estres(ESTRES_DEL_DIA).expect("el estrés");
        for diario in [&del_sueno, &del_estres] {
            assert_eq!(diario.training_status, None);
            assert_eq!(diario.vo2max, None);
        }
        let del_pulso = pulso_reposo(PULSO_DE_TRES_DIAS).expect("el pulso");
        assert!(del_pulso
            .iter()
            .all(|d| d.training_status.is_none() && d.vo2max.is_none()));
    }

    #[test]
    fn un_diario_de_la_misma_fecha_no_es_nuevo() {
        let guardados = parsear_diarios(DIARIOS_GUARDADOS).expect("los guardados");
        let traidos = vec![
            estres(ESTRES_DEL_DIA).expect("el estrés del 2026-10-01, que ya está guardado"),
            estres(r#"{"calendarDate": "2026-10-05", "averageStressLevel": 20}"#)
                .expect("el del 2026-10-05, que no"),
        ];
        let nuevos = diarios_nuevos(&guardados, &traidos);
        assert_eq!(nuevos.len(), 1);
        assert_eq!(
            nuevos[0].fecha, "2026-10-05",
            "el mismo día es el mismo diario, aunque llegue con otro dato"
        );
    }

    #[test]
    fn con_nada_guardado_todos_los_diarios_son_nuevos() {
        let traidos = pulso_reposo(PULSO_DE_TRES_DIAS).expect("el pulso del rango");
        assert_eq!(diarios_nuevos(&[], &traidos).len(), 2);
    }

    #[test]
    fn la_misma_fecha_dos_veces_en_la_tanda_sale_una_vez() {
        let traidos = vec![
            estres(ESTRES_DEL_DIA).expect("el estrés"),
            sueno(SUENO_DEL_DIA).expect("el sueño del mismo día"),
        ];
        assert_eq!(
            diarios_nuevos(&[], &traidos).len(),
            1,
            "los dos son del 2026-10-01: es UN diario nuevo, no dos"
        );
    }

    #[test]
    fn el_resumen_cuenta_lo_que_hay_y_da_las_fechas_en_orden_de_llegada() {
        let actividades = parsear_actividades(TRES_ACTIVIDADES).expect("las actividades");
        let diarios = un_diario_por_fecha(
            &pulso_reposo(PULSO_DE_TRES_DIAS).expect("el pulso del rango"),
        );
        let resumen = resumen_de_nuevo(&actividades, &diarios);
        assert_eq!(resumen.actividades, 3);
        assert_eq!(resumen.diarios, 2);
        assert_eq!(resumen.fechas_de_diarios, vec!["2026-10-01", "2026-10-03"]);
        assert_eq!(
            format!("{} actividades y {} diarios nuevos", resumen.actividades, resumen.diarios),
            "3 actividades y 2 diarios nuevos",
            "que es la frase que quiere decir la pantalla, sin contar nada ella"
        );
    }

    #[test]
    fn el_resumen_de_nada_son_ceros_y_ninguna_fecha() {
        let resumen = resumen_de_nuevo(&[], &[]);
        assert_eq!(resumen.actividades, 0);
        assert_eq!(resumen.diarios, 0);
        assert!(resumen.fechas_de_diarios.is_empty());
    }
}
