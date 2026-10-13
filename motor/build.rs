//! Compila la interfaz de Slint al construir el motor.
//!
//! POR QUÉ HACE FALTA: los ficheros `.slint` son TEXTO, y hay que convertirlos en código antes de
//! compilar. Si esto falla, el error sale aquí y no dentro del programa, que es donde se entiende.
//!
//! ============================================================================
//! LA INFORMACIÓN DE DEPURACIÓN, Y POR QUÉ SE ACTIVA (2026-10-13)
//! ============================================================================
//!
//! POR DEFECTO VA APAGADA, y eso deja la interfaz **imposible de medir**. Slint lo dice con estas
//! palabras cuando se intenta:
//!
//!   > «The use of the ElementHandle API requires the presence of debug info in Slint compiler
//!   > generated code.»
//!
//! O sea: **sin esto no se puede preguntar dónde ha caído un elemento ni cuánto mide.** Y sin poder
//! medir, la única forma de saber cómo quedó una pantalla es **abrirla y mirarla** — que es
//! exactamente lo que la norma prohíbe (AU-63): *si la única forma de saber cómo quedó es que otra
//! persona lo mire y lo cuente, el método está mal.*
//!
//! POR QUÉ NO ES UN LUJO: costó una tarde entera de idas y vueltas con capturas de pantalla,
//! corrigiendo **a ciegas** ocho diferencias entre el diseño aprobado y lo que salía en pantalla.
//!
//! Y NO ENCARECE LA APLICACIÓN: `i-slint-backend-testing` está en `dev-dependencies`, así que el
//! binario que se entrega no lo lleva. Esto solo añade información al código generado.

fn main() {
    let config = slint_build::CompilerConfiguration::new()
        // La información de depuración es lo que permite MEDIR la geometría desde las pruebas.
        .with_debug_info(true);
    // UN SOLO PUNTO DE ENTRADA. Compilar dos ficheros por separado NO vale: Slint genera un solo
    // modulo con lo ULTIMO que compilas, y el primero desaparece. El sintoma es enganoso -- compila
    // bien y luego el programa no encuentra el tipo.
    slint_build::compile_with_config("ui/raiz.slint", config)
        .expect("la interfaz de Slint no compila");
}
