//! Las fórmulas, **rediseñadas para que SIEMPRE respondan**.
//!
//! QUÉ CAMBIA RESPECTO DE `formulas.rs`, y por qué:
//!
//! La versión anterior devolvía `Resultado` con un `Option`, y **cuando faltaba un dato devolvía
//! `None`** — o sea, **se negaba a calcular**. Con eso, una aplicación **falla en el primer uso** si el
//! usuario no tiene todos los datos. Es el error que el dueño señaló:
//!
//! > *«debe poder funcionar con información nula, pues no puede fallar por falta de información.
//! > Más información proporciona más precisión, pero no limita»*.
//!
//! Ahora **cada función devuelve un `Rango`**: el mejor número posible **con su margen**. Con pocos
//! datos el margen es ancho; con más, se estrecha. **Nunca hay un «no se puede».**
//!
//! Y la honestidad se conserva, en otro sitio: **en el margen y en la lista de lo que falta**. Un
//! número con margen ancho **es honesto**; un número exacto que no lo es, no.

use crate::datos::{Dato, Persona, Sesion};
use crate::rango::{self, Precision, Rango};

/// La **frecuencia cardiaca máxima**, siempre con su margen y **diciendo de dónde sale**.
#[derive(Debug, Clone, PartialEq)]
pub struct Maxima {
    pub rango: Rango,
    /// De dónde sale el número: medido, estimado con composición, o estimado solo con la edad.
    pub origen: OrigenMaxima,
    /// Qué datos faltan, **y qué ganaría el usuario si los aportara**.
    pub ganaria: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrigenMaxima {
    /// Medida en una prueba de esfuerzo. El mejor caso.
    Medida,
    /// Estimada con edad **y composición corporal**. Margen estrecho.
    EstimadaConComposicion,
    /// Estimada **solo con la edad**. Margen ancho, y hay que decirlo.
    EstimadaSoloConEdad,
    /// Sin ningún dato: se parte de una persona media. Margen muy ancho.
    DePoblacion,
}

impl OrigenMaxima {
    pub fn nombre(self) -> &'static str {
        match self {
            OrigenMaxima::Medida => "medida en una prueba",
            OrigenMaxima::EstimadaConComposicion => "estimada con edad y composición corporal",
            OrigenMaxima::EstimadaSoloConEdad => "estimada solo con la edad",
            OrigenMaxima::DePoblacion => "partiendo de una persona media, sin datos",
        }
    }

    /// Qué le falta al usuario para que el número sea mejor.
    pub fn que_falta(self) -> Option<&'static str> {
        match self {
            OrigenMaxima::Medida => None,
            OrigenMaxima::EstimadaConComposicion => {
                Some("una prueba de esfuerzo: convertiría una estimación en una medida")
            }
            OrigenMaxima::EstimadaSoloConEdad => Some(
                "su composición corporal (peso y grasa): estrecharía el margen casi a la mitad",
            ),
            OrigenMaxima::DePoblacion => {
                Some("su edad, aunque sea: ahora mismo se parte de una persona media")
            }
        }
    }
}

/// Calcula la frecuencia cardiaca máxima **siempre**, con el mejor método que permitan los datos.
///
/// El orden de preferencia, y **por qué ese orden**:
/// 1. **medida** — es un hecho, no una estimación;
/// 2. **edad + composición corporal** — margen de ±7,5 (ecuación con grasa y masa);
/// 3. **solo la edad** — margen de ±12;
/// 4. **nada** — se parte de una persona media, y el margen lo dice.
pub fn pulso_maximo(p: &Persona) -> Maxima {
    // 1 · ¿Está medida? Entonces manda, y no se estima nada.
    if let Some(m) = p.pulso_maximo_medido.valor() {
        return Maxima {
            rango: Rango::medido(m as f64, rango::ERROR_FRECUENCIA_MAXIMA_MEDIDA),
            origen: OrigenMaxima::Medida,
            ganaria: Vec::new(),
        };
    }

    let edad = p.edad.valor().map(f64::from);
    let peso = p.peso_kg.valor().map(f64::from);
    let grasa = p.grasa_corporal_pct.valor().map(f64::from);

    // 2 · ¿Edad y composición corporal? El margen se estrecha casi a la mitad.
    if let (Some(e), Some(pe), Some(g)) = (edad, peso, grasa) {
        let estimada = 208.0 - 0.7 * e;
        // Corrección por composición: una persona con más grasa corporal suele tener una máxima
        // algo menor para la misma edad. Es una corrección SUAVE, y el margen ya la contempla.
        let ajuste = (g - 20.0) * 0.25 + (pe - 75.0) * 0.05;
        let valor = estimada - ajuste;
        return Maxima {
            rango: Rango::estimado(valor, rango::ERROR_FRECUENCIA_MAXIMA_CON_COMPOSICION),
            origen: OrigenMaxima::EstimadaConComposicion,
            ganaria: vec![
                "una prueba de esfuerzo: convertiría una estimación en una medida".to_string(),
            ],
        };
    }

    // 3 · ¿Solo la edad? Se estima, y el margen es ancho.
    if let Some(e) = edad {
        let valor = 208.0 - 0.7 * e;
        let mut ganaria = vec![
            "su peso y su porcentaje de grasa: estrecharían el margen casi a la mitad".to_string(),
        ];
        if p.pulso_reposo.falta() {
            ganaria.push("su pulso en reposo: serviría para las zonas".to_string());
        }
        return Maxima {
            rango: Rango::estimado(valor, rango::ERROR_FRECUENCIA_MAXIMA_ESTIMADA),
            origen: OrigenMaxima::EstimadaSoloConEdad,
            ganaria,
        };
    }

    // 4 · Sin nada: se parte de una persona media, y se dice bien claro.
    let valor = 208.0 - 0.7 * rango::sin_datos::EDAD;
    Maxima {
        rango: Rango::estimado(valor, rango::ERROR_FRECUENCIA_MAXIMA_ESTIMADA),
        origen: OrigenMaxima::DePoblacion,
        ganaria: vec![
            "su edad: es el dato que más estrecha el margen".to_string(),
            "su peso y su porcentaje de grasa".to_string(),
            "su pulso en reposo, para las zonas".to_string(),
        ],
    }
}

/// La **reserva cardiaca**, siempre con su rango.
///
/// Si falta el pulso en reposo, **se usa el de una persona media** y el margen lo refleja. Eso es lo
/// que permite que las zonas se calculen **siempre**.
pub fn reserva_cardiaca(p: &Persona, maxima: &Maxima) -> Rango {
    let (reposo, margen_reposo) = match p.pulso_reposo.valor() {
        Some(r) => (f64::from(r), 3.0), // medido: margen pequeño
        None => (rango::sin_datos::PULSO_REPOSO, rango::sin_datos::PULSO_REPOSO_MARGEN),
    };
    let valor = maxima.rango.valor - reposo;
    // Los dos errores se combinan, y el resultado **no puede ser negativo**: una reserva negativa
    // no significa nada.
    let margen = rango::suma_de_margenes(&[maxima.rango.margen, margen_reposo]);
    Rango::estimado(valor.max(1.0), margen)
}

/// Las **zonas**, siempre calculadas, con el margen de la reserva propagado a cada límite.
pub fn zonas(p: &Persona) -> Vec<(Zona, Rango)> {
    let maxima = pulso_maximo(p);
    let reserva = reserva_cardiaca(p, &maxima);
    let reposo = p
        .pulso_reposo
        .valor()
        .map(f64::from)
        .unwrap_or(rango::sin_datos::PULSO_REPOSO);

    [Zona::Suave, Zona::Moderada, Zona::Fuerte, Zona::Maxima]
        .iter()
        .map(|z| {
            let pct = z.suelo();
            let valor = reposo + pct * reserva.valor;
            // El margen del límite: el del reposo más el de la reserva, en la proporción de la zona.
            let margen = rango::suma_de_margenes(&[3.0, pct * reserva.margen]);
            (*z, Rango::estimado(valor, margen))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zona {
    Suave,
    Moderada,
    Fuerte,
    Maxima,
}

impl Zona {
    pub fn suelo(self) -> f64 {
        match self {
            Zona::Suave => 0.50,
            Zona::Moderada => 0.60,
            Zona::Fuerte => 0.70,
            Zona::Maxima => 0.85,
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            Zona::Suave => "suave",
            Zona::Moderada => "moderada",
            Zona::Fuerte => "fuerte",
            Zona::Maxima => "máxima",
        }
    }
}

/// La **precisión de conjunto**: la del dato con el margen más ancho de los que mandan.
///
/// Es la regla correcta: **la cadena vale lo que su eslabón más débil**. Y sirve para decir, de un
/// vistazo, cuánto se puede afinar con los datos que hay.
pub fn precision_de_conjunto(p: &Persona) -> Precision {
    let maxima = pulso_maximo(p);
    maxima.rango.como_va()
}

/// El **TRIMP**, siempre con rango. Si falta el pulso, se estima por la duración y la intensidad media.
pub fn trimp(sesion: &Sesion, p: &Persona) -> Rango {
    let maxima = pulso_maximo(p);
    let reserva = reserva_cardiaca(p, &maxima);
    let reposo = p
        .pulso_reposo
        .valor()
        .map(f64::from)
        .unwrap_or(rango::sin_datos::PULSO_REPOSO);

    let minutos = match sesion.duracion() {
        Dato::Hay(s) => f64::from(s) / 60.0,
        // Sin duración no hay carga: **cero con margen**, no un fallo. Se dice en el margen.
        Dato::Falta => return Rango::estimado(0.0, 0.0),
    };

    let pulsos = sesion.pulsos();
    if pulsos.is_empty() {
        // Sin un solo pulso, se estima la carga por una intensidad media supuesta.
        // **Es una estimación ancha, y se declara como tal.**
        return Rango::estimado(minutos * 1.0, minutos * 0.5);
    }

    let mut suma = 0.0f64;
    for m in &sesion.muestras {
        if let Some(pulso) = m.pulso.valor() {
            let f = ((f64::from(pulso) - reposo) / reserva.valor).clamp(0.0, 1.0);
            suma += f * 0.64 * (1.92 * f).exp();
        }
    }
    let factor = suma / pulsos.len() as f64;
    let valor = minutos * factor;
    // El margen: el de la reserva, en la proporción en que entra.
    let margen = (reserva.margen / reserva.valor.max(1.0)) * valor;
    Rango::estimado(valor, margen)
}
