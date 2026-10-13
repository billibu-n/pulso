//! **El HRV de sus 102 actividades**, que está dentro del FIT y Garmin Connect no enseña.
//!
//! `cargo run --bin hrv_real -- ../datos/garmin_files/storage`

use motor::{fit, hrv};
use std::path::Path;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or("../datos/garmin_files/storage".into());
    let (bien, _) = fit::leer_carpeta(Path::new(&carpeta));

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  EL HRV: el dato que está en el FIT y Garmin Connect no enseña             ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

    let mut todos: Vec<hrv::Hrv> = Vec::new();
    let mut total_intervalos = 0usize;
    let mut total_descartados = 0usize;

    for (_, l) in &bien {
        let iv = hrv::intervalos_de(&l.sesion);
        if iv.is_empty() {
            continue;
        }
        total_intervalos += iv.len();
        let h = hrv::calcular(iv);
        total_descartados += h.descartados;
        todos.push(h);
    }

    println!("  actividades con HRV:        {}", todos.len());
    println!("  intervalos entre latidos:   {total_intervalos}");
    println!("  descartados por artefactos: {total_descartados}  ({:.1} %)",
             100.0 * total_descartados as f64 / total_intervalos.max(1) as f64);

    // La comprobacion: el pulso deducido del RR tiene que cuadrar con el que midio el aparato
    println!("\n▌ LA COMPROBACIÓN: ¿el pulso deducido del HRV cuadra con el del aparato?\n");
    println!("  {:<10} {:>12} {:>12} {:>10}", "latidos", "pulso HRV", "pulso Garmin", "diferencia");
    println!("  {}", "-".repeat(48));
    let mut comparadas = 0; let mut cuadran = 0;
    for (i, (_, l)) in bien.iter().enumerate() {
        let iv = hrv::intervalos_de(&l.sesion);
        if iv.len() < 120 { continue; }
        let h = hrv::calcular(iv);
        if let Some(declarado) = l.declarado.pulso_medio.valor() {
            let deducido = h.pulso_deducido();
            let dif = deducido - f64::from(declarado);
            comparadas += 1;
            if dif.abs() <= 5.0 { cuadran += 1; }
            if comparadas <= 8 {
                println!("  {:<10} {:>12.1} {:>12} {:>+10.1}", h.latidos, deducido, declarado, dif);
            }
        }
        let _ = i;
    }
    println!("\n  cuadran (dentro de 5 ppm): {cuadran} de {comparadas}");

    // Los numeros del HRV
    println!("\n\n▌ SU HRV, RESUMIDO\n");
    let validos: Vec<&hrv::Hrv> = todos.iter().filter(|h| h.latidos >= 300).collect();
    if !validos.is_empty() {
        let rmssd: Vec<f64> = validos.iter().map(|h| h.rmssd.valor).collect();
        let coef: Vec<f64> = validos.iter().map(|h| h.coeficiente.valor).collect();
        let media = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        println!("  sesiones con HRV fiable (300+ latidos): {}", validos.len());
        println!("  RMSSD medio:                {:.1} ms", media(&rmssd));
        println!("  RMSSD más bajo:             {:.1} ms", rmssd.iter().cloned().fold(f64::MAX, f64::min));
        println!("  RMSSD más alto:             {:.1} ms", rmssd.iter().cloned().fold(f64::MIN, f64::max));
        println!("  coeficiente de variación:   {:.1} %", media(&coef));
    }
    println!();
}
