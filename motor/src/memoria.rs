//! **La memoria entre arranques, INCREMENTAL.**
//!
//! POR QUE EXISTE, y esta MEDIDO: abrir la aplicacion con 102 ficheros de verdad tardaba **5,50 s**, y
//! de esos 5,50 s, **5,50 son LEER los ficheros**. Contar 549.842 muestras y calcular la deriva de las
//! 102 sesiones es **instantaneo**. O sea que el problema no era calcular: era **volver a leer lo que
//! no ha cambiado**.
//!
//! QUE CAMBIO EL 2026-10-14, y por que: la primera version guardaba SOLO el resultado y una huella. Si
//! la huella no coincidia -- y NO coincide en cuanto el ciclista añade la sesion de hoy -- **se releia
//! el historico entero para sumar un fichero**. El dueño lo pidio asi de claro:
//!
//! > «por cada activacion de la aplicacion, no tenga que releer de nuevo toda la informacion de hace
//! > meses y ir sumando la informacion mas reciente».
//!
//! Ahora se guarda **una ficha por fichero** (nombre, tamaño y su APORTE al total), y al arrancar solo
//! se leen las fichas **nuevas o cambiadas**. El contrato esta en docs/CONTRATO-memoria-incremental.md.
//!
//! LO QUE **NO** SE HACE: guardar los datos crudos, ni en un formato binario propio, ni comprimidos.
//! Se guardan aportes de nueve numeros, y un fichero de texto con eso se puede **leer y comprobar a
//! ojo** -- que es lo que este proyecto pide de todo lo que guarda.

use crate::analisis::Ficha;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Lo que se guarda en disco: **las fichas y de cuando son**.
///
/// La fecha del guardado NO decide nada; se guarda para poder **documentar** de cuando es la memoria.
/// Un total sin fecha no se puede poner en contexto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guardado {
    /// La version del formato. Si un dia cambia lo que se guarda, esto lo delata.
    ///
    /// POR QUE: un fichero de memoria viejo, leido por un programa nuevo, daria numeros equivocados
    /// **sin avisar**. Con la version, se detecta y se vuelve a calcular. Es mas barato recalcular
    /// que dar un dato falso.
    pub version: u32,
    /// Cuando se calculo, en segundos desde el 1 de enero de 1970 (UTC). None si el reloj no se pudo
    /// leer, que no es motivo para no guardar.
    pub guardado_en_s: Option<i64>,
    /// Una ficha por fichero recordado.
    pub fichas: Vec<Ficha>,
}

/// La version del formato de la memoria. **Se sube a mano cuando cambia lo que se guarda.**
///
/// LA VERSION 2 ES LA DE LOS APORTES: la 1 guardaba solo el resultado y una huella, y no se puede
/// convertir a fichas sin releer. Se sube, y una memoria de la version 1 se descarta y se recalcula.
pub const VERSION: u32 = 2;

/// Lee el fichero de memoria entero.
///
/// Devuelve None -- **y no un error** -- cuando no hay memoria, cuando esta corrupta o cuando es de
/// otra version. En los tres casos la respuesta correcta es la misma: **volver a leer**. Un error aqui
/// no es un problema que el usuario tenga que resolver.
pub fn leer_guardado(ruta: &Path) -> Option<Guardado> {
    let texto = std::fs::read_to_string(ruta).ok()?;
    let g: Guardado = serde_json::from_str(&texto).ok()?;
    if g.version != VERSION {
        return None;
    }
    Some(g)
}

/// **Lo que se recuerda**, listo para pasarselo a analisis::actualizar.
///
/// Si no hay memoria, esta corrupta o es de otra version, devuelve una lista VACIA: eso significa
/// «no recuerdo nada», y el llamante leera todo. Es exactamente lo que hay que hacer la primera vez.
pub fn leer_fichas(ruta: &Path) -> Vec<Ficha> {
    leer_guardado(ruta).map(|g| g.fichas).unwrap_or_default()
}

/// Escribe la memoria. Si no se puede escribir, **no pasa nada**: se pierde la memoria, no el resultado.
///
/// POR QUE NO DEVUELVE ERROR: la memoria es una **comodidad**, no un dato. Que no se pueda guardar
/// significa que la proxima vez tardara unos segundos mas, y eso no es un fallo que haya que enseñarle
/// a nadie. Devolver un error obligaria a decidir que hacer con el, y no hay nada que hacer.
pub fn guardar(ruta: &Path, fichas: &[Ficha]) {
    let g = Guardado {
        version: VERSION,
        guardado_en_s: ahora_s(),
        fichas: fichas.to_vec(),
    };
    // Si la carpeta no existe se crea. En el telefono, la carpeta de la aplicacion puede no existir
    // todavia, y no poder guardar por eso seria un fallo tonto.
    if let Some(padre) = ruta.parent() {
        let _ = std::fs::create_dir_all(padre);
    }
    if let Ok(texto) = serde_json::to_string_pretty(&g) {
        let _ = std::fs::write(ruta, texto);
    }
}

/// El instante de ahora, en segundos epoch. None si el reloj del sistema va antes de 1970.
fn ahora_s() -> Option<i64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}
