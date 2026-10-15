//! **Captura la ventana SIN abrirla.**
//!
//! CONTRATO (cinco lineas):
//! - RECIBE: la ruta del PPM de salida y, opcional, --carpeta CARPETA (y --ejemplo).
//! - DEVUELVE: un PPM (P6) de 411x880 con lo que pintaria la aplicacion, con SUS datos reales.
//! - NO HACE: no abre ventana (renderiza en memoria), no decide, no escribe el perfil; lo unico que
//!   escribe es el PPM.
//!
//! POR QUE EXISTE: el dueño pidio VER la aplicacion, y abrir una ventana en su equipo esta prohibido.
//! Esto la renderiza fuera de pantalla; la conversion a PNG la hace \`convert\` (ImageMagick).

slint::include_modules!();

// EL MISMO PINTADO QUE LA APLICACION (Fase 2). Aqui vivia una COPIA de `fn pintar`, y una copia es
// una promesa de divergencia: el dia que la aplicacion cambiara, **la captura mentiria**, y la
// captura es justo lo que se mira para juzgar el diseño. Ahora el camino es UNO.
#[path = "compartido/pintar.rs"]
mod pintar;

use motor::cara;
use motor::datos::Persona;
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const ANCHO: u32 = 411;
const ALTO: u32 = 880;

/// La plataforma que entrega la ventana de render en memoria (la que usan las pruebas de Slint).
struct Plataforma(Rc<MinimalSoftwareWindow>);

impl Platform for Plataforma {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.0.clone())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--ayuda" || a == "-h" || a == "--help") {
        println!("capturar-ui <salida.ppm> [--carpeta CARPETA] [--ejemplo] [--seccion N]");
        println!("  Renderiza la aplicacion fuera de pantalla. No abre ninguna ventana.");
        println!("  --seccion: 0 Resumen (defecto), 1 Zonas, 2 Sesiones, 3 Datos.");
        return;
    }
    let Some(salida) = args.get(1).filter(|a| !a.starts_with("--")) else {
        eprintln!("Falta la ruta del PPM de salida.");
        std::process::exit(2);
    };
    let salida = PathBuf::from(salida);
    let carpeta = args
        .iter()
        .position(|a| a == "--carpeta")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "../datos/garmin_files/storage".to_string());
    // --ejemplo: pinta el CASO SINTETICO (inventado) y sin sesiones, para VER el diseno con datos
    // personales sin mezclar los suyos. Lo que salga lleva el aviso de que es sintetico.
    let ejemplo = args.iter().any(|a| a == "--ejemplo");
    // --seccion N: pinta ESA seccion (0 Resumen, 1 Zonas, 2 Sesiones, 3 Datos) en vez de la primera.
    // Existe porque el dueno pidio VER la aplicacion entera, y sin esto solo se veia el Resumen:
    // el resto de la ventana quedaba sin poder mirarse, que es justo lo que hay que juzgar.
    let seccion: i32 = args
        .iter()
        .position(|a| a == "--seccion")
        .and_then(|i| args.get(i + 1))
        .map(|v| {
            v.parse().unwrap_or_else(|_| {
                eprintln!("--seccion quiere un numero: 0 Resumen, 1 Zonas, 2 Sesiones, 3 Datos");
                std::process::exit(2);
            })
        })
        .unwrap_or(0);

    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    if slint::platform::set_platform(Box::new(Plataforma(window.clone()))).is_err() {
        eprintln!("no se pudo instalar la plataforma de render");
        std::process::exit(1);
    }
    window.set_size(slint::PhysicalSize::new(ANCHO, ALTO));

    let ventana = Ventana::new().expect("la ventana tiene que construirse");
    ventana.set_seccion_activa(seccion);

    // LOS MISMOS DATOS QUE LA APLICACION: su perfil y su carpeta, por el mismo camino.
    let (persona, real) = if ejemplo {
        let ruta = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../datos-de-ejemplo/caso-60-ciclismo.json");
        let texto = std::fs::read_to_string(&ruta).unwrap_or_else(|e| {
            eprintln!("no se pudo leer el caso sintetico {}: {e}", ruta.display());
            std::process::exit(1);
        });
        let p = motor::adaptador::leer_perfil(&texto).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });
        (p, None)
    } else {
        let (ruta_perfil, _) = motor::profile::ruta_del_perfil();
        let p = motor::profile::leer(&ruta_perfil).unwrap_or_else(|e| {
            eprintln!("(perfil: {e}); se pinta el estado SIN datos, que es el real de hoy");
            Persona::desconocida()
        });
        let ruta_memoria = ruta_perfil.with_file_name("analisis.json");
        (p, analisis_de(Path::new(&carpeta), &ruta_memoria))
    };
    let ensenado = cara::lo_que_ensena(&persona, real.as_ref());
    pintar::pintar(&ventana, &ensenado);

    ventana.show().expect("la ventana tiene que mostrarse");
    let mut buffer = vec![PremultipliedRgbaColor::default(); (ANCHO * ALTO) as usize];
    let dibujado = window.draw_if_needed(|r| {
        r.render(&mut buffer, ANCHO as usize);
    });
    if !dibujado {
        eprintln!("aviso: la ventana no se marcd como sucia; el PPM podria salir vacio");
    }
    escribir_ppm(&salida, &buffer).expect("escribir el PPM");
    println!("escrito {} ({}x{})", salida.display(), ANCHO, ALTO);
}

/// El mismo camino que `pulso`: memoria incremental y, si hay datos, se guarda la memoria.
fn analisis_de(carpeta: &Path, ruta_memoria: &Path) -> Option<motor::analisis::Real> {
    if !carpeta.exists() {
        return None;
    }
    let previas = motor::memoria::leer_fichas(ruta_memoria);
    let (real, fichas, _) = motor::analisis::actualizar(carpeta, &previas);
    if real.hay_datos() {
        motor::memoria::guardar(ruta_memoria, &fichas);
        Some(real)
    } else {
        None
    }
}

/// El PPM P6, sin dependencia nueva: es el formato mas simple que entiende `convert`.
fn escribir_ppm(ruta: &Path, buffer: &[PremultipliedRgbaColor]) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(15 + buffer.len() * 3);
    bytes.extend_from_slice(format!("P6\n{ANCHO} {ALTO}\n255\n").as_bytes());
    for p in buffer {
        bytes.extend_from_slice(&des_premultiplicado(*p));
    }
    std::fs::write(ruta, bytes)
}

/// El renderizador entrega color PREMULTIPLICADO; el PPM quiere el color tal cual.
fn des_premultiplicado(p: PremultipliedRgbaColor) -> [u8; 3] {
    let a = p.alpha as u32;
    if a == 0 {
        return [0, 0, 0];
    }
    [
        (p.red as u32 * 255 / a).min(255) as u8,
        (p.green as u32 * 255 / a).min(255) as u8,
        (p.blue as u32 * 255 / a).min(255) as u8,
    ]
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_pixel_opaco_se_des_premultiplica_sin_cambiar_y_el_transparente_es_negro() {
        assert_eq!(
            des_premultiplicado(PremultipliedRgbaColor { red: 10, green: 20, blue: 30, alpha: 255 }),
            [10, 20, 30]
        );
        assert_eq!(
            des_premultiplicado(PremultipliedRgbaColor { red: 0, green: 0, blue: 0, alpha: 0 }),
            [0, 0, 0]
        );
        // Semitransparente: 5 * 255 / 128 = 9, que es el color original.
        assert_eq!(
            des_premultiplicado(PremultipliedRgbaColor { red: 5, green: 5, blue: 5, alpha: 128 }),
            [9, 9, 9]
        );
    }
}
