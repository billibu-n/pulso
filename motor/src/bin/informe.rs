//! **La cadena completa: de sus ficheros FIT al informe, en el formato que se pida.**
//!
//! USO, y las dos formas siguen valiendo (la de siempre, y la de la entrega):
//!
//! ```text
//! cargo run --bin informe -- ../datos/garmin_files/storage [salida.md]
//! cargo run --bin informe -- informe --formato html --salida ../informe/informe.html --fecha 2026-10-17
//! cargo run --bin informe -- export --tipo=actividades --salida ../datos/export.csv
//! cargo run --bin informe -- export --tipo=potencia  --salida ../datos/export-potencia.csv
//! cargo run --bin informe -- export --tipo=zonas     --salida ../datos/export-zonas.csv
//! ```
//!
//! Esto es lo que el cliente recibe. Y fíjate en el recorrido: **no hay nada escrito a mano** —
//! los ficheros entran, el motor calcula, y el informe sale. El HTML y el CSV **no calculan nada**:
//! convierten lo que el motor ya decidió (ver `informe_html` y `export`).
//!
//! SI `--salida` NO SE DA, el artefacto va a la SALIDA ESTANDAR y sin adornos, para poder hacer
//! `> fichero.html` sin que se cuele el banner de diagnostico.

use motor::datos::{Dato, Persona};
use motor::{export, fit, informe, informe_html, tendencia};
use std::path::Path;

/// Lo que pide la linea de ordenes.
struct Orden {
    que: Que,
    carpeta: String,
    salida: Option<String>,
    formato: String,
    tipo: String,
    fecha: String,
}

#[derive(PartialEq)]
enum Que {
    Informe,
    Export,
}

fn leer_orden() -> Result<Orden, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut o = Orden {
        que: Que::Informe,
        carpeta: "../datos/garmin_files/storage".into(),
        salida: None,
        formato: "md".into(),
        tipo: "actividades".into(),
        fecha: String::new(),
    };
    let mut posicionales: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        // `--clave=valor` y `--clave valor` se aceptan las dos: escribir `=` de menos no debe
        // convertir una orden correcta en un informe en Markdown que nadie entiende.
        let (clave, inline) = match a.split_once('=') {
            Some((k, v)) if k.starts_with("--") => (k, Some(v.to_string())),
            _ => (a, None),
        };
        match clave {
            "export" => o.que = Que::Export,
            "informe" => o.que = Que::Informe,
            "--formato" => o.formato = valor_de(&args, &mut i, inline.as_deref(), clave)?,
            "--tipo" => o.tipo = valor_de(&args, &mut i, inline.as_deref(), clave)?,
            "--carpeta" => o.carpeta = valor_de(&args, &mut i, inline.as_deref(), clave)?,
            "--salida" => o.salida = Some(valor_de(&args, &mut i, inline.as_deref(), clave)?),
            "--fecha" => o.fecha = valor_de(&args, &mut i, inline.as_deref(), clave)?,
            x if x.starts_with("--") => return Err(format!("opcion desconocida: {x}")),
            x => posicionales.push(x.to_string()),
        }
        i += 1;
    }
    // Compatibilidad con la forma de siempre: `informe CARPETA [SALIDA]`.
    if let Some(p) = posicionales.first() {
        o.carpeta = p.clone();
    }
    if let Some(s) = posicionales.get(1) {
        o.salida = Some(s.clone());
    }
    Ok(o)
}

/// El valor de una opcion: `--clave=valor` o `--clave valor`.
fn valor_de(args: &[String], i: &mut usize, inline: Option<&str>, clave: &str) -> Result<String, String> {
    if let Some(v) = inline {
        return Ok(v.to_string());
    }
    *i += 1;
    args.get(*i).cloned().ok_or_else(|| format!("falta el valor de {clave}"))
}

fn main() {
    let o = match leer_orden() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    let (ficheros, malos) = fit::leer_carpeta(Path::new(&o.carpeta));
    if ficheros.is_empty() {
        eprintln!("No se leyó ningún FIT de {}", o.carpeta);
        std::process::exit(1);
    }

    // ---- La persona: lo que el FIT puede dar, más lo que hay que aportar ----
    //
    // EL FIT NO TRAE la edad, el peso ni el umbral: eso no lo mide el aparato. Aquí se ponen los datos
    // que el usuario puede aportar, y **si no están, se dejan vacíos** — el motor calcula igual.
    let mut p = Persona::desconocida();
    p.edad = Dato::Hay(60);
    p.pulso_reposo = Dato::Hay(48);
    p.peso_kg = Dato::Hay(70.2);
    p.altura_cm = Dato::Hay(178.0);
    p.grasa_corporal_pct = Dato::Hay(19.1);
    p.umbral_lactato = Dato::Hay(169);

    // Y la frecuencia máxima: **se busca en TODAS las sesiones**, que es donde está.
    let maxima_real = ficheros
        .iter()
        .filter_map(|(_, l)| l.declarado.pulso_maximo.valor())
        .max();
    if let Some(m) = maxima_real {
        p.pulso_maximo_medido = Dato::Hay(m);
    }

    // ---- La última sesión CON DATOS SUFICIENTES, que es sobre la que se informa ----
    //
    // Se corrigió el 2026-10-13: la primera versión cogía la ÚLTIMA sin más, y si esa no traía pulso
    // y potencia juntos, el informe decía «la deriva no se pudo medir» — **cuando en las otras 101 sí
    // se podía**. El orden de los ficheros no dice nada sobre cuál sirve.
    let ultima = ficheros
        .iter()
        .rev()
        .find(|(_, l)| {
            l.sesion.pulsos().len() >= 100
                && l.sesion.muestras.iter().filter(|m| m.potencia_w.valor().is_some()).count() >= 100
        })
        .map(|(_, l)| &l.sesion)
        .or_else(|| ficheros.last().map(|(_, l)| &l.sesion));

    // ---- LA EXPORTACION: CSV de actividades, potencia o zonas ----
    if o.que == Que::Export {
        let csv = match o.tipo.as_str() {
            "actividades" => export::actividades_csv(&ficheros),
            "potencia" => match ultima {
                Some(s) => export::potencia_csv(s),
                None => {
                    eprintln!("No hay sesion de la que exportar potencia");
                    std::process::exit(1);
                }
            },
            "zonas" => export::zonas_csv(&motor::umbral::zonas_del_umbral(&p)),
            otro => {
                eprintln!("tipo desconocido: {otro} (actividades | potencia | zonas)");
                std::process::exit(2);
            }
        };
        entregar(&csv, o.salida.as_deref());
        return;
    }

    // ---- La serie del consumo de oxígeno, para la tendencia ----
    // El FIT no la trae: la estima Garmin y se guarda aparte. Si no está, no hay tendencia.
    let serie_vo2: Option<tendencia::Serie> = None;

    let h = informe::hechos(&p, ultima, serie_vo2.as_ref());
    let texto = informe::generar(&h);

    // ---- Y la comprobacion que evita el fallo silencioso ----
    let legitimos: Vec<f64> = {
        let mut v = vec![
            h.umbral.rango.valor, h.umbral.rango.margen, h.maxima.valor, h.maxima.margen,
        ];
        for (_, desde, hasta) in &h.zonas_ciclismo {
            v.push(desde.valor);
            v.push(desde.margen);
            v.push(hasta.valor);
        }
        if let Some((p, m, _, n)) = &h.deriva {
            v.push(*p);
            v.push(*m);
            v.push(*n as f64);
        }
        if let Some(e) = &h.eficiencia {
            v.push(e.valor);
            v.push(e.margen);
        }
        if let Some(vv) = &h.vo2 {
            v.push(vv.valor);
            v.push(vv.margen);
        }
        v
    };
    let problemas = informe::comprobar_que_no_inventa(&texto, &legitimos);

    let artefacto = match o.formato.as_str() {
        "html" => {
            let titulo = "Análisis de entrenamiento";
            informe_html::a_html(&texto, titulo, &o.fecha)
        }
        "md" | "markdown" => texto.clone(),
        otro => {
            eprintln!("formato desconocido: {otro} (md | html)");
            std::process::exit(2);
        }
    };

    match &o.salida {
        Some(ruta) => {
            // El diagnostico va por stderr para no ensuciar el artefacto, y SOLO cuando se escribe a
            // fichero: en la salida estandar el artefacto tiene que ir limpio.
            informe_de_consola(&o, ficheros.len(), malos.len(), maxima_real, h, &problemas);
            entregar(&artefacto, Some(ruta));
        }
        None => {
            // A la salida estandar, LIMPIO: `> informe.html` no puede llevar banner.
            print!("{artefacto}");
        }
    }
}

fn entregar(contenido: &str, ruta: Option<&str>) {
    match ruta {
        Some(r) => {
            if let Some(dir) = Path::new(r).parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Err(e) = std::fs::write(r, contenido) {
                eprintln!("No se pudo escribir {r}: {e}");
                std::process::exit(1);
            }
            println!("  escrito: {r} ({} bytes)", contenido.len());
        }
        None => print!("{contenido}"),
    }
}

fn informe_de_consola(
    o: &Orden,
    ficheros: usize,
    malos: usize,
    maxima_real: Option<u16>,
    h: informe::Hechos,
    problemas: &[String],
) {
    eprintln!("DE SUS FICHEROS AL INFORME, sin nada escrito a mano");
    eprintln!("  formato:            {}", o.formato);
    eprintln!("  ficheros leidos:    {ficheros}");
    eprintln!("  ficheros con error: {malos}");
    eprintln!("  maxima real en TODAS las sesiones: {}", maxima_real.unwrap_or(0));
    eprintln!("  datos de la persona: {} de {}", h.persona_datos.0, h.persona_datos.1);
    eprintln!();
    eprintln!("  ¿EL INFORME INVENTA ALGUN NUMERO?");
    if problemas.is_empty() {
        eprintln!("    No: todos los numeros del texto salieron de un calculo del motor.");
    } else {
        for x in problemas.iter().take(5) {
            eprintln!("    aviso: {x}");
        }
    }
}
