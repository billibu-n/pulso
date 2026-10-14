//! **El analisis de la carpeta entera, y la memoria INCREMENTAL.**
//!
//! POR QUE ESTA AQUI Y NO DENTRO DEL BINARIO DE LA VENTANA: estaba dentro, y tenia dos consecuencias
//! malas. La primera es que **la memoria entre arranques no podia guardarlo**, porque no habia forma
//! de nombrarlo desde fuera. La segunda es que **no se podia probar sin abrir la ventana**.
//!
//! Y AHORA, ADEMAS, EL APORTE POR FICHERO. La primera memoria guardaba SOLO el total; si el ciclista
//! añadia la sesion de hoy, el total no cuadraba y habia que **releer los tres meses de historico**.
//! Este modulo separa las dos cosas:
//!
//! - **Aporte**: lo que suma **un** fichero (nueve numeros, sin las 549.842 muestras).
//! - **combinar**: junta los aportes y da el **Real**. Es el UNICO sitio donde vive la cuenta.
//!
//! Con eso, arrancar consiste en comparar **nombre y tamaño** de cada FIT con lo recordado y **leer
//! solo lo nuevo o lo cambiado** (ver decidir y actualizar). El contrato esta en
//! docs/CONTRATO-memoria-incremental.md, y se escribio ANTES de este codigo.
//!
//! LO QUE ESTE MODULO **NO** HACE: no dibuja, no formatea numeros y no decide que enseñar. Devuelve
//! numeros; quien los enseñe decide como se escriben.

use crate::datos::Dato;
use crate::fit::{self, Lectura};
use crate::{hrv, patron};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Lo que se saca de la carpeta entera, con **las fechas del periodo**.
///
/// Es Serialize/Deserialize porque es lo que se guarda entre arranques -- bueno, lo que se guarda ahora
/// son los APORTES, pero el Real se reconstruye de ellos y tiene que poder viajar entero.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Real {
    /// Cuantas sesiones se leyeron.
    pub sesiones: usize,
    /// Cuantas lecturas (muestras) hay en total.
    pub muestras: usize,
    /// Cuantos intervalos entre latidos hay en total.
    pub rr: usize,
    /// La deriva media, en porcentaje. None si no habia ninguna sesion con muestras suficientes.
    pub deriva_media: Option<f64>,
    /// Los vatios por pulso, de media.
    pub eficiencia_media: Option<f64>,
    /// Cuantas sesiones se les escapa la deriva por encima del 10 %.
    pub escapados: usize,
    /// La media del RMSSD. **OJO**: medido durante el ejercicio, no en reposo (ver docs/).
    pub hrv_medio: Option<f64>,
    /// La frecuencia mas alta vista **en los datos**.
    pub maxima_vista: Option<u16>,
    /// **Los ficheros que NO se pudieron leer**, con su motivo.
    ///
    /// ESTE CAMPO ES LA HONESTIDAD DEL MODULO, y sin el estaria mintiendo: si de 102 ficheros se leen
    /// 99, todos los promedios salen de 99, y **nada en la pantalla lo diria**. Un dato que falta hay
    /// que decirlo. Antes esto se tiraba a la basura en el binario de la ventana.
    pub no_leidos: Vec<(String, String)>,
    /// **La fecha de la sesion mas antigua**, en segundos epoch. None si ninguna la traia.
    ///
    /// LA FECHA ES COHERENCIA, NO ADORNO: sin ella, un total de tres meses y un total de hoy se
    /// parecen demasiado, y la memoria no puede decir que periodo cubre.
    pub desde: Option<i64>,
    /// **La fecha de la sesion mas reciente**, en segundos epoch. None si ninguna la traia.
    pub hasta: Option<i64>,
}

impl Real {
    /// ¿Hay algo que analizar?
    pub fn hay_datos(&self) -> bool {
        self.sesiones > 0
    }

    /// ¿Se leyo TODO lo que habia?
    pub fn esta_completo(&self) -> bool {
        self.no_leidos.is_empty()
    }
}

/// **Lo que UNA sesion aporta al total**: nueve numeros, y ni una muestra.
///
/// ESTA ES LA PIEZA QUE HACE POSIBLE NO RELEER. Un aporte se puede guardar, reutilizar y sumar sin
/// volver a abrir el FIT. Lo que no cabe en un aporte -- las muestras, los intervalos -- no se guarda:
/// se guarda **lo que aportan**, que es lo que el total necesita.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Aporte {
    /// Cuando empezo la sesion, en segundos epoch. None si el fichero no lo traia.
    pub inicio_s: Option<i64>,
    /// Cuantas muestras trajo.
    pub muestras: usize,
    /// Cuantos intervalos R-R trajo.
    pub rr: usize,
    /// La frecuencia maxima que declara el fichero.
    pub maxima: Option<u16>,
    /// La deriva, **solo si hay muestras bastantes** para que signifique algo.
    pub deriva: Option<f64>,
    /// ¿La deriva pasa del 10 %?
    pub escapado: bool,
    /// Los vatios por pulso, si valen algo.
    pub eficiencia: Option<f64>,
    /// El RMSSD, **solo si hay latidos bastantes**.
    pub hrv: Option<f64>,
}

impl Aporte {
    /// Calcula el aporte de un fichero ya leido.
    ///
    /// LAS TRES REGLAS DE CORTE (20 muestras, valor > 0, 120 latidos) NO SON NUEVAS: son las que ya
    /// usaba el motor, y se respetan aqui para que el total no cambie al pasar a la via incremental.
    pub fn de_lectura(l: &Lectura) -> Self {
        let mut a = Aporte {
            inicio_s: l.inicio_s,
            muestras: l.sesion.muestras.len(),
            rr: l.sesion.intervalos_rr_ms.len(),
            ..Aporte::default()
        };
        if let Some(m) = l.declarado.pulso_maximo.valor() {
            a.maxima = Some(m);
        }
        let d = patron::deriva(&l.sesion);
        if d.muestras >= 20 {
            a.deriva = Some(d.porcentaje());
            a.escapado = d.porcentaje() > 10.0;
        }
        let e = patron::eficiencia(&l.sesion);
        if e.valor > 0.0 {
            a.eficiencia = Some(e.valor);
        }
        let h = hrv::calcular(hrv::intervalos_de(&l.sesion));
        if h.latidos >= 120 {
            a.hrv = Some(h.rmssd.valor);
        }
        a
    }
}

/// **Lo que se recuerda de UN fichero**: su nombre, su tamaño y su aporte (o el error).
///
/// El tamaño va aqui porque es lo que decide si el fichero se puede reutilizar: **nombre y tamaño**.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ficha {
    pub nombre: String,
    pub tamano: u64,
    /// None si el fichero NO se pudo leer; entonces el motivo esta en error.
    pub aporte: Option<Aporte>,
    pub error: Option<String>,
}

impl Ficha {
    pub fn bien(nombre: String, tamano: u64, aporte: Aporte) -> Self {
        Self { nombre, tamano, aporte: Some(aporte), error: None }
    }
    pub fn mal(nombre: String, tamano: u64, error: String) -> Self {
        Self { nombre, tamano, aporte: None, error: Some(error) }
    }
}

/// Que hacer con un fichero que esta en la carpeta: reutilizar su ficha o leerlo.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Recordada(Ficha),
    Leer(String),
}

/// **La regla de decision, y es una sola**: se reutiliza si **nombre y tamaño** coinciden.
///
/// Es una funcion PURA sobre dos listas, para poder probarla sin disco y sin ficheros: es el corazon de
/// la memoria incremental, y si se equivoca, o se relee de mas o --peor-- se da por bueno lo que cambio.
///
/// **El tamaño y no la fecha**, a proposito: copiar un fichero cambia su fecha y no su contenido.
pub fn decidir(actual: &[(String, u64)], previas: &[Ficha]) -> Vec<Decision> {
    use std::collections::HashMap;
    let por_nombre: HashMap<&str, &Ficha> =
        previas.iter().map(|f| (f.nombre.as_str(), f)).collect();
    actual
        .iter()
        .map(|(nombre, tam)| match por_nombre.get(nombre.as_str()) {
            Some(f) if f.tamano == *tam => Decision::Recordada((*f).clone()),
            _ => Decision::Leer(nombre.clone()),
        })
        .collect()
}

/// Cuantas fichas se recordaron, cuantas se leyeron y cuantas no se pudieron leer.
///
/// NO ES UN ADORNO: es **la prueba de que el ahorro ocurre de verdad**. Sin este numero, «no relee» es
/// una promesa; con el, es una medida.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cuantas {
    pub recordadas: usize,
    pub leidas: usize,
    pub no_leidas: usize,
}

/// **El unico sitio donde vive la cuenta**: junta los aportes y da el Real.
///
/// Se separa de la lectura a proposito: asi se puede probar sin tocar el disco, y ademas es lo que
/// permite reconstruir el total a partir de lo recordado.
pub fn combinar(aportes: &[Aporte]) -> Real {
    let mut r = Real::default();
    let mut derivas: Vec<f64> = Vec::new();
    let mut efs: Vec<f64> = Vec::new();
    let mut hrvs: Vec<f64> = Vec::new();

    for a in aportes {
        r.sesiones += 1;
        r.muestras += a.muestras;
        r.rr += a.rr;
        if let Some(m) = a.maxima {
            r.maxima_vista = Some(r.maxima_vista.map_or(m, |x: u16| x.max(m)));
        }
        if let Some(d) = a.deriva {
            derivas.push(d);
        }
        if a.escapado {
            r.escapados += 1;
        }
        if let Some(e) = a.eficiencia {
            efs.push(e);
        }
        if let Some(h) = a.hrv {
            hrvs.push(h);
        }
        if let Some(t) = a.inicio_s {
            r.desde = Some(r.desde.map_or(t, |x| x.min(t)));
            r.hasta = Some(r.hasta.map_or(t, |x| x.max(t)));
        }
    }

    let media = |v: &[f64]| -> Option<f64> {
        if v.is_empty() {
            None
        } else {
            Some(v.iter().sum::<f64>() / v.len() as f64)
        }
    };
    r.deriva_media = media(&derivas);
    r.eficiencia_media = media(&efs);
    r.hrv_medio = media(&hrvs);
    r
}

/// Calcula el resultado a partir de las sesiones ya leidas.
///
/// Se separa de de_carpeta a proposito: asi **se puede probar sin tocar el disco**. Ahora es un caso
/// particular de combinar, para que la cuenta no viva en dos sitios.
pub fn de_lecturas(ficheros: &[(String, Lectura)], no_leidos: Vec<(String, String)>) -> Real {
    let aportes: Vec<Aporte> = ficheros.iter().map(|(_, l)| Aporte::de_lectura(l)).collect();
    let mut r = combinar(&aportes);
    r.no_leidos = no_leidos;
    r
}

/// Lee la carpeta entera y calcula. **Esto es lo que tarda 5,50 s**, y por eso existe la memoria.
pub fn de_carpeta(carpeta: &Path) -> Real {
    let (ficheros, no_leidos) = fit::leer_carpeta(carpeta);
    de_lecturas(&ficheros, no_leidos)
}

/// **Actualiza el analisis leyendo SOLO lo nuevo o lo cambiado**, y devuelve tambien las fichas.
///
/// ES LA PUERTA QUE USA LA APLICACION AL ARRANCAR. Con previas vacias lee todo (primera vez); con las
/// fichas guardadas, reutiliza lo que no cambio.
pub fn actualizar(carpeta: &Path, previas: &[Ficha]) -> (Real, Vec<Ficha>, Cuantas) {
    let actual = fit::listar(carpeta);
    let decisiones = decidir(&actual, previas);
    let mut fichas: Vec<Ficha> = Vec::with_capacity(decisiones.len());
    let mut cuantas = Cuantas::default();

    for d in decisiones {
        match d {
            Decision::Recordada(f) => {
                cuantas.recordadas += 1;
                if f.aporte.is_none() {
                    cuantas.no_leidas += 1;
                }
                fichas.push(f);
            }
            Decision::Leer(nombre) => {
                cuantas.leidas += 1;
                let tamano = actual
                    .iter()
                    .find(|(n, _)| n == &nombre)
                    .map(|(_, t)| *t)
                    .unwrap_or(0);
                match fit::leer(&carpeta.join(&nombre)) {
                    Ok(l) => fichas.push(Ficha::bien(nombre, tamano, Aporte::de_lectura(&l))),
                    Err(e) => {
                        cuantas.no_leidas += 1;
                        fichas.push(Ficha::mal(nombre, tamano, e));
                    }
                }
            }
        }
    }

    let aportes: Vec<Aporte> = fichas.iter().filter_map(|f| f.aporte.clone()).collect();
    let mut real = combinar(&aportes);
    real.no_leidos = fichas
        .iter()
        .filter_map(|f| f.error.clone().map(|e| (f.nombre.clone(), e)))
        .collect();
    (real, fichas, cuantas)
}

/// La frecuencia maxima que se puede defender: la del aparato **solo si el usuario no la declaro**.
///
/// POR QUE ESTA REGLA ES UNA FUNCION Y NO UN if SUELTO: es una decision del dueño (2026-10-13) y
/// tiene que estar **en un solo sitio**, dicha una vez. El aparato aporta lo que el usuario no puede
/// saber; el usuario aporta lo que el aparato no puede saber. **Cuando los dos hablan del mismo dato,
/// manda el que lo declaro a proposito.**
pub fn maxima_que_manda(declarada: &Dato<u16>, vista: Option<u16>) -> Dato<u16> {
    match (declarada.valor(), vista) {
        (Some(d), _) => Dato::Hay(d),
        (None, Some(v)) => Dato::Hay(v),
        (None, None) => Dato::Falta,
    }
}
