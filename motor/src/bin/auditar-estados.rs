//! INSTRUMENTO DE AUDITORIA DE INTERACCION (2026-10-15). Se QUEDA: no es temporal.
//!
//! Nacio el 2026-10-15 como temporal y su cabecera decia «se BORRA al terminar». **Dejo de ser
//! temporal el mismo dia**, por dos motivos medidos:
//!  1. Es el **unico** instrumento que mide **estados de interaccion**. `capturar-ui` pinta la
//!     ventana quieta, y una captura quieta **no puede** mostrar un hover: el hover solo existe
//!     mientras el puntero esta encima.
//!  2. Encontro un defecto real que ninguna prueba veia: pasar el puntero por encima de la
//!     navegacion daba **0 px distintos** — la aplicacion no daba ninguna senal de que se pudiera
//!     pulsar (la queja del dueno: «no hay correccion del cursor»).
//!
//! **Vive en `src/bin/` a proposito y no en un crate aparte**: comparte el camino de pintado
//! (`compartido/pintar.rs`) con la aplicacion. Un crate propio obligaria a duplicarlo, y una copia
//! es una promesa de divergencia: el dia que la app cambie, el instrumento mentiria.
//!
//! COMPLEMENTA a `tests/medir-estados.rs`, que es el PORTERO (se pone rojo si el hover desaparece).
//! Este instrumento es para MIRAR: deja las capturas y su diferencial de pixeles.
//!
//! Provoca ESTADOS DE INTERACCION (hover, pulsado, foco, scroll) sobre la ventana EN MEMORIA y
//! captura un PPM por estado, en la carpeta que se le diga. No abre ventana, no toca datos del
//! dueno: usa el caso SINTETICO (datos-de-ejemplo), igual que `capturar-ui --ejemplo`.
//!
//! Uso:  auditar-estados <carpeta-salida>          bateria completa de estados
//!       auditar-estados <carpeta> scrollhover X Y DELTA   scroll y luego hover en (X,Y)
//!
//! Cada captura imprime `repintado=true/false`: false = la ventana no se marco sucia (aviso del
//! padre: un estado que no repinta puede salir desfasado), y se anota en el informe.

slint::include_modules!();

#[path = "compartido/pintar.rs"]
mod pintar;

use motor::cara;
use motor::datos::Persona;
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::platform::{Platform, PlatformError, WindowAdapter, WindowEvent};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const ANCHO: u32 = 411;
const ALTO: u32 = 880;

struct Plataforma(Rc<MinimalSoftwareWindow>);

impl Platform for Plataforma {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.0.clone())
    }
}

fn capturar(
    window: &Rc<MinimalSoftwareWindow>,
    salida: &Path,
    nombre: &str,
    buffer: &mut Vec<PremultipliedRgbaColor>,
) {
    let repintado = window.draw_if_needed(|r| {
        r.render(buffer, ANCHO as usize);
    });
    let ruta = salida.join(format!("{nombre}.ppm"));
    let mut bytes = Vec::with_capacity(15 + buffer.len() * 3);
    bytes.extend_from_slice(format!("P6\n{ANCHO} {ALTO}\n255\n").as_bytes());
    for p in buffer.iter() {
        let a = p.alpha as u32;
        let rgb = if a == 0 {
            [0u8, 0, 0]
        } else {
            [
                ((p.red as u32 * 255 / a).min(255)) as u8,
                ((p.green as u32 * 255 / a).min(255)) as u8,
                ((p.blue as u32 * 255 / a).min(255)) as u8,
            ]
        };
        bytes.extend_from_slice(&rgb);
    }
    std::fs::write(&ruta, bytes).expect("escribir el ppm");
    eprintln!("{nombre}: repintado={repintado}");
}

fn mover(window: &MinimalSoftwareWindow, x: f32, y: f32) {
    window.dispatch_event(WindowEvent::PointerMoved {
        position: slint::LogicalPosition::new(x, y),
    });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let salida = PathBuf::from(args.get(1).cloned().unwrap_or_default());
    std::fs::create_dir_all(&salida).expect("crear carpeta de salida");

    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    if slint::platform::set_platform(Box::new(Plataforma(window.clone()))).is_err() {
        eprintln!("no se pudo instalar la plataforma");
        std::process::exit(1);
    }
    window.set_size(slint::PhysicalSize::new(ANCHO, ALTO));

    let ventana = Ventana::new().expect("la ventana tiene que construirse");

    // EL CASO SINTETICO, nunca los datos del dueno.
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../datos-de-ejemplo/caso-60-ciclismo.json");
    let texto = std::fs::read_to_string(&ruta).expect("leer el caso sintetico");
    let persona: Persona =
        motor::adaptador::leer_perfil(&texto).expect("el caso sintetico tiene que ser valido");
    let ensenado = cara::lo_que_ensena(&persona, None);
    pintar::pintar(&ventana, &ensenado);

    ventana.show().expect("mostrar");
    let mut buffer = vec![PremultipliedRgbaColor::default(); (ANCHO * ALTO) as usize];

        if args.iter().any(|a| a == "scrollhover") {
        // Modo segunda pasada: scroll de la seccion Datos y hover donde se indique.
        //
        // DOS FALLOS MEDIDOS EL 2026-10-15, y se corrigen aqui en vez de repetirlos:
        //  1. Se leia `args[3..5]` dando por hecho que delante venian `programa` y `salida`. Con
        //     `salida scrollhover 205 400 -300` (lo que se teclea de verdad) los numeros caen en
        //     3, 4 y 5; pero con una palabra de mas el programa se desalinea y lee `scrollhover`
        //     como si fuera una coordenada. Se leen POR NOMBRE apuntando al final, que es estable.
        //  2. `args[n]` PANIQUEA si falta el argumento; `args.get(n)` avisa y dice que falta. Un
        //     instrumento que revienta en vez de explicarse cuesta un diagnostico entero.
        let numeros: Vec<String> = args
            .iter()
            .skip_while(|a| a.as_str() != "scrollhover")
            .skip(1) // la propia palabra
            .take(3) // 0: x, 1: y, 2: delta
            .cloned()
            .collect();
        let leer = |i: usize, nombre: &str| -> f32 {
            numeros
                .get(i)
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or_else(|| {
                    eprintln!(
                        "falta o no es un numero el argumento `{nombre}`.\n\
                         Uso: auditar-estados <carpeta-de-salida> scrollhover <x> <y> <delta>"
                    );
                    std::process::exit(2);
                })
        };
        let (hx, hy, delta) = (leer(0, "x"), leer(1, "y"), leer(2, "delta"));
        ventana.set_seccion_activa(3);
        mover(&window, 205.0, 400.0);
        window.dispatch_event(WindowEvent::PointerScrolled {
            position: slint::LogicalPosition::new(205.0, 400.0),
            delta_x: 0.0,
            delta_y: delta,
        });
        capturar(&window, &salida, "03-scroll-con-hover", &mut buffer);
        mover(&window, hx, hy);
        capturar(&window, &salida, "03-hover-sincronizar", &mut buffer);
        window.dispatch_event(WindowEvent::PointerPressed {
            position: slint::LogicalPosition::new(hx, hy),
            button: slint::platform::PointerEventButton::Left,
        });
        capturar(&window, &salida, "03-pressed-sincronizar", &mut buffer);
        window.dispatch_event(WindowEvent::PointerReleased {
            position: slint::LogicalPosition::new(hx, hy),
            button: slint::platform::PointerEventButton::Left,
        });
        mover(&window, 5.0, 400.0);
        for i in 4..=12 {
            window.dispatch_event(WindowEvent::KeyPressed { text: "\t".into() });
            capturar(&window, &salida, &format!("03-foco-tab{i}"), &mut buffer);
        }
        return;
    }

    // ================= SECCION 3 · DATOS (los unicos controles visibles) =================
    ventana.set_seccion_activa(3);
    capturar(&window, &salida, "03-datos-base", &mut buffer);

    // EL RIEL DE SECCIONES (escritorio): la navegacion. Anadido el 2026-10-15 al arreglar el
    // cursor y el hover: el instrumento NO lo medía, asi que el cambio no se habria visto.
    // Las filas van a x≈140 (el riel), con 36 de alto y espaciado 4 desde y=20.
    mover(&window, 140.0, 100.0); // «Sesiones», una que NO esta activa
    capturar(&window, &salida, "03-hover-riel-sesiones", &mut buffer);
    mover(&window, 140.0, 60.0); // «Zonas», tampoco activa
    capturar(&window, &salida, "03-hover-riel-zonas", &mut buffer);

    mover(&window, 205.0, 637.0); // «Guardar mis datos»
    capturar(&window, &salida, "03-hover-guardar", &mut buffer);

    window.dispatch_event(WindowEvent::PointerPressed {
        position: slint::LogicalPosition::new(205.0, 637.0),
        button: slint::platform::PointerEventButton::Left,
    });
    capturar(&window, &salida, "03-pressed-guardar", &mut buffer);
    window.dispatch_event(WindowEvent::PointerReleased {
        position: slint::LogicalPosition::new(205.0, 637.0),
        button: slint::platform::PointerEventButton::Left,
    });
    capturar(&window, &salida, "03-soltado-guardar", &mut buffer);

    mover(&window, 205.0, 559.0); // combo «Masculino»
    capturar(&window, &salida, "03-hover-combo", &mut buffer);
    mover(&window, 112.0, 196.0); // campo Edad
    capturar(&window, &salida, "03-hover-campo-edad", &mut buffer);
    mover(&window, 309.0, 214.0); // campo Peso
    capturar(&window, &salida, "03-hover-campo-peso", &mut buffer);
    mover(&window, 205.0, 723.0); // «Buscar mis ficheros»
    capturar(&window, &salida, "03-hover-buscar", &mut buffer);

    // FOCO: Tab x3 desde sin-foco. Cada paso, su captura.
    mover(&window, 5.0, 500.0); // sacar el puntero de encima
    capturar(&window, &salida, "03-foco-0", &mut buffer);
    for i in 1..=3 {
        window.dispatch_event(WindowEvent::KeyPressed { text: "\t".into() });
        capturar(&window, &salida, &format!("03-foco-tab{i}"), &mut buffer);
    }

    // SCROLL para alcanzar «Sincronizar».
    mover(&window, 205.0, 400.0);
    window.dispatch_event(WindowEvent::PointerScrolled {
        position: slint::LogicalPosition::new(205.0, 400.0),
        delta_x: 0.0,
        delta_y: -300.0,
    });
    capturar(&window, &salida, "03-scroll-300", &mut buffer);

    // ================= SECCION 0 · RESUMEN (nav movil) =================
    ventana.set_seccion_activa(0);
    capturar(&window, &salida, "00-resumen-base", &mut buffer);
    mover(&window, 155.0, 840.0); // hueco Zonas
    capturar(&window, &salida, "00-hover-nav-zonas", &mut buffer);
    mover(&window, 53.0, 840.0); // hueco Resumen
    capturar(&window, &salida, "00-hover-nav-resumen", &mut buffer);
    mover(&window, 358.0, 840.0); // hueco Datos
    capturar(&window, &salida, "00-hover-nav-datos", &mut buffer);
    mover(&window, 155.0, 840.0); // volver a Zonas...
    window.dispatch_event(WindowEvent::PointerPressed {
        position: slint::LogicalPosition::new(155.0, 840.0),
        button: slint::platform::PointerEventButton::Left,
    });
    capturar(&window, &salida, "00-pressed-nav-zonas", &mut buffer);
    window.dispatch_event(WindowEvent::PointerReleased {
        position: slint::LogicalPosition::new(155.0, 840.0),
        button: slint::platform::PointerEventButton::Left,
    });
    // Al soltar, el click cambia de seccion: se mide si la navegacion RESPONDE.
    capturar(&window, &salida, "00-tras-click-nav-zonas", &mut buffer);

    println!("listo");
}
