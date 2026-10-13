// ¿QUE TRAEN LOS CAMPOS DE "POSICION" Y "FASE" DE PEDALEO, en TODOS los ficheros?
//
// POR QUE: el desequilibrio izquierda/derecha que el cliente VE en su aplicacion **no esta** en el
// campo `left_right_balance` (ese trae el numero de serie del sensor). Esta repartido en los campos
// `*_position` (potencia y cadencia de cada pierna) y `*_power_phase` (la fase del pedaleo).
// Hay que medirlo en todos antes de prometer nada.
use std::fs::File;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "../datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta).unwrap()
        .filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false)).collect();
    rutas.sort();

    let interesantes = ["avg_power_position", "max_power_position", "avg_cadence_position",
                        "avg_left_pco", "avg_right_pco", "enhanced_avg_respiration_rate",
                        "total_anaerobic_training_effect", "training_load_peak", "avg_vam"];
    let mut cuantos: std::collections::BTreeMap<&str, usize> = Default::default();
    let mut ejemplos: Vec<(String, String, String)> = Vec::new();
    let mut pot_izq: Vec<f64> = Vec::new();
    let mut pot_der: Vec<f64> = Vec::new();

    for ruta in &rutas {
        let Ok(mut f) = File::open(ruta) else { continue };
        let Ok(datos) = fitparser::from_reader(&mut f) else { continue };
        let nombre = ruta.file_name().unwrap().to_string_lossy().to_string();
        for r in datos.iter() {
            if format!("{:?}", r.kind()) != "Session" { continue; }
            for c in r.fields() {
                let n = c.name();
                if let Some(_) = interesantes.iter().find(|x| **x == n) {
                    *cuantos.entry(interesantes.iter().find(|x| **x == n).unwrap()).or_insert(0) += 1;
                    if ejemplos.len() < 6 {
                        ejemplos.push((nombre[10..26].to_string(), n.to_string(), format!("{:?}", c.value())));
                    }
                    if n == "avg_power_position" {
                        if let fitparser::Value::Array(v) = c.value() {
                            let nums: Vec<f64> = v.iter().filter_map(|x| match x {
                                fitparser::Value::UInt8(a) => Some(f64::from(*a)),
                                fitparser::Value::UInt16(a) => Some(f64::from(*a)),
                                _ => None }).collect();
                            if nums.len() == 2 {
                                pot_izq.push(nums[0]); pot_der.push(nums[1]);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("=== DE {} FICHEROS, EN CUANTOS VIENE CADA CAMPO ===", rutas.len());
    for (campo, n) in &cuantos { println!("  {n:>4}  {campo}"); }
    println!("\n=== EJEMPLOS ===");
    for (f, campo, v) in &ejemplos { println!("  {f} {campo:<32} {v}"); }

    if !pot_izq.is_empty() {
        println!("\n=== POTENCIA MEDIA DE CADA PIERNA ({} sesiones) ===", pot_izq.len());
        let media = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        println!("  izquierda: media {:.1} W", media(&pot_izq));
        println!("  derecha:   media {:.1} W", media(&pot_der));
        // Y EL PORCENTAJE, que es lo que el cliente ve en su aplicacion.
        let mut pares: Vec<(f64, f64)> = pot_izq.iter().cloned().zip(pot_der.iter().cloned())
            .filter(|(i, d)| *i + *d > 0.0).collect();
        pares.sort_by(|a, b| (a.0 / (a.0 + a.1)).partial_cmp(&(b.0 / (b.0 + b.1))).unwrap());
        if !pares.is_empty() {
            let pct_izq: Vec<f64> = pares.iter().map(|(i, d)| i / (i + d) * 100.0).collect();
            println!("\n=== PORCENTAJE DE POTENCIA EN LA PIERNA IZQUIERDA ===");
            println!("  minimo {:.1} % · mediana {:.1} % · maximo {:.1} %",
                pct_izq[0], pct_izq[pct_izq.len()/2], pct_izq[pct_izq.len()-1]);
            let desviados = pct_izq.iter().filter(|p| (**p - 50.0).abs() > 2.0).count();
            println!("  sesiones con desequilibrio mayor del 2 %: {desviados} de {}", pct_izq.len());
        }
    }
}
