//! Compila la interfaz de Slint al construir el motor.
//!
//! POR QUÉ HACE FALTA: los ficheros `.slint` son TEXTO, y hay que convertirlos en código antes de
//! compilar. Si esto falla, el error sale aqui y no dentro del programa, que es donde se entiende.
fn main() {
    slint_build::compile("ui/ventana.slint").expect("la interfaz de Slint no compila");
}
