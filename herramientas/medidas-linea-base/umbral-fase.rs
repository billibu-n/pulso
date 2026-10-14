// ¿Cuanta gente cae EXACTAMENTE en el corte? Es lo que decide si el corte tiene una direccion sola.
use std::fs::File;
fn da(a: f64, b: f64) -> f64 { let r = (a-b).rem_euclid(360.0); if r > 180.0 { 360.0-r } else { r } }
fn cuatro(v: &fitparser::Value) -> Option<[f64;4]> {
    let fitparser::Value::Array(a) = v else { return None };
    let n: Vec<f64> = a.iter().filter_map(|x| match x {
        fitparser::Value::UInt8(b)=>Some(f64::from(*b)), fitparser::Value::UInt16(b)=>Some(f64::from(*b)),
        fitparser::Value::Float32(b)=>Some(f64::from(*b)), fitparser::Value::Float64(b)=>Some(*b), _=>None }).collect();
    if n.len()==4 { Some([n[0],n[1],n[2],n[3]]) } else { None }
}
fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "/home/billibu/projects/pulso/datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta).unwrap().filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x=="fit").unwrap_or(false)).collect();
    rutas.sort();
    let mut v: Vec<f64> = Vec::new();
    for r in &rutas {
        let Ok(mut f) = File::open(r) else { continue };
        let Ok(d) = fitparser::from_reader(&mut f) else { continue };
        let (mut i, mut j) = (None, None);
        for rec in d.iter() {
            if format!("{:?}", rec.kind()) != "Session" { continue; }
            for c in rec.fields() { match c.name() {
                "avg_left_power_phase" => i = cuatro(c.value()),
                "avg_right_power_phase" => j = cuatro(c.value()), _=>{} } }
        }
        if let (Some(a),Some(b)) = (i,j) {
            v.push(a.iter().zip(b.iter()).map(|(x,y)| da(*x,*y)).fold(0.0f64,f64::max));
        }
    }
    v.sort_by(|a,b| a.partial_cmp(b).unwrap());
    let base = v[v.len()/2];
    let corte = base*2.0;
    println!("n={} base={base} corte={corte}", v.len());
    println!("  == corte (exactamente): {}", v.iter().filter(|x| (**x-corte).abs()<1e-9).count());
    println!("  >  corte (estricto)  : {}", v.iter().filter(|x| **x>corte).count());
    println!("  >= corte (alcanzar)  : {}", v.iter().filter(|x| **x>=corte).count());
    println!("\n  LOS VALORES DISTINTOS QUE APARECEN (es una escala, no un continuo):");
    let mut u: Vec<f64> = v.clone(); u.dedup();
    for x in &u { println!("    {x:>8.4}  x{}", v.iter().filter(|y| (**y-*x).abs()<1e-9).count()); }
}
