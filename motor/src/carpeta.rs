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
