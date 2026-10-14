//! El **lector de FIT**: traduce el fichero del aparato a **nuestro** modelo.
//!
//! POR QUÉ ESTA PIEZA ES LA QUE FALTABA: hasta ahora el motor leía un JSON que **yo preparaba a
//! mano**. Con esto, **sus ficheros entran directamente**, y el motor deja de depender de que alguien
//! los convierta.
//!
//! QUÉ TRAE UN FIT, y se comprobó abriendo uno de los suyos (no de memoria):
//!
//! | mensaje | cuántos en una sesión | qué es |
//! |---|---:|---|
//! | `Record` | 7 815 | **las muestras, una por segundo** |
//! | `Hrv` | 8 023 | **la variabilidad del pulso, latido a latido** |
//! | `Lap` | 48 | las vueltas |
//! | `TimeInZone` | 49 | el tiempo en cada zona |
//! | `Session` | 1 | los totales |
//!
//! **El FIT ya trae los totales y las zonas que calculó Garmin.** Eso es importante y hay que decirlo:
//! **el motor no necesita recalcular lo que ya viene** — lo que aporta es **lo que NO viene**, que son
//! los patrones: la deriva, la eficiencia y el desfase.
//!
//! Y LOS DOS SE COMPARAN: si el total de Garmin y el nuestro no coinciden, **uno de los dos está mal**,
//! y eso es una comprobación que sale gratis.

use crate::datos::{Biomecanica, Dato, Muestra, Persona, Sesion};
use std::fs::File;
use std::path::Path;

/// Lo que se saca de un fichero FIT.
#[derive(Debug, Clone)]
pub struct Lectura {
    pub sesion: Sesion,
    /// Los totales que **el propio fichero declara**, para poder compararlos con los nuestros.
    pub declarado: Declarado,
    /// Cuántos mensajes de cada tipo traía, para saber qué había dentro.
    pub mensajes: Vec<(String, usize)>,
    /// **Cuántos valores venían marcados como `Invalid`**, por campo.
    ///
    /// ES UN CONTADOR APARTE Y NO UN DATO, y esa distinción es la lección: `fitparser` tiene una
    /// variante propia, `Value::Invalid`, para el hueco que el aparato escribe cuando **no midió**.
    /// El lector la trata como ausencia —que es lo correcto— pero **tratarla en silencio es lo que
    /// hace imposible distinguir «este sensor no mide» de «este sensor no midió hoy»**. Guardando
    /// la cuenta, esa pregunta se contesta: medido sobre sus 102 ficheros son **103 huecos, y los
    /// tres campos que los traen son todos de postura** (`avg_power_position`, `max_power_position`,
    /// `avg_cadence_position`). O sea que no es un aparato roto: es que **la postura de pie no
    /// siempre se usa**, y cuando no se usa, el hueco se escribe en vez de un cero.
    ///
    /// Va aquí y no en `Sesion` porque **no es un dato de la sesión**: es una avería del fichero, y
    /// una avería se cuenta, no se promedia.
    pub invalidos: Vec<(String, usize)>,
    /// **Cuándo empezó la sesión**, en segundos desde el 1 de enero de 1970 (UTC). `None` si el
    /// fichero no lo trae.
    ///
    /// ESTE DATO ES LO QUE HACE QUE LA MEMORIA PUEDA DECIR QUÉ PERIODO CUBRE. Sin fecha, un total de
    /// tres meses y un total de hoy se parecen demasiado. Va en `Lectura` y no en `Sesion` porque es
    /// **un dato del fichero**, como los totales declarados: el modelo del motor no necesita la hora
    /// para calcular, pero la memoria sí para no mentir con el periodo.
    pub inicio_s: Option<i64>,
}

/// Los totales que el FIT trae ya calculados. **Se conservan para comparar, no para sustituir.**
#[derive(Debug, Clone, Default)]
pub struct Declarado {
    pub duracion_s: Dato<u32>,
    pub pulso_medio: Dato<u16>,
    pub pulso_maximo: Dato<u16>,
    pub potencia_media: Dato<u16>,
    pub potencia_maxima: Dato<u16>,
    pub distancia_m: Dato<f64>,
    pub calorias: Dato<u16>,
    pub deporte: Dato<String>,
    /// **La potencia normalizada** que declara el fichero (`normalized_power`), en vatios.
    ///
    /// Es la potencia «equivalente» que pesa los esfuerzos por su coste fisiológico, y es la base del
    /// TSS. Y NO ES NUESTRA: **el FIT la trae calculada**, así que aquí se guarda tal cual. El día que
    /// el motor la calcule, se comparan — y si no cuadran, **uno de los dos está mal**. Esa
    /// comparación es la única razón por la que este número entra en el modelo.
    ///
    /// Se declara en **dos sitios del fichero** y en distinto número de ficheros: en `Session` viene
    /// en 83, y en `Lap` en otros 83. Por eso el nombre dice de dónde salió.
    pub potencia_normalizada: Dato<f64>,
    /// **El TSS** que declara el fichero (`training_stress_score`).
    ///
    /// No se recalcula ni se corrige: se conserva para poder contrastarlo. Y va en `f64` aunque
    /// parezca un entero, porque **no lo es**: su máximo medido (222) no cabe en un byte, y
    /// redondearlo aquí sería perder precisión antes de comparar.
    pub tss: Dato<f64>,
    /// **El factor de intensidad** (`intensity_factor`): la potencia normalizada partido por el
    /// umbral. Es el número del que sale el TSS, y viene ya calculado.
    pub factor_intensidad: Dato<f64>,
    /// **El umbral de potencia (FTP)** que declara el fichero (`threshold_power`), en vatios.
    ///
    /// Vale **213 W en las 83 sesiones**, y `ftpAutoDetected: true` dice que **lo detectó el reloj**,
    /// no el cliente. Ese detalle importa: un umbral estimado por el aparato y un umbral medido en
    /// una prueba **no valen lo mismo**, y el motor tiene que poder decirlo.
    pub potencia_umbral: Dato<u16>,
    /// **La cadencia media** que declara el fichero (`avg_cadence`), en rpm.
    ///
    /// Entra por una razón concreta: la cadencia de las muestras **no se puede comprobar sola**. Un
    /// lector que lea mal el campo —o que lo lea de otro mensaje— cuadra consigo mismo y no se nota.
    /// Con la media declarada al lado, la de las muestras se contrasta **contra el fichero**, y esa
    /// comprobación es la que vale.
    ///
    /// OJO CON LA DEFINICIÓN, y esto costó una medición descubrirlo: **el aparato la calcula SIN los
    /// ceros** —su perfil dice «`total_cycles / total_timer_time` **if** `nonzero_avg_cadence`»—. En
    /// uno de sus ficheros la media es 83,5 de las dos maneras, pero en otro la declarada es 75, con
    /// ceros sale 67,9 y sin ceros sale 75,4. Por eso `Sesion::cadencias()` salta los ceros: para
    /// poder comparar con el aparato en vez de discutirle.
    pub cadencia_media: Dato<f64>,
}

/// Lee un fichero FIT y lo traduce a nuestro modelo.
///
/// **Nunca falla por datos raros**: lo que no esté, queda como `Falta`, y el motor ya sabe trabajar
/// así.
pub fn leer(ruta: &Path) -> Result<Lectura, String> {
    let mut f = File::open(ruta).map_err(|e| format!("no se pudo abrir {}: {e}", ruta.display()))?;
    let datos = fitparser::from_reader(&mut f).map_err(|e| format!("el FIT no se pudo leer: {e:?}"))?;

    let mut muestras: Vec<Muestra> = Vec::new();
    let mut intervalos_rr: Vec<f64> = Vec::new();
    let mut declarado = Declarado::default();
    let mut biomecanica = Biomecanica::vacia();
    let mut invalidos: std::collections::BTreeMap<String, usize> = Default::default();
    let mut cuenta: std::collections::BTreeMap<String, usize> = Default::default();
    // La fecha de la sesión: primero la del primer `Record`, y si el mensaje `Session` trae su
    // `start_time`, manda ese (es el que el reloj declara como comienzo, no el primer latido).
    let mut inicio_s: Option<i64> = None;

    for r in datos.iter() {
        let tipo = format!("{:?}", r.kind());
        *cuenta.entry(tipo.clone()).or_insert(0) += 1;

        match tipo.as_str() {
            "Record" => {
                let mut segundo: Option<u32> = None;
                let mut pulso = Dato::Falta;
                let mut potencia = Dato::Falta;
                let mut cadencia = Dato::Falta;
                let mut distancia = Dato::Falta;
                let mut temperatura = Dato::Falta;

                for c in r.fields() {
                    match c.name() {
                        "timestamp" => {
                            // El FIT trae la marca de tiempo. Se usa el SEGUNDO ABSOLUTO, y luego se
                            // resta el primero: lo que importa es el tiempo DENTRO de la sesión.
                            if let fitparser::Value::Timestamp(t) = c.value() {
                                segundo = Some(t.timestamp() as u32);
                                if inicio_s.is_none() {
                                    inicio_s = Some(t.timestamp() as i64);
                                }
                            }
                        }
                        "heart_rate" => pulso = entero(c.value()),
                        "power" => potencia = entero(c.value()),
                        // LA CADENCIA YA NO SE TIRA. Antes se leía y se descartaba con un «el motor
                        // no la usa»; el motor ahora la usa, y mientras no la usó se estaba tirando
                        // el 90 % de un dato que ya estaba leído. Va en `u8` porque el FIT la declara
                        // de un byte.
                        "cadence" => cadencia = como_u8(c.value(), "Record.cadence", &mut invalidos),
                        // La distancia por muestra: sin ella no se puede preguntar DÓNDE pasó algo.
                        "distance" => distancia = real_o_falta(c.value()),
                        // La temperatura, que está en los 102 ficheros y el motor ignoraba. Va en
                        // `f32` porque el FIT la da con decimales y es un dato de ambiente, no una
                        // medida de precisión.
                        "temperature" => {
                            temperatura = match real(c.value()) {
                                Some(v) => Dato::Hay(v as f32),
                                None => Dato::Falta,
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(seg) = segundo {
                    muestras.push(Muestra {
                        segundo: seg,
                        pulso,
                        potencia_w: potencia,
                        cadencia_rpm: cadencia,
                        distancia_m: distancia,
                        temperatura_c: temperatura,
                    });
                }
            }
            // EL HRV: cada mensaje trae un array de intervalos entre latidos, en SEGUNDOS.
            // Se pasan a milisegundos, que es como se trabaja. Y los `Invalid` se saltan: son huecos
            // del aparato, no latidos.
            "Hrv" => {
                for c in r.fields() {
                    if c.name() == "time" {
                        if let fitparser::Value::Array(v) = c.value() {
                            for x in v {
                                if let fitparser::Value::Float64(s) = x {
                                    intervalos_rr.push(s * 1000.0);
                                }
                            }
                        }
                    }
                }
            }
            "Session" => {
                for c in r.fields() {
                    match c.name() {
                        // La fecha que declara el reloj para la sesión. Manda sobre la del primer
                        // registro: una sesión puede empezar antes del primer latido.
                        "start_time" => {
                            if let fitparser::Value::Timestamp(t) = c.value() {
                                inicio_s = Some(t.timestamp() as i64);
                            }
                        }
                        "total_elapsed_time" | "total_timer_time" => {
                            if declarado.duracion_s.falta() {
                                if let Some(v) = real(c.value()) {
                                    declarado.duracion_s = Dato::Hay(v as u32);
                                }
                            }
                        }
                        "avg_heart_rate" => declarado.pulso_medio = entero(c.value()),
                        "max_heart_rate" => declarado.pulso_maximo = entero(c.value()),
                        "avg_power" => declarado.potencia_media = entero(c.value()),
                        "max_power" => declarado.potencia_maxima = entero(c.value()),
                        "total_distance" => {
                            if let Some(v) = real(c.value()) {
                                declarado.distancia_m = Dato::Hay(v);
                            }
                        }
                        "total_calories" => declarado.calorias = entero(c.value()),
                        "sport" => {
                            if let fitparser::Value::String(s) = c.value() {
                                declarado.deporte = Dato::Hay(s.clone());
                            }
                        }
                        // ---- LOS TOTALES NUEVOS: se conservan para comparar, no para sustituir ----
                        "normalized_power" => {
                            declarado.potencia_normalizada = real_o_falta(c.value());
                        }
                        "training_stress_score" => declarado.tss = real_o_falta(c.value()),
                        "intensity_factor" => declarado.factor_intensidad = real_o_falta(c.value()),
                        "threshold_power" => {
                            declarado.potencia_umbral = entero(c.value());
                        }
                        "avg_cadence" => declarado.cadencia_media = real_o_falta(c.value()),
                        // ---- LA POSTURA: sentado de pie. **NO es izquierda/derecha.** ----
                        //
                        // `avg_power_position` y `avg_cadence_position` son DOS huecos indexados por
                        // `rider_position_type` (`Seated = 0`, `Standing = 1`), y por eso el índice 0
                        // va a un campo que dice «sentado»: **quien lea el índice 0 aquí no puede
                        // creer que es la pierna izquierda**, porque el nombre del campo lo impide.
                        //
                        // Se piden los DOS huecos válidos (ver `dos_huecos`): medido en sus ficheros,
                        // hay arrays como `Array([UInt16(147), Invalid])` — uno medido y otro no— y
                        // en ese caso **el que falta queda `Falta`**, nunca un cero.
                        "avg_power_position" => {
                            let v = dos_huecos(c.value(), c.name(), &mut invalidos);
                            biomecanica.potencia_media_sentado = de_hueco(v[0], a_entero_u16);
                            biomecanica.potencia_media_de_pie = de_hueco(v[1], a_entero_u16);
                        }
                        "max_power_position" => {
                            let v = dos_huecos(c.value(), c.name(), &mut invalidos);
                            biomecanica.potencia_maxima_sentado = de_hueco(v[0], a_entero_u16);
                            biomecanica.potencia_maxima_de_pie = de_hueco(v[1], a_entero_u16);
                        }
                        "avg_cadence_position" => {
                            let v = dos_huecos(c.value(), c.name(), &mut invalidos);
                            biomecanica.cadencia_media_sentado = de_hueco(v[0], Dato::Hay);
                            biomecanica.cadencia_media_de_pie = de_hueco(v[1], Dato::Hay);
                        }
                        // ---- LA FASE DE PEDALEO: esto SÍ es por pierna ----
                        //
                        // Cuatro ángulos por pierna. **Los cuatro o ninguno**: una fase a medias no
                        // se promedia, porque el ángulo que falta se leería como cero grados y cero
                        // grados es una posición del pedal, no un hueco.
                        "avg_left_power_phase" => {
                            biomecanica.fase_pierna_izquierda =
                                cuatro_grados(c.value(), c.name(), &mut invalidos);
                        }
                        "avg_right_power_phase" => {
                            biomecanica.fase_pierna_derecha =
                                cuatro_grados(c.value(), c.name(), &mut invalidos);
                        }
                        // ---- LA RESPIRACIÓN, QUE NO SE HABÍA PERDIDO ----
                        //
                        // Se mira primero la versión `enhanced` (la que trae decimales) y si no está,
                        // la corta. Son EL MISMO dato en dos precisiones, no dos datos: quedarse con
                        // las dos sería contar dos veces lo mismo.
                        "enhanced_avg_respiration_rate" => {
                            biomecanica.respiracion_media = real_o_falta(c.value());
                        }
                        "avg_respiration_rate" => {
                            if biomecanica.respiracion_media.falta() {
                                biomecanica.respiracion_media = real_o_falta(c.value());
                            }
                        }
                        "enhanced_max_respiration_rate" => {
                            biomecanica.respiracion_maxima = real_o_falta(c.value());
                        }
                        "max_respiration_rate" => {
                            if biomecanica.respiracion_maxima.falta() {
                                biomecanica.respiracion_maxima = real_o_falta(c.value());
                            }
                        }
                        "total_anaerobic_training_effect" => {
                            biomecanica.carga_anaerobica = real_o_falta(c.value());
                        }
                        "training_load_peak" => {
                            biomecanica.pico_de_carga = real_o_falta(c.value());
                        }
                        // ---- LO QUE **NO** SE LEE, Y POR QUÉ ----
                        //
                        // `avg_left_pco` y `avg_right_pco` (el centro de presión del pedal) se
                        // IGNORAN a propósito. No es un olvido: valen **0,00 mm en las 83 sesiones y
                        // nunca difieren**, o sea que el sensor no los está midiendo — devuelve su
                        // centinela. Guardarlos como dato metería un cero que parece una medida, y
                        // ese es exactamente el error que este motor existe para no cometer. Si algún
                        // día alguien cambia de potenciómetro y los valores se mueven, se añaden; hoy
                        // no hay nada que añadir.
                        _ => {}
                    }
                }
            }
            // La potencia normalizada viene en DOS mensajes distintos segun el fichero: en `Session`
            // (el total de la sesion) y en `Lap` (la vuelta, que en una salida de una sola vuelta es
            // lo mismo). Medido: 83 ficheros la traen en `Session`. Se guarda la primera que
            // aparezca, y **se dice de donde salio** para que nadie compare dos cosas distintas.
            "Lap" => {
                for c in r.fields() {
                    if c.name() == "normalized_power" && declarado.potencia_normalizada.falta() {
                        declarado.potencia_normalizada = real_o_falta(c.value());
                    }
                }
            }
            _ => {}
        }
    }

    // Los segundos absolutos se convierten a tiempo DENTRO de la sesión, y si el fichero declarara
    // una duración distinta, manda la de las muestras: es la que se puede comprobar.
    if let (Some(primero), Some(ultimo)) = (
        muestras.iter().map(|m| m.segundo).min(),
        muestras.iter().map(|m| m.segundo).max(),
    ) {
        for m in &mut muestras {
            m.segundo = m.segundo.saturating_sub(primero);
        }
        if declarado.duracion_s.falta() {
            declarado.duracion_s = Dato::Hay(ultimo.saturating_sub(primero));
        }
    }

    // La distancia TOTAL declarada se sigue guardando aparte, en `Declarado`, para poder compararla
    // con la de la ultima muestra: si no cuadran, una de las dos esta mal — y esa comprobacion sale
    // gratis. Lo que ya no se hace es tirar la distancia de cada muestra.

    // La sesión se arma con los CONSTRUCTORES, no con un literal de struct. Es lo que hace que
    // añadir un campo al modelo —como la biomecánica hoy— no obligue a tocar este sitio otra vez: se
    // añade un `con_...` y el resto del lector sigue igual.
    let mut sesion = Sesion::nueva(muestras).con_rr(intervalos_rr).con_biomecanica(biomecanica);
    sesion.duracion_s = declarado.duracion_s;

    Ok(Lectura {
        sesion,
        declarado,
        mensajes: cuenta.into_iter().collect(),
        invalidos: invalidos.into_iter().collect(),
        inicio_s,
    })
}

/// **Los FIT de una carpeta con su TAMAÑO, sin leer ninguno**, ordenados por nombre.
///
/// ES LA BASE DE LA MEMORIA INCREMENTAL: mirar el tamaño de 102 ficheros es instantáneo, y con eso se
/// decide **cuáles hay que leer de verdad**. Los ficheros de otro tipo no cuentan: no se leen, así que
/// no cambian el resultado.
///
/// **EL TAMAÑO Y NO LA FECHA**, a propósito: la fecha cambia al copiar un fichero aunque el contenido
/// sea el mismo, y con la fecha, copiar sus datos invalidaría media memoria sin motivo.
pub fn listar(carpeta: &Path) -> Vec<(String, u64)> {
    let mut v: Vec<(String, u64)> = match std::fs::read_dir(carpeta) {
        Ok(d) => d
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "fit").unwrap_or(false))
            .filter_map(|e| {
                let nombre = e.file_name().to_string_lossy().to_string();
                let tam = e.metadata().ok()?.len();
                Some((nombre, tam))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    v.sort();
    v
}

/// Un valor que no está **se cuenta**, para poder distinguir «el aparato no lo mide» de «hoy no lo
/// midió». `fitparser` marca esos huecos con su propia variante, `Value::Invalid`.
fn contar_invalido(campo: &str, invalidos: &mut std::collections::BTreeMap<String, usize>) {
    *invalidos.entry(campo.to_string()).or_insert(0) += 1;
}

/// Saca un entero de FIT **como `u8`**, que es como el formato declara la cadencia.
///
/// POR QUÉ NO VALE `entero()`: allí un `SInt8` negativo se recorta a 0, y aquí un valor negativo no
/// es una cadencia baja — es un valor imposible, y **lo imposible se trata como ausencia**. Un `-1`
/// convertido en `0` rpm viajaría hasta la media sin que nadie lo viera.
fn como_u8(v: &fitparser::Value, campo: &str, invalidos: &mut std::collections::BTreeMap<String, usize>) -> Dato<u8> {
    match v {
        fitparser::Value::UInt8(x) => Dato::Hay(*x),
        fitparser::Value::UInt16(x) => Dato::Hay((*x).min(u16::from(u8::MAX)) as u8),
        fitparser::Value::SInt8(x) if *x >= 0 => Dato::Hay(*x as u8),
        fitparser::Value::SInt16(x) if *x >= 0 => Dato::Hay((*x).min(i16::from(u8::MAX)) as u8),
        // `Value::Invalid` es la variante con la que `fitparser` dice «aquí no hay medida». Se trata
        // como ausencia, que es lo correcto, **pero se cuenta**: así se puede decir cuántas veces el
        // aparato escribió un hueco, que es un dato sobre el sensor y no sobre la sesión.
        fitparser::Value::Invalid => {
            contar_invalido(campo, invalidos);
            Dato::Falta
        }
        _ => Dato::Falta,
    }
}

/// Un número con decimales, o `Falta`. **Un `Invalid` nunca se convierte en cero.**
fn real_o_falta(v: &fitparser::Value) -> Dato<f64> {
    match real(v) {
        Some(x) => Dato::Hay(x),
        None => Dato::Falta,
    }
}

/// Un número con decimales, redondeado, para un campo que el modelo guarda entero.
///
/// Pública para que la prueba del error de `avg_power_position` pueda poner el valor real del fichero
/// en el campo que dice el formato, sin tener que pasar por un fichero de disco.
pub fn a_entero_u16(v: f64) -> Dato<u16> {
    if v < 0.0 {
        return Dato::Falta;
    }
    Dato::Hay(v.round().min(f64::from(u16::MAX)) as u16)
}

/// Convierte **un hueco de un array** en un dato del modelo: `None` es `Falta`, y nunca un cero.
pub fn de_hueco<T>(hueco: Option<f64>, convertir: impl Fn(f64) -> Dato<T>) -> Dato<T> {
    match hueco {
        Some(v) => convertir(v),
        None => Dato::Falta,
    }
}

/// **Los dos huecos de un campo que el FIT indexa por postura** (`rider_position_type`).
///
/// LOS HUECOS SON `[SENTADO, DE PIE]`, **no** `[izquierda, derecha]`, y por eso la función se llama
/// `dos_huecos` y no `izquierda_y_derecha`: el nombre de una función también puede mentir, y este es
/// el sitio exacto donde se cometió el error la primera vez.
///
/// COMPROBAR LA FORMA ANTES DE USARLA no es una precaución teórica: medido en sus ficheros, hay
/// arrays como `Array([UInt16(147), Invalid])` — **un hueco medido y el otro no**. En ese caso el
/// hueco que falta queda `None` y el que está se conserva. Lo que **no** se hace nunca es rellenar el
/// que falta con un cero: un cero en «potencia de pie» se promedia con las demás sesiones y hunde la
/// media, y el resultado sigue pareciendo un número.
///
/// Y si el array no tiene dos huecos —uno, tres, o ninguno—, no se inventa una postura: **los dos
/// quedan ausentes** y se cuenta la forma rara.
///
/// Es pública porque **la prueba que deja escrito el error** tiene que poder pasarle un
/// `Array([UInt16(147), Invalid])` sin depender de que exista un fichero así en el disco.
pub fn dos_huecos(
    v: &fitparser::Value,
    campo: &str,
    invalidos: &mut std::collections::BTreeMap<String, usize>,
) -> [Option<f64>; 2] {
    let fitparser::Value::Array(xs) = v else {
        return [None, None];
    };
    if xs.len() != 2 {
        contar_invalido(campo, invalidos);
        return [None, None];
    }
    let mut fuera = [None, None];
    for (i, x) in xs.iter().enumerate() {
        fuera[i] = real(x);
        if matches!(x, fitparser::Value::Invalid) {
            contar_invalido(campo, invalidos);
        }
    }
    fuera
}

/// **Los cuatro ángulos de la fase de una pierna**, o `Falta`.
///
/// SON LOS CUATRO O NINGUNO. El índice 0 es el ángulo de inicio del empuje y el 2 el de fin, así que
/// «casi todos» no sirve: una fase con un hueco dentro daría un arco de empuje calculado contra un
/// cero, o sea **una pierna que no empuja nada**, que es una conclusión sobre su rodilla sacada de un
/// hueco del fichero.
///
/// Y ya vienen **en grados**: el lector de FIT aplica la escala del formato (0,7111111 sobre el valor
/// crudo) antes de entregarlos. Medido en un fichero suyo: `357,19 · 184,22 · 185,63 · 90,00`. **No
/// hay que multiplicar por nada**, y hacerlo daría cifras 1,4 veces mayores que parecerían igual de
/// razonables.
///
/// Pública por el mismo motivo que `dos_huecos`: la prueba del hueco dentro de la fase necesita
/// fabricar el array, y un array así puede no existir hoy en el disco.
pub fn cuatro_grados(
    v: &fitparser::Value,
    campo: &str,
    invalidos: &mut std::collections::BTreeMap<String, usize>,
) -> Dato<[f64; 4]> {
    let fitparser::Value::Array(xs) = v else {
        return Dato::Falta;
    };
    if xs.len() != 4 {
        contar_invalido(campo, invalidos);
        return Dato::Falta;
    }
    let mut grados = [0.0_f64; 4];
    for (i, x) in xs.iter().enumerate() {
        match real(x) {
            Some(n) => grados[i] = n,
            None => {
                // Un solo hueco invalida la fase ENTERA: media fase no se promedia.
                contar_invalido(campo, invalidos);
                return Dato::Falta;
            }
        }
    }
    Dato::Hay(grados)
}

/// Saca un entero de un valor de FIT, sea del tipo que sea. Devuelve `Falta` si no es un número.
fn entero(v: &fitparser::Value) -> Dato<u16> {
    match v {
        fitparser::Value::UInt8(x) => Dato::Hay(u16::from(*x)),
        fitparser::Value::UInt16(x) => Dato::Hay(*x),
        fitparser::Value::UInt32(x) => Dato::Hay(*x as u16),
        fitparser::Value::SInt8(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::SInt16(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::SInt32(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::Float32(x) => Dato::Hay(x.round().max(0.0) as u16),
        fitparser::Value::Float64(x) => Dato::Hay(x.round().max(0.0) as u16),
        // Un valor de un tipo que no se espera **no se adivina**: se trata como ausente.
        _ => Dato::Falta,
    }
}

/// Saca un número con decimales de un valor de FIT.
fn real(v: &fitparser::Value) -> Option<f64> {
    match v {
        fitparser::Value::Float32(x) => Some(f64::from(*x)),
        fitparser::Value::Float64(x) => Some(*x),
        fitparser::Value::UInt8(x) => Some(f64::from(*x)),
        fitparser::Value::UInt16(x) => Some(f64::from(*x)),
        fitparser::Value::UInt32(x) => Some(f64::from(*x)),
        fitparser::Value::SInt8(x) => Some(f64::from(*x)),
        fitparser::Value::SInt16(x) => Some(f64::from(*x)),
        fitparser::Value::SInt32(x) => Some(f64::from(*x)),
        _ => None,
    }
}

/// Lee **todos** los FIT de una carpeta, ordenados por nombre.
///
/// Los que no se puedan leer **se cuentan y se dicen**, en vez de perderse en silencio: un fichero que
/// no se lee es un dato que falta, y **lo que falta hay que decirlo**.
pub fn leer_carpeta(carpeta: &Path) -> (Vec<(String, Lectura)>, Vec<(String, String)>) {
    leer_carpeta_hasta(carpeta, usize::MAX)
}

/// Igual, pero **parando a los `tope` ficheros**.
///
/// POR QUÉ EXISTE: leer 102 ficheros de verdad tarda unos segundos, y una prueba que tarda deja de
/// ejecutarse. Con un tope, las pruebas comprueban **el lector** —que es lo que hay que comprobar— sin
/// pagar el precio de leer el histórico entero. **Una prueba lenta es una prueba que se deja de
/// correr**, y entonces deja de proteger.
///
/// Y EL TOPE SE APLICA **DESPUÉS DE ELEGIR LOS FIT**, que es un fallo que ya se midió y engañaba:
/// la primera versión ordenaba y truncaba antes de filtrar, y en su carpeta **los 102 primeros
/// ficheros por nombre son los `.json` de Garmin**, no las actividades. Con `tope = 1` o `12` —que es
/// justo lo que usan las pruebas— **no leía ni un FIT y devolvía «0 leídos» sin dar un solo error**.
/// Eso es lo peor que puede hacer un lector: parecer que funciona. Lo cazó una prueba nueva que
/// contaba los ficheros con cadencia y se encontró con cero.
pub fn leer_carpeta_hasta(carpeta: &Path, tope: usize) -> (Vec<(String, Lectura)>, Vec<(String, String)>) {
    let mut bien = Vec::new();
    let mut mal = Vec::new();
    let mut rutas: Vec<_> = match std::fs::read_dir(carpeta) {
        Ok(d) => d
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            // Se ELIGE primero y se RECORTA después. Al revés, el tope se gasta en ficheros que no
            // son FIT y la función devuelve vacío como si no hubiera datos.
            .filter(|r| r.extension().map(|e| e == "fit").unwrap_or(false))
            .collect(),
        Err(e) => return (bien, vec![(carpeta.display().to_string(), format!("{e}"))]),
    };
    rutas.sort();
    rutas.truncate(tope);
    for r in rutas {
        match leer(&r) {
            Ok(l) => bien.push((r.file_name().unwrap_or_default().to_string_lossy().to_string(), l)),
            Err(e) => mal.push((r.file_name().unwrap_or_default().to_string_lossy().to_string(), e)),
        }
    }
    (bien, mal)
}

/// Un perfil de persona **a partir de lo que el FIT declara**, para cuando no hay más.
///
/// El FIT **no trae** la edad, el peso ni la grasa: eso no lo mide el aparato. Lo que sí trae es el
/// **pulso máximo de la sesión**, que sirve como suelo — y el motor ya sabe que eso es una estimación.
pub fn persona_desde(lectura: &Lectura) -> Persona {
    let mut p = Persona::desconocida();
    if let Dato::Hay(m) = lectura.declarado.pulso_maximo {
        p.pulso_maximo_medido = Dato::Hay(m);
    }
    p
}
