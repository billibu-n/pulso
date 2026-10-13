//! Las fórmulas, **cada una con su fuente**.
//!
//! La regla de este fichero: **si no se puede citar, no se calcula**. Cada función dice de dónde sale,
//! y cada una admite que **puede no poder calcularse** — devuelve `Dato::Falta`, no un cero.
//!
//! Las fuentes completas están en `docs/contexto.md`, y sus enlaces en `docs/contexto.refs.yaml`.

use crate::datos::{Dato, Persona, Sesion};

/// La frecuencia cardíaca **máxima**, y **de dónde sale**.
///
/// Hay tres caminos, y **no valen lo mismo**:
///
/// 1. **Medida** en una prueba de esfuerzo → es un hecho.
/// 2. **Estimada** con la edad → **tiene error**, y por eso el motor lo dice.
/// 3. **Ninguno** → no se puede.
///
/// La ecuación que se usa cuando se estima es la de **Tanaka**:
/// `FCmáx = 208 − (0,7 × edad)`, que se prefiere a la clásica `220 − edad` porque **se ajustó sobre
/// datos medidos** en vez de ser una regla de dedo.
///
/// **Y aquí está el caso que me importa**: esa ecuación se ajustó sobre **una población concreta**, y
/// aplicarla a **otra** mete un error que **no se puede cuantificar sin saber de quién hablamos**. El
/// cliente mencionó la **etnia** entre sus datos — y tiene razón en mencionarla: **el motor no la va a
/// inventar ni la va a ignorar en silencio**. Si falta, **lo dice**.
pub fn pulso_maximo(p: &Persona) -> Resultado<u16> {
    // 1. Si está medido, manda. No se estima lo que se sabe.
    if let Dato::Hay(m) = p.pulso_maximo_medido {
        return Resultado::seguro(m, "Medida en una prueba de esfuerzo", &[]);
    }

    // 2. Si hay edad, se estima — y se declara el error.
    let edad = match p.edad.valor() {
        Some(e) => e,
        None => {
            return Resultado::no_se_puede(
                "No hay frecuencia máxima medida ni edad para estimarla",
                &["la frecuencia máxima medida (prueba de esfuerzo)", "la edad"],
            )
        }
    };

    let estimada = 208.0 - (0.7 * f32::from(edad));
    let valor = estimada.round() as u16;

    // El error de una estimación por edad NO es cero, y quien lo use tiene que saberlo.
    let mut falta = Vec::new();
    if p.sexo.falta() {
        falta.push("el sexo: hay ecuaciones específicas que aquí no se pueden aplicar");
    }

    Resultado::con_reserva(
        valor,
        "Estimada con la ecuación de Tanaka: 208 − (0,7 × edad)",
        "Una estimación por edad tiene error frente a una prueba de esfuerzo. No se usa para prescribir intensidades altas sin comprobación.",
        &falta,
    )
}

/// **Reserva de frecuencia cardíaca**: la diferencia entre la máxima y la de reposo.
///
/// Es la base del **método de Karvonen** para fijar zonas de intensidad: las zonas se calculan sobre
/// **el margen que le queda al corazón**, no sobre el total. Una persona con 50 pulsaciones en reposo y
/// otra con 80 **no entrenan igual al mismo porcentaje**, y esto es lo que lo recoge.
pub fn reserva_cardiaca(p: &Persona) -> Resultado<u16> {
    let maxima = match pulso_maximo(p) {
        Resultado { valor: Some(v), .. } => v,
        _ => {
            return Resultado::no_se_puede(
                "Sin frecuencia máxima no hay reserva cardíaca",
                &["la frecuencia máxima"],
            )
        }
    };
    let reposo = match p.pulso_reposo.valor() {
        Some(r) => r,
        None => {
            return Resultado::no_se_puede(
                "Sin frecuencia de reposo no hay reserva cardíaca",
                &["la frecuencia cardíaca en reposo"],
            )
        }
    };

    // Un reposo por encima de la máxima no es un dato: es un error de entrada, y se dice.
    if reposo >= maxima {
        return Resultado::no_se_puede(
            "La frecuencia de reposo es mayor o igual que la máxima: los datos no cuadran",
            &["una frecuencia de reposo y una máxima coherentes"],
        );
    }

    Resultado::seguro(maxima - reposo, "Máxima menos reposo (base del método de Karvonen)", &[])
}

/// Una **zona de intensidad** por el método de Karvonen.
///
/// Los límites que se usan —60 %, 70 %, 80 %, 90 % de la reserva— son los **habituales** para separar
/// intensidad suave, moderada, fuerte y máxima. **No son una ley física**: son convenciones de
/// prescripción, y por eso van escritas aquí y no escondidas en una constante suelta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zona {
    Suave,
    Moderada,
    Fuerte,
    Maxima,
}

impl Zona {
    /// El porcentaje de la reserva que define el **suelo** de cada zona.
    pub fn suelo(self) -> f32 {
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

/// Los **límites en pulsaciones** de cada zona, para esta persona.
///
/// Devuelve `Falta` si no se puede calcular la reserva. **No devuelve zonas inventadas.**
pub fn zonas(p: &Persona) -> Resultado<Vec<(Zona, u16, u16)>> {
    let reposo = match p.pulso_reposo.valor() {
        Some(r) => r,
        None => {
            return Resultado::no_se_puede(
                "Sin frecuencia de reposo no se pueden fijar zonas",
                &["la frecuencia cardíaca en reposo"],
            )
        }
    };
    let reserva = match reserva_cardiaca(p) {
        Resultado { valor: Some(v), .. } => v,
        Resultado { falta, .. } => {
            let mut f = falta;
            f.insert(0, "la reserva cardíaca (hace falta la máxima y la de reposo)".to_string());
            return Resultado::no_se_puede("Sin reserva cardíaca no hay zonas", &f);
        }
    };

    let en_pulsos = |pct: f32| -> u16 { (f32::from(reposo) + pct * f32::from(reserva)).round() as u16 };

    let zs = vec![
        (Zona::Suave, en_pulsos(0.50), en_pulsos(0.60)),
        (Zona::Moderada, en_pulsos(0.60), en_pulsos(0.70)),
        (Zona::Fuerte, en_pulsos(0.70), en_pulsos(0.85)),
        (Zona::Maxima, en_pulsos(0.85), en_pulsos(1.00)),
    ];

    Resultado::seguro(
        zs,
        "Método de Karvonen: reposo + (porcentaje × reserva). Límites al 50/60/70/85 %",
        &[],
    )
}

/// El **TRIMP**: cuánto esfuerzo ha supuesto una sesión, juntando **duración e intensidad**.
///
/// Es el método de **Banister**, y sirve para **comparar sesiones distintas** y ver si el plan
/// progresa. Se calcula como **minutos × un factor que sube con la intensidad**, y ese factor crece de
/// forma **exponencial**: entrenar fuerte cansa **mucho** más que entrenar el doble de tiempo suave.
///
/// Aquí se usa la forma **simplificada por fracción de reserva**, que es la que se puede aplicar sin
/// análisis de sangre: `TRIMP = minutos × fracción × 0,64 × e^(1,92 × fracción)`.
pub fn trimp(sesion: &Sesion, p: &Persona) -> Resultado<f32> {
    let reserva = match reserva_cardiaca(p) {
        Resultado { valor: Some(v), .. } => f32::from(v),
        _ => {
            return Resultado::no_se_puede(
                "Sin reserva cardíaca no se puede calcular la carga",
                &["la frecuencia máxima", "la frecuencia cardíaca en reposo"],
            )
        }
    };
    let reposo = f32::from(p.pulso_reposo.valor().unwrap_or(0));

    let pulsos: Vec<u16> = sesion.pulsos();
    if pulsos.is_empty() {
        return Resultado::no_se_puede(
            "La sesión no trae ni una muestra de pulso",
            &["pulsaciones durante la sesión"],
        );
    }

    // Cuántas muestras faltaban de las que había: si son muchas, la carga es una estimación.
    let total = sesion.muestras.len();
    let sin_pulso = total - pulsos.len();
    let proporcion_sin = sin_pulso as f32 / total.max(1) as f32;

    // El factor de cada muestra, y se promedia. Las muestras sin pulso NO se cuentan como cero.
    let mut suma = 0.0f32;
    for m in &sesion.muestras {
        let pulso = match m.pulso.valor() {
            Some(v) => f32::from(v),
            None => continue, // no se rellena: se salta
        };
        let fraccion = ((pulso - reposo) / reserva).clamp(0.0, 1.0);
        suma += fraccion * 0.64 * (1.92 * fraccion).exp();
    }
    let factor_medio = suma / pulsos.len() as f32;

    let minutos = match sesion.duracion() {
        Dato::Hay(s) => s as f32 / 60.0,
        Dato::Falta => {
            return Resultado::no_se_puede(
                "No hay duración: ni declarada ni deducible de las muestras",
                &["la duración de la sesión"],
            )
        }
    };

    let carga = minutos * factor_medio;

    if proporcion_sin > 0.2 {
        if proporcion_sin > 0.5 {
            // Con más de la mitad sin pulso, el número no se sostiene: no se da.
            return Resultado::no_se_puede(
                &format!(
                    "Más de la mitad de la sesión ({:.0} %) no trae pulso: la carga no se puede afirmar",
                    proporcion_sin * 100.0
                ),
                ["pulsaciones en toda la sesión"],
            );
        }
        let f = vec!["pulsaciones en parte de la sesión".to_string()];
        return Resultado::con_reserva(
            carga,
            "TRIMP de Banister, forma simplificada por fracción de reserva",
            &format!("Falta el pulso en un {:.0} % de la sesión: la carga real es algo mayor que esta.", proporcion_sin * 100.0),
            f,
        );
    }

    Resultado::seguro(carga, "TRIMP de Banister, forma simplificada por fracción de reserva", &[])
}

/// El resultado de un cálculo, **con lo que se puede y lo que no**.
///
/// Este tipo es la razón de que el motor exista. Un cálculo **no devuelve un número**: devuelve
/// **un número, su fuente, su reserva, y lo que falta**.
#[derive(Debug, Clone, PartialEq)]
pub struct Resultado<T> {
    /// El valor. **`None` significa que no se puede afirmar**, y es una respuesta legítima.
    pub valor: Option<T>,
    /// De dónde sale. **Vacío solo cuando no hay valor.**
    pub fuente: String,
    /// Qué reserva tiene el número, si la tiene.
    pub reserva: Option<String>,
    /// Qué falta para poder afirmarlo mejor.
    pub falta: Vec<String>,
}

impl<T> Resultado<T> {
    pub fn seguro(valor: T, fuente: &str, reserva: &[&str]) -> Self {
        Self {
            valor: Some(valor),
            fuente: fuente.to_string(),
            reserva: reserva.first().map(|s| s.to_string()),
            falta: Vec::new(),
        }
    }

    /// Con valor, pero con una reserva. Acepta `&["a"]` o un `Vec<String>`.
    pub fn con_reserva<I, S>(valor: T, fuente: &str, reserva: &str, falta: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            valor: Some(valor),
            fuente: fuente.to_string(),
            reserva: Some(reserva.to_string()),
            falta: falta.into_iter().map(|s| s.as_ref().to_string()).collect(),
        }
    }

    /// Sin valor. Acepta cualquier lista de textos: `&["a", "b"]` o un `&Vec<String>`.
    pub fn no_se_puede<I, S>(motivo: &str, falta: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            valor: None,
            fuente: motivo.to_string(),
            reserva: None,
            falta: falta.into_iter().map(|s| s.as_ref().to_string()).collect(),
        }
    }

    pub fn se_puede(&self) -> bool {
        self.valor.is_some()
    }
}
