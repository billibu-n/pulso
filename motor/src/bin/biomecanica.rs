// LA PRUEBA DE QUE EL LECTOR LEE: mide, sobre los ficheros de verdad, los campos que el motor
// ignoraba — y **en cuántas sesiones viene cada uno**.
//
// POR QUE ESTE PROGRAMA SE QUEDA EN EL REPOSITORIO: no es una demostracion, es un INSTRUMENTO. Los
// numeros de este resumen (cuantas sesiones traen la fase, cuanto se desvian las dos piernas, cuantas
// veces el reloj escribio un hueco) son la base de todo lo que se afirme despues sobre su pedalada.
// Si un dia el lector se rompe o el reloj cambia de firmware, **aqui se ve**, y sin esto habria que
// volver a averiguarlo a mano.
//
// Y HAY TRES TRAMPAS QUE ESTE PROGRAMA ESTA ESCRITO PARA NO VOLVER A PISAR. Las tres están medidas
// en `docs/ENCARGO-farellones-2026.md` §3, y las tres producen numeros plausibles y falsos:
//
//   1. `avg_power_position` **NO es izquierda/derecha**: es **[sentado, de pie]** (`rider_position_type`:
//      `Seated = 0`, `Standing = 1`). Leido al reves, su `[145, 410]` parece un 37/63 de desequilibrio.
//   2. `left_right_balance` **no es un porcentaje**: es el numero de serie del potenciometro. Por eso
//      este programa **no lo lee**.
//   3. `avg_left_pco` / `avg_right_pco` valen **0,00 siempre**: es el centinela de un sensor que no
//      mide el centro de presion, no una medida de cero milimetros.
//
// Uso:  cargo run -q --release --bin biomecanica -- ../datos/garmin_files/storage
use motor::datos::Dato;
use motor::fit::Lectura;
use std::path::Path;

/// Cuenta cuantas sesiones traen un dato, para poder decir **en cuantas falta** y no solo el promedio.
fn hay<T>(d: &Dato<T>) -> bool {
    d.lo_hay()
}

fn media(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// Los cuatro angulos de una fase, redondeados a una decima para poder leerlos.
fn redondear(v: &[f64; 4]) -> Vec<f64> {
    v.iter().map(|x| (x * 10.0).round() / 10.0).collect()
}

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "../datos/garmin_files/storage".into());
    let (lecturas, fallos) = motor::fit::leer_carpeta(Path::new(&carpeta));

    println!("=== BIOMECANICA Y CARGA, MEDIDO SOBRE {} FICHEROS ===", lecturas.len() + fallos.len());
    println!("  leidos ......... {}", lecturas.len());
    println!("  NO leidos ...... {}   <-- si no es 0, todo lo de abajo sale de menos ficheros", fallos.len());
    for (f, e) in &fallos {
        println!("      {f}: {e}");
    }

    // Un contador por campo: en cuantas sesiones viene. No hay lista escrita a mano — se rellena sola
    // al recorrer, para que el dia que se añada un campo no haya que acordarse de añadirlo aqui.
    let mut en_cuantas: std::collections::BTreeMap<&str, usize> = Default::default();
    let mut invalidos: std::collections::BTreeMap<String, usize> = Default::default();

    let mut fases_izq: Vec<[f64; 4]> = Vec::new();
    let mut fases_der: Vec<[f64; 4]> = Vec::new();
    let mut sesiones_con_las_dos = 0usize;
    let mut sesiones_que_difieren = 0usize;
    let mut asimetrias: Vec<f64> = Vec::new();

    let (mut p_sentado, mut p_depie, mut c_sentado, mut c_depie) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut resp_media, mut resp_maxima) = (Vec::new(), Vec::new());
    let (mut carga_anaerobica, mut pico_carga) = (Vec::new(), Vec::new());
    let mut temperaturas: Vec<f64> = Vec::new();
    let mut cadencias_muestra: Vec<f64> = Vec::new();

    // Los totales que el fichero DECLARA, para comparar. No sustituyen a nada: se cuentan.
    let (mut np, mut tss, mut fi, mut ftp) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for (_, l) in &lecturas {
        let Lectura { sesion, declarado, invalidos: inv, .. } = l;
        let b = &sesion.biomecanica;

        let marcar = |mapa: &mut std::collections::BTreeMap<&'static str, usize>, campo: &'static str| {
            *mapa.entry(campo).or_insert(0) += 1;
        };

        if hay(&b.potencia_media_sentado) { marcar(&mut en_cuantas, "potencia_media_sentado"); }
        if hay(&b.potencia_media_de_pie) { marcar(&mut en_cuantas, "potencia_media_de_pie"); }
        if hay(&b.potencia_maxima_sentado) { marcar(&mut en_cuantas, "potencia_maxima_sentado"); }
        if hay(&b.potencia_maxima_de_pie) { marcar(&mut en_cuantas, "potencia_maxima_de_pie"); }
        if hay(&b.cadencia_media_sentado) { marcar(&mut en_cuantas, "cadencia_media_sentado"); }
        if hay(&b.cadencia_media_de_pie) { marcar(&mut en_cuantas, "cadencia_media_de_pie"); }
        if hay(&b.fase_pierna_izquierda) { marcar(&mut en_cuantas, "fase_pierna_izquierda"); }
        if hay(&b.fase_pierna_derecha) { marcar(&mut en_cuantas, "fase_pierna_derecha"); }
        if hay(&b.respiracion_media) { marcar(&mut en_cuantas, "respiracion_media"); }
        if hay(&b.respiracion_maxima) { marcar(&mut en_cuantas, "respiracion_maxima"); }
        if hay(&b.carga_anaerobica) { marcar(&mut en_cuantas, "carga_anaerobica"); }
        if hay(&b.pico_de_carga) { marcar(&mut en_cuantas, "pico_de_carga"); }

        if let Dato::Hay(v) = b.potencia_media_sentado { p_sentado.push(f64::from(v)); }
        if let Dato::Hay(v) = b.potencia_media_de_pie { p_depie.push(f64::from(v)); }
        if let Dato::Hay(v) = b.cadencia_media_sentado { c_sentado.push(v); }
        if let Dato::Hay(v) = b.cadencia_media_de_pie { c_depie.push(v); }
        if let Dato::Hay(v) = b.respiracion_media { resp_media.push(v); }
        if let Dato::Hay(v) = b.respiracion_maxima { resp_maxima.push(v); }
        if let Dato::Hay(v) = b.carga_anaerobica { carga_anaerobica.push(v); }
        if let Dato::Hay(v) = b.pico_de_carga { pico_carga.push(v); }

        // LA FASE: solo entra en la comparacion la sesion que trae LAS DOS piernas. Comparar una
        // pierna contra un hueco daria una asimetria que es del fichero, no de sus piernas.
        if let Some((i, d)) = b.fases() {
            sesiones_con_las_dos += 1;
            fases_izq.push(i);
            fases_der.push(d);
            // Difieren si CUALQUIERA de los cuatro angulos se separa mas de 1 grado, y la separacion
            // va POR EL CAMINO CORTO: los angulos dan la vuelta a los 360, y restarlos tal cual
            // convierte dos piernas casi iguales (357 grados una, 0 la otra) en 357 grados de
            // asimetria. Se mide con la misma funcion que usa el modelo, para no tener dos verdades.
            let difiere = i
                .iter()
                .zip(d.iter())
                .any(|(a, b)| motor::datos::distancia_angular(*a, *b) > 1.0);
            if difiere {
                sesiones_que_difieren += 1;
            }
            if let Some(a) = b.asimetria_grados() {
                asimetrias.push(a);
            }
        }

        if let Some(t) = sesion.temperatura_media() {
            temperaturas.push(f64::from(t));
        }
        cadencias_muestra.extend(sesion.cadencias().iter().map(|c| f64::from(*c)));

        if let Dato::Hay(v) = declarado.potencia_normalizada { np.push(v); }
        if let Dato::Hay(v) = declarado.tss { tss.push(v); }
        if let Dato::Hay(v) = declarado.factor_intensidad { fi.push(v); }
        if let Dato::Hay(v) = declarado.potencia_umbral { ftp.push(f64::from(v)); }

        for (campo, n) in inv {
            *invalidos.entry(campo.clone()).or_insert(0) += n;
        }
    }

    println!("\n=== EN CUANTAS SESIONES VIENE CADA CAMPO ===");
    for (campo, n) in &en_cuantas {
        println!("  {n:>4} de {}  {campo}", lecturas.len());
    }
    let n_cadencias = cadencias_muestra.len();
    let n_temperaturas = temperaturas.len();
    println!("  {n_cadencias:>4} de {}  cadencias_en_las_muestras ({n_cadencias} valores)", lecturas.len());
    println!("  {n_temperaturas:>4} de {}  temperatura_media_de_la_sesion", lecturas.len());

    println!("\n=== LA FASE DEL PEDALEO, POR PIERNA (grados) ===");
    println!("  sesiones con LAS DOS fases: {sesiones_con_las_dos} de {}", lecturas.len());
    if sesiones_con_las_dos > 0 {
        // MEDIA CIRCULAR, y no la aritmetica, PORQUE LOS ANGULOS DAN LA VUELTA A LOS 360: sumar
        // 357 grados y 4 grados y dividir entre dos da 180, o sea «justo lo contrario» de donde
        // estan las dos. La media de direcciones se hace con senos y cosenos, que es lo que hace
        // esto. Y el numero de sesiones que dan la vuelta se DICE, porque una media circular sobre
        // angulos repartidos por toda la circunferencia tampoco significa nada.
        let circular = |v: &Vec<[f64; 4]>, k: usize| -> (f64, f64) {
            let (mut s, mut c) = (0.0_f64, 0.0_f64);
            for f in v.iter() {
                let r = f[k].to_radians();
                s += r.sin();
                c += r.cos();
            }
            let n = v.len() as f64;
            (s / n, c / n)
        };
        let angulo = |s: f64, c: f64| -> f64 {
            let a = s.atan2(c).to_degrees();
            if a < 0.0 { a + 360.0 } else { a }
        };
        // La resultante mide si los angulos estan juntos (1) o repartidos (0). Por debajo de 0,5 la
        // media no describe a nadie y hay que decirlo en vez de enseñarla como si describiera.
        let resultante = |s: f64, c: f64| (s * s + c * c).sqrt();
        for (nombre, v) in [("IZQUIERDA", &fases_izq), ("DERECHA  ", &fases_der)] {
            let mut linea = String::new();
            for (k, etiqueta) in ["inicio", "pico", "fin", "centro"].iter().enumerate() {
                let (s, c) = circular(v, k);
                let r = resultante(s, c);
                let texto = if r < 0.5 {
                    // Cuando los angulos estan repartidos por toda la vuelta, decir «la media es X»
                    // es inventar un punto de encuentro que no existe.
                    "repartidos".to_string()
                } else {
                    format!("{:.1}", angulo(s, c))
                };
                linea.push_str(&format!(" {etiqueta} {texto}"));
            }
            println!("  fase {nombre} media:{linea}");
        }
        // La vuelta del cero, contada: es la que explica por que la media de arriba no es la de una
        // resta, y es un dato del sensor (donde tiene el cero), no de sus piernas.
        let vueltas = fases_izq
            .iter()
            .zip(fases_der.iter())
            .filter(|(i, d)| (i[0] - d[0]).abs() > 300.0)
            .count();
        println!("  sesiones donde el cero cae en distinto lado (el arranque salta 360): {vueltas}");
        println!("  sesiones en que las dos piernas DIFIEREN (mas de 1 grado): {sesiones_que_difieren} de {sesiones_con_las_dos}");
        asimetrias.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "  asimetria entre las dos piernas, por el camino corto: mediana {:.1} grados · peor caso {:.1} grados",
            asimetrias[asimetrias.len() / 2],
            asimetrias[asimetrias.len() - 1]
        );
        if let (Some(i), Some(d)) = (fases_izq.first(), fases_der.first()) {
            println!("  ejemplo del primer fichero:");
            println!("      izquierda: {:?}", redondear(i));
            println!("      derecha:   {:?}", redondear(d));
        }
    } else {
        println!("  (ninguna: sin las dos fases no se puede decir nada de la asimetria)");
    }

    println!("\n=== POR POSTURA (sentado / de pie) — NO es izquierda/derecha ===");
    println!("  potencia media sentado: {:.1} W   ({} sesiones)", media(&p_sentado), p_sentado.len());
    println!("  potencia media de pie:  {:.1} W   ({} sesiones)", media(&p_depie), p_depie.len());
    println!("  cadencia media sentado: {:.1} rpm ({} sesiones)", media(&c_sentado), c_sentado.len());
    println!("  cadencia media de pie:  {:.1} rpm ({} sesiones)", media(&c_depie), c_depie.len());

    println!("\n=== RESPIRACION (respiraciones por minuto) ===");
    println!("  media de las medias ... {:.1}   ({} sesiones)", media(&resp_media), resp_media.len());
    println!("  media de las maximas .. {:.1}   ({} sesiones)", media(&resp_maxima), resp_maxima.len());

    println!("\n=== CARGA ===");
    println!("  carga anaerobica (media de las sesiones) ... {:.2}   ({} sesiones)", media(&carga_anaerobica), carga_anaerobica.len());
    println!("  pico de carga (media de las sesiones) ..... {:.1}   ({} sesiones)", media(&pico_carga), pico_carga.len());

    println!("\n=== TEMPERATURA Y CADENCIA, DE LAS MUESTRAS ===");
    println!("  temperatura media de la sesion: {:.1} C   ({} sesiones la traen)", media(&temperaturas), temperaturas.len());
    println!("  cadencia media de las muestras: {:.1} rpm ({} valores)", media(&cadencias_muestra), cadencias_muestra.len());

    // LA DISTANCIA SE COMPRUEBA CONTRA SI MISMA, y es la comprobacion que sale gratis: la distancia
    // de la ULTIMA muestra y el total que declara el fichero son el mismo viaje medido dos veces. Si
    // no cuadran, una de las dos esta mal — y como el motor ya lee las dos, no cuadrarlas en silencio
    // seria quedarse con la duda teniendo la respuesta delante.
    let mut con_distancia = 0usize;
    let mut cuadran = 0usize;
    let mut peor_diferencia = 0.0_f64;
    for (_, l) in &lecturas {
        let ultima = l.sesion.muestras.iter().filter_map(|m| m.distancia_m.valor()).next_back();
        let declarada = l.declarado.distancia_m.valor();
        if let (Some(u), Some(d)) = (ultima, declarada) {
            con_distancia += 1;
            let diferencia = (u - d).abs();
            if diferencia < 20.0 {
                cuadran += 1;
            } else {
                peor_diferencia = peor_diferencia.max(diferencia);
            }
        }
    }
    println!(
        "  distancia: la ultima muestra cuadra con el total declarado en {cuadran} de {con_distancia} sesiones"
    );
    if peor_diferencia > 0.0 {
        println!("    (peor diferencia: {peor_diferencia:.0} m — no se corrige, se dice)");
    }

    println!("\n=== LO QUE EL FICHERO DECLARA (se conserva para comparar, no para sustituir) ===");
    println!("  potencia normalizada ... {} sesiones, media {:.1} W", np.len(), media(&np));
    println!("  TSS ................... {} sesiones, media {:.1}", tss.len(), media(&tss));
    println!("  factor de intensidad .. {} sesiones, media {:.3}", fi.len(), media(&fi));
    println!("  umbral de potencia .... {} sesiones, media {:.0} W", ftp.len(), media(&ftp));

    // Y EL CONTRASTE QUE VALE: la cadencia guardada en las muestras contra la que declara el fichero.
    // Un lector que lea mal un campo cuadra consigo mismo; contra el fichero, no cuadra.
    let mut comparadas = 0usize;
    let mut cuadran = 0usize;
    let mut sin_pedaleo = 0usize;
    let mut peor_cadencia = 0.0_f64;
    for (_, l) in &lecturas {
        let cadencias = l.sesion.cadencias();
        let declarada = match l.declarado.cadencia_media.valor() {
            Some(d) if !cadencias.is_empty() => d,
            Some(_) => {
                // El fichero declara cadencia y las muestras vienen todas a cero: el potenciometro no
                // la mando. Se cuenta APARTE, porque no es un fallo del lector.
                sin_pedaleo += 1;
                continue;
            }
            None => continue,
        };
        comparadas += 1;
        let media = cadencias.iter().map(|c| f64::from(*c)).sum::<f64>() / cadencias.len() as f64;
        if (media - declarada).abs() <= 2.0 {
            cuadran += 1;
        } else {
            peor_cadencia = peor_cadencia.max((media - declarada).abs());
        }
    }
    println!(
        "  cadencia media ......... la de las muestras cuadra con la declarada en {cuadran} de {comparadas} sesiones"
    );
    if peor_cadencia > 0.0 {
        println!("    (peor diferencia: {peor_cadencia:.1} rpm — no se corrige, se dice)");
    }
    println!("  sesiones con la cadencia en TODO CERO en las muestras: {sin_pedaleo}");

    // EL CONTADOR DE HUECOS. Es la respuesta a «el sensor no lo mide» frente a «hoy no lo midio»: si
    // un campo sale aqui con miles de `Invalid` y otro con cero, no son lo mismo aunque los dos
    // acaben en `Falta` dentro del modelo.
    let total_invalidos: usize = invalidos.values().sum();
    println!("\n=== VALORES `Invalid` DEL FICHERO, TRATADOS COMO AUSENCIA ===");
    println!("  total: {total_invalidos} valores en {} campos", invalidos.len());
    for (campo, n) in &invalidos {
        println!("  {n:>6}  {campo}");
    }

    // Y la trampa que se dice en voz alta cada vez que este programa se ejecuta, porque cuesta dos
    // conclusiones falsas y volver a pisarla es gratis.
    println!("\n=== LO QUE ESTE PROGRAMA **NO** MIDE, A PROPOSITO ===");
    println!("  reparto de potencia izquierda/derecha: NO VIENE en el FIT (lo calcula Garmin en su");
    println!("    servidor). `left_right_balance` trae el NUMERO DE SERIE del potenciometro.");
    println!("  centro de presion (PCO): `avg_left_pco`/`avg_right_pco` valen 0,00 siempre: es el");
    println!("    centinela de un sensor que no lo mide. Guardarlo seria un cero con forma de dato.");
    println!("  Lo unico que dice algo de UNA pierna es la FASE, que si se mide aqui arriba.");
}
