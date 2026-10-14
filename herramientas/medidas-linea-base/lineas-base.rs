// LAS LINEAS BASE: ¿que corte separa de verdad, en SUS datos?
//
// POR QUE: el encargo exige comparar cada sesion CON SU PROPIA linea base y no contra tablas de
// poblacion. Pero "su propia linea base" hay que definirla con numeros, no con adjetivos. Aqui se
// prueban los candidatos sobre sus 102 ficheros y se mira cuantas sesiones separa cada uno.
//
// Y hay una razon de fondo para preferir la MAD (desviacion absoluta mediana) a la desviacion tipica:
// su distribucion tiene colas (una deriva de -23,6 % y otra de +34,7 %), y la desviacion tipica se deja
// arrastrar por ellas. La MAD no.
use std::fs::File;

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
    Some((((w1 / p1) - (w2 / p2)) / (w1 / p1)) * 100.0)
}

fn cuartos(v: &mut Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    (v[n / 4], v[n / 2], v[(3 * n) / 4])
}

fn diferencia_circular(a: f64, b: f64) -> f64 {
    let mut d = (a - b) % 360.0;
    if d > 180.0 {
        d -= 360.0;
    }
    if d < -180.0 {
        d += 360.0;
    }
    d
}

fn cuatro(v: &fitparser::Value) -> Option<[f64; 4]> {
    let fitparser::Value::Array(a) = v else { return None };
    let n: Vec<f64> = a
        .iter()
        .filter_map(|x| match x {
            fitparser::Value::UInt8(b) => Some(f64::from(*b)),
            fitparser::Value::UInt16(b) => Some(f64::from(*b)),
            fitparser::Value::Float32(b) => Some(f64::from(*b)),
            fitparser::Value::Float64(b) => Some(*b),
            _ => None,
        })
        .collect();
    if n.len() == 4 {
        Some([n[0], n[1], n[2], n[3]])
    } else {
        None
    }
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
    let mut asimetrias: Vec<f64> = Vec::new();

    for ruta in &rutas {
        let Ok(mut f) = File::open(ruta) else { continue };
        let Ok(datos) = fitparser::from_reader(&mut f) else { continue };
        let mut pares: Vec<(f64, f64)> = Vec::new();
        let mut izq: Option<[f64; 4]> = None;
        let mut der: Option<[f64; 4]> = None;

        for r in datos.iter() {
            let tipo = format!("{:?}", r.kind());
            if tipo == "Record" {
                let mut p: Option<f64> = None;
                let mut w: Option<f64> = None;
                for c in r.fields() {
                    match (c.name(), c.value()) {
                        ("heart_rate", fitparser::Value::UInt8(x)) => p = Some(f64::from(*x)),
                        ("power", fitparser::Value::UInt16(x)) => w = Some(f64::from(*x)),
                        ("power", fitparser::Value::UInt8(x)) => w = Some(f64::from(*x)),
                        _ => {}
                    }
                }
                if let (Some(a), Some(b)) = (p, w) {
                    if a > 0.0 && b > 0.0 {
                        pares.push((a, b));
                    }
                }
            }
            if tipo == "Session" {
                for c in r.fields() {
                    match c.name() {
                        "avg_left_power_phase" => izq = cuatro(c.value()),
                        "avg_right_power_phase" => der = cuatro(c.value()),
                        _ => {}
                    }
                }
            }
        }

        if let Some(d) = deriva(&pares) {
            derivas.push(d);
        }
        if let (Some(a), Some(b)) = (izq, der) {
            asimetrias.push(
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| diferencia_circular(*x, *y).abs())
                    .fold(0.0f64, f64::max),
            );
        }
    }

    // ---- LA DERIVA ----
    println!("════ A. LA DERIVA: {} sesiones ════", derivas.len());
    let mut d = derivas.clone();
    let (q1, med, q3) = cuartos(&mut d);
    let desviaciones: Vec<f64> = d.iter().map(|x| (x - med).abs()).collect();
    let mut dv = desviaciones.clone();
    dv.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mad = dv[dv.len() / 2];
    println!("  mediana {med:.2} · Q1 {q1:.2} · Q3 {q3:.2} · MAD {mad:.2}");
    println!("  (MAD = desviacion absoluta mediana: no la mueven las colas)");
    for k in [1.0, 1.5, 2.0] {
        let corte = med + k * mad;
        let n = d.iter().filter(|x| **x > corte).count();
        println!("    mediana + {k} x MAD = {corte:>6.2} %  ->  separa {n:>3} de {}", d.len());
    }
    println!("  y los cortes PUBLICADOS, sobre sus datos:");
    for k in [5.0, 10.0] {
        let n = d.iter().filter(|x| **x > k).count();
        println!("    mas del {k:>4.1} %: {n:>3} de {}", d.len());
    }

    // ---- LA ASIMETRIA ----
    println!("\n════ B. LA ASIMETRIA DE FASE: {} sesiones ════", asimetrias.len());
    let mut a = asimetrias.clone();
    let (aq1, amed, aq3) = cuartos(&mut a);
    let adesv: Vec<f64> = a.iter().map(|x| (x - amed).abs()).collect();
    let mut adv = adesv.clone();
    adv.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let amad = adv[adv.len() / 2];
    let amedia = a.iter().sum::<f64>() / a.len() as f64;
    let avar = a.iter().map(|x| (x - amedia).powi(2)).sum::<f64>() / a.len() as f64;
    println!("  mediana {amed:.2} · Q1 {aq1:.2} · Q3 {aq3:.2} · MAD {amad:.2} · media {amedia:.2} · desv {:.2}", avar.sqrt());
    for k in [1.0, 2.0, 2.5, 3.0] {
        let corte = amed + k * amad;
        let n = a.iter().filter(|x| **x > corte).count();
        println!("    mediana + {k} x MAD = {corte:>6.2} grados  ->  separa {n:>3} de {}", a.len());
    }

    println!("\n  ¿EL CORTE CAE EN TIERRA DE NADIE? (cuantas sesiones quedan pegadas al corte)");
    for k in [2.0, 2.5, 3.0] {
        let corte = amed + k * amad;
        let cerca = a.iter().filter(|x| (**x - corte).abs() < 1.0).count();
        println!("    corte {corte:>6.2}: {cerca} sesiones a menos de 1 grado");
    }
}
