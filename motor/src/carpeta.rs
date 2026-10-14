//! **Elegir la carpeta de sus ficheros, sin depender de ningun dialogo del sistema.**
//!
//! POR QUE EXISTE: hoy la carpeta entra por linea de ordenes, y eso obliga al cliente a saber donde
//! estan sus ficheros. Esta pieza es la **logica del selector**: mira el disco y devuelve nombres. El
//! panel que los dibuja vive en la capa 5.
//!
//! POR QUE NO SE USA UN DIALOGO DEL SISTEMA: medido, Slint 1.18.1 no trae FileDialog ni FolderDialog, y
//! una dependencia nueva hay que autorizarla. El contrato esta en docs/CONTRATO-carpeta.md.
//!
//! LO QUE **NO** HACE: no decide cual es la carpeta buena, no guarda nada, no crea ni borra carpetas y
//! no filtra por contenido. Solo enseña lo que hay.

use std::path::{Path, PathBuf};

/// **Las subcarpetas inmediatas**, ordenadas, sin ocultas y sin ficheros.
///
/// Un error de permisos o una carpeta que no existe devuelven **vacio, no un error**: para quien elige,
/// "aqui no puedo entrar" y "aqui no hay nada" se enseñan igual, y el detalle no es cosa suya.
pub fn subcarpetas(dir: &Path) -> Vec<String> {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut v: Vec<String> = entradas
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let nombre = e.file_name().to_string_lossy().to_string();
            // Las ocultas no se enseñan: llenan la lista de ruido y el cliente no las busca.
            if nombre.starts_with('.') {
                None
            } else {
                Some(nombre)
            }
        })
        .collect();
    v.sort();
    v
}

/// **La carpeta de arriba.** None en la raiz: no se puede subir mas, y se dice en vez de dar una ruta
/// rara. Una ruta relativa sin padre tampoco sube.
pub fn ruta_padre(dir: &Path) -> Option<PathBuf> {
    match dir.parent() {
        Some(p) if !p.as_os_str().is_empty() => Some(p.to_path_buf()),
        _ => None,
    }
}

/// ¿Es una carpeta? Se pregunta al disco, no al nombre.
pub fn es_carpeta(ruta: &Path) -> bool {
    ruta.is_dir()
}

/// Una subcarpeta, con **cuantos FIT tiene**. El numero es la INTUICION del selector: el cliente
/// reconoce su carpeta de datos de un vistazo, sin recordar la ruta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrada {
    pub nombre: String,
    pub cuantos_fit: usize,
}

/// **Lo que el panel tiene que dibujar**, ya calculado. Es el modelo del selector, y por eso se puede
/// probar sin abrir ninguna ventana: quien dibuja no tiene que saber leer el disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vista {
    /// La carpeta que se esta mirando. None si la ruta no era una carpeta.
    pub ruta: Option<PathBuf>,
    /// La carpeta de arriba, si se puede subir. None en la raiz.
    pub subir_a: Option<PathBuf>,
    /// Las subcarpetas, cada una con su numero de FIT.
    pub entradas: Vec<Entrada>,
    /// **Cuantos FIT hay EN ESTA carpeta**: si hay alguno, ya se puede usar tal cual.
    pub fit_aqui: usize,
    /// Que decir si no hay nada que elegir. None cuando si hay.
    pub aviso: Option<String>,
}

/// **Abre una carpeta y devuelve lo que hay que dibujar**, sin dibujar nada.
///
/// Cuenta los FIT de las subcarpetas con la MISMA definicion que el lector -- fit::listar -- para que el
/// numero que enseña el selector sea el que luego se va a analizar. Una pista que no cuadra con el
/// resultado es peor que no darla.
pub fn abrir(dir: &Path) -> Vista {
    if !es_carpeta(dir) {
        return Vista {
            ruta: None,
            subir_a: None,
            entradas: Vec::new(),
            fit_aqui: 0,
            aviso: Some("esa carpeta no existe o no se puede leer".to_string()),
        };
    }
    let entradas: Vec<Entrada> = subcarpetas(dir)
        .into_iter()
        .map(|nombre| Entrada {
            cuantos_fit: crate::fit::listar(&dir.join(&nombre)).len(),
            nombre,
        })
        .collect();
    let fit_aqui = crate::fit::listar(dir).len();
    let aviso = if fit_aqui > 0 {
        None
    } else if entradas.iter().any(|e| e.cuantos_fit > 0) {
        None
    } else {
        Some("aqui no se ven FIT todavia; entra en la carpeta donde esten".to_string())
    };
    Vista {
        ruta: Some(dir.to_path_buf()),
        subir_a: ruta_padre(dir),
        entradas,
        fit_aqui,
        aviso,
    }
}
