//! **El motor funcionando SIN NINGÚN DATO, y ganando precisión con cada dato.**
//!
//! `cargo run --bin sin_datos`
//!
//! Esto demuestra la corrección del dueño:
//!
//! > *«debe poder funcionar con información nula, pues no puede fallar por falta de información.
//! > Más información proporciona más precisión, pero no limita»*.
//!
//! Se calculan las zonas CUATRO VECES, con cada vez más datos. **Ninguna falla.** Y se ve cómo el
//! margen se estrecha — que es exactamente lo que se pidió.

use motor::datos::{Persona, Sexo};
use motor::formulas_v2::{self, OrigenMaxima};
use motor::hay;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  EL MOTOR, CON INFORMACIÓN NULA Y CON MÁS: siempre responde                ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");

    // ---- 1 · Sin absolutamente nada ----
    let p1 = Persona::desconocida();

    // ---- 2 · Solo la edad ----
    let mut p2 = Persona::desconocida();
    p2.edad = hay(60u8);

    // ---- 3 · Edad y composición corporal (lo que da una báscula) ----
    let mut p3 = p2.clone();
    p3.peso_kg = hay(70.2f32);
    p3.grasa_corporal_pct = hay(19.1f32);
    p3.altura_cm = hay(178.0f32);
    p3.sexo = hay(Sexo::Masculino);

    // ---- 4 · Y además, la máxima medida ----
    let mut p4 = p3.clone();
    p4.pulso_maximo_medido = hay(179u16);
    p4.pulso_reposo = hay(48u16);

    let casos: [(&str, &Persona); 4] = [
        ("1 · SIN NINGÚN DATO de la persona", &p1),
        ("2 · con la EDAD", &p2),
        ("3 · con la edad y la COMPOSICIÓN CORPORAL", &p3),
        ("4 · con todo, y la máxima MEDIDA", &p4),
    ];

    for (titulo, p) in casos {
        println!("\n\n▌ {titulo}");
        println!("   datos que tiene: {}", p.cuantos_datos());

        let m = formulas_v2::pulso_maximo(p);
        println!(
            "\n   frecuencia máxima:  {:.0} ± {:.0} ppm   (precisión {})",
            m.rango.valor,
            m.rango.margen,
            m.rango.como_va().nombre()
        );
        println!("   de dónde sale:      {}", m.origen.nombre());
        println!(
            "   qué significa:      {}",
            m.rango.como_va().explicacion()
        );

        // LAS ZONAS, SIEMPRE.
        let z = formulas_v2::zonas(p);
        println!("\n   LAS ZONAS, calculadas igual:");
        for (zona, r) in &z {
            println!(
                "     {:<10} {:>3.0} ± {:<2.0} ppm        (entre {:.0} y {:.0})",
                zona.nombre(),
                r.valor,
                r.margen,
                r.minimo(),
                r.maximo()
            );
        }

        if let Some(f) = m.origen.que_falta() {
            println!("\n   para afinarlo haría falta: {f}");
        } else {
            println!("\n   no falta nada: el número está medido");
        }
    }

    // ---- Y la comparación, que es el argumento entero ----
    println!("\n\n╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  EL MARGEN, SEGÚN LOS DATOS QUE HAYA                                       ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");
    println!("   {:<46} {:>10}   {}", "situación", "margen", "precisión");
    println!("   {}", "-".repeat(74));
    for (titulo, p) in casos {
        let m = formulas_v2::pulso_maximo(p);
        let ancho = (m.rango.margen * 2.0) as usize;
        println!(
            "   {:<46} {:>6.1} ppm   {:<10} {}",
            titulo.trim_start_matches(|c: char| c.is_numeric() || c == ' ' || c == '·'),
            m.rango.margen,
            m.rango.como_va().nombre(),
            "█".repeat(ancho / 2)
        );
    }
    println!(
        "\n   ==> se estrecha de {:.1} a {:.1} ppm solo con los datos que el usuario puede dar.",
        formulas_v2::pulso_maximo(&p1).rango.margen,
        formulas_v2::pulso_maximo(&p4).rango.margen
    );
    println!("   ==> y NINGUNO de los cuatro casos deja de responder.\n");
}
