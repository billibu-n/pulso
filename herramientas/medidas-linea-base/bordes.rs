// LOS BORDES, MEDIDOS: ¿cuantas sesiones de verdad caen EXACTAMENTE en 100 TSS?
use std::fs::File;
fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "/home/billibu/projects/pulso/datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta).unwrap().filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x=="fit").unwrap_or(false)).collect();
    rutas.sort();
    let mut tss: Vec<f64> = Vec::new();
    for r in &rutas {
        let Ok(mut f) = File::open(r) else { continue };
        let Ok(d) = fitparser::from_reader(&mut f) else { continue };
        for rec in d.iter() {
            if format!("{:?}", rec.kind()) != "Session" { continue; }
            for c in rec.fields() {
                if c.name() == "training_stress_score" {
                    let v = match c.value() {
                        fitparser::Value::UInt8(x)=>Some(f64::from(*x)), fitparser::Value::UInt16(x)=>Some(f64::from(*x)),
                        fitparser::Value::UInt32(x)=>Some(f64::from(*x)), fitparser::Value::Float32(x)=>Some(f64::from(*x)),
                        fitparser::Value::Float64(x)=>Some(*x), _=>None };
                    if let Some(v)=v { tss.push(v); }
                }
            }
        }
    }
    println!("sesiones con TSS: {}", tss.len());
    println!("  == 100 exacto: {}", tss.iter().filter(|t| (**t-100.0).abs()<1e-9).count());
    println!("  >  100      : {}", tss.iter().filter(|t| **t>100.0).count());
    println!("  >= 100      : {}", tss.iter().filter(|t| **t>=100.0).count());
    let mut v = tss.clone(); v.sort_by(|a,b| a.partial_cmp(b).unwrap());
    println!("  los 6 mas cercanos a 100: {:?}", v.iter().filter(|t| (**t-100.0).abs()<12.0).collect::<Vec<_>>());
    println!("  mediana {:.0} · maximo {:.0}", v[v.len()/2], v[v.len()-1]);
}
