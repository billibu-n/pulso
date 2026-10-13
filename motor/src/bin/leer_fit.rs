//! **Lee sus 102 ficheros FIT y comprueba que el motor los entiende.**
//!
//! `cargo run --bin leer_fit -- ../datos/garmin_files/storage`
//!
//! LO QUE HACE, y por qué así:
//! 1. Lee **todos** los FIT de la carpeta.
//! 2. Compara **lo que el fichero declara** (los totales de Garmin) con **lo que el motor calcula**.
//!    Si no coinciden, **uno de los dos está mal** — y esa comprobación sale gratis.
//! 3. Y calcula **lo que el FIT NO trae**: la deriva y la eficiencia.

use motor::{fit, patron};
use std::path::Path;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or("../datos/garmin_files/storage".into());
    let (bien, mal) = fit::leer_carpeta(Path::new(&carpeta));

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  EL LECTOR DE FIT, CONTRA SUS FICHEROS DE VERDAD                           ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");
    println!("  ficheros leídos:  {}", bien.len());
    println!("  ficheros fallidos: {}", mal.len());
    for (n, e) in mal.iter().take(3) {
        println!("    ✗ {n}: {e}");
    }

    let total_muestras: usize = bien.iter().map(|(_, l)| l.sesion.muestras.len()).sum();
    println!("  muestras en total: {total_muestras}\n");

    // ---- La comprobacion cruzada: lo que dice Garmin contra lo que calcula el motor ----
    println!("▌ LA COMPROBACIÓN CRUZADA: ¿lo que calcula el motor cuadra con lo que dice el fichero?\n");
    println!("  {:<34} {:>8} {:>8} {:>8}", "sesión", "motor", "Garmin", "diferencia");
    println!("  {}", "-".repeat(64));

    let mut coinciden = 0;
    let mut no_coinciden = 0;
    for (nombre, l) in bien.iter().take(10) {
        let calculado: Vec<u16> = l.sesion.pulsos();
        if calculado.is_empty() {
            continue;
        }
        let media_motor = calculado.iter().map(|v| *v as f64).sum::<f64>() / calculado.len() as f64;
        match l.declarado.pulso_medio.valor() {
            Some(declarado) => {
                let dif = media_motor - f64::from(declarado);
                if dif.abs() <= 2.0 {
                    coinciden += 1;
                } else {
                    no_coinciden += 1;
                }
                let corto: String = nombre.chars().skip(22).take(12).collect();
                println!(
                    "  {:<34} {:>8.1} {:>8} {:>+8.1}",
                    corto, media_motor, declarado, dif
                );
            }
            None => println!("  {:<34} {:>8.1} {:>8}", &nombre[..34.min(nombre.len())], media_motor, "(no lo trae)"),
        }
    }
    println!("\n  coinciden (dentro de 2 ppm): {coinciden}    no coinciden: {no_coinciden}");

    // ---- Los mensajes que trae un FIT, en total ----
    println!("\n\n▌ QUÉ TRAEN LOS FICHEROS, SUMADO\n");
    let mut totales: std::collections::BTreeMap<String, usize> = Default::default();
    for (_, l) in &bien {
        for (k, v) in &l.mensajes {
            *totales.entry(k.clone()).or_insert(0) += v;
        }
    }
    for (k, v) in totales.iter().take(10) {
        println!("  {k:<24} {v:>9}");
    }

    // ---- Y lo que el FIT NO trae ----
    println!("\n\n▌ LO QUE EL FIT NO TRAE, CALCULADO POR EL MOTOR\n");
    let mut derivas: Vec<f64> = Vec::new();
    let mut escapados = 0;
    for (_, l) in &bien {
        let d = patron::deriva(&l.sesion);
        if d.muestras >= 100 {
            derivas.push(d.porcentaje());
            if d.porcentaje() > 10.0 {
                escapados += 1;
            }
        }
    }
    if !derivas.is_empty() {
        derivas.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let media = derivas.iter().sum::<f64>() / derivas.len() as f64;
        println!("  sesiones medidas:            {}", derivas.len());
        println!("  deriva media:                {media:.1} %");
        println!("  la más baja:                 {:.1} %", derivas.first().unwrap());
        println!("  la más alta:                 {:.1} %", derivas.last().unwrap());
        println!("  mediana:                     {:.1} %", derivas[derivas.len() / 2]);
        println!("  con el pulso ESCAPADO (+10 %): {escapados}");
    }
    println!();
}
