// LA CUENTA FINAL DE LA FASE, con la misma regla que el modulo: alinear el corte al peldaño y
// comparar con una tolerancia mayor que el ruido del lector (8,8e-8) y mucho menor que el peldaño.
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
    let paso = 360.0/256.0;
    let corte = ((base*2.0)/paso).round()*paso;
    let tol = 1e-6 * corte.abs().max(1.0);
    println!("n={} base={base:.10} corte(alineado)={corte:.10} tolerancia={tol:.3e}", v.len());
    let superan = v.iter().filter(|x| if (**x-corte).abs()<=tol { false } else { **x>corte }).count();
    let en_el_corte = v.iter().filter(|x| (**x-corte).abs()<=tol).count();
    let ruido: Vec<f64> = v.iter().map(|x| (x-corte).abs()).filter(|d| *d>0.0 && *d<1e-5).collect();
    println!("  EN el corte (no avisan): {en_el_corte}");
    println!("  SUPERAN el corte (avisan): {superan}");
    println!("  ruido del lector medido: min {:?} max {:?}", ruido.iter().cloned().fold(f64::INFINITY,f64::min), ruido.iter().cloned().fold(0.0f64,f64::max));
    println!("  (peldaño = {paso} -> la tolerancia es {:.0} veces menor)", paso/tol);
}
