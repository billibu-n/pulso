//! **La ventana, con el motor dentro.**
//!
//! `cargo run --bin ventana`
//!
//! Y UNA REGLA QUE SE CUMPLE AQUI: **esta ventana NO calcula nada.** Llama al motor, recibe numeros
//! ya calculados con su margen y su fuente, y los **formatea para enseñarlos**. La unica aritmetica
//! que hay es **convertir un limite en un ancho de barra** (0 a 100), porque eso es dibujar, no
//! calcular.
//!
//! SI AQUI HUBIERA UNA CUENTA, el dia que se cambie una formula del motor **la pantalla mentiria sin
//! avisar** — y nadie lo notaria, porque seguiria enseñando numeros plausibles.

slint::include_modules!();

use motor::datos::{Dato, Persona};
use motor::{fit, hrv, patron, umbral};
use std::path::Path;

/// La persona de la que se parte. **Cuando el motor lea su perfil de verdad, esto se llena solo.**
fn persona() -> Persona {
    let mut p = Persona::desconocida();
    p.edad = Dato::Hay(60);
    p.pulso_reposo = Dato::Hay(48);
    p.peso_kg = Dato::Hay(70.2);
    p.altura_cm = Dato::Hay(178.0);
    p.grasa_corporal_pct = Dato::Hay(19.1);
    p.umbral_lactato = Dato::Hay(169);
    p.pulso_maximo_medido = Dato::Hay(183);
    p
}

/// **Los resultados REALES, leídos de sus ficheros.**
///
/// POR QUE ESTO ES LO QUE FALTABA: la primera version de la ventana enseñaba cuatro cifras escritas
/// a mano en el codigo. **El motor calcula 625 resultados y la ventana enseñaba cuatro.** Una
/// pantalla con numeros escritos a mano no es una aplicacion: es un dibujo.
///
/// Y si la carpeta no está, **la ventana sigue abriendo** y lo dice -- porque el escenario «no hay
/// datos» tambien hay que soportarlo.
struct Real {
    sesiones: usize,
    muestras: usize,
    rr: usize,
    deriva_media: Option<f64>,
    eficiencia_media: Option<f64>,
    escapados: usize,
    hrv_medio: Option<f64>,
    maxima_vista: Option<u16>,
}

fn leer_real(carpeta: &Path) -> Option<Real> {
    let (ficheros, _) = fit::leer_carpeta(carpeta);
    if ficheros.is_empty() {
        return None;
    }
    let mut r = Real {
        sesiones: ficheros.len(),
        muestras: 0,
        rr: 0,
        deriva_media: None,
        eficiencia_media: None,
        escapados: 0,
        hrv_medio: None,
        maxima_vista: None,
    };
    let mut derivas: Vec<f64> = Vec::new();
    let mut efs: Vec<f64> = Vec::new();
    let mut hrvs: Vec<f64> = Vec::new();

    for (_, l) in &ficheros {
        r.muestras += l.sesion.muestras.len();
        r.rr += l.sesion.intervalos_rr_ms.len();
        if let Some(m) = l.declarado.pulso_maximo.valor() {
            r.maxima_vista = Some(r.maxima_vista.map_or(m, |x: u16| x.max(m)));
        }
        let d = patron::deriva(&l.sesion);
        if d.muestras >= 20 {
            derivas.push(d.porcentaje());
            if d.porcentaje() > 10.0 {
                r.escapados += 1;
            }
        }
        let e = patron::eficiencia(&l.sesion);
        if e.valor > 0.0 {
            efs.push(e.valor);
        }
        let h = hrv::calcular(hrv::intervalos_de(&l.sesion));
        if h.latidos >= 120 {
            hrvs.push(h.rmssd.valor);
        }
    }
    let media = |v: &[f64]| if v.is_empty() { None } else { Some(v.iter().sum::<f64>() / v.len() as f64) };
    r.deriva_media = media(&derivas);
    r.eficiencia_media = media(&efs);
    r.hrv_medio = media(&hrvs);
    Some(r)
}

/// **Un numero con la coma decimal**, como se escribe en espanol.
///
/// POR QUE EXISTE: el formato de Rust pone el PUNTO (`0.98`), y en la pantalla se leia «0.98» al lado
/// de un «1,02» escrito a mano. **Dos formas en la misma pantalla se ven como un descuido**, y el
/// dueño lo vio en la comparacion. Aqui se decide UNA vez como se escriben los numeros.
fn num(v: f64, decimales: usize) -> String {
    format!("{v:.decimales$}").replace('.', ",")
}

fn main() -> Result<(), slint::PlatformError> {
    let ventana = Ventana::new()?;

    // ---- LOS DATOS DE VERDAD, de sus ficheros ----
    let carpeta = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../datos/garmin_files/storage".to_string());
    let real = leer_real(Path::new(&carpeta));

    let mut p = persona();
    // La frecuencia maxima se busca en TODAS las sesiones: es donde esta.
    if let Some(r) = &real {
        if let Some(m) = r.maxima_vista {
            p.pulso_maximo_medido = Dato::Hay(m);
        }
    }

    // --- Lo que ENTREGA el motor. Aqui no se calcula: se pide. ---
    let u = umbral::umbral(&p);
    let zonas = umbral::zonas_del_umbral(&p);

    // Las cuatro cifras, ya formateadas. **Los margenes van CON el numero**, no en una nota al pie:
    // es lo que este motor tiene de distinto.
    let deriva = real.as_ref().and_then(|r| r.deriva_media);
    let eficiencia = real.as_ref().and_then(|r| r.eficiencia_media);
    let las_cifras: Vec<slint::SharedString> = vec![
        format!("{:.0}±{:.0}", u.rango.valor, u.rango.margen).into(),
        format!("{:.0}", p.pulso_maximo_medido.valor().unwrap_or(0)).into(),
        eficiencia.map_or("—".to_string(), |e| num(e, 2)).into(),
        deriva.map_or("—".to_string(), |d| format!("{}{}", if d >= 0.0 { "+" } else { "-" }, num(d.abs(), 1))).into(),
    ];
    ventana.set_cifras(las_cifras.as_slice().into());
    let etiquetas: Vec<slint::SharedString> = vec![
        "umbral (pulsaciones)".into(),
        "frecuencia máxima".into(),
        "vatios por pulso".into(),
        "deriva %".into(),
    ];
    ventana.set_etiquetas(etiquetas.as_slice().into());

    // Las zonas: nombre, TRAMO, y el nivel de la barra.
    // EL TRAMO, no el limite: una zona empieza donde acaba la otra, y dos numeros iguales seguidos
    // no se entienden. Ese defecto ya salio una vez en el informe.
    let nombres: Vec<slint::SharedString> =
        zonas.iter().map(|(z, _, _)| z.nombre().into()).collect();
    let tramos: Vec<slint::SharedString> = zonas
        .iter()
        .map(|(_, desde, hasta)| {
            if (hasta.valor - desde.valor).abs() < 1.0 {
                format!("hasta {:.0}", hasta.valor).into()
            } else {
                format!("{:.0} – {:.0}", desde.valor, hasta.valor).into()
            }
        })
        .collect();
    // El nivel: el ancho de la barra, en proporcion a la zona mas alta. Es DIBUJAR, no calcular.
    let tope = zonas.iter().map(|(_, _, h)| h.valor).fold(f64::MIN, f64::max);
    let niveles: Vec<i32> = zonas
        .iter()
        .map(|(_, _, h)| (h.valor / tope * 100.0).round() as i32)
        .collect();

    ventana.set_zonas_nombres(nombres.as_slice().into());
    ventana.set_zonas_tramos(tramos.as_slice().into());
    ventana.set_zonas_niveles(niveles.as_slice().into());
    // Se guardan para poder imprimirlos si se pide: los MISMOS que van a la pantalla.
    let copia_nombres = nombres.clone();
    let copia_tramos = tramos.clone();
    let copia_niveles = niveles.clone();

    // La precision, en UNA linea. Y **de donde sale el dato**, que es lo que lo hace creible.
    ventana.set_precision(
        format!(
            "Precisión {}: las zonas salen del {}, que es un dato medido.",
            u.rango.como_va().nombre(),
            u.referencia.nombre()
        )
        .into(),
    );

    // Lo que se deduce de los datos: **lo que el fichero del aparato NO trae.**
    ventana.set_aviso(
        match (&real, deriva) {
            (Some(r), Some(d)) => format!(
                "{} % de deriva media en {} sesiones: {} se le escapan. Sacado de {} lecturas.",
                num(d, 1),
                r.sesiones,
                r.escapados,
                r.muestras
            )
            .into(),
            _ => "Sin datos todavía: la deriva y la eficiencia aparecerán aquí.".into(),
        },
    );

    // Y el vacio, con lo que HAY, no con un texto fijo.
    ventana.set_vacio(
        match &real {
            Some(r) => format!(
                "{} sesiones · {} lecturas · {} intervalos de HRV",
                r.sesiones, r.muestras, r.rr
            ),
            None => format!("Sin datos en {carpeta}"),
        }
        .into(),
    );

    // ---- Y SI SE PIDE, SE IMPRIME LO MISMO QUE VA A LA PANTALLA ----
    //
    // POR QUE EXISTE: la ventana se ve, pero **no se puede comprobar desde fuera**. Imprimir los
    // MISMOS valores que recibe permite verificar que enseña los datos de verdad y no unos escritos
    // a mano -- que es lo que hacia la primera version.
    if std::env::args().any(|a| a == "--texto") {
        println!("--- LO QUE LA VENTANA RECE Y ENSEÑA ---");
        for i in 0..las_cifras.len() {
            println!("  {:<12} {}", las_cifras[i], etiquetas[i]);
        }
        println!();
        for i in 0..copia_nombres.len() {
            println!(
                "  {:<16} {:<14} barra {}%",
                copia_nombres[i], copia_tramos[i], copia_niveles[i]
            );
        }
        println!();
        println!("  {}", ventana.get_precision());
        println!("  {}", ventana.get_aviso());
        println!("  {}", ventana.get_vacio());
        println!();
        println!("  ==> los MISMOS valores que salen en pantalla, no un resumen aparte.");
        // Y AQUI SE TERMINA, SIN ABRIR LA VENTANA.
        //
        // ESTE FALLO SE CORRIGIO EL 2026-10-13, y lo cazo el dueno: `--texto` imprimia los valores
        // y DESPUES seguia hasta `run()`, asi que **abria una ventana en su pantalla** -- en una
        // comprobacion que solo queria leer unos numeros. Se quedo alli, molestando.
        //
        // Un modo que SOLO COMPRUEBA no puede abrir nada: si la comprobacion tiene efectos que se
        // ven, deja de poder hacerse en cualquier momento.
        return Ok(());
    }

    // El boton de tema: cambia UNA propiedad, y ninguna pieza se toca.
    let v = ventana.as_weak();
    ventana.on_elegir(move |i| {
        if let Some(w) = v.upgrade() {
            // La seccion 3 (Datos) alterna el tema mientras no haya mas pantallas.
            if i == 3 {
                w.set_oscuro(!w.get_oscuro());
            }
        }
    });

    ventana.run()
}
