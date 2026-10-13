// LA PREGUNTA QUE DECIDE EL MODULO DE LA RODILLA: ¿de donde sale el desequilibrio?
//
// DOS INTERPRETACIONES QUE SE PARECEN Y NO SON LO MISMO, y equivocarse tiene consecuencias:
//   - `avg_power_position` = potencia media **por POSTURA** (sentado / de pie). Documentado por Garmin.
//   - `avg_cadence_position` = cadencia media **por POSTURA**, por la misma razon.
//
// Y LO QUE **SI** ES POR PIERNA, sin ambiguedad:
//   - `avg_left_power_phase` y `avg_right_power_phase` -- la FASE del pedaleo de cada pierna.
//   - `avg_left_pco` / `avg_right_pco` -- el CENTRO DE PRESION sobre el pedal, en mm.
//
// Este programa mide LOS TRES, para saber cual sirve de verdad.
use std::fs::File;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "../datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta).unwrap()
        .filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false)).collect();
    rutas.sort();

    let mut pco_izq: Vec<f64> = Vec::new();
    let mut pco_der: Vec<f64> = Vec::new();
    let mut fase_izq: Vec<Vec<f64>> = Vec::new();
    let mut fase_der: Vec<Vec<f64>> = Vec::new();
    let mut sentado: Vec<f64> = Vec::new();
    let mut depie: Vec<f64> = Vec::new();
    let mut cad_sentado: Vec<f64> = Vec::new();
    let mut cad_depie: Vec<f64> = Vec::new();

    for ruta in &rutas {
        let Ok(mut f) = File::open(ruta) else { continue };
        let Ok(datos) = fitparser::from_reader(&mut f) else { continue };
        for r in datos.iter() {
            if format!("{:?}", r.kind()) != "Session" { continue; }
            for c in r.fields() {
                let nums: Vec<f64> = match c.value() {
                    fitparser::Value::Array(v) => v.iter().filter_map(|x| match x {
                        fitparser::Value::UInt8(a) => Some(f64::from(*a)),
                        fitparser::Value::UInt16(a) => Some(f64::from(*a)),
                        fitparser::Value::Float64(a) => Some(*a),
                        fitparser::Value::SInt8(a) => Some(f64::from(*a)),
                        _ => None }).collect(),
                    fitparser::Value::SInt8(a) => vec![f64::from(*a)],
                    _ => Vec::new(),
                };
                match c.name() {
                    "avg_left_pco" => if let Some(v) = nums.first() { pco_izq.push(*v); },
                    "avg_right_pco" => if let Some(v) = nums.first() { pco_der.push(*v); },
                    "avg_left_power_phase" => if nums.len() == 4 { fase_izq.push(nums); },
                    "avg_right_power_phase" => if nums.len() == 4 { fase_der.push(nums); },
                    "avg_power_position" => if nums.len() == 2 {
                        if nums[0] > 0.0 { sentado.push(nums[0]); }
                        if nums[1] > 0.0 { depie.push(nums[1]); }
                    },
                    "avg_cadence_position" => if nums.len() == 2 {
                        if nums[0] > 0.0 { cad_sentado.push(nums[0]); }
                        if nums[1] > 0.0 { cad_depie.push(nums[1]); }
                    },
                    _ => {}
                }
            }
        }
    }

    let media = |v: &[f64]| if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 };

    println!("=== POR POSTURA (sentado / de pie) — NO es izquierda/derecha ===");
    println!("  potencia sentado: {} sesiones, media {:.1} W", sentado.len(), media(&sentado));
    println!("  potencia de pie:  {} sesiones, media {:.1} W", depie.len(), media(&depie));
    println!("  cadencia sentado: media {:.1} rpm · de pie: media {:.1} rpm", media(&cad_sentado), media(&cad_depie));

    println!("\n=== POR PIERNA — esto SI es izquierda/derecha ===");
    println!("  PCO (centro de presion, mm):");
    println!("    izquierda: {} valores, media {:.2} mm", pco_izq.len(), media(&pco_izq));
    println!("    derecha:   {} valores, media {:.2} mm", pco_der.len(), media(&pco_der));
    let distintos = pco_izq.iter().zip(pco_der.iter()).filter(|(i, d)| i != d).count();
    println!("    sesiones donde izquierda y derecha DIFIEREN: {distintos} de {}", pco_izq.len().min(pco_der.len()));
    println!("    (si son todas iguales, el sensor no los distingue)");

    println!("\n  FASE de pedaleo (4 angulos por pierna):");
    println!("    izquierda: {} sesiones", fase_izq.len());
    println!("    derecha:   {} sesiones", fase_der.len());
    if let (Some(i), Some(d)) = (fase_izq.first(), fase_der.first()) {
        println!("    ejemplo izq: {:?}", i.iter().map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>());
        println!("    ejemplo der: {:?}", d.iter().map(|x| (x * 10.0).round() / 10.0).collect::<Vec<_>>());
    }
    let difieren = fase_izq.iter().zip(fase_der.iter())
        .filter(|(i, d)| i.iter().zip(d.iter()).any(|(a, b)| (a - b).abs() > 1.0)).count();
    println!("    sesiones donde las dos fases DIFIEREN: {difieren} de {}", fase_izq.len().min(fase_der.len()));
}
