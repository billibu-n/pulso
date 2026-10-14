// LA DERIVA, CON DOS LECTORES DISTINTOS, SESION A SESION. Uno de los dos esta mal.
use std::fs::File;
fn deriva_patron(pares: &[(f64,f64)]) -> Option<f64> {
    if pares.len() < 20 { return None; }
    let m = pares.len()/2;
    let med = |v: &[(f64,f64)]| { (v.iter().map(|x|x.0).sum::<f64>()/v.len() as f64, v.iter().map(|x|x.1).sum::<f64>()/v.len() as f64) };
    let (p1,w1)=med(&pares[..m]); let (p2,w2)=med(&pares[m..]);
    if p1<=0.0||p2<=0.0 { return None; }
    Some((((w1/p1)-(w2/p2))/(w1/p1))*100.0)
}
fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "/home/billibu/projects/pulso/datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta).unwrap().filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x=="fit").unwrap_or(false)).collect();
    rutas.sort();
    let mut todas: Vec<(String,f64,usize)> = Vec::new();
    for r in &rutas {
        let Ok(mut f) = File::open(r) else { continue };
        let Ok(d) = fitparser::from_reader(&mut f) else { continue };
        let nombre = r.file_name().unwrap().to_string_lossy().get(10..30).unwrap_or("?").to_string();
        let mut pares: Vec<(f64,f64)> = Vec::new();
        let mut tipos_pot: std::collections::BTreeMap<String,usize> = Default::default();
        for rec in d.iter() {
            if format!("{:?}", rec.kind()) != "Record" { continue; }
            let (mut p, mut w) = (None, None);
            for c in rec.fields() {
                match (c.name(), c.value()) {
                    ("heart_rate", fitparser::Value::UInt8(x)) => p = Some(f64::from(*x)),
                    ("power", v) => {
                        *tipos_pot.entry(format!("{:?}", std::mem::discriminant(v)).chars().take(20).collect::<String>()).or_insert(0)+=1;
                        w = match v {
                            fitparser::Value::UInt8(x)=>Some(f64::from(*x)),
                            fitparser::Value::UInt16(x)=>Some(f64::from(*x)),
                            fitparser::Value::UInt32(x)=>Some(f64::from(*x)),
                            fitparser::Value::Float32(x)=>Some(f64::from(*x)),
                            fitparser::Value::Float64(x)=>Some(*x), _=>None };
                    }
                    _=>{}
                }
            }
            if let (Some(a),Some(b))=(p,w) { if a>0.0&&b>0.0 { pares.push((a,b)); } }
        }
        if let Some(x)=deriva_patron(&pares) { todas.push((nombre,x,pares.len())); }
    }
    let mut v: Vec<f64> = todas.iter().map(|(_,x,_)| *x).collect();
    v.sort_by(|a,b| a.partial_cmp(b).unwrap());
    println!("n={} mediana={:.4} media={:.4}", v.len(), v[v.len()/2], v.iter().sum::<f64>()/v.len() as f64);
    println!("suma de pares = {}", todas.iter().map(|(_,_,n)| n).sum::<usize>());
    println!("\nlas 8 con mas pares:");
    let mut o = todas.clone(); o.sort_by(|a,b| b.2.cmp(&a.2));
    for (n,x,p) in o.iter().take(8) { println!("  {n} deriva {x:>8.2} con {p} pares"); }
}
