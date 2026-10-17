//! **Medir el ESTADO DE INTERACCION de la interfaz: hover y pulsado.**
//!
//! POR QUE EXISTE (medido el 2026-10-15, y es el caso que lo pario):
//! el dueno uso la aplicacion por primera vez y dijo, textual:
//!
//! > «no hay correccion del cursor al colocarlo sobre una seccion especifica».
//!
//! Se midio: **cero** declaraciones de `mouse-cursor` en toda la interfaz, y pasar el puntero por
//! encima de la navegacion producia **0 px distintos** — el programa no daba NINGUNA senal de que se
//! pudiera pulsar. Nada de esto lo cazaba ninguna prueba, y **no por descuido**: `medir-interfaz.rs`
//! mide **geometria**, y esto no es geometria. Una captura quieta **no puede** mostrar un estado que
//! solo existe mientras el puntero esta encima.
//!
//! ESTA PRUEBA ES EL PORTERO DE LA REGLA AU-101: «el estado de interaccion se mide, y no se ve en una
//! captura quieta: hover, foco y pulsado se comprueban PROVOCANDOLOS».
//!
//! COMO MIDE, y por que asi:
//!  1. Renderiza la ventana a un buffer de pixeles **con el renderizador de memoria**.
//!  2. Envia un `PointerMoved` DE VERDAD, con `dispatch_event`.
//!  3. Compara pixel a pixel. **Si el hover deja de pintarse, esta prueba se pone roja.**
//!
//! QUE **NO** COMPRUEBA, y se dice en vez de disimularlo: **EL CURSOR**.
//! Slint 1.18 **no expone el cursor en su API publica** — `MouseCursorInner` vive en el modulo
//! `cursor` de `i-slint-core`, que **no se reexporta** desde `slint` (medido el 2026-10-15), ni
//! `WindowAdapter` tiene un `set_mouse_cursor` que se pueda interceptar. Asi que el cursor se
//! comprueba **por su declaracion** (`mouse-cursor: pointer` en el codigo: si se borra, esto no
//! salta) y **visualmente**, con `motor/src/bin/auditar-estados.rs`, que deja las capturas en
//! `datos/capturas/estados-interaccion/`. **Un limite medido se escribe; no se tapa.**

slint::include_modules!();

// La ruta es relativa a ESTE fichero: `#[path]` se resuelve asi, no desde la raiz del crate.
#[path = "../src/bin/compartido/pintar.rs"]
mod pintar;

use motor::cara;
use motor::datos::Persona;
use motor::profile;
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::platform::{Platform, PlatformError, WindowAdapter, WindowEvent};
use slint::LogicalPosition;
use std::rc::Rc;

/// La seccion MOVIL es la que mide 411 de ancho: **es la que el cliente vera en el telefono**.
const ANCHO: u32 = 411;
const ALTO: u32 = 880;

struct Plataforma(Rc<MinimalSoftwareWindow>);

impl Platform for Plataforma {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.0.clone())
    }
}

/// La ventana con el MISMO camino de pintado que la aplicacion (no un fixture: AU-05).
fn ventana_de_pantalla_estrecha() -> (Rc<MinimalSoftwareWindow>, Ventana) {
    let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Plataforma(w.clone())))
        .expect("la plataforma se instala una sola vez por prueba");
    w.set_size(slint::PhysicalSize::new(ANCHO, ALTO));
    let v = Ventana::new().expect("la ventana tiene que construirse");
    let mut p = Persona::desconocida();
    // El `Result` SE COMPRUEBA, no se ignora (aviso de compilacion cazado el 2026-10-17): si estos
    // datos no se aplican, la prueba mediria una persona distinta de la que dice medir.
    profile::aplicar(&mut p, &[("edad", "52"), ("umbral_lactato", "169")])
        .expect("los datos de la persona de la prueba tienen que aplicarse");
    let ensenado = cara::lo_que_ensena(&p, None);
    pintar::pintar(&v, &ensenado);
    let _ = v.show();
    (w, v)
}

/// Los pixeles que la ventana pintaria AHORA. Devuelve `false` si la ventana no se marcó sucia: un
/// render que no ocurre deja el buffer VIEJO, y creer que se midio seria peor que no medir.
fn pintar_ahora(w: &Rc<MinimalSoftwareWindow>, buffer: &mut [PremultipliedRgbaColor]) -> bool {
    let mut pinto = false;
    w.draw_if_needed(|r| {
        r.render(buffer, ANCHO as usize);
        pinto = true;
    });
    pinto
}

/// Cuantos pixeles cambian entre dos renders. Es la MEDIDA de «se nota o no se nota».
/// Se compara campo a campo porque `PremultipliedRgbaColor` no implementa `PartialEq` (medido).
fn pixeles_distintos(a: &[PremultipliedRgbaColor], b: &[PremultipliedRgbaColor]) -> usize {
    a.iter()
        .zip(b.iter())
        .filter(|(x, y)| (x.red, x.green, x.blue, x.alpha) != (y.red, y.green, y.blue, y.alpha))
        .count()
}

fn mover(w: &Rc<MinimalSoftwareWindow>, x: f32, y: f32) {
    w.dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(x, y),
    });
}

/// El centro del hueco `nav-hueco-i`, en coordenadas de ventana.
/// Se PREGUNTA por el nombre, no se escribe una coordenada a mano: una coordenada a mano se queda
/// obsoleta el dia que el reparto cambie (AU-63).
fn centro_de_un_hueco_de_la_barra(v: &Ventana, i: usize) -> (f32, f32) {
    use i_slint_backend_testing::ElementQuery;
    let nombre = format!("nav-hueco-{i}");
    // POR `accessible-id`, no por `match_id`: medido el 2026-10-15, `match_id` va por el id del
    // COMPONENTE (encuentra `ArmazonMovil::cabecera`) y NO por el `accessible-id`, asi que con el
    // nombre accesible no encuentra nada. La forma de la casa es esta (AU-63).
    let todos = ElementQuery::from_root(v).find_all();
    let campo = todos
        .iter()
        .find(|e| e.accessible_id().map(|s| s.to_string()) == Some(nombre.clone()))
        .unwrap_or_else(|| panic!("no se encontro el hueco de navegacion `{nombre}`"));
    let p = campo.absolute_position();
    let t = campo.size();
    (p.x + t.width / 2.0, p.y + t.height / 2.0)
}

/// El minimo de pixeles que tiene que moverse una senal de hover para considerarse una senal.
/// **Medido el 2026-10-15**: el halo de la barra movil mueve **737 px**. El defecto que esta prueba
/// vigila daba **0 px**, que es el silencio total. El umbral queda holgadamente por debajo del valor
/// real y muy por encima de cero: asi un cambio de color no lo dispara, y perder la senal si.
const MINIMO_PX_DE_HOVER: usize = 100;

#[test]
fn la_barra_de_navegacion_da_senal_al_pasar_el_puntero_encima() {
    let (w, v) = ventana_de_pantalla_estrecha();
    let mut antes = vec![PremultipliedRgbaColor::default(); (ANCHO * ALTO) as usize];
    let mut despues = vec![PremultipliedRgbaColor::default(); (ANCHO * ALTO) as usize];
    let mut apagado = vec![PremultipliedRgbaColor::default(); (ANCHO * ALTO) as usize];

    // 1) SIN puntero encima: el estado de partida. Se mueve a una esquina, lejos de todo.
    mover(&w, 5.0, 5.0);
    assert!(
        pintar_ahora(&w, &mut antes),
        "la ventana no se marcó sucia sin puntero encima: el render no es fiable, y sin render \
         fiable esta prueba no mide nada"
    );

    // 2) CON el puntero encima de «Zonas» (el 1 de la barra). Es una seccion que NO esta activa: en
    //    la activa la senal no debe encenderse, porque ya lleva su pildora.
    let (x, y) = centro_de_un_hueco_de_la_barra(&v, 1);
    mover(&w, x, y);
    assert!(
        pintar_ahora(&w, &mut despues),
        "con el puntero encima, la ventana TIENE que repintarse (AU-101)"
    );

    let cambiados = pixeles_distintos(&antes, &despues);
    assert!(
        cambiados > MINIMO_PX_DE_HOVER,
        "pasar el puntero por encima de la barra de navegacion cambia {cambiados} px (el minimo es \
         {MINIMO_PX_DE_HOVER}). El 2026-10-15 esto daba **0 px**, que es exactamente la queja del \
         dueno: «no hay correccion del cursor». Si vuelve a 0, la senal de interaccion se ha perdido \
         (AU-101): la barra es lo PRIMERO que toca el dedo en el telefono"
    );
    println!("  barra de navegacion · hover sobre «Zonas» · {cambiados} px distintos");

    // 3) EL CASO LIMITE, y el que mas importa: apartarlo tiene que APAGAR la senal.
    //    Una senal que se queda encendida es peor que no tenerla, porque miente.
    mover(&w, 5.0, 5.0);
    let _ = pintar_ahora(&w, &mut apagado);
    let vuelta = pixeles_distintos(&antes, &apagado);
    assert_eq!(
        vuelta, 0,
        "al apartar el puntero la pantalla tiene que quedar EXACTAMENTE como estaba, y quedan \
         {vuelta} px encendidos: una senal de hover que no se apaga miente (AU-101)"
    );
    println!("  barra de navegacion · al apartar el puntero · {vuelta} px (tiene que ser 0)");
}
