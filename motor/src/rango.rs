//! **El rango**: todo cálculo devuelve un número **y su margen**.
//!
//! ESTE MÓDULO CAMBIA EL PRINCIPIO DEL MOTOR, y nace de una corrección del dueño:
//!
//! > *«No es posible que solo funcione la aplicación con toda la data, debe poder funcionar con
//! > información nula, pues no puede fallar por falta de información. Más información proporciona más
//! > precisión, pero no limita.»*
//!
//! TENÍA RAZÓN, Y EL MOTOR ESTABA MAL. Antes, si faltaba un dato, el motor **se negaba a calcular**:
//! había **17 sitios** donde decía «no se puede» y solo **8** donde daba un número. Eso es un motor
//! diseñado para **negarse**, y una aplicación así **se abandona en el primer uso**.
//!
//! LA CONFUSIÓN QUE HAY QUE DESHACER: **«no inventar» NO es lo mismo que «no responder»**.
//! - **Inventar** es dar un número **como si fuera exacto** cuando no lo es. Eso sigue prohibido.
//! - **Responder** es dar **el mejor número posible, con su margen**. Eso es lo correcto siempre.
//!
//! CÓMO: cada dato que entra lleva **un valor y una incertidumbre**. Las fórmulas **propagan** esa
//! incertidumbre, y el resultado sale **con su rango**. Con pocos datos el rango es ancho; con más
//! datos se estrecha. **Pero nunca deja de haber respuesta.**
//!
//! Y LAS INCERTIDUMBRES NO SE INVENTAN: salen de la literatura, y cada una lleva su fuente. Las que no
//! tienen fuente publicada **se declaran como estimación propia**, y se dice.

use std::fmt;

/// Un valor **con su margen**. Es la unidad de todo el motor.
///
/// `valor` es la mejor estimación; `margen` es cuánto puede desviarse. **No es un intervalo de
/// confianza formal**: es el error que la fuente publica, o el que se declara cuando no hay fuente.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rango {
    pub valor: f64,
    pub margen: f64,
}

impl Rango {
    /// Un valor **medido**: su margen es el error de la medición, y suele ser pequeño.
    pub fn medido(valor: f64, margen: f64) -> Self {
        Self { valor, margen: margen.abs() }
    }

    /// Un valor **estimado por una fórmula**, con el error que publica su fuente.
    pub fn estimado(valor: f64, error_de_la_fuente: f64) -> Self {
        Self { valor, margen: error_de_la_fuente.abs() }
    }

    pub fn minimo(&self) -> f64 {
        self.valor - self.margen
    }

    pub fn maximo(&self) -> f64 {
        self.valor + self.margen
    }

    /// **Cuánta precisión tiene**, de 0 a 1: 1 es «exacto», 0 es «no dice nada».
    ///
    /// OJO, y es una corrección del 2026-10-13: esto es **relativo**, y **sirve solo para comparar dos
    /// márgenes sobre el mismo valor**. Para decidir si un número es fiable hay que mirar
    /// [`como_va`](Self::como_va), que usa el **margen absoluto**.
    ///
    /// El motivo, con un caso real: un margen de **±12 pulsaciones** sobre 166 da una precisión
    /// relativa de **0,928** — que suena bien. Pero **12 pulsaciones mueven las zonas enteras**: no es
    /// «buena», es **amplia**. Medir en porcentaje **engaña**, y por eso se cambió.
    pub fn precision(&self) -> f64 {
        if self.valor.abs() < f64::EPSILON {
            return 0.0;
        }
        (1.0 - (self.margen / self.valor.abs())).clamp(0.0, 1.0)
    }

    /// Cómo de ancho es el margen, **en palabras y por su tamaño real**.
    ///
    /// LOS CORTES ESTÁN EN PULSACIONES, y no en porcentaje, porque es lo que de verdad importa: lo que
    /// cambia una zona de entrenamiento **son los latidos, no el porcentaje**.
    ///
    /// Y son **una decisión**, escrita aquí y a la vista — el mismo criterio que los cortes del salto de
    /// esfuerzo. Se calibran así:
    ///
    /// - **±3 o menos**: es una medición, o casi. `Alta`.
    /// - **±8 o menos**: el margen más estrecho que da una ecuación con composición corporal (7,5).
    ///   `Buena`.
    /// - **±14 o menos**: el margen de una ecuación solo con la edad (12). `Aceptable`.
    /// - **más de ±14**: faltan datos. `Amplia`.
    pub fn como_va(&self) -> Precision {
        let m = self.margen;
        if m <= 3.0 {
            Precision::Alta
        } else if m <= 8.0 {
            Precision::Buena
        } else if m <= 14.0 {
            Precision::Aceptable
        } else {
            Precision::Amplia
        }
    }
}

/// Lo ancho que es el margen, dicho en palabras.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    /// El dato está medido, o el margen es mínimo.
    Alta,
    /// Estimado con una fórmula buena, o con varios datos de apoyo.
    Buena,
    /// Estimado con poco: sirve para orientarse.
    Aceptable,
    /// Faltan casi todos los datos: sirve como punto de partida, y hay que decirlo.
    Amplia,
}

impl Precision {
    pub fn nombre(self) -> &'static str {
        match self {
            Precision::Alta => "alta",
            Precision::Buena => "buena",
            Precision::Aceptable => "aceptable",
            Precision::Amplia => "amplia",
        }
    }

    /// Qué decirle al usuario, en una frase.
    pub fn explicacion(self) -> &'static str {
        match self {
            Precision::Alta => "el dato está medido, o el margen es mínimo",
            Precision::Buena => "el margen es pequeño: sirve para decidir",
            Precision::Aceptable => "el margen es apreciable: úsalo como orientación",
            Precision::Amplia => {
                "faltan datos: el margen es ancho, así que tómalo como punto de partida y no como una cifra exacta"
            }
        }
    }
}

impl fmt::Display for Rango {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0} ± {:.0}", self.valor, self.margen)
    }
}

/// **Propagación de la incertidumbre**, para las fórmulas de este motor.
///
/// Cuando un resultado sale de una suma o una resta de datos con error, **los errores se suman en
/// cuadratura** (raíz de la suma de cuadrados). Es la regla habitual, y **es más generosa que sumarlos
/// a pelo**: sumar los márgenes directos daría un rango demasiado ancho.
pub fn suma_de_margenes(margenes: &[f64]) -> f64 {
    margenes.iter().map(|m| m * m).sum::<f64>().sqrt()
}

/// El error que se le supone a la **frecuencia cardiaca máxima estimada por la edad**.
///
/// DE DÓNDE SALE, y no es un número inventado: los estudios que validan estas ecuaciones publican
/// desviaciones **de hasta 12 pulsaciones por minuto** para las fórmulas clásicas, y hay trabajos
/// recientes que encuentran que **ninguna ecuación sirve por encima de los 40 años**.
///
/// **Y esto es lo que hace que el margen sirva para algo**: con este error, las zonas salen con un
/// rango de ±12, que es **honesto** — y el usuario ve **por qué le conviene medirse**.
pub const ERROR_FRECUENCIA_MAXIMA_ESTIMADA: f64 = 12.0;

/// El error de la **frecuencia cardiaca máxima cuando se incluye la composición corporal**.
///
/// DE DÓNDE SALE: hay ecuaciones que añaden **grasa corporal, masa y nivel de actividad** a la edad, y
/// publican un error de **±7,5 pulsaciones** — **casi la mitad** que la fórmula por edad sola.
///
/// **Y esto es exactamente lo que el dueño pide**: *«más información la hace más precisa»*. Aquí se ve
/// con un número: **pasar de edad sola a edad más composición corporal estrecha el margen de 12 a 7,5**.
pub const ERROR_FRECUENCIA_MAXIMA_CON_COMPOSICION: f64 = 7.5;

/// El error de una **medición directa** en una prueba de esfuerzo. Es pequeño, y se declara.
pub const ERROR_FRECUENCIA_MAXIMA_MEDIDA: f64 = 2.0;

/// **La precisión del motor cuando NO hay ningún dato de la persona.**
///
/// Aquí está la respuesta a *«debe poder funcionar con información nula»*: **el motor arranca con los
/// valores de una persona media**, y lo dice. Es el punto de partida — y **cada dato que llegue
/// estrecha el margen**.
///
/// **No es una invención**: son valores de población, y **por eso el margen es ancho**. Lo que estaría
/// prohibido es presentarlos como si fueran de esa persona.
pub mod sin_datos {
    /// Edad con la que arranca el motor si no se sabe: **la mediana de la población adulta**.
    /// Se elige 40, y por eso el margen de todo lo que dependa de ella será ancho.
    pub const EDAD: f64 = 40.0;
    /// Pulsaciones en reposo de una persona media, sin entrenar.
    pub const PULSO_REPOSO: f64 = 70.0;
    /// Margen del pulso de reposo de población: alto, porque varía mucho entre personas.
    pub const PULSO_REPOSO_MARGEN: f64 = 12.0;
    /// Peso de referencia.
    pub const PESO: f64 = 75.0;
    /// Margen del peso de población.
    pub const PESO_MARGEN: f64 = 15.0;
}
