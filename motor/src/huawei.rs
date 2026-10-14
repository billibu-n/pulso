//! **El adaptador de la exportación de salud de Huawei.**
//!
//! QUÉ ES Y DE QUÉ CAPA: es un **adaptador** (capa 2 de 5). Traduce el formato de fuera al modelo del
//! motor, y **no decide nada**. El motor **no conoce** este fichero: el día que Huawei cambie su
//! formato, se cambia **esta pieza** y no se toca una línea del centro.
//!
//! QUÉ RECIBE: la **ruta de un ZIP tal cual sale del correo de Huawei**, o la de una **carpeta** ya
//! descomprimida. La pieza decide sola: si es un directorio, lo recorre; si es un fichero, lo abre como
//! ZIP. **Abrir el ZIP es trabajo de la aplicación, no del cliente** (decisión del dueño, 2026-10-14:
//! «la idea es minimizar el trabajo del cliente»); se lee en memoria y no se escribe nada en disco.
//!
//! QUÉ DEVUELVE: un resultado con una lista de «Composicion», una medición por fecha, en kg y %.
//!
//! QUÉ NO HACE: no calcula medias ni tendencias; no decide avisos; no escribe; no toca la red; no conoce
//! a «Persona» ni a «Hoy». **Un campo que no viene queda en «Falta»; NUNCA en un cero.**
//!
//! DE DÓNDE SALE EL FORMATO: «docs/HUAWEI-exportacion.md», medido contra el XLSX de descripción del
//! propio export, tres parsers independientes y la API de Salud de Huawei. El contrato está en
//! «docs/CONTRATO-huawei.md», y se escribió **antes** que este código.

use crate::datos::{Composicion, Dato, Fecha};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// La clave del punto que trae la composición corporal. **Se filtra por ella, no por «type»**: el mismo
/// dato aparece como type 10006 y como type 8, así que filtrar por type perdería mediciones.
pub const CLAVE_COMPOSICION: &str = "WEIGHT_BODYFAT_BROAD";

/// Lo que puede salir mal. **Tipado**, para que quien llame decida, en vez de un texto suelto.
#[derive(Debug)]
pub enum ErrorHuawei {
    /// La carpeta no existe.
    NoExiste(PathBuf),
    /// El fichero no es un ZIP legible (o está corrupto).
    NoEsUnZip(String),
    /// La carpeta existe pero no tiene ningún fichero de datos de Huawei.
    SinFicheros(PathBuf),
    /// El texto no es JSON, o no tiene la forma del export.
    Formato(String),
    /// Un fichero no se pudo leer.
    Io(String),
}

impl fmt::Display for ErrorHuawei {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorHuawei::NoExiste(p) => write!(f, "no existe la carpeta {}", p.display()),
            ErrorHuawei::NoEsUnZip(m) => write!(f, "no se pudo abrir el ZIP: {m}"),
            ErrorHuawei::SinFicheros(p) => write!(
                f,
                "en {} no hay ningún fichero health detail data",
                p.display()
            ),
            ErrorHuawei::Formato(m) => write!(f, "el JSON no tiene la forma del export: {m}"),
            ErrorHuawei::Io(m) => write!(f, "no se pudo leer: {m}"),
        }
    }
}

impl std::error::Error for ErrorHuawei {}

// ---------------------------------------------------------------------------------------------
// La forma del fichero, tal como la mide docs/HUAWEI-exportacion.md
// ---------------------------------------------------------------------------------------------

#[derive(Deserialize)]
struct Registro {
    /// El id del dispositivo. **No se filtra por type**, que varía; sí se usa para deduplicar.
    #[serde(rename = "deviceCode", default)]
    device_code: Option<i64>,
    /// La zona del registro, +HHMM o -HHMM. **No se deduce del reloj de la máquina.**
    #[serde(rename = "timeZone", default)]
    time_zone: Option<String>,
    #[serde(rename = "samplePoints", default)]
    sample_points: Vec<Punto>,
}

#[derive(Deserialize)]
struct Punto {
    #[serde(default)]
    key: Option<String>,
    /// Inicio de la medida, en **milisegundos epoch UTC**.
    #[serde(rename = "startTime", default)]
    start_time: Option<i64>,
    /// La carga útil. **Puede ser un texto que contiene JSON** (doble codificación) o ya un objeto.
    #[serde(default)]
    value: Option<serde_json::Value>,
}

/// Los campos de composición que el motor necesita, **todos opcionales**.
///
/// Lo que no se modela aquí **no se pierde por el camino**: el export completo sigue en su carpeta. Esta
/// pieza solo entrega lo que Composicion declara, y no inventa el resto.
#[derive(Deserialize, Default)]
struct Cuerpo {
    #[serde(rename = "bodyWeight", default)]
    peso_kg: Option<f64>,
    #[serde(rename = "bodyFatRate", default)]
    grasa_pct: Option<f64>,
    #[serde(rename = "moistureRate", default)]
    agua_pct: Option<f64>,
    #[serde(rename = "muscleMass", default)]
    musculo_kg: Option<f64>,
    #[serde(rename = "boneSalt", default)]
    hueso_kg: Option<f64>,
}

impl Cuerpo {
    fn a_composicion(&self, ms: i64, tz: &str) -> Composicion {
        Composicion {
            fecha: fecha_desde_ms(ms, tz),
            peso_kg: dato_kg(self.peso_kg),
            grasa_corporal_pct: dato_pct(self.grasa_pct),
            agua_corporal_pct: dato_pct(self.agua_pct),
            masa_muscular_kg: dato_kg(self.musculo_kg),
            masa_osea_kg: dato_kg(self.hueso_kg),
        }
    }
}

/// Un número en kg **solo si es plausible**; si no, queda en Falta. Ver Composicion::plausible_kg.
fn dato_kg(v: Option<f64>) -> Dato<f32> {
    v.and_then(|x| Composicion::plausible_kg(x as f32))
        .map(Dato::Hay)
        .unwrap_or(Dato::Falta)
}

/// Lo mismo para un porcentaje.
fn dato_pct(v: Option<f64>) -> Dato<f32> {
    v.and_then(|x| Composicion::plausible_pct(x as f32))
        .map(Dato::Hay)
        .unwrap_or(Dato::Falta)
}

// ---------------------------------------------------------------------------------------------
// La lectura
// ---------------------------------------------------------------------------------------------

/// **Lee el texto de un fichero health detail data** y saca las composiciones corporales.
///
/// Es una **función pura**: por eso se puede probar sin tocar el disco y sin ningún ZIP. leer_carpeta
/// la usa por dentro.
pub fn leer_texto(texto: &str) -> Result<Vec<Composicion>, ErrorHuawei> {
    // Un array de registros, o un solo registro suelto (el export aparece de las dos formas).
    let registros: Vec<Registro> = match serde_json::from_str(texto) {
        Ok(v) => v,
        Err(_) => match serde_json::from_str::<Registro>(texto) {
            Ok(r) => vec![r],
            Err(e) => return Err(ErrorHuawei::Formato(e.to_string())),
        },
    };

    let mut salida: Vec<Composicion> = Vec::new();
    // El mismo dato puede venir como type 8 y type 10006: se deduplica por (dispositivo, instante).
    let mut vistos: BTreeSet<(Option<i64>, i64)> = BTreeSet::new();

    for r in &registros {
        let tz = r.time_zone.as_deref().unwrap_or("+0000");
        for p in &r.sample_points {
            if p.key.as_deref() != Some(CLAVE_COMPOSICION) {
                continue;
            }
            let Some(ms) = p.start_time else { continue };
            if !vistos.insert((r.device_code, ms)) {
                continue;
            }
            let Some(cuerpo) = cuerpo_del_punto(p.value.as_ref()) else {
                continue;
            };
            salida.push(cuerpo.a_composicion(ms, tz));
        }
    }

    ordenar(&mut salida);
    Ok(salida)
}

/// **La puerta única de esta pieza**: acepta un ZIP o una carpeta y saca las mediciones.
///
/// Existe para que quien use el adaptador **no tenga que saber** en qué forma llegó la exportación: si
/// es un directorio, lo recorre; si es un fichero, lo abre como ZIP. **El cliente no descomprime nada.**
pub fn leer(ruta: &Path) -> Result<Vec<Composicion>, ErrorHuawei> {
    if !ruta.exists() {
        return Err(ErrorHuawei::NoExiste(ruta.to_path_buf()));
    }
    if ruta.is_dir() {
        leer_carpeta(ruta)
    } else {
        leer_zip(ruta)
    }
}

/// **Lee un ZIP de exportación directamente**, sin descomprimirlo a disco.
pub fn leer_zip(ruta: &Path) -> Result<Vec<Composicion>, ErrorHuawei> {
    let archivo = std::fs::File::open(ruta).map_err(|e| ErrorHuawei::Io(e.to_string()))?;
    let mut zip = zip::ZipArchive::new(archivo).map_err(|e| ErrorHuawei::NoEsUnZip(e.to_string()))?;

    let mut todas: Vec<Composicion> = Vec::new();
    let mut encontrados = 0usize;
    for i in 0..zip.len() {
        // Un punto ilegible NO aborta el resto: se ignora y se sigue (misma regla que en la carpeta).
        let Ok(mut f) = zip.by_index(i) else { continue };
        if !f.is_file() || !es_fichero_de_datos(Path::new(f.name())) {
            continue;
        }
        encontrados += 1;
        let mut texto = String::new();
        if std::io::Read::read_to_string(&mut f, &mut texto).is_err() {
            continue;
        }
        if let Ok(cs) = leer_texto(&texto) {
            todas.extend(cs);
        }
    }
    if encontrados == 0 {
        return Err(ErrorHuawei::SinFicheros(ruta.to_path_buf()));
    }
    ordenar(&mut todas);
    Ok(todas)
}

/// **Lee una carpeta de exportación** (ya descomprimida) y junta todas las mediciones.
pub fn leer_carpeta(carpeta: &Path) -> Result<Vec<Composicion>, ErrorHuawei> {
    if !carpeta.is_dir() {
        return Err(ErrorHuawei::NoExiste(carpeta.to_path_buf()));
    }

    let mut ficheros: Vec<PathBuf> = Vec::new();
    buscar_json(carpeta, &mut ficheros);
    if ficheros.is_empty() {
        return Err(ErrorHuawei::SinFicheros(carpeta.to_path_buf()));
    }
    ficheros.sort();

    let mut todas: Vec<Composicion> = Vec::new();
    for f in ficheros {
        // **Un fichero ilegible o corrupto NO aborta el resto**: se ignora y se sigue. Un export con un
        // lote roto no puede dejar sin datos los otros diez.
        let Ok(texto) = std::fs::read_to_string(&f) else {
            continue;
        };
        if let Ok(cs) = leer_texto(&texto) {
            todas.extend(cs.iter().copied());
        }
    }

    ordenar(&mut todas);
    Ok(todas)
}

/// Orden determinista: de la medición más antigua a la más reciente.
fn ordenar(v: &mut [Composicion]) {
    v.sort_by_key(|c| match c.fecha {
        Dato::Hay(f) => (f.anio as i64, f.mes as i64, f.dia as i64),
        Dato::Falta => (i64::MAX, 0, 0),
    });
}

/// Recorre la carpeta **a cualquier profundidad**: el export puede venir anidado bajo
/// HUAWEI_HEALTH_<fecha>/, y la carpeta buena se busca por nombre, no por nivel fijo.
fn buscar_json(dir: &Path, salida: &mut Vec<PathBuf>) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            buscar_json(&p, salida);
        } else if es_fichero_de_datos(&p) {
            salida.push(p);
        }
    }
}

fn es_fichero_de_datos(p: &Path) -> bool {
    let Some(nombre) = p.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let n = nombre.to_lowercase();
    n.starts_with("health detail data") && n.ends_with(".json")
}

/// Parsea la carga útil de un punto: **texto con JSON dentro, o ya un objeto**.
///
/// Un value vacío, "null" o "{}" **no es una medición**: se descarta el punto, y no se devuelve un cero.
fn cuerpo_del_punto(v: Option<&serde_json::Value>) -> Option<Cuerpo> {
    let v = v?;
    match v {
        serde_json::Value::String(s) => {
            let t = s.trim();
            if t.is_empty() || t == "null" || t == "{}" {
                return None;
            }
            serde_json::from_str::<Cuerpo>(s).ok()
        }
        serde_json::Value::Object(_) => serde_json::from_value::<Cuerpo>(v.clone()).ok(),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------
// La fecha: ms epoch + la zona DEL REGISTRO (no la del equipo)
// ---------------------------------------------------------------------------------------------

/// **La fecha local de una medida**, a partir de los ms UTC y la zona del propio registro.
///
/// POR QUÉ NO SE USA EL RELOJ DE LA MÁQUINA: el export trae su propia zona, y la fecha de un pesaje es la
/// del lugar donde se hizo. Deducirla del equipo daría un día distinto al viajar, que es justo el error
/// que el proyecto ya corrigió con la fecha de nacimiento.
pub fn fecha_desde_ms(ms: i64, tz: &str) -> Dato<Fecha> {
    let local = ms + offset_en_ms(tz);
    let dias = local.div_euclid(86_400_000);
    let f = fecha_desde_dias(dias);
    if f.es_valida() {
        Dato::Hay(f)
    } else {
        Dato::Falta
    }
}

/// El desfase de "+0300" / "-0330" en milisegundos. Una zona que no se entiende vale 0 (UTC), y no se
/// inventa un desfase.
fn offset_en_ms(tz: &str) -> i64 {
    let t = tz.trim();
    let (signo, resto) = match t.strip_prefix('-') {
        Some(r) => (-1i64, r),
        None => (1i64, t.strip_prefix('+').unwrap_or(t)),
    };
    let digitos: String = resto.chars().filter(|c| c.is_ascii_digit()).collect();
    if digitos.len() < 4 {
        return 0;
    }
    let h: i64 = digitos[0..2].parse().unwrap_or(0);
    let m: i64 = digitos[2..4].parse().unwrap_or(0);
    signo * (h * 3600 + m * 60) * 1000
}

/// El día de calendario a partir de los días transcurridos desde 1970-01-01.
///
/// Es el algoritmo **civil-from-days** (Howard Hinnant), que evita depender de una biblioteca de fechas
/// para convertir un entero en tres números. **Se prueba en tests/huawei.rs**, incluido un cambio de
/// año, porque una fórmula de calendario escrita a mano es exactamente lo que hay que probar.
fn fecha_desde_dias(z: i64) -> Fecha {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as i64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8; // [1, 12]
    let anio = if m <= 2 { y + 1 } else { y };
    Fecha::nueva(anio.max(0) as u16, m, d)
}
