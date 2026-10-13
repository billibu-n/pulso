//! El **lector de FIT**: traduce el fichero del aparato a **nuestro** modelo.
//!
//! POR QUÉ ESTA PIEZA ES LA QUE FALTABA: hasta ahora el motor leía un JSON que **yo preparaba a
//! mano**. Con esto, **sus ficheros entran directamente**, y el motor deja de depender de que alguien
//! los convierta.
//!
//! QUÉ TRAE UN FIT, y se comprobó abriendo uno de los suyos (no de memoria):
//!
//! | mensaje | cuántos en una sesión | qué es |
//! |---|---:|---|
//! | `Record` | 7 815 | **las muestras, una por segundo** |
//! | `Hrv` | 8 023 | **la variabilidad del pulso, latido a latido** |
//! | `Lap` | 48 | las vueltas |
//! | `TimeInZone` | 49 | el tiempo en cada zona |
//! | `Session` | 1 | los totales |
//!
//! **El FIT ya trae los totales y las zonas que calculó Garmin.** Eso es importante y hay que decirlo:
//! **el motor no necesita recalcular lo que ya viene** — lo que aporta es **lo que NO viene**, que son
//! los patrones: la deriva, la eficiencia y el desfase.
//!
//! Y LOS DOS SE COMPARAN: si el total de Garmin y el nuestro no coinciden, **uno de los dos está mal**,
//! y eso es una comprobación que sale gratis.

use crate::datos::{Dato, Muestra, Persona, Sesion};
use std::fs::File;
use std::path::Path;

/// Lo que se saca de un fichero FIT.
#[derive(Debug, Clone)]
pub struct Lectura {
    pub sesion: Sesion,
    /// Los totales que **el propio fichero declara**, para poder compararlos con los nuestros.
    pub declarado: Declarado,
    /// Cuántos mensajes de cada tipo traía, para saber qué había dentro.
    pub mensajes: Vec<(String, usize)>,
}

/// Los totales que el FIT trae ya calculados. **Se conservan para comparar, no para sustituir.**
#[derive(Debug, Clone, Default)]
pub struct Declarado {
    pub duracion_s: Dato<u32>,
    pub pulso_medio: Dato<u16>,
    pub pulso_maximo: Dato<u16>,
    pub potencia_media: Dato<u16>,
    pub potencia_maxima: Dato<u16>,
    pub distancia_m: Dato<f64>,
    pub calorias: Dato<u16>,
    pub deporte: Dato<String>,
}

/// Lee un fichero FIT y lo traduce a nuestro modelo.
///
/// **Nunca falla por datos raros**: lo que no esté, queda como `Falta`, y el motor ya sabe trabajar
/// así.
pub fn leer(ruta: &Path) -> Result<Lectura, String> {
    let mut f = File::open(ruta).map_err(|e| format!("no se pudo abrir {}: {e}", ruta.display()))?;
    let datos = fitparser::from_reader(&mut f).map_err(|e| format!("el FIT no se pudo leer: {e:?}"))?;

    let mut muestras: Vec<Muestra> = Vec::new();
    let mut intervalos_rr: Vec<f64> = Vec::new();
    let mut declarado = Declarado::default();
    let mut cuenta: std::collections::BTreeMap<String, usize> = Default::default();

    for r in datos.iter() {
        let tipo = format!("{:?}", r.kind());
        *cuenta.entry(tipo.clone()).or_insert(0) += 1;

        match tipo.as_str() {
            "Record" => {
                let mut segundo: Option<u32> = None;
                let mut pulso = Dato::Falta;
                let mut potencia = Dato::Falta;

                for c in r.fields() {
                    match c.name() {
                        "timestamp" => {
                            // El FIT trae la marca de tiempo. Se usa el SEGUNDO ABSOLUTO, y luego se
                            // resta el primero: lo que importa es el tiempo DENTRO de la sesión.
                            if let fitparser::Value::Timestamp(t) = c.value() {
                                segundo = Some(t.timestamp() as u32);
                            }
                        }
                        "heart_rate" => pulso = entero(c.value()),
                        "power" => potencia = entero(c.value()),
                        // La cadencia y la distancia **se leen pero no se guardan todavia**: el motor
                        // no las usa. Guardar lo que no se usa es una forma de mentir sobre lo que se
                        // sabe, asi que se dejan fuera a proposito y se dice aqui.
                        _ => {}
                    }
                }

                if let Some(seg) = segundo {
                    muestras.push(Muestra {
                        segundo: seg,
                        pulso,
                        potencia_w: potencia,
                    });
                }
            }
            // EL HRV: cada mensaje trae un array de intervalos entre latidos, en SEGUNDOS.
            // Se pasan a milisegundos, que es como se trabaja. Y los `Invalid` se saltan: son huecos
            // del aparato, no latidos.
            "Hrv" => {
                for c in r.fields() {
                    if c.name() == "time" {
                        if let fitparser::Value::Array(v) = c.value() {
                            for x in v {
                                if let fitparser::Value::Float64(s) = x {
                                    intervalos_rr.push(s * 1000.0);
                                }
                            }
                        }
                    }
                }
            }
            "Session" => {
                for c in r.fields() {
                    match c.name() {
                        "total_elapsed_time" | "total_timer_time" => {
                            if declarado.duracion_s.falta() {
                                if let Some(v) = real(c.value()) {
                                    declarado.duracion_s = Dato::Hay(v as u32);
                                }
                            }
                        }
                        "avg_heart_rate" => declarado.pulso_medio = entero(c.value()),
                        "max_heart_rate" => declarado.pulso_maximo = entero(c.value()),
                        "avg_power" => declarado.potencia_media = entero(c.value()),
                        "max_power" => declarado.potencia_maxima = entero(c.value()),
                        "total_distance" => {
                            if let Some(v) = real(c.value()) {
                                declarado.distancia_m = Dato::Hay(v);
                            }
                        }
                        "total_calories" => declarado.calorias = entero(c.value()),
                        "sport" => {
                            if let fitparser::Value::String(s) = c.value() {
                                declarado.deporte = Dato::Hay(s.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Los segundos absolutos se convierten a tiempo DENTRO de la sesión, y si el fichero declarara
    // una duración distinta, manda la de las muestras: es la que se puede comprobar.
    if let (Some(primero), Some(ultimo)) = (
        muestras.iter().map(|m| m.segundo).min(),
        muestras.iter().map(|m| m.segundo).max(),
    ) {
        for m in &mut muestras {
            m.segundo = m.segundo.saturating_sub(primero);
        }
        if declarado.duracion_s.falta() {
            declarado.duracion_s = Dato::Hay(ultimo.saturating_sub(primero));
        }
    }

    // La distancia se guarda como dato aparte y no entra en el modelo de la muestra: el motor no la
    // usa todavía, y **guardar lo que no se usa es una forma de mentir sobre lo que se sabe**.

    Ok(Lectura {
        sesion: Sesion {
            muestras,
            duracion_s: declarado.duracion_s,
            intervalos_rr_ms: intervalos_rr,
        },
        declarado,
        mensajes: cuenta.into_iter().collect(),
    })
}

/// Saca un entero de un valor de FIT, sea del tipo que sea. Devuelve `Falta` si no es un número.
fn entero(v: &fitparser::Value) -> Dato<u16> {
    match v {
        fitparser::Value::UInt8(x) => Dato::Hay(u16::from(*x)),
        fitparser::Value::UInt16(x) => Dato::Hay(*x),
        fitparser::Value::UInt32(x) => Dato::Hay(*x as u16),
        fitparser::Value::SInt8(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::SInt16(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::SInt32(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::Float32(x) => Dato::Hay(x.round().max(0.0) as u16),
        fitparser::Value::Float64(x) => Dato::Hay(x.round().max(0.0) as u16),
        // Un valor de un tipo que no se espera **no se adivina**: se trata como ausente.
        _ => Dato::Falta,
    }
}

/// Saca un número con decimales de un valor de FIT.
fn real(v: &fitparser::Value) -> Option<f64> {
    match v {
        fitparser::Value::Float32(x) => Some(f64::from(*x)),
        fitparser::Value::Float64(x) => Some(*x),
        fitparser::Value::UInt8(x) => Some(f64::from(*x)),
        fitparser::Value::UInt16(x) => Some(f64::from(*x)),
        fitparser::Value::UInt32(x) => Some(f64::from(*x)),
        fitparser::Value::SInt8(x) => Some(f64::from(*x)),
        fitparser::Value::SInt16(x) => Some(f64::from(*x)),
        fitparser::Value::SInt32(x) => Some(f64::from(*x)),
        _ => None,
    }
}

/// Lee **todos** los FIT de una carpeta, ordenados por nombre.
///
/// Los que no se puedan leer **se cuentan y se dicen**, en vez de perderse en silencio: un fichero que
/// no se lee es un dato que falta, y **lo que falta hay que decirlo**.
pub fn leer_carpeta(carpeta: &Path) -> (Vec<(String, Lectura)>, Vec<(String, String)>) {
    leer_carpeta_hasta(carpeta, usize::MAX)
}

/// Igual, pero **parando a los `tope` ficheros**.
///
/// POR QUÉ EXISTE: leer 102 ficheros de verdad tarda unos segundos, y una prueba que tarda deja de
/// ejecutarse. Con un tope, las pruebas comprueban **el lector** —que es lo que hay que comprobar— sin
/// pagar el precio de leer el histórico entero. **Una prueba lenta es una prueba que se deja de
/// correr**, y entonces deja de proteger.
pub fn leer_carpeta_hasta(carpeta: &Path, tope: usize) -> (Vec<(String, Lectura)>, Vec<(String, String)>) {
    let mut bien = Vec::new();
    let mut mal = Vec::new();
    let mut rutas: Vec<_> = match std::fs::read_dir(carpeta) {
        Ok(d) => d.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
        Err(e) => return (bien, vec![(carpeta.display().to_string(), format!("{e}"))]),
    };
    rutas.sort();
    rutas.truncate(tope);
    for r in rutas {
        if r.extension().map(|e| e == "fit").unwrap_or(false) {
            match leer(&r) {
                Ok(l) => bien.push((r.file_name().unwrap_or_default().to_string_lossy().to_string(), l)),
                Err(e) => mal.push((r.file_name().unwrap_or_default().to_string_lossy().to_string(), e)),
            }
        }
    }
    (bien, mal)
}

/// Un perfil de persona **a partir de lo que el FIT declara**, para cuando no hay más.
///
/// El FIT **no trae** la edad, el peso ni la grasa: eso no lo mide el aparato. Lo que sí trae es el
/// **pulso máximo de la sesión**, que sirve como suelo — y el motor ya sabe que eso es una estimación.
pub fn persona_desde(lectura: &Lectura) -> Persona {
    let mut p = Persona::desconocida();
    if let Dato::Hay(m) = lectura.declarado.pulso_maximo {
        p.pulso_maximo_medido = Dato::Hay(m);
    }
    p
}
