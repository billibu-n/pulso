//! Mide la tendencia de una serie, con el motor.
//!
//! `cargo run --bin tendencia -- ../datos/serie-vo2.json`
//!
//! POR QUÉ EXISTE: «¿está mejorando?» es la pregunta central del encargo, y **con un solo mes no se
//! puede contestar**. Este programa la contesta con datos, y **dice cuántos puntos usó** — porque una
//! tendencia de dos puntos y una de cincuenta no valen lo mismo.

use motor::fuentes::{Procedencia, Valor};
use motor::tendencia::{tolerancia, Direccion, Serie};
use serde::Deserialize;

#[derive(Deserialize)]
struct Raiz { series: Vec<SerieJson> }
#[derive(Deserialize)]
struct SerieJson { que: String, puntos: Vec<PuntoJson> }
#[derive(Deserialize)]
struct PuntoJson { valor: f64, fecha: String }

fn main() {
    let ruta = std::env::args().nth(1).unwrap_or("../datos/serie-vo2.json".into());
    let texto = std::fs::read_to_string(&ruta).unwrap_or_else(|e| {
        eprintln!("No se pudo leer {ruta}: {e}");
        std::process::exit(1);
    });
    let raiz: Raiz = serde_json::from_str(&texto).unwrap_or_else(|e| {
        eprintln!("El JSON no se entiende: {e}");
        std::process::exit(1);
    });

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  LA TENDENCIA, medida por el motor con datos reales                  ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");

    for sj in raiz.series {
        let s = Serie::nueva(
            &sj.que,
            sj.puntos.iter()
                .map(|p| Valor::nuevo(p.valor, Procedencia::AparatoDeEntrenamiento, &p.fecha))
                .collect(),
        );
        println!();
        match s.tendencia().prestado() {
            Some(t) => {
                println!("  {}", t.frase(tolerancia::VO2));
                println!();
                println!("    puntos usados ....... {}", t.puntos);
                println!("    desde ............... {}", t.desde);
                println!("    hasta ............... {}", t.hasta);
                println!("    primero ............. {:.1}", t.primero);
                println!("    último .............. {:.1}", t.ultimo);
                println!("    cambio .............. {:+.1}  ({:+.1} %)", t.cambio, t.relativo * 100.0);
                if let Some(m) = &t.minimo {
                    println!("    el más bajo ......... {:.1}  ({})", m.valor, m.fecha);
                }
                let d = t.direccion(tolerancia::VO2);
                println!();
                println!("    dirección ........... {}", match d {
                    Direccion::Sube => "SUBE",
                    Direccion::Baja => "BAJA",
                    Direccion::Estable => "SE MANTIENE (dentro de la tolerancia)",
                });
            }
            None => println!("  NO SE PUEDE: hacen falta al menos dos puntos en el tiempo"),
        }
    }
    println!();
}
