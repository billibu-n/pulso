// LAS ZONAS DE POTENCIA DE ESTE CLIENTE, MEDIDAS SOBRE SUS FICHEROS DE VERDAD.
//
// POR QUE EXISTE: el motor calculaba las zonas SOLO con el pulso, y este cliente entrena con medidor
// de potencia. Este instrumento contesta, con numeros y no con opiniones, a las cinco preguntas que
// deciden si el modulo `potencia` sirve:
//
//   1. ¿Que zonas salen de sus 213 W de FTP, y de donde sale ese numero?
//   2. ¿Cuanto TSS tiene por sesion, y cuantas veces salta SU regla de los 100?
//   3. ¿Como va su carga semana a semana, y hacia donde va?
//   4. ¿Cuantas sesiones NO traen potencia? (la pregunta que impide inventar)
//   5. ¿Cuadra el reparto de zonas que declara el fichero con la duracion de la sesion?
//
// SE CONSERVA COMO INSTRUMENTO, igual que `medir-carga` o `potencia-real`: **estos numeros hay que
// poder volver a medirlos**. Si algun dia se toca `potencia.rs` y las zonas se mueven, aqui se ve.
//
// Y SE LLAMA `potencia` PORQUE ES EL INSTRUMENTO DEL MODULO `potencia`, no porque mida lo mismo que
// `potencia-real`: aquel **mira que campos hay** en los ficheros (y por eso sigue siendo el primero
// que hay que correr); este **aplica el modulo** y comprueba sus numeros.
//
// Uso:  cargo run -q --release --bin potencia -- ../datos/garmin_files/storage
use motor::fit;
use motor::potencia::{self, EnZona, SesionConCarga};
use std::path::Path;

fn main() {
    let carpeta = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../datos/garmin_files/storage".to_string());
    let carpeta = Path::new(&carpeta);

    // ── 1 · SE LEEN LOS FICHEROS ────────────────────────────────────────────────────────────────
    let mut rutas: Vec<std::path::PathBuf> = match std::fs::read_dir(carpeta) {
        Ok(d) => d
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false))
            .collect(),
        Err(e) => {
            eprintln!("no se pudo leer la carpeta {}: {e}", carpeta.display());
            std::process::exit(1);
        }
    };
    rutas.sort();
    if rutas.is_empty() {
        println!("no hay ni un fichero .fit en {}", carpeta.display());
        return;
    }

    // ── 2 · LO QUE DECLARA CADA FICHERO, y lo que traen sus muestras ────────────────────────────
    //
    // DOS LECTURAS POR FICHERO, y se dice por que: la primera saca lo que el fichero DECLARA sobre
    // potencia (umbral, TSS, zonas), y la segunda usa el lector del motor para saber si hay VATIOS en
    // las muestras. Son dos preguntas distintas —«que dice el aparato» y «que midio el sensor»— y
    // mezclarlas en una sola pasada obligaria a duplicar el lector de muestras.
    let mut declarados: Vec<(String, String, potencia::DeclaradoDePotencia, bool)> = Vec::new();
    let mut declarado_global: Option<fit::Declarado> = None;
    let mut no_leidos = 0usize;

    for ruta in &rutas {
        let Ok(d) = potencia::leer_declarado_de_potencia(ruta) else {
            no_leidos += 1;
            continue;
        };
        let nombre = ruta.file_name().unwrap_or_default().to_string_lossy().to_string();
        let fecha = fecha_del_nombre(&nombre).unwrap_or_else(|| "(sin fecha)".to_string());

        let hay_vatios = match fit::leer(ruta) {
            Ok(l) => {
                // El umbral y el TSS del fichero, para la comprobacion del TSS. Se toma el de la
                // PRIMERA sesion que lo traiga: es el mismo en las 83, y esta medido.
                if declarado_global.is_none() && l.declarado.potencia_umbral.lo_hay() {
                    let mut g = l.declarado.clone();
                    // Y LA DURACION SE CORRIGE A LA QUE HACE FALTA PARA LA CUENTA, diciendolo: la
                    // comprobacion del TSS usa el **tiempo en movimiento**, y `fit::Declarado`
                    // guarda el transcurrido. Sin esta linea la comprobacion acusaria al fichero de
                    // un error que es nuestro (medido: cuadran 31 en vez de 55).
                    if let Some(t) = d.duracion_en_movimiento_s.valor() {
                        g.duracion_s = motor::Dato::Hay(t as u32);
                    }
                    declarado_global = Some(g);
                }
                l.sesion
                    .muestras
                    .iter()
                    .any(|m| m.potencia_w.valor().unwrap_or(0) > 0)
            }
            Err(_) => {
                no_leidos += 1;
                false
            }
        };
        declarados.push((nombre, fecha, d, hay_vatios));
    }

    println!("=== {} FICHEROS FIT EN {} ===", rutas.len(), carpeta.display());
    if no_leidos > 0 {
        println!("  y {no_leidos} no se pudieron leer: ese dato falta y se dice");
    }

    // ── 3 · LAS ZONAS DE POTENCIA ───────────────────────────────────────────────────────────────
    let Some(declarado_global) = declarado_global else {
        println!("\nNO HAY NI UN FICHERO CON UMBRAL DE POTENCIA.");
        println!("  Y por eso NO se dan zonas: no se inventan. Lo que falta, y para que:");
        if let Some(q) = potencia::Referencia::SinUmbral.que_falta() {
            println!("    · {q}");
        }
        return;
    };

    println!("\n=== LA REFERENCIA, Y DE DONDE SALE ===");
    match potencia::umbral_de_potencia(&declarado_global) {
        motor::Dato::Hay(ftp) => {
            println!("  {}", ftp.frase());
            println!(
                "  margen: ±{:.1} W ({}). {}",
                ftp.rango.margen,
                ftp.referencia.nombre(),
                potencia::ZonaPotencia::fundamento()
            );
            println!(
                "  el margen NO sale de este aparato: es el error tipico de medicion de 7 W de\n  \
                 Borszcz y otros, Int J Sports Med 2020;41(03):175-181, DOI 10.1055/a-1018-1965\n  \
                 (25 ciclistas entrenados, dos pruebas separadas por 7 dias, CV 2,9 %).\n  \
                 Se usa como SUELO del margen, y se declara que es de otro protocolo."
            );

            println!("\n=== LAS ZONAS DE POTENCIA, CON SU TRAMO (no solo el suelo) ===");
            println!(
                "  {:>2}  {:<24} {:>14}  {:<10}  {}",
                "z", "zona", "vatios", "% del FTP", "que es"
            );
            for (z, desde, hasta) in potencia::zonas_de_potencia(&ftp) {
                let vatios = match &hasta {
                    // La septima zona no tiene techo, y se DICE en vez de inventar un numero. El
                    // signo «+» es la unica forma honesta de escribirla en una linea.
                    None => format!("{:.0} +", desde.valor),
                    Some(h) => format!("{:.0} - {:.0}", desde.valor, h.valor),
                };
                let pct = match z.techo() {
                    None => format!("{:.0} +", z.fraccion() * 100.0),
                    Some(t) => format!("{:.0} - {:.0}", z.fraccion() * 100.0, t * 100.0),
                };
                let margen = match &hasta {
                    None => format!("±{:.1}", desde.margen),
                    Some(h) => format!("±{:.1}", h.margen.max(desde.margen)),
                };
                println!(
                    "  {:>2}  {:<24} {:>14}  {:<10}  {}",
                    numero_de_zona(z),
                    z.nombre(),
                    format!("{vatios} {margen}"),
                    pct,
                    z.explicacion()
                );
            }
            println!(
                "\n  (el margen de cada corte es el del FTP propagado, no un error distinto por zona)"
            );
            println!(
                "  FUENTE de los cortes: {}\n  \
                 el nivel 7 NO tiene techo porque la fuente le pone «n/a» en potencia: son esfuerzos\n  \
                 de menos de 25 s que estresan mas el sistema musculoesqueletico que el metabolico.",
                potencia::ZonaPotencia::fundamento()
            );
        }
        motor::Dato::Falta => {
            println!("  no hay umbral de potencia: no se pueden dar zonas, y no se inventan");
        }
    }

    // ── 4 · EL TSS POR SESION ───────────────────────────────────────────────────────────────────
    let con_potencia = declarados.iter().filter(|(_, _, _, p)| *p).count();
    let mut tss_por_sesion: Vec<(motor::Dato<f64>, bool)> = Vec::new();
    let mut sesiones_para_semanas: Vec<SesionConCarga> = Vec::new();
    for (_, fecha, d, hay_vatios) in &declarados {
        // EL TSS SE LEE, NO SE RECALCULA: esta linea es la regla del proyecto entera, y por eso el
        // dato sale tal cual del fichero y no pasa por ninguna formula.
        tss_por_sesion.push((d.tss, *hay_vatios));
        if fecha != "(sin fecha)" {
            sesiones_para_semanas.push(match d.tss.valor() {
                Some(t) => SesionConCarga::nueva(fecha, t),
                None => SesionConCarga::sin_datos(),
            });
        }
    }

    let resumen = potencia::ResumenDeCarga::calcular(
        &tss_por_sesion,
        potencia::UMBRAL_DE_CARGA_DEL_CLIENTE,
    );
    println!("\n=== LA CARGA: EL TSS QUE TRAE EL FICHERO ===");
    println!("  {}", resumen.frase());
    println!(
        "  EL TSS NO SE RECALCULA: se lee de `Session.training_stress_score` y se conserva.\n  \
         Si algun dia se recalcula, tiene que ser para COMPROBAR que coincide, no para sustituirlo."
    );

    println!("\n=== LA REGLA DEL CLIENTE, APLICADA ===");
    println!("  {}", potencia::fundamento_del_aviso_de_carga());
    println!(
        "  corte: superar {:.0} TSS (estricto: {:.0} exacto NO es carga alta)  ->  {} sesiones",
        potencia::UMBRAL_DE_CARGA_DEL_CLIENTE,
        potencia::UMBRAL_DE_CARGA_DEL_CLIENTE,
        resumen.por_encima_del_umbral
    );

    // ── 5 · LA COMPROBACION DEL TSS, que es lo unico que se recalcula ───────────────────────────
    println!("\n=== COMPROBACION: ¿el TSS del fichero es NP·IF·100/FTP? ===");
    if let Some(c) = potencia::comprobar_tss(&declarado_global, potencia::TOLERANCIA_TSS) {
        println!("  (la sesion que se comprueba es la primera con potencia)");
        println!("  {}", c.frase());
    } else {
        println!("  no se puede comprobar: falta alguna de las piezas (NP, IF, FTP o duracion)");
    }
    // Y sobre TODAS, que es donde el numero dice algo.
    let mut cuadran = 0;
    let mut comparadas = 0;
    let mut peor = 0.0f64;
    for ruta in &rutas {
        let Ok(l) = fit::leer(ruta) else { continue };
        // MISMA CORRECCION QUE ARRIBA, y por el mismo motivo medido: `fit::Declarado.duracion_s`
        // guarda el tiempo TRANSCURRIDO, y la cuenta del TSS necesita el tiempo EN MOVIMIENTO.
        let Ok(local) = potencia::leer_declarado_de_potencia(ruta) else { continue };
        let mut g = l.declarado.clone();
        if let Some(t) = local.duracion_en_movimiento_s.valor() {
            g.duracion_s = motor::Dato::Hay(t as u32);
        }
        match potencia::comprobar_tss(&g, potencia::TOLERANCIA_TSS) {
            Some(c) => {
                comparadas += 1;
                if c.cuadra { cuadran += 1; }
                if c.diferencia > peor {
                    peor = c.diferencia;
                }
            }
            None => {}
        }
    }
    if comparadas > 0 {
        println!(
            "  sobre las {comparadas} sesiones que traen las cuatro piezas: cuadran {cuadran} \
             ({:.0} %), y la peor diferencia son {peor:.1} puntos de TSS",
            cuadran as f64 / comparadas as f64 * 100.0
        );
    }

    // ── 6 · LA CARGA SEMANAL Y SU TENDENCIA ─────────────────────────────────────────────────────
    let semanas = potencia::carga_semanal(&sesiones_para_semanas);
    println!("\n=== LA CARGA SEMANAL, Y HACIA DONDE VA ===");
    println!("  {:>10}  {:>8}  {:>9}  {}", "semana", "TSS", "sesiones", "sin TSS");
    for s in &semanas {
        println!(
            "  {:>10}  {:>8.0}  {:>9}  {}",
            s.lunes,
            s.tss_total,
            s.sesiones,
            if s.sin_tss > 0 {
                format!("{} (no se suman como cero)", s.sin_tss)
            } else {
                "-".to_string()
            }
        );
    }
    match potencia::tendencia_de_carga(&semanas) {
        motor::Dato::Hay(t) => {
            println!("\n  {}", t.frase(potencia::TOLERANCIA_CARGA));
            println!(
                "  (la tolerancia es el {:.0} %, la MISMA que usa el volumen en `tendencia.rs`: \n   \
                 dos tolerancias distintas para lo mismo serian una precision falsa)",
                potencia::TOLERANCIA_CARGA * 100.0
            );
        }
        motor::Dato::Falta => {
            println!("\n  con menos de dos semanas NO hay tendencia, y decirlo es la respuesta correcta");
        }
    }
    let (ultimas4, n4) = potencia::carga_de_las_ultimas(&semanas, 4);
    println!("\n  carga de las ultimas {n4} semanas: {ultimas4:.0} TSS");

    // ── 7 · LAS SESIONES SIN POTENCIA: el dato que impide inventar ──────────────────────────────
    println!("\n=== LAS SESIONES QUE NO TRAEN POTENCIA ===");
    println!(
        "  con potencia en las muestras ... {con_potencia} de {}",
        declarados.len()
    );
    println!(
        "  SIN potencia en las muestras ... {} de {}",
        declarados.len() - con_potencia,
        declarados.len()
    );
    let sin_pot: Vec<&(String, String, potencia::DeclaradoDePotencia, bool)> = declarados
        .iter()
        .filter(|(_, _, _, p)| !*p)
        .collect();
    println!("  y son estas, con lo que SI traen:");
    for (_nombre, fecha, d, _) in &sin_pot {
        let zonas = potencia::tiempo_en_zonas(&d.tiempo_en_potencia_s)
            .map(|z| format!("{} zonas declaradas", z.len()))
            .unwrap_or_else(|| "sin reparto de zonas".to_string());
        println!(
            "    {fecha}  TSS {:>6}  umbral {:>6}  {}",
            d.tss
                .valor()
                .map(|t| format!("{t:.0}"))
                .unwrap_or_else(|| "(falta)".to_string()),
            d.umbral_w
                .valor()
                .map(|u| format!("{u} W"))
                .unwrap_or_else(|| "(falta)".to_string()),
            zonas
        );
    }
    if sin_pot.is_empty() {
        println!("    (ninguna: todas traen potencia)");
    }
    if let Some(q) = potencia::que_falta_para_potencia(true, false) {
        println!("\n  que falta en esas sesiones, y para que: {q}");
    }

    // ── 8 · EL REPARTO DE ZONAS QUE DECLARA EL FICHERO, Y SU COMPROBACION ───────────────────────
    println!("\n=== EL REPARTO DE ZONAS DEL FICHERO, Y SI CUADRA CON LA DURACION ===");
    println!(
        "  POR QUE SE COMPRUEBA: si el reparto de zonas no suma la sesion, uno de los dos numeros\n  \
         esta mal. Es la comprobacion que `fit.rs` describe: sale gratis y caza un lector roto."
    );
    let mut cuadran_zonas = 0;
    let mut con_reparto = 0;
    let mut ejemplos = 0;
    let mut suma_zonas_sin_potencia = 0.0f64;
    let mut con_tiempo_sin_zona = 0usize;
    let mut total_sin_zona = 0.0f64;
    for (nombre, fecha, d, hay_vatios) in &declarados {
        let Some(z) = potencia::tiempo_en_zonas(&d.tiempo_en_potencia_s) else {
            continue;
        };
        let Some(duracion) = d.duracion_en_movimiento_s.valor() else {
            continue;
        };
        con_reparto += 1;
        let sin_zona = potencia::segundos_sin_zona(&d.tiempo_en_potencia_s);
        if sin_zona > 0.5 {
            con_tiempo_sin_zona += 1;
            total_sin_zona += sin_zona;
        }
        let c = potencia::comparar_zonas_con_duracion(&z, duracion, 0.01);
        if c.cuadra {
            cuadran_zonas += 1;
        }
        if !hay_vatios {
            suma_zonas_sin_potencia += c.suma_de_zonas_s;
        }
        if ejemplos < 3 && *hay_vatios {
            println!(
                "\n  {fecha} ({}): {}",
                &nombre[..nombre.len().min(26)],
                c.frase()
            );
            imprimir_reparto(&z, duracion);
            if sin_zona > 0.5 {
                println!(
                    "        {:<24} {:>10}  {}",
                    "(tramos 8 a 10 del aparato)",
                    potencia::segundos_legibles(sin_zona),
                    "la fuente que citamos define SIETE zonas: estos no se nombran ni se reparten"
                );
            }
            let pulso: Vec<(String, f64)> = d
                .tiempo_en_pulso_s
                .prestado()
                .map(|v| {
                    v.iter()
                        .enumerate()
                        .map(|(i, s)| (format!("zona {}", i + 1), *s))
                        .collect()
                })
                .unwrap_or_default();
            if let Some(frase) = potencia::comparar_potencia_y_pulso(&z, &pulso) {
                println!("      POTENCIA frente a PULSO: {frase}");
            }
            ejemplos += 1;
        }
    }
    println!(
        "\n  de {con_reparto} sesiones con reparto de zonas, cuadran con la duracion (dentro del 1 %): \
         {cuadran_zonas}"
    );
    println!(
        "  y {con_tiempo_sin_zona} de ellas traen tiempo en los tramos 8 a 10 del array, que la tabla\n  \
         de Coggan NO define: {} en total. NO se reparten a ojo entre las siete zonas —eso cuadraria\n  \
         el total inventando— y por eso esta comprobacion no puede dar el 100 %.",
        potencia::segundos_legibles(total_sin_zona)
    );
    if suma_zonas_sin_potencia > 0.0 {
        println!(
            "  OJO: las sesiones sin potencia suman {suma_zonas_sin_potencia:.0} s de zonas de potencia, \
             y eso NO puede ser: no hay vatios que repartir"
        );
    } else {
        println!(
            "  y las sesiones SIN potencia suman 0 s de zonas de potencia: el aparato deja el reparto\n  \
             vacio, que es lo correcto — por eso el motor devuelve «falta» y no una lista de ceros."
        );
    }
}

/// El numero de zona, para poder leer la tabla de un vistazo y compararla con la fuente.
fn numero_de_zona(z: potencia::ZonaPotencia) -> usize {
    potencia::ZonaPotencia::todas()
        .iter()
        .position(|x| *x == z)
        .map(|i| i + 1)
        .unwrap_or(0)
}

/// El reparto del tiempo, zona a zona, con su porcentaje.
fn imprimir_reparto(zonas: &[EnZona], total_s: f64) {
    for z in zonas {
        if z.segundos <= 0.0 {
            continue;
        }
        println!(
            "        {:<24} {:>10}  {:>5.1} %",
            z.zona.nombre(),
            z.tiempo_legible(),
            z.fraccion_de(total_s) * 100.0
        );
    }
}

/// La fecha de una sesion, **sacada del nombre del fichero**.
///
/// POR QUE DEL NOMBRE Y NO DEL CONTENIDO: el nombre lo pone Garmin con el formato
/// `..._ACTIVITY_<id>_<AAAA-MM-DD>T...Z.fit`, y esta medido en los 102. La fecha del contenido
/// (`Session.start_time`) seria mejor, pero **`fit.rs` todavia no la expone**, y este binario no
/// puede tocar ese fichero. Se declara aqui para que se vea de donde sale, y si un dia el nombre no
/// la trae, devuelve `None` y la sesion **no entra en ninguna semana** en vez de caer en la equivocada.
fn fecha_del_nombre(nombre: &str) -> Option<String> {
    let pos = nombre.find("_ACTIVITY_")? + "_ACTIVITY_".len();
    let resto = &nombre[pos..];
    let guion = resto.find('_')?;
    let fecha = &resto[guion + 1..];
    // Se corta en la `T`: lo que queda delante es `AAAA-MM-DD`.
    let dia = fecha.split('T').next()?;
    potencia::lunes_de_la_semana(dia)?;
    Some(dia.to_string())
}
