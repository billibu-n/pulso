//! **La cadena completa: de sus ficheros FIT al informe.**
//!
//! `cargo run --bin informe -- ../datos/garmin_files/storage [salida.md]`
//!
//! Esto es lo que el cliente recibe. Y fíjate en el recorrido: **no hay nada escrito a mano** —
//! los ficheros entran, el motor calcula, y el informe sale.

use motor::datos::{Dato, Persona};
use motor::{fit, informe, tendencia};
use std::path::Path;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or("../datos/garmin_files/storage".into());
    let salida = std::env::args().nth(2);

    let (ficheros, malos) = fit::leer_carpeta(Path::new(&carpeta));
    if ficheros.is_empty() {
        eprintln!("No se leyó ningún FIT de {carpeta}");
        std::process::exit(1);
    }

    // ---- La persona: lo que el FIT puede dar, más lo que hay que aportar ----
    //
    // EL FIT NO TRAE la edad, el peso ni el umbral: eso no lo mide el aparato. Aquí se ponen los datos
    // que el usuario puede aportar, y **si no están, se dejan vacíos** — el motor calcula igual.
    let mut p = Persona::desconocida();
    p.edad = Dato::Hay(60);
    p.pulso_reposo = Dato::Hay(48);
    p.peso_kg = Dato::Hay(70.2);
    p.altura_cm = Dato::Hay(178.0);
    p.grasa_corporal_pct = Dato::Hay(19.1);
    p.umbral_lactato = Dato::Hay(169);

    // Y la frecuencia máxima: **se busca en TODAS las sesiones**, que es donde está.
    // Esto corrige un fallo anterior: se usaba la de una sola sesión, y salía corta.
    let maxima_real = ficheros
        .iter()
        .filter_map(|(_, l)| l.declarado.pulso_maximo.valor())
        .max();
    if let Some(m) = maxima_real {
        p.pulso_maximo_medido = Dato::Hay(m);
    }

    // ---- La última sesión CON DATOS SUFICIENTES, que es sobre la que se informa ----
    //
    // Se corrigió el 2026-10-13: la primera versión cogía la ÚLTIMA sin más, y si esa no traía pulso
    // y potencia juntos, el informe decía «la deriva no se pudo medir» — **cuando en las otras 101 sí
    // se podía**. El orden de los ficheros no dice nada sobre cuál sirve.
    let ultima = ficheros
        .iter()
        .rev()
        .find(|(_, l)| {
            l.sesion.pulsos().len() >= 100
                && l.sesion.muestras.iter().filter(|m| m.potencia_w.valor().is_some()).count() >= 100
        })
        .map(|(_, l)| &l.sesion)
        .or_else(|| ficheros.last().map(|(_, l)| &l.sesion));

    // ---- La serie del consumo de oxígeno, para la tendencia ----
    // El FIT no la trae: la estima Garmin y se guarda aparte. Si no está, no hay tendencia.
    let serie_vo2: Option<tendencia::Serie> = None;

    let h = informe::hechos(&p, ultima, serie_vo2.as_ref());
    let texto = informe::generar(&h);

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  DE SUS FICHEROS AL INFORME, sin nada escrito a mano                       ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");
    println!("  ficheros leídos:   {}", ficheros.len());
    println!("  ficheros con error: {}", malos.len());
    println!("  máxima real encontrada en TODAS las sesiones: {}", maxima_real.unwrap_or(0));
    println!("  sesión del informe: {} muestras con pulso",
             ultima.map(|s| s.pulsos().len()).unwrap_or(0));
    println!("  datos de la persona: {} de {}", h.persona_datos.0, h.persona_datos.1);
    println!();

    // ---- Y la comprobacion que evita el fallo silencioso ----
    let legitimos: Vec<f64> = {
        let mut v = vec![
            h.umbral.rango.valor, h.umbral.rango.margen, h.maxima.valor, h.maxima.margen,
        ];
        for (_, desde, hasta) in &h.zonas_ciclismo {
            v.push(desde.valor);
            v.push(desde.margen);
            v.push(hasta.valor);
        }
        if let Some((p, m, _, n)) = &h.deriva {
            v.push(*p);
            v.push(*m);
            v.push(*n as f64);
        }
        if let Some(e) = &h.eficiencia {
            v.push(e.valor);
            v.push(e.margen);
        }
        if let Some(vv) = &h.vo2 {
            v.push(vv.valor);
            v.push(vv.margen);
        }
        v
    };
    let problemas = informe::comprobar_que_no_inventa(&texto, &legitimos);
    println!("  ▌ ¿EL INFORME INVENTA ALGÚN NÚMERO?");
    if problemas.is_empty() {
        println!("    No: todos los números del texto salieron de un cálculo del motor.");
    } else {
        for x in problemas.iter().take(5) {
            println!("    ⚠ {x}");
        }
    }

    match salida {
        Some(ruta) => {
            std::fs::write(&ruta, &texto).unwrap_or_else(|e| eprintln!("No se pudo escribir: {e}"));
            println!("\n  Informe escrito en: {ruta}");
        }
        None => {
            println!("\n\n{}", "-".repeat(78));
            println!("{texto}");
        }
    }
}
