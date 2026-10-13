//! El **salto**: cuánto sube una sesión respecto al mes anterior.
//!
//! DE DÓNDE SALE ESTO, y es lo que le da valor: del **estudio más grande que existe** sobre lesiones en
//! corredores —**más de 5.000 personas, 87 países, 18 meses**, publicado en el *British Journal of
//! Sports Medicine* y **patrocinado por Garmin**—. Sus conclusiones:
//!
//! - **La relación de carga aguda y crónica (ACWR) NO mostró relación clara con las lesiones.** Por eso
//!   **este motor no la usa como base**, aunque venga ya calculada en los relojes.
//! - **El riesgo crece de forma EXPONENCIAL** cuando una sesión se alarga respecto a **la más larga del
//!   último mes**. Las lesiones no se cocinan despacio: **muchas ocurren de golpe, en una sola sesión**.
//!
//! FUENTE: `docs/ESFUERZO-y-progresion.md`, con los enlaces.
//!
//! OJO, Y ESTO ES IMPORTANTE: el estudio mide **DISTANCIA** en corredores. Aquí se ofrece también la
//! **carga** (TRIMP), porque el cliente hace **ciclismo y fuerza**, donde la distancia no siempre
//! existe. **Cada medida dice de dónde sale**, y la de carga se declara como **extensión**, no como lo
//! que el estudio midió.

use crate::datos::{Dato, Persona, Sesion};
use crate::formulas::{self, Resultado};

/// El salto de una sesión respecto a lo más largo del mes anterior.
#[derive(Debug, Clone, PartialEq)]
pub struct Salto {
    /// Cuánto dura (o cuánta carga tiene) la sesión de hoy.
    pub hoy: f32,
    /// Lo más alto de los 30 días anteriores, en la misma unidad.
    pub referencia: f32,
    /// La proporción: 1,0 es «igual que lo más largo del mes».
    pub proporcion: f32,
    /// Qué se mide: duración, carga, etc. **Se declara, no se supone.**
    pub unidad: String,
}

impl Salto {
    /// El porcentaje de subida. 0,5 es «un 50 % más».
    pub fn subida(&self) -> f32 {
        self.proporcion - 1.0
    }
}

/// El nivel de riesgo del salto, **con los nombres de lo que significa**, no con números sueltos.
///
/// Los cortes están **elegidos a mano y a la vista**, porque el estudio da **una curva, no un número**.
/// Y por eso el informe dice también **que el corte es una decisión**, no un hallazgo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NivelSalto {
    /// Por debajo de lo que ya hacía: se mantiene o se recupera.
    Mantiene,
    /// Sube poco: es la **sobrecarga progresiva**, que es como se mejora.
    Progresion,
    /// Sube bastante: se puede, pero **con aviso**.
    Aviso,
    /// Sube mucho: **aquí es donde se lesiona la gente**.
    Peligro,
}

impl NivelSalto {
    pub fn nombre(self) -> &'static str {
        match self {
            NivelSalto::Mantiene => "mantiene",
            NivelSalto::Progresion => "progresión",
            NivelSalto::Aviso => "aviso",
            NivelSalto::Peligro => "peligro",
        }
    }

    /// Qué significa, en una frase que se le puede decir a una persona.
    pub fn explicacion(self) -> &'static str {
        match self {
            NivelSalto::Mantiene => {
                "está en lo que ya hacía: mantiene, y eso es lo que toca para recuperar o sostener"
            }
            NivelSalto::Progresion => {
                "sube poco respecto a su mes: es la sobrecarga progresiva, que es como se mejora"
            }
            NivelSalto::Aviso => {
                "sube bastante de golpe. Se puede hacer, pero conviene saberlo: el riesgo ya no es despreciable"
            }
            NivelSalto::Peligro => {
                "sube mucho respecto a su mes. El estudio del que sale esto encontró que el riesgo de lesión crece de forma exponencial en este tramo"
            }
        }
    }
}

/// Los cortes, en un solo sitio y a la vista. **Son una decisión, no un hallazgo del estudio.**
const CORTE_PROGRESION: f32 = 1.10; // hasta un 10 % más
const CORTE_AVISO: f32 = 1.30; // hasta un 30 % más
const CORTE_PELIGRO: f32 = 1.50; // a partir de aquí, peligro

/// Calcula el salto de la sesión frente a las anteriores.
///
/// `anteriores` son las sesiones de **los 30 días previos**, sin incluir la de hoy.
pub fn salto(
    hoy: &Sesion,
    anteriores: &[Sesion],
    persona: &Persona,
    por_carga: bool,
) -> Resultado<(Salto, NivelSalto)> {
    if anteriores.is_empty() {
        return Resultado::no_se_puede(
            "No hay sesiones anteriores con las que comparar",
            ["al menos un mes de sesiones anteriores"],
        );
    }

    // La unidad: se declara cuál se usa, y qué respalda cada una.
    let (valor_hoy, valor_ref, unidad, fuente, reserva) = if por_carga {
        let carga = |s: &Sesion| formulas::trimp(s, persona).valor;
        let hoy_c = match carga(hoy) {
            Some(v) => v,
            None => {
                return Resultado::no_se_puede(
                    "No se puede calcular la carga de la sesión de hoy, así que no hay salto que medir",
                    ["pulsaciones y duración en la sesión de hoy"],
                )
            }
        };
        // De las anteriores, solo valen las que tienen carga calculable. Se dice cuántas se cayeron.
        let mut refs: Vec<f32> = Vec::new();
        let mut caidas = 0;
        for s in anteriores {
            match carga(s) {
                Some(v) => refs.push(v),
                None => caidas += 1,
            }
        }
        if refs.is_empty() {
            return Resultado::no_se_puede(
                "Ninguna sesión anterior tiene carga calculable",
                ["pulsaciones y duración en las sesiones anteriores"],
            );
        }
        let max = refs.iter().cloned().fold(f32::MIN, f32::max);
        let reserva = if caidas > 0 {
            Some(format!(
                "Se han quedado fuera {caidas} sesiones anteriores por no tener carga calculable: la referencia real puede ser más alta."
            ))
        } else {
            None
        };
        (
            hoy_c,
            max,
            "carga (TRIMP)".to_string(),
            "Comparación de la carga de la sesión con la más alta de los 30 días anteriores",
            reserva,
        )
    } else {
        let dur = |s: &Sesion| match s.duracion() {
            Dato::Hay(v) => Some(v as f32),
            Dato::Falta => None,
        };
        let hoy_d = match dur(hoy) {
            Some(v) => v,
            None => {
                return Resultado::no_se_puede(
                    "No se puede saber cuánto duró la sesión de hoy",
                    ["la duración de la sesión de hoy"],
                )
            }
        };
        let mut refs: Vec<f32> = anteriores.iter().filter_map(dur).collect();
        if refs.is_empty() {
            return Resultado::no_se_puede(
                "Ninguna sesión anterior tiene duración",
                ["la duración de las sesiones anteriores"],
            );
        }
        refs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let max = *refs.last().unwrap();
        (
            hoy_d,
            max,
            "duración (segundos)".to_string(),
            "Comparación de la duración de la sesión con la más larga de los 30 días anteriores",
            None,
        )
    };

    if valor_ref <= 0.0 {
        return Resultado::no_se_puede(
            "La referencia del mes anterior es cero: no se puede comparar",
            ["sesiones anteriores con valor mayor que cero"],
        );
    }

    let proporcion = valor_hoy / valor_ref;
    let nivel = if proporcion <= 1.0 {
        NivelSalto::Mantiene
    } else if proporcion < CORTE_PROGRESION {
        NivelSalto::Progresion
    } else if proporcion < CORTE_AVISO {
        NivelSalto::Progresion
    } else if proporcion < CORTE_PELIGRO {
        NivelSalto::Aviso
    } else {
        NivelSalto::Peligro
    };

    let s = Salto { hoy: valor_hoy, referencia: valor_ref, proporcion, unidad };

    let aviso_cortes = "Los cortes (10 %, 30 %, 50 %) son una DECISIÓN de este motor, elegida a la vista: el estudio da una curva de riesgo, no un umbral.";

    match reserva {
        Some(r) => Resultado::con_reserva(
            (s, nivel),
            fuente,
            &format!("{aviso_cortes} {r}"),
            ["sesiones anteriores completas"],
        ),
        None => Resultado::con_reserva((s, nivel), fuente, aviso_cortes, ["nada: la comparación es completa"]),
    }
}

/// Aviso sobre la **frecuencia cardíaca máxima estimada por edad**, que es un problema real.
///
/// POR QUÉ EXISTE: el motor estima la máxima con la ecuación de **Tanaka** (`208 − 0,7 × edad`) cuando
/// no hay una medición. Y esa estimación tiene un **error individual grande**: hay estudios sobre
/// poblaciones amplias y sanas —el **HUNT Fitness Study**— y revisiones sobre la exactitud de estas
/// fórmulas que lo documentan.
///
/// **El caso que lo hace importante**: un **deportista de 60 años** suele tener una máxima **más alta**
/// que la media de su edad. Si el motor estima 166 y su máxima real es 178, **todas sus zonas salen
/// bajas** y le estaría diciendo que entrena más suave de lo que entrena. **Un error de 12 pulsaciones
/// es prescribir la intensidad equivocada.**
///
/// Y a partir de cierta edad hay una razón añadida, que **no es de rendimiento sino de seguridad**: una
/// prueba de esfuerzo es la forma de saber que se puede exigir el corazón con tranquilidad.
pub fn aviso_de_la_estimacion(persona: &Persona) -> Option<String> {
    // Si está medida, no hay nada que avisar: se sabe.
    if persona.pulso_maximo_medido.valor().is_some() {
        return None;
    }
    if persona.edad.valor().is_none() {
        return None;
    }

    let edad = persona.edad.valor().unwrap_or(0);
    let mut aviso = String::from(
        "La frecuencia cardíaca máxima está ESTIMADA por la edad, no medida. \
         La estimación tiene un error individual grande: dos personas de la misma edad pueden tener \
         máximas muy distintas, y la fórmula no puede saber cuál es cuál.",
    );

    if edad >= 50 {
        aviso.push_str(
            " Y a esta edad importa más: una persona entrenada suele tener la máxima MÁS ALTA que la \
             media de su edad, así que las zonas calculadas pueden quedarse CORTAS y hacerle entrenar \
             más suave de lo que cree. Una prueba de esfuerzo da el número real, y además es la forma \
             de saber que se puede exigir el corazón con tranquilidad.",
        );
    }

    Some(aviso)
}
