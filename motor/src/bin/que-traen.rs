// ¿DONDE APARECE `resting_heart_rate` EN LOS FICHEROS DE SESION, Y QUE VALE?
//
// Un agente encontro el campo en 102 de 102 ficheros con el valor 52, y 1.679 apariciones. Eso choca
// con el 48 que este proyecto tenia medido como pulso en reposo. Antes de decir cual es el bueno, hay
// que ver DONDE esta cada uno y QUE SIGNIFICA cada uno.
use std::fs::File;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "../datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta).unwrap()
        .filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false)).collect();
    rutas.sort();

    let mut por_mensaje: std::collections::BTreeMap<String, (usize, std::collections::BTreeSet<String>)> = Default::default();
    let mut en_sesion = 0usize;
    let mut sesiones = 0usize;

    for ruta in &rutas {
        let Ok(mut f) = File::open(ruta) else { continue };
        let Ok(datos) = fitparser::from_reader(&mut f) else { continue };
        sesiones += 1;
        let mut esta_en_sesion = false;
        for r in datos.iter() {
            let tipo = format!("{:?}", r.kind());
            for c in r.fields() {
                if c.name() == "resting_heart_rate" {
                    let e = por_mensaje.entry(tipo.clone()).or_default();
                    e.0 += 1;
                    e.1.insert(format!("{:?}", c.value()));
                    if tipo == "Session" { esta_en_sesion = true; }
                }
            }
        }
        if esta_en_sesion { en_sesion += 1; }
    }

    println!("=== FICHEROS DE SESION: {sesiones} ===");
    println!("\n=== `resting_heart_rate`: EN QUE MENSAJE Y CON QUE VALOR ===");
    for (msg, (n, valores)) in &por_mensaje {
        let v: Vec<_> = valores.iter().take(4).cloned().collect();
        println!("  {n:>6} apariciones en {msg:<28} valores: {v:?}");
    }
    println!("\n  ¿aparece en el mensaje `Session`? ... {} de {sesiones} ficheros", en_sesion);
    println!("\n  Y EL PULSO MINIMO REAL de cada sesion (que es otra cosa):");
}
