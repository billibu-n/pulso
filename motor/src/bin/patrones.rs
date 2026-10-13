//! **Los patrones, calculados por el motor con datos reales.**
//!
//! `cargo run --bin patrones -- ../datos/sesiones.json`
//!
//! Esto es lo que el FIT **no** trae: la deriva, la eficiencia y el desfase salen de **comparar** el
//! pulso con la potencia, y eso solo lo puede hacer quien tenga las dos series.

use motor::adaptador;
use motor::patron::{self, NivelDeriva};

fn main() {
    let ruta = std::env::args().nth(1).unwrap_or("../datos/sesiones.json".into());
    let texto = std::fs::read_to_string(&ruta).unwrap_or_else(|e| {
        eprintln!("No se pudo leer {ruta}: {e}");
        std::process::exit(1);
    });
    let semanas = adaptador::leer_semanas(&texto).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  LO QUE EL FIT NO TRAE: los patrones, deducidos de los sensores            ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

    let mut deriva_alta = 0;
    let mut deriva_baja = 0;
    let mut n = 0;

    println!(
        "  {:<12} {:>5} {:>10} {:>8} {:>16} {:>10}",
        "fecha", "min", "deriva", "margen", "nivel", "eficiencia"
    );
    println!("  {}", "-".repeat(70));

    for sem in &semanas {
        for s in &sem.sesiones {
            let sesion = s.a_sesion();
            let d = patron::deriva(&sesion);
            let ef = patron::eficiencia(&sesion);
            if d.muestras < 20 || ef.valor <= 0.0 {
                continue;
            }
            n += 1;
            if d.porcentaje() > 10.0 {
                deriva_alta += 1;
            } else if d.porcentaje() < 5.0 {
                deriva_baja += 1;
            }
            let nivel = match d.nivel() {
                NivelDeriva::Buena => "aguanta",
                NivelDeriva::Aparece => "aparece",
                NivelDeriva::SeEscapa => "SE ESCAPA",
            };
            println!(
                "  {:<12} {:>5.0} {:>9.1}% {:>7.1} {:>16} {:>10.3}",
                s.tipo.chars().take(12).collect::<String>(),
                s.duracion_s as f64 / 60.0,
                d.porcentaje(),
                d.rango.margen,
                nivel,
                ef.valor
            );
        }
    }

    println!("\n  {} sesiones con datos suficientes", n);
    println!("  con la deriva POR ENCIMA del 10 % (se le escapa el pulso): {deriva_alta}");
    println!("  con la deriva POR DEBAJO del 5 % (aguanta):                 {deriva_baja}");

    // ---- El desfase, con una sesion larga ----
    println!("\n\n▌ EL DESFASE: cuánto tarda el pulso en responder a la potencia\n");
    let mut medido = false;
    for sem in semanas.iter().rev() {
        for s in &sem.sesiones {
            let sesion = s.a_sesion();
            if let motor::Dato::Hay(r) = patron::desfase(&sesion, 60) {
                println!("  En la sesion de {:.0} minutos, el pulso responde a la potencia con unos
  {:.0} ± {:.0} segundos de retardo.", s.duracion_s as f64/60.0, r.valor, r.margen);
                println!();
                println!("  POR QUE IMPORTA: si el pulso tarda, una subida de potencia al final de una");
                println!("  sesion NO da tiempo a que se refleje -- y pareceria que fue facil.");
                medido = true;
                break;
            }
        }
        if medido { break; }
    }
    if !medido { println!("  No hay datos suficientes para medirlo."); }
    println!();
}
