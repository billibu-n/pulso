// ¿QUE ES «EL PULSO EN REPOSO» DE ESTE CLIENTE, Y DE QUE SIRVE?
//
// LA PREGUNTA DEL DUEÑO: «no importa lo que diga directamente, importa los datos de la medicion...
// compruebalo tu y ve de que sirve este dato, y separalo si es necesario si genera mucho conflicto».
//
// Y HAY CONFLICTO, medido: el encargo dice 71 ppm de promedio historico, y en el proyecto habia
// apuntado 48. Los dos no pueden ser el mismo dato. Este programa lo separa en TRES cosas distintas,
// que es lo unico honesto:
//
//   1. PULSO MINIMO EN ENTRENAMIENTO -- se puede medir de los FIT, y es real.
//   2. PULSO EN REPOSO DE VERDAD    -- lo mide el reloj durmiendo, y sus 165 ficheros vienen VACIOS.
//   3. PULSO DESPIERTO DE UNA MEDICION SUELTA -- lo que da una bascula o una toma puntual. El 71.
use std::fs::File;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "../datos/garmin_files/storage".into());
    let mut rutas: Vec<_> = std::fs::read_dir(&carpeta).unwrap()
        .filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false)).collect();
    rutas.sort();

    // El minimo de cada sesion, y el minimo de los PRIMEROS MINUTOS (que es lo mas parecido al reposo
    // que se puede sacar de un entrenamiento: aun no ha empezado el esfuerzo).
    let mut minimos: Vec<(String, u16, Option<u16>)> = Vec::new();

    for ruta in &rutas {
        let Ok(mut f) = File::open(ruta) else { continue };
        let Ok(datos) = fitparser::from_reader(&mut f) else { continue };
        let nombre = ruta.file_name().unwrap().to_string_lossy().to_string();
        let fecha = nombre.split('_').nth(3).unwrap_or("?").to_string();
        let mut pulsos: Vec<(u32, u16)> = Vec::new();
        for r in datos.iter() {
            if format!("{:?}", r.kind()) != "Record" { continue; }
            let mut seg: Option<u32> = None;
            let mut p: Option<u16> = None;
            for c in r.fields() {
                match c.name() {
                    "timestamp" => if let fitparser::Value::Timestamp(t) = c.value() { seg = Some(t.timestamp() as u32); },
                    "heart_rate" => if let fitparser::Value::UInt8(v) = c.value() { p = Some(u16::from(*v)); },
                    _ => {}
                }
            }
            if let (Some(s), Some(p)) = (seg, p) { if p > 0 { pulsos.push((s, p)); } }
        }
        if pulsos.is_empty() { continue; }
        let t0 = pulsos[0].0;
        let min_total = pulsos.iter().map(|(_, p)| *p).min().unwrap();
        // Los primeros 5 minutos: antes de que el esfuerzo se note.
        let primeros: Vec<u16> = pulsos.iter().filter(|(s, _)| s - t0 <= 300).map(|(_, p)| *p).collect();
        let min_primeros = primeros.iter().copied().min();
        minimos.push((fecha, min_total, min_primeros));
    }

    println!("=== SESIONES CON PULSO: {} ===", minimos.len());
    println!("\n=== 1. PULSO MINIMO DE CADA SESION (lo mas bajo visto entrenando) ===");
    let mut todos: Vec<u16> = minimos.iter().map(|(_, m, _)| *m).collect();
    todos.sort();
    println!("  minimo {} · mediana {} · maximo {}", todos[0], todos[todos.len()/2], todos[todos.len()-1]);
    let por_debajo_60 = todos.iter().filter(|p| **p < 60).count();
    println!("  sesiones donde el pulso BAJO de 60: {por_debajo_60} de {}", todos.len());

    println!("\n=== 2. PULSO MINIMO EN LOS PRIMEROS 5 MINUTOS (lo mas parecido al reposo) ===");
    let mut primeros: Vec<u16> = minimos.iter().filter_map(|(_, _, p)| *p).collect();
    primeros.sort();
    if !primeros.is_empty() {
        println!("  minimo {} · mediana {} · maximo {}", primeros[0], primeros[primeros.len()/2], primeros[primeros.len()-1]);
    }

    println!("\n=== 3. LOS 12 MAS BAJOS, con su fecha (para ver si son las sesiones suaves) ===");
    let mut orden: Vec<_> = minimos.clone();
    orden.sort_by_key(|(_, _, p)| *p);
    for (fecha, total, primeros) in orden.iter().take(12) {
        println!("  {fecha}  minimo total {total:>3}  · primeros 5 min {primeros:?}");
    }

    println!("\n=== 4. EVOLUCION: ¿el minimo SUBE con los meses? (eso seria fatiga acumulada) ===");
    let mut por_mes: std::collections::BTreeMap<String, Vec<u16>> = Default::default();
    for (fecha, _, p) in &minimos {
        if let Some(p) = p { por_mes.entry(fecha[..7].to_string()).or_default().push(*p); }
    }
    for (mes, v) in &por_mes {
        let media = v.iter().map(|x| f64::from(*x)).sum::<f64>() / v.len() as f64;
        let min = v.iter().min().unwrap();
        println!("  {mes}  {:.1} ppm de media · el mas bajo {min} · {} sesiones", media, v.len());
    }
}
