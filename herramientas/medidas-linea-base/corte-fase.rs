// EL CORTE DE LA FASE: los mismos datos, contados por los dos caminos, y el valor exacto del corte.
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
    let corte = ((base*2.0)/(360.0/256.0)).round()*(360.0/256.0);
    println!("MEDIDO A MANO: n={} base={base:.10} corte={corte:.10}", v.len());
    for (etq, cond) in [("> corte", 0), (">= corte", 1)] {
        let n = v.iter().filter(|x| if cond==0 { **x > corte } else { **x >= corte }).count();
        println!("  {etq}: {n}");
    }
    // La MAD, que es lo que el modulo usa para la DERIVA pero NO para la fase. Comprobacion.
    let desv: Vec<f64> = v.iter().map(|x| (x-base).abs()).collect();
    let mut dv = desv.clone(); dv.sort_by(|a,b| a.partial_cmp(b).unwrap());
    println!("  (por curiosidad: MAD de la fase = {:.10}, y base + 2*MAD = {:.10})", dv[dv.len()/2], base + 2.0*dv[dv.len()/2]);

    // EL MODULO, para comparar.
    let h = motor::avisos::de_carpeta(std::path::Path::new(&carpeta));
    println!("\nEL MODULO: base={:?} corte={:?}", h.base_de_fase(), h.corte_de_fase());
    let mut mv: Vec<f64> = h.sesiones.iter().filter_map(|s| s.asimetria_grados).collect();
    mv.sort_by(|a,b| a.partial_cmp(b).unwrap());
    println!("  n={} base medida={:.10}", mv.len(), mv[mv.len()/2]);
    let c = h.corte_de_fase().unwrap();
    println!("  > corte : {}", mv.iter().filter(|x| **x > c).count());
    println!("  >= corte: {}", mv.iter().filter(|x| **x >= c).count());
    println!("  los valores == corte: {:?}", mv.iter().filter(|x| (**x-c).abs()<1e-9).collect::<Vec<_>>());
}
