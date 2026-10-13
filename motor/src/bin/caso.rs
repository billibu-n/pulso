//! Lee el caso sintético del ciclista de 60 años y lo pasa por el motor.
//!
//! `cargo run --bin caso -- ../datos-sinteticos/caso-60-ciclismo.json`
//!
//! POR QUÉ ESTO ES IMPORTANTE: es el motor enfrentándose **al caso del cliente** (60 años, ciclismo
//! competitivo) con datos inventados, **antes** de tener los suyos. Cuando lleguen de verdad, se cambia
//! el fichero y **este mismo programa** dice qué pasa.

use motor::adaptador;
use motor::esfuerzo::{self, NivelSalto};
use motor::formulas;
use motor::juicio::Juicio;

fn main() {
    let ruta = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../datos-sinteticos/caso-60-ciclismo.json".to_string());

    let texto = std::fs::read_to_string(&ruta).unwrap_or_else(|e| {
        eprintln!("No se pudo leer {ruta}: {e}");
        std::process::exit(1);
    });

    let persona = adaptador::leer_perfil(&texto).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let semanas = adaptador::leer_semanas(&texto).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  EL CASO: ciclista de 60 años, competitivo, datos SINTÉTICOS            ║");
    println!("║  (inventados: el cliente aún no ha dado los suyos)                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");

    // ---- 1. El aviso de la estimación, que es lo más urgente en su caso ----
    println!("\n▌ 1 · EL AVISO SOBRE SU FRECUENCIA MÁXIMA\n");
    match esfuerzo::aviso_de_la_estimacion(&persona) {
        Some(a) => {
            println!("  ⚠  {a}");
            let est = formulas::pulso_maximo(&persona);
            if let Some(v) = est.valor {
                println!("\n  El motor estima: {v} ppm");
                println!("  Fuente: {}", est.fuente);
            }
        }
        None => println!("  (la máxima está medida: no hay nada que avisar)"),
    }

    // ---- 2. Las zonas ----
    println!("\n▌ 2 · SUS ZONAS DE ENTRENAMIENTO\n");
    let z = formulas::zonas(&persona);
    match z.valor {
        Some(zs) => {
            for (zona, a, b) in zs {
                println!("  {:<10} {:>4} - {:<4} ppm", zona.nombre(), a, b);
            }
            println!("\n  Fuente: {}", z.fuente);
        }
        None => {
            println!("  NO SE PUEDEN CALCULAR");
            for f in &z.falta {
                println!("    falta: {f}");
            }
        }
    }

    // ---- 3. El salto, semana a semana ----
    println!("\n▌ 3 · EL SALTO DE CADA SEMANA RESPECTO AL MES ANTERIOR\n");
    println!("  El cliente hace ciclismo. Aquí se mide la CARGA, que vale para cualquier deporte,");
    println!("  y también la DURACIÓN, que es lo que midió el estudio. Las dos, para compararlas.\n");

    let mut anteriores: Vec<motor::Sesion> = Vec::new();

    for sem in &semanas {
        let sesiones: Vec<motor::Sesion> = sem.sesiones.iter().map(|s| s.a_sesion()).collect();
        println!("  ── semana {} ──────────────────────────────────────────", sem.semana);

        for (i, s) in sesiones.iter().enumerate() {
            let nombre = &sem.sesiones[i].tipo;
            let dur_min = s.duracion().valor().unwrap_or(0) / 60;

            let por_carga = esfuerzo::salto(s, &anteriores, &persona, true);
            let por_dur = esfuerzo::salto(s, &anteriores, &persona, false);

            let texto_carga = match &por_carga.valor {
                Some((sal, nivel)) => format!(
                    "carga {:>6.1} vs {:>6.1} = {:>5.0} %  [{}]",
                    sal.hoy, sal.referencia, sal.subida() * 100.0, nivel.nombre()
                ),
                None => format!("carga: NO SE PUEDE — {}", por_carga.fuente),
            };
            let texto_dur = match &por_dur.valor {
                Some((sal, nivel)) => format!(
                    "{:>4} min vs {:>4} min = {:>5.0} %  [{}]",
                    sal.hoy / 60.0, sal.referencia / 60.0, sal.subida() * 100.0, nivel.nombre()
                ),
                None => "duración: sin referencia".to_string(),
            };

            println!("     {:<16} ({:>3} min)", nombre, dur_min);
            println!("        {texto_carga}");
            println!("        {texto_dur}");

            // Cuando el nivel es peligro, se dice POR QUÉ, con la explicación del estudio.
            if let Some((_, nivel)) = &por_carga.valor {
                if *nivel == NivelSalto::Peligro || *nivel == NivelSalto::Aviso {
                    println!("        ⚠  {}", nivel.explicacion());
                }
            }
        }

        anteriores.extend(sesiones);
        println!();
    }

    // ---- 4. El juicio completo sobre la última sesión ----
    println!("▌ 4 · EL JUICIO COMPLETO SOBRE LA ÚLTIMA SESIÓN\n");
    if let Some(ultima) = anteriores.last() {
        println!("{}", Juicio::juzgar(&persona, ultima).informe());
    }
}
