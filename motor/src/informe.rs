//! **El informe**: los hechos del motor, colocados para que se lean.
//!
//! QUÉ ES ESTE MÓDULO, y qué NO es:
//!
//! - **NO calcula.** Recibe los números **ya calculados** por el motor y los ordena. Si aquí hubiera una
//!   sola cuenta, el día que se cambie una fórmula **el informe mentiría sin avisar**.
//! - **NO redacta con un modelo.** Genera el texto **con plantillas**, a partir de los hechos. Eso tiene
//!   dos consecuencias que importan: **es reproducible** —los mismos datos dan el mismo informe— y **es
//!   auditable**: cada afirmación se puede rastrear hasta el cálculo que la produjo.
//!
//! POR QUÉ ASÍ Y NO CON UNA IA, y lo decidió el dueño: *«al final lo que le interesa es una plantilla e
//! información bien distribuida»*. Una plantilla **no improvisa**, y en algo que afecta al cuerpo de una
//! persona, **que no improvise es una ventaja**.
//!
//! LA ESTRUCTURA DEL INFORME, y por qué ese orden: **primero lo que se puede afirmar, después el
//! detalle, y al final lo que falta**. Un informe que empieza por los avisos metodológicos no se lee.

use crate::datos::Persona;
use crate::formulas_v2;
use crate::patron::NivelDeriva;
use crate::rango::Rango;
use crate::umbral;

/// Todo lo que el informe necesita, **ya calculado**.
///
/// Se pasa como una estructura y no se calcula dentro, a propósito: **así el informe se puede probar sin
/// datos**, y el cálculo se puede cambiar sin tocar el texto.
#[derive(Debug, Clone)]
pub struct Hechos {
    pub persona_datos: (usize, usize),
    pub umbral: umbral::Umbral,
    pub zonas_ciclismo: Vec<(umbral::ZonaCiclismo, Rango, Rango)>,
    pub zonas_fc: Vec<(formulas_v2::Zona, Rango)>,
    pub maxima: Rango,
    pub origen_maxima: String,
    pub vo2: Option<Rango>,
    /// La deriva y la eficiencia de la última sesión, si se midieron.
    pub deriva: Option<(f64, f64, NivelDeriva, usize)>,
    pub eficiencia: Option<Rango>,
    /// Y la tendencia del consumo de oxígeno, si hay histórico.
    pub tendencia_vo2: Option<String>,
    /// Lo que falta, en palabras: **es lo último del informe, y no lo primero**.
    pub falta: Vec<String>,
}

/// Construye los hechos **a partir del motor**, sin calcular nada nuevo.
pub fn hechos(
    p: &Persona,
    ultima: Option<&crate::datos::Sesion>,
    vo2_serie: Option<&crate::tendencia::Serie>,
) -> Hechos {
    let maxima = formulas_v2::pulso_maximo(p);
    let u = umbral::umbral(p);

    let mut falta: Vec<String> = Vec::new();
    if let Some(f) = u.referencia.que_falta() {
        falta.push(f.to_string());
    }
    for g in &maxima.ganaria {
        if !falta.contains(g) {
            falta.push(g.clone());
        }
    }

    let (deriva, eficiencia) = match ultima {
        Some(s) => {
            let d = crate::patron::deriva(s);
            let e = crate::patron::eficiencia(s);
            let der = if d.muestras >= 20 {
                Some((d.porcentaje(), d.rango.margen, d.nivel(), d.muestras))
            } else {
                None
            };
            let ef = if e.valor > 0.0 { Some(e) } else { None };
            (der, ef)
        }
        None => (None, None),
    };

    let tendencia_vo2 = vo2_serie.and_then(|s| {
        s.tendencia()
            .prestado()
            .map(|t| t.frase(crate::tendencia::tolerancia::VO2))
    });

    Hechos {
        persona_datos: umbral::cuantos_datos_utiles(p),
        umbral: u,
        zonas_ciclismo: umbral::zonas_del_umbral(p),
        zonas_fc: formulas_v2::zonas(p),
        maxima: maxima.rango,
        origen_maxima: maxima.origen.nombre().to_string(),
        vo2: umbral::vo2_maximo(p).prestado().cloned(),
        deriva,
        eficiencia,
        tendencia_vo2,
        falta,
    }
}

/// Genera el informe en texto, con tablas. **No calcula: coloca lo que le dan.**
pub fn generar(h: &Hechos) -> String {
    let mut o = String::new();

    o.push_str("# Análisis de entrenamiento\n\n");
    o.push_str("Informe generado a partir de los datos de entrenamiento. ");
    o.push_str("Cada número sale de una fórmula con su margen, y el margen está escrito.\n\n");

    // ---- 1 · LO QUE SE PUEDE AFIRMAR ----
    o.push_str("## Lo que se puede afirmar\n\n");
    let precision = h.umbral.rango.como_va();
    o.push_str(&format!(
        "La referencia para las zonas es el **{}**, con un valor de **{:.0} ± {:.0} pulsaciones** \
         (precisión {}).\n\n",
        h.umbral.referencia.nombre(),
        h.umbral.rango.valor,
        h.umbral.rango.margen,
        precision.nombre()
    ));
    o.push_str(&format!("*{}*\n\n", precision.explicacion()));

    // ---- 2 · LAS ZONAS ----
    o.push_str("## Sus zonas de entrenamiento\n\n");
    o.push_str("| zona | pulsaciones | qué es |\n");
    o.push_str("|---|---:|---|\n");
    for (z, desde, hasta) in &h.zonas_ciclismo {
        // Un TRAMO, no un límite: es lo que hace que el informe se entienda.
        let tramo = if (hasta.valor - desde.valor).abs() < 1.0 {
            format!("hasta {:.0}", hasta.valor)
        } else {
            format!("{:.0} – {:.0}", desde.valor, hasta.valor)
        };
        o.push_str(&format!("| **{}** | {} | {} |\n", z.nombre(), tramo, z.explicacion()));
    }
    o.push_str("\n: Zonas de ciclismo, fijadas sobre el umbral. El margen del umbral es de ± ");
    o.push_str(&format!("{:.0} pulsaciones.\n\n", h.umbral.rango.margen));

    // ---- 3 · LA FRECUENCIA MÁXIMA ----
    o.push_str("## Su frecuencia cardíaca máxima\n\n");
    o.push_str(&format!(
        "**{:.0} ± {:.0} pulsaciones**, {}.\n\n",
        h.maxima.valor, h.maxima.margen, h.origen_maxima
    ));

    // ---- 4 · LOS PATRONES: LO QUE EL FICHERO NO TRAE ----
    o.push_str("## Lo que se deduce de sus datos\n\n");
    o.push_str("Estos números **no vienen en el fichero del aparato**: salen de comparar el pulso con la potencia.\n\n");

    match &h.deriva {
        Some((porcentaje, margen, nivel, muestras)) => {
            o.push_str(&format!(
                "**Deriva del pulso: {:+.1} ± {:.1} %** — {}.\n\n",
                porcentaje,
                margen,
                nivel.nombre()
            ));
            o.push_str(&format!("*{}*\n\n", nivel.explicacion()));
            o.push_str(&format!("Medido sobre {} muestras.\n\n", muestras));
        }
        None => o.push_str("La deriva del pulso: no se pudo medir en la última sesión.\n\n"),
    }

    if let Some(e) = &h.eficiencia {
        o.push_str(&format!(
            "**Eficiencia: {:.3} ± {:.3} vatios por pulsación.**\n\n",
            e.valor, e.margen
        ));
        o.push_str("Es la medida de la mejora: si sube con el tiempo, hace el mismo trabajo con menos pulso.\n\n");
    }

    if let Some(t) = &h.tendencia_vo2 {
        o.push_str(&format!("**Consumo máximo de oxígeno:** {t}\n\n"));
    }

    if let Some(v) = &h.vo2 {
        o.push_str(&format!(
            "**Consumo máximo de oxígeno: {:.0} ± {:.0} ml/kg/min.**\n\n",
            v.valor, v.margen
        ));
    }

    // ---- 5 · LO QUE FALTA, AL FINAL ----
    o.push_str("## Para que el análisis sea más preciso\n\n");
    if h.falta.is_empty() {
        o.push_str("No falta ningún dato: el análisis está en su mejor versión posible con lo que hay.\n\n");
    } else {
        o.push_str("No falta información para **dar un resultado** — el resultado está arriba. ");
        o.push_str("Falta para que sea **más preciso**:\n\n");
        for f in &h.falta {
            let mut s = f.to_string();
            if let Some(c) = s.get_mut(0..1) {
                c.make_ascii_uppercase();
            }
            o.push_str(&format!("- {s}\n"));
        }
        o.push('\n');
    }
    o.push_str(&format!(
        "Datos disponibles: **{} de {}**.\n\n",
        h.persona_datos.0, h.persona_datos.1
    ));

    o.push_str("---\n\n");
    o.push_str(
        "*Este informe describe el rendimiento y la carga de entrenamiento a partir de los datos \
         registrados. No es un diagnóstico médico: cualquier decisión sobre intensidad o salud \
         corresponde a un profesional sanitario.*\n",
    );

    o
}

/// Un resumen corto, para enseñarlo sin leerlo entero.
pub fn resumen(h: &Hechos) -> String {
    let mut o = String::new();
    o.push_str(&format!(
        "Zonas fijadas sobre el {}, con {:.0} ± {:.0} ppm ({}).\n",
        h.umbral.referencia.nombre(),
        h.umbral.rango.valor,
        h.umbral.rango.margen,
        h.umbral.rango.como_va().nombre()
    ));
    if let Some((p, m, n, _)) = &h.deriva {
        o.push_str(&format!("Deriva del pulso: {p:+.1} ± {m:.1} % — {}.\n", n.nombre()));
    }
    if let Some(e) = &h.eficiencia {
        o.push_str(&format!("Eficiencia: {:.3} W por pulsación.\n", e.valor));
    }
    if !h.falta.is_empty() {
        o.push_str(&format!("Faltan {} datos para afinarlo.\n", h.falta.len()));
    }
    o
}

/// ¿El informe afirma algo que el motor no puede sostener?
///
/// EXISTE PARA CUANDO ALGUIEN AÑADA TEXTO AL INFORME, y es la guarda que evita el fallo silencioso:
/// **si el texto lleva un número que no salió de un cálculo, esta comprobación lo dice.** Es la misma
/// idea que el motor revisando lo que escribiría una IA — pero aplicada al informe de plantilla.
pub fn comprobar_que_no_inventa(informe: &str, numeros_legitimos: &[f64]) -> Vec<String> {
    let mut problemas = Vec::new();
    for palabra in informe.split(|c: char| !c.is_ascii_digit() && c != '.' && c != ',') {
        let limpio = palabra.replace(',', ".");
        if let Ok(v) = limpio.parse::<f64>() {
            // Se ignoran los números pequeños: son los "1" de las listas y los índices.
            if v.abs() < 10.0 {
                continue;
            }
            let conocido = numeros_legitimos.iter().any(|n| (n - v).abs() <= 1.0);
            if !conocido {
                problemas.push(format!("el informe menciona {v}, y ningún cálculo lo produjo"));
            }
        }
    }
    problemas
}
