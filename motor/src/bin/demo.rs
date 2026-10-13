//! Demostración del motor, para verlo funcionar SIN interfaz.
//!
//! Se ejecuta con `cargo run --bin demo`.
//!
//! Hace **la misma sesión tres veces**, y cada vez quita un dato más. Así se ven las **tres
//! respuestas** del motor, que son el centro de todo esto:
//!
//!   1. con todos los datos  → **SE PUEDE AFIRMAR**
//!   2. sin un dato que cambia el número → **SE PUEDE, CON RESERVA**
//!   3. sin un dato sin el cual no tiene sentido → **NO SE PUEDE**, y dice qué haría falta
//!
//! Y lo importante está en el tercer caso: **el motor no inventa nada**. Dice que no puede.

use motor::datos::{Muestra, Persona, Sesion, Sexo};
use motor::juicio::Juicio;
use motor::hay;

/// Una sesión de 40 minutos: calentamiento suave, subida, y un tramo fuerte.
fn sesion_de_cuarenta_minutos() -> Sesion {
    let mut muestras = Vec::new();
    let mut segundo = 0u32;
    // 10 min suaves (120), 15 min moderados (145), 10 min fuertes (165), 5 min suaves (115)
    for (minutos, pulso) in [(10, 120u16), (15, 145), (10, 165), (5, 115)] {
        for _ in 0..(minutos * 6) {
            muestras.push(Muestra {
                segundo,
                pulso: hay(pulso),
                potencia_w: hay(200),
            });
            segundo += 10;
        }
    }
    Sesion { muestras, duracion_s: hay(2400) }
}

fn main() {
    let sesion = sesion_de_cuarenta_minutos();

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  EL MOTOR: la misma sesion, con cada vez menos datos                 ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");

    // ---- CASO 1: todo lo que hace falta ----
    let completa = Persona {
        edad: hay(40u8),
        pulso_reposo: hay(55u16),
        pulso_maximo_medido: hay(186u16), // medido: manda sobre la estimacion
        peso_kg: hay(68.0f32),
        sexo: hay(Sexo::Femenino),
    };

    println!("\n\n▌ CASO 1 · CON LA FRECUENCIA MAXIMA MEDIDA\n");
    println!("{}", Juicio::juzgar(&completa, &sesion).informe());

    // ---- CASO 2: sin la medida, hay que estimarla con la edad ----
    let estimada = Persona {
        pulso_maximo_medido: motor::Dato::Falta,
        ..completa.clone()
    };

    println!("\n▌ CASO 2 · SIN LA MEDIDA: SE ESTIMA CON LA EDAD\n");
    println!("{}", Juicio::juzgar(&estimada, &sesion).informe());

    // ---- CASO 3: sin edad ni medida -> no se puede, y se dice ----
    let sin_nada = Persona {
        edad: motor::Dato::Falta,
        pulso_maximo_medido: motor::Dato::Falta,
        ..completa.clone()
    };

    println!("\n▌ CASO 3 · SIN EDAD Y SIN MEDIDA\n");
    println!("{}", Juicio::juzgar(&sin_nada, &sesion).informe());

    // ---- CASO 4: una persona de la que no sabemos nada ----
    println!("\n▌ CASO 4 · NO SABEMOS NADA DE LA PERSONA\n");
    println!("{}", Juicio::juzgar(&Persona::desconocida(), &sesion).informe());
}
