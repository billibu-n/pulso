//! **El umbral de lactato**: el dato que manda sobre la fórmula.
//!
//! POR QUÉ ESTE MÓDULO ES IMPORTANTE, y está respaldado por la literatura reciente:
//!
//! En ciclismo **las zonas no se fijan sobre la frecuencia máxima, sino sobre el umbral de lactato** —
//! el punto en el que el esfuerzo empieza a acumularse más rápido de lo que el cuerpo elimina. Es un
//! dato **medido**, y por tanto mucho más fiable que cualquier estimación por edad.
//!
//! Y hay un trabajo de 2025 que **valida siete ecuaciones** de frecuencia máxima contra **634 personas
//! de 18 a 85 años**, y encuentra que **ninguna sirve por encima de los 40**. Otro, con **50 ciclistas**,
//! encuentra que los **porcentajes fijos de la frecuencia máxima** reflejan mal la fisiología de cada
//! persona, con variaciones del **6 % al 29 %**.
//!
//! CONCLUSIÓN QUE ORDENA ESTE MÓDULO: **si hay umbral, el umbral manda.** La fórmula por edad baja a
//! **último recurso**. Y como el rediseño exige, **nunca se deja de responder**: si no hay umbral, se
//! estima, y el margen lo dice.

use crate::datos::{Dato, Persona};
use crate::rango::{self, Rango};

/// De dónde sale la referencia con la que se fijan las zonas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Referencia {
    /// **El umbral de lactato, medido.** Es el mejor caso, y el que usa el ciclismo de verdad.
    UmbralMedido,
    /// Estimado como una fracción de la frecuencia máxima. Es una aproximación, y se dice.
    EstimadoDeLaMaxima,
    /// Sin umbral ni máxima medida: se parte de una persona media.
    DePoblacion,
}

impl Referencia {
    pub fn nombre(self) -> &'static str {
        match self {
            Referencia::UmbralMedido => "umbral de lactato medido",
            Referencia::EstimadoDeLaMaxima => "estimado a partir de la frecuencia máxima",
            Referencia::DePoblacion => "partiendo de una persona media",
        }
    }

    /// Qué gana el usuario si aporta el dato que falta.
    pub fn que_falta(self) -> Option<&'static str> {
        match self {
            Referencia::UmbralMedido => None,
            Referencia::EstimadoDeLaMaxima => Some(
                "su umbral de lactato, de una prueba: es el dato con el que se fijan las zonas en ciclismo, y es medido",
            ),
            Referencia::DePoblacion => {
                Some("una prueba de esfuerzo, o al menos su edad y su composición corporal")
            }
        }
    }
}

/// La **referencia para las zonas**, siempre con su valor, su margen y su origen.
#[derive(Debug, Clone, PartialEq)]
pub struct Umbral {
    pub rango: Rango,
    pub referencia: Referencia,
}

impl Umbral {
    pub fn valor(&self) -> f64 {
        self.rango.valor
    }
}

/// El margen que se le supone a un **umbral medido en una prueba**.
///
/// Es pequeño comparado con una estimación, y **no cero**: una prueba tiene su propio error. El valor
/// sale de la reproducibilidad típica de las pruebas de lactato, que es de unos pocos vatios o unas
/// pocas pulsaciones de una vez a otra.
pub const MARGEN_UMBRAL_MEDIDO: f64 = 3.0;

/// La fracción de la frecuencia máxima a la que suele estar el umbral, **cuando no se ha medido**.
///
/// ES UNA ESTIMACIÓN, y va escrita como tal: en personas entrenadas el umbral suele quedar entre el
/// **80 % y el 90 %** de la máxima. Se toma el centro, y el margen cubre los extremos.
pub const FRACCION_UMBRAL_DE_LA_MAXIMA: f64 = 0.85;

/// **El umbral de lactato**, con la mejor información disponible.
///
/// EL ORDEN DE PREFERENCIA, y por qué ese:
/// 1. **medido** — es un hecho, y es como se fijan las zonas en ciclismo;
/// 2. **estimado de la máxima** — sirve, pero se declara que es una aproximación;
/// 3. **de población** — el punto de partida, con el margen más ancho.
pub fn umbral(p: &Persona) -> Umbral {
    // 1 · ¿Esta medido? Entonces manda, y no se estima nada.
    if let Some(u) = p.umbral_lactato.valor() {
        return Umbral {
            rango: Rango::medido(f64::from(u), MARGEN_UMBRAL_MEDIDO),
            referencia: Referencia::UmbralMedido,
        };
    }

    // 2 · ¿Hay frecuencia máxima? Se estima el umbral como una fracción suya, y se dice.
    let maxima = crate::formulas_v2::pulso_maximo(p);
    if maxima.origen != crate::formulas_v2::OrigenMaxima::DePoblacion {
        let valor = maxima.rango.valor * FRACCION_UMBRAL_DE_LA_MAXIMA;
        // El margen: el de la máxima, más el de la propia fracción (que varía del 80 al 90 %).
        let margen = rango::suma_de_margenes(&[
            maxima.rango.margen * FRACCION_UMBRAL_DE_LA_MAXIMA,
            maxima.rango.valor * 0.05, // la fracción varía un 5 % arriba o abajo
        ]);
        return Umbral {
            rango: Rango::estimado(valor, margen),
            referencia: Referencia::EstimadoDeLaMaxima,
        };
    }

    // 3 · Sin nada: se parte de una persona media.
    let valor = maxima.rango.valor * FRACCION_UMBRAL_DE_LA_MAXIMA;
    Umbral {
        rango: Rango::estimado(valor, maxima.rango.margen * FRACCION_UMBRAL_DE_LA_MAXIMA + 8.0),
        referencia: Referencia::DePoblacion,
    }
}

/// Las **zonas de ciclismo**, fijadas sobre el umbral — que es como se hace de verdad.
///
/// LOS CORTES, y de dónde salen: son los **habituales en ciclismo**, expresados como porcentaje del
/// umbral. Y son **una convención de prescripción**, no una ley física: por eso van escritos y a la
/// vista, con esta nota.
///
/// | zona | % del umbral | qué es |
/// |---|---:|---|
/// | **Z1 recuperación** | hasta 81 % | rodar suave, recuperar |
/// | **Z2 resistencia** | 81 – 90 % | el fondo, donde se construye la base |
/// | **Z3 tempo** | 90 – 94 % | trabajo sostenido |
/// | **Z4 umbral** | 94 – 100 % | justo en el umbral |
/// | **Z5 consumo máximo** | 100 – 120 % | por encima |
pub fn zonas_del_umbral(p: &Persona) -> Vec<(ZonaCiclismo, Rango, Rango)> {
    let u = umbral(p);
    ZonaCiclismo::todas()
        .iter()
        .map(|z| {
            // UNA ZONA ES UN TRAMO, NO UN LÍMITE. Esto se corrigió el 2026-10-13: la primera versión
            // devolvía solo el suelo, y entonces «recuperación» y «resistencia» salían con **el mismo
            // número** en el informe, porque las dos empiezan donde acaba la anterior. Un informe que
            // dice dos veces 137 **no se entiende**, y el fallo era de lectura, no de cálculo.
            let desde = u.rango.valor * z.fraccion();
            let hasta = u.rango.valor * z.techo();
            let m_desde = u.rango.margen * z.fraccion();
            let m_hasta = u.rango.margen * z.techo();
            (*z, Rango::estimado(desde, m_desde), Rango::estimado(hasta, m_hasta))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZonaCiclismo {
    Recuperacion,
    Resistencia,
    Tempo,
    Umbral,
    ConsumoMaximo,
}

impl ZonaCiclismo {
    pub fn todas() -> [ZonaCiclismo; 5] {
        [
            ZonaCiclismo::Recuperacion,
            ZonaCiclismo::Resistencia,
            ZonaCiclismo::Tempo,
            ZonaCiclismo::Umbral,
            ZonaCiclismo::ConsumoMaximo,
        ]
    }

    /// El suelo de cada zona, como fracción del umbral.
    pub fn fraccion(self) -> f64 {
        match self {
            ZonaCiclismo::Recuperacion => 0.81,
            ZonaCiclismo::Resistencia => 0.81,
            ZonaCiclismo::Tempo => 0.90,
            ZonaCiclismo::Umbral => 0.94,
            ZonaCiclismo::ConsumoMaximo => 1.00,
        }
    }

    /// **El techo de la zona**, como fracción del umbral. Sin esto, dos zonas seguidas darían el
    /// mismo número, porque una empieza donde acaba la otra.
    pub fn techo(self) -> f64 {
        match self {
            ZonaCiclismo::Recuperacion => 0.81,
            ZonaCiclismo::Resistencia => 0.90,
            ZonaCiclismo::Tempo => 0.94,
            ZonaCiclismo::Umbral => 1.00,
            ZonaCiclismo::ConsumoMaximo => 1.20,
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            ZonaCiclismo::Recuperacion => "recuperación",
            ZonaCiclismo::Resistencia => "resistencia",
            ZonaCiclismo::Tempo => "tempo",
            ZonaCiclismo::Umbral => "umbral",
            ZonaCiclismo::ConsumoMaximo => "consumo máximo",
        }
    }

    pub fn explicacion(self) -> &'static str {
        match self {
            ZonaCiclismo::Recuperacion => "rodar suave: sirve para recuperar, no para mejorar",
            ZonaCiclismo::Resistencia => "el fondo: aquí se construye la base aeróbica",
            ZonaCiclismo::Tempo => "trabajo sostenido: cuesta, pero se aguanta un rato largo",
            ZonaCiclismo::Umbral => "justo en el umbral: es la zona que más mejora la resistencia",
            ZonaCiclismo::ConsumoMaximo => "por encima del umbral: se aguanta poco, y cansa mucho",
        }
    }
}

/// El **consumo máximo de oxígeno**, con su margen, si la persona lo tiene.
///
/// El FIT **no lo trae** — lo estima Garmin y lo guarda aparte. Y **no se inventa**: si no está, se dice
/// que falta en vez de dar un número de población, porque para este dato **no hay una referencia útil
/// que dar**: depende demasiado de la persona.
pub fn vo2_maximo(p: &Persona) -> Dato<Rango> {
    match p.vo2_maximo.valor() {
        Some(v) => {
            // El margen: las estimaciones por reloj tienen un error de unos 3-5 ml/kg/min frente a
            // una prueba de laboratorio. Se usa 4, que es el centro de ese rango.
            Dato::Hay(Rango::estimado(f64::from(v), 4.0))
        }
        None => Dato::Falta,
    }
}

/// **Cuánta información tiene la persona**, para poder decirlo de un vistazo.
///
/// Cuenta los datos que **de verdad cambian el resultado**, y no todos: tener el nombre no mejora
/// ninguna estimación.
pub fn cuantos_datos_utiles(p: &Persona) -> (usize, usize) {
    let utiles = [
        p.edad.lo_hay(),
        p.pulso_reposo.lo_hay(),
        p.pulso_maximo_medido.lo_hay(),
        p.peso_kg.lo_hay(),
        p.altura_cm.lo_hay(),
        p.grasa_corporal_pct.lo_hay(),
        p.masa_muscular_kg.lo_hay(),
        p.umbral_lactato.lo_hay(),
        p.vo2_maximo.lo_hay(),
    ];
    (utiles.iter().filter(|x| **x).count(), utiles.len())
}
