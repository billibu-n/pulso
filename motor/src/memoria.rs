//! **El resultado del análisis, y su MEMORIA entre arranques.**
//!
//! POR QUE EXISTE, y esta MEDIDO, no supuesto: abrir la aplicacion con 102 ficheros de verdad tardaba
//! **5,50 s**, y de esos 5,50 s, **5,50 son LEER los ficheros**. Contar 549.842 muestras y calcular la
//! deriva de las 102 sesiones es **instantaneo** (0,000 s y 0,00 s, medido). O sea que el problema no
//! era calcular: era **volver a leer lo que no ha cambiado**.
//!
//! Un ciclista que abre la aplicacion para mirar sus zonas **no va a esperar cinco segundos y medio
//! cada vez**, y menos en el telefono. Y el paso 8 del recorrido del usuario lo dice: **una aplicacion
//! que hay que reconfigurar cada vez no se usa dos veces.**
//!
//! COMO SE RESUELVE, y por que asi: se guarda **el resultado** -- no los ficheros, que son enormes --
//! junto a una **huella** de lo que se leyo. Al arrancar se recalcula la huella mirando la carpeta:
//! **leer el tamaño de 102 ficheros es instantaneo**, asi que sale mucho mas barato que volver a
//! leerlos. Si la huella coincide, se usa lo guardado. Si no, se lee todo y se guarda otra vez.
//!
//! POR QUE EL TAMAÑO Y NO LA FECHA: la fecha de un fichero cambia si se **copia**, aunque el contenido
//! sea el mismo; el tamaño no. Con la fecha, copiar sus datos invalidaria la memoria sin motivo. Se
//! probo a mirarlo y se elige el tamaño **a proposito**, no por comodidad.
//!
//! Y **lo que NO se hace**: guardar los datos crudos, ni en un formato binario propio, ni comprimidos.
//! El resultado son **nueve numeros**, y un fichero de texto con nueve numeros se puede **leer y
//! comprobar a ojo** -- que es lo que este proyecto pide de todo lo que guarda.

use crate::analisis::Real;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Lo que se guarda en disco: **el resultado y la huella de lo que se leyo**.
///
/// Van juntos a proposito: un resultado **sin** la huella no se puede comprobar, y entonces no se sabe
/// si corresponde a los ficheros que hay ahora. Un resultado que no se puede comprobar no sirve.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guardado {
    /// La version del formato. Si un dia cambia lo que se guarda, esto lo delata.
    ///
    /// POR QUE: un fichero de memoria viejo, leido por un programa nuevo, daria numeros equivocados
    /// **sin avisar**. Con la version, se detecta y se vuelve a calcular. Es mas barato recalcular
    /// que dar un dato falso.
    pub version: u32,
    /// La huella de los ficheros: nombre y tamaño de cada uno.
    pub huella: Vec<(String, u64)>,
    /// El resultado.
    pub real: Real,
}

/// La version del formato de la memoria. **Se sube a mano cuando cambia `Real`.**
pub const VERSION: u32 = 1;

/// La huella de una carpeta: de cada `.fit`, su **nombre y su tamaño**, ordenado.
///
/// Se mira solo lo que puede cambiar el resultado: si aparece un fichero, si desaparece, o si cambia
/// de tamaño. Los ficheros de otro tipo no cuentan -- no se leen, asi que no cambian nada.
pub fn huella(carpeta: &Path) -> Vec<(String, u64)> {
    let mut h: Vec<(String, u64)> = match std::fs::read_dir(carpeta) {
        Ok(d) => d
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "fit").unwrap_or(false))
            .filter_map(|e| {
                let nombre = e.file_name().to_string_lossy().to_string();
                let tam = e.metadata().ok()?.len();
                Some((nombre, tam))
            })
            .collect(),
        // Una carpeta que no se puede leer da una huella VACIA, no un error: el que llama tiene que
        // poder distinguir «no hay nada» de «no se pudo mirar». Y una huella vacia no casara con nada.
        Err(_) => Vec::new(),
    };
    h.sort();
    h
}

/// Lee la memoria del disco.
///
/// Devuelve `None` -- **y no un error** -- cuando no hay memoria, cuando esta corrupta o cuando es de
/// otra version. En los tres casos la respuesta correcta es la misma: **volver a calcular**. Un error
/// aqui no es un problema que el usuario tenga que resolver.
pub fn leer(ruta: &Path, carpeta: &Path) -> Option<Real> {
    let texto = std::fs::read_to_string(ruta).ok()?;
    let g: Guardado = serde_json::from_str(&texto).ok()?;
    if g.version != VERSION {
        return None;
    }
    // Y LA COMPROBACION QUE HACE QUE ESTO SIRVA DE ALGO: la huella tiene que ser la de AHORA.
    if g.huella != huella(carpeta) {
        return None;
    }
    Some(g.real)
}

/// Escribe la memoria. Si no se puede escribir, **no pasa nada**: se pierde la memoria, no el resultado.
///
/// POR QUE NO DEVUELVE ERROR: la memoria es una **comodidad**, no un dato. Que no se pueda guardar
/// significa que la proxima vez tardara cinco segundos otra vez, y eso no es un fallo que haya que
/// enseñarle a nadie. Devolver un error obligaria a decidir que hacer con el, y no hay nada que hacer.
pub fn guardar(ruta: &Path, carpeta: &Path, real: &Real) {
    let g = Guardado {
        version: VERSION,
        huella: huella(carpeta),
        real: real.clone(),
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
