// LA LINEA BASE DE ESTE CLIENTE, medida de sus 102 ficheros, SIN depender del crate.
//
// POR QUE: el modulo `avisos` va a comparar cada sesion contra SU PROPIA linea base y no contra tablas
// de poblacion (lo exige `docs/ENCARGO-farellones-2026.md` §4). Para elegir un corte honesto hay que
// saber primero como se reparten sus numeros. Este programa los reparte.
//
// La deriva se calcula con LA MISMA FORMULA que `motor/src/patron.rs` (eficiencia de la primera mitad
// contra la segunda, en vatios por pulsacion). No se inventa otro calculo: si aqui saliera otro numero,
// el modulo estaria midiendo otra cosa.
use std::collections::BTreeMap;
use std::fs::File;

/// La deriva de una sesion: la misma cuenta que `patron::deriva`.
/// Devuelve `None` si no hay 20 pares (pulso, potencia), que es el minimo que usa el motor.
fn deriva(pares: &[(f64, f64)]) -> Option<f64> {
    if pares.len() < 20 {
        return None;
    }
    let mitad = pares.len() / 2;
    let media = |v: &[(f64, f64)]| -> (f64, f64) {
        let p = v.iter().map(|x| x.0).sum::<f64>() / v.len() as f64;
        let w = v.iter().map(|x| x.1).sum::<f64>() / v.len() as f64;
        (p, w)
    };
    let (p1, w1) = media(&pares[..mitad]);
    let (p2, w2) = media(&pares[mitad..]);
    if p1 <= 0.0 || p2 <= 0.0 {
        return None;
    }
    let e1 = w1 / p1;
    let e2 = w2 / p2;
    Some(((e1 - e2) / e1) * 100.0)
}

fn cuartos(v: &[f64]) -> (f64, f64, f64) {
    let n = v.len();
    (v[n / 4], v[n / 2], v[(3 * n) / 4])
}

fn main() {
    let carpeta = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/billibu/projects/pulso/datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false))
        .collect();
    rutas.sort();

    let mut derivas: Vec<f64> = Vec::new();
    let mut pares_por_sesion: Vec<usize> = Vec::new();
    let mut sesiones = 0usize;
    let mut sin_potencia = 0usize;
    let mut con_pocos_pares = 0usize;

    for ruta in &rutas {
        let Ok(mut f) = File::open(ruta) else { continue };
        let Ok(datos) = fitparser::from_reader(&mut f) else { continue };
        sesiones += 1;

        let mut pares: Vec<(f64, f64)> = Vec::new();
        let mut hay_potencia = false;
        for r in datos.iter() {
            if format!("{:?}", r.kind()) != "Record" {
                continue;
            }
            let mut p: Option<f64> = None;
            let mut w: Option<f64> = None;
            for c in r.fields() {
                match c.value() {
                    fitparser::Value::UInt8(x) if c.name() == "heart_rate" => p = Some(f64::from(*x)),
                    fitparser::Value::UInt16(x) if c.name() == "power" => w = Some(f64::from(*x)),
                    fitparser::Value::UInt8(x) if c.name() == "power" => w = Some(f64::from(*x)),
                    _ => {}
                }
            }
            if w.is_some() {
                hay_potencia = true;
            }
            if let (Some(a), Some(b)) = (p, w) {
                if a > 0.0 && b > 0.0 {
                    pares.push((a, b));
                }
            }
        }
        if !hay_potencia {
            sin_potencia += 1;
            continue;
        }
        match deriva(&pares) {
            Some(d) => {
                derivas.push(d);
                pares_por_sesion.push(pares.len());
            }
            None => con_pocos_pares += 1,
        }
    }

    println!("=== DERIVA: {} ficheros, {sesiones} sesiones ===", rutas.len());
    println!("  sin sensor de potencia ......... {sin_potencia}");
    println!("  con potencia pero <20 pares .... {con_pocos_pares}");
    println!("  con deriva calculable .......... {}", derivas.len());

    let mut d = derivas.clone();
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let media = d.iter().sum::<f64>() / d.len() as f64;
    let var = d.iter().map(|x| (x - media).powi(2)).sum::<f64>() / d.len() as f64;
    let desv = var.sqrt();
    let (q1, med, q3) = cuartos(&d);
    println!("\n  minimo {:.2} · Q1 {:.2} · MEDIANA {:.2} · Q3 {:.2} · maximo {:.2}", d[0], q1, med, q3, d[d.len() - 1]);
    println!("  media {media:.2} · desviacion {desv:.2}");
    println!("  (el documento del encargo dice: media 9,7 % · mediana 10,8 % · 43 por encima del 10 %)");
    for k in [0.0, 5.0, 10.0, 12.0, 15.0, 20.0] {
        let n = d.iter().filter(|x| **x > k).count();
        println!("    por encima del {k:>4.1} %: {n:>3}");
    }
    println!("  media + 1 desviacion = {:.2} %", media + desv);

    // La deriva NEGATIVA: la eficiencia MEJORO en la segunda mitad. Es un dato real y hay que verlo.
    let negativas = d.iter().filter(|x| **x < 0.0).count();
    println!("  sesiones con deriva NEGATIVA (mejoro en la segunda mitad): {negativas}");
    println!("  (si son muchas, la cuenta de `media + desviacion` no describe a este cliente)");

    // Y la pregunta del corte: ¿la mediana mas una desviacion separa algo util, o cae en tierra de nadie?
    println!("\n  LA DISTRIBUCION, EN 10 TRAMOS:");
    let min = d[0];
    let max = d[d.len() - 1];
    let paso = (max - min) / 10.0;
    for i in 0..10 {
        let a = min + paso * i as f64;
        let b = a + paso;
        let n = d.iter().filter(|x| **x >= a && **x < b).count();
        let barra = "#".repeat(n);
        println!("    {a:>7.1} a {b:>7.1}: {n:>3} {barra}");
    }
}
