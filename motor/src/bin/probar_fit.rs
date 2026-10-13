//! Lee un FIT y enseña QUE TRAE, para poder escribir el lector sobre datos reales.
//!
//! `cargo run --bin probar_fit -- <fichero.fit>`

use std::fs::File;

fn main() {
    let ruta = match std::env::args().nth(1) {
        Some(r) => r,
        None => {
            eprintln!("Uso: probar_fit <fichero.fit>");
            std::process::exit(1);
        }
    };
    let mut f = File::open(&ruta).unwrap_or_else(|e| {
        eprintln!("No se pudo abrir {ruta}: {e}");
        std::process::exit(1);
    });

    let datos = fitparser::from_reader(&mut f).unwrap_or_else(|e| {
        eprintln!("El FIT no se pudo leer: {e:?}");
        std::process::exit(1);
    });

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  QUÉ TRAE UN FIT, POR DENTRO                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!("\n  {ruta}");
    println!("  mensajes en el fichero: {}\n", datos.len());

    // Cuántos mensajes de cada tipo
    let mut cuenta: std::collections::BTreeMap<String, usize> = Default::default();
    for r in &datos {
        *cuenta.entry(format!("{:?}", r.kind())).or_insert(0) += 1;
    }
    println!("  POR TIPO DE MENSAJE:");
    for (k, v) in cuenta.iter().take(12) {
        println!("    {k:<24} {v}");
    }

    // Los campos de los registros: lo que de verdad nos interesa
    println!("\n  CAMPOS DE LOS MENSAJES 'Record' (las muestras):");
    let mut campos: std::collections::BTreeMap<String, (usize, String)> = Default::default();
    for r in datos.iter().filter(|r| format!("{:?}", r.kind()) == "Record") {
        for c in r.fields() {
            let e = campos.entry(c.name().to_string()).or_insert((0, c.units().to_string()));
            e.0 += 1;
        }
    }
    for (nombre, (n, unidades)) in &campos {
        println!("    {nombre:<28} {n:>6} veces   unidades: {unidades}");
    }

    // Y un ejemplo de los primeros registros
    println!("\n  LOS TRES PRIMEROS REGISTROS, ENTEROS:");
    for r in datos.iter().filter(|r| format!("{:?}", r.kind()) == "Record").take(3) {
        println!("    ---");
        for c in r.fields() {
            println!("      {:<28} {:?}", c.name(), c.value());
        }
    }

    // El HRV: la variabilidad latido a latido
    println!("\n  MENSAJES 'Hrv' (la variabilidad latido a latido):");
    let hrvs: Vec<_> = datos.iter().filter(|r| format!("{:?}", r.kind()) == "Hrv").collect();
    println!("    cuantos: {}", hrvs.len());
    for (i, r) in hrvs.iter().take(2).enumerate() {
        println!("    --- mensaje {i} ---");
        for c in r.fields() {
            match c.value() {
                fitparser::Value::Array(v) => println!("      {:<22} Array de {} valores: {:?}", c.name(), v.len(), &v[..v.len().min(5)]),
                otro => println!("      {:<22} {:?}", c.name(), otro),
            }
        }
    }

    // Y los mensajes de sesión, que traen los totales
    println!("\n  CAMPOS DEL MENSAJE 'Session' (los totales):");
    for r in datos.iter().filter(|r| format!("{:?}", r.kind()) == "Session") {
        for c in r.fields() {
            println!("      {:<28} {:?}", c.name(), c.value());
        }
    }
}
