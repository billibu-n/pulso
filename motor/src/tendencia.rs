//! La **tendencia**: ¿está mejorando, o solo entrenando?
//!
//! POR QUÉ EXISTE: es **la pregunta central del encargo** —«mejora continua», dijo el cliente— y es la
//! que **no se puede contestar con un solo mes**. Con un mes se comparan sesiones entre sí; para saber si
//! **mejora** hacen falta **dos momentos separados en el tiempo**.
//!
//! Y hay una trampa en la que es fácil caer: **subir el volumen NO es mejorar**. En los datos de este
//! cliente se ve con claridad — en julio entrenó **39 horas** (el mes de más volumen) y su consumo máximo
//! de oxígeno **bajó**. Mejoró **después**, cuando el volumen bajó. Un motor que confunda **más** con
//! **mejor** le diría que entrene más justo cuando necesita recuperar.
//!
//! LA REGLA: la tendencia **no dice si algo es bueno o malo**. Dice **hacia dónde va**, y **cuánto**, y
//! **con qué datos**. La interpretación va aparte, y con su fundamento.

use crate::datos::Dato;
use crate::fuentes::Valor;

/// Una serie de valores en el tiempo, para medir hacia dónde va.
#[derive(Debug, Clone, PartialEq)]
pub struct Serie {
    /// Los puntos, **en orden cronológico**.
    pub puntos: Vec<Valor<f64>>,
    /// Qué se está midiendo, en palabras: «consumo máximo de oxígeno», «vatios por pulsación».
    pub que: String,
}

impl Serie {
    pub fn nueva(que: &str, puntos: Vec<Valor<f64>>) -> Self {
        let mut s = Self { puntos, que: que.to_string() };
        s.puntos.sort_by(|a, b| a.fecha.cmp(&b.fecha));
        s
    }

    pub fn hay_suficiente(&self) -> bool {
        self.puntos.len() >= 2
    }

    /// La tendencia, **con sus tres números**: el primero, el último, y el cambio.
    ///
    /// Devuelve `Falta` si no hay al menos dos puntos: **con uno no hay tendencia**, y decirlo es la
    /// respuesta correcta.
    pub fn tendencia(&self) -> Dato<Tendencia> {
        if !self.hay_suficiente() {
            return Dato::Falta;
        }
        let primero = self.puntos.first().unwrap();
        let ultimo = self.puntos.last().unwrap();
        let cambio = ultimo.valor - primero.valor;
        let relativo = if primero.valor.abs() > f64::EPSILON {
            cambio / primero.valor
        } else {
            0.0
        };

        // El mínimo y el máximo, porque **la forma importa**: una serie que baja y luego sube
        // cuenta algo distinto que una que solo sube.
        let minimo = self.puntos.iter().cloned().fold(None, |acc: Option<Valor<f64>>, p| {
            Some(match acc {
                Some(a) if a.valor <= p.valor => a,
                _ => p,
            })
        });
        let maximo = self.puntos.iter().cloned().fold(None, |acc: Option<Valor<f64>>, p| {
            Some(match acc {
                Some(a) if a.valor >= p.valor => a,
                _ => p,
            })
        });

        Dato::Hay(Tendencia {
            que: self.que.clone(),
            puntos: self.puntos.len(),
            desde: primero.fecha.clone(),
            hasta: ultimo.fecha.clone(),
            primero: primero.valor,
            ultimo: ultimo.valor,
            cambio,
            relativo,
            minimo,
            maximo,
        })
    }
}

/// El resultado de medir una tendencia.
#[derive(Debug, Clone, PartialEq)]
pub struct Tendencia {
    pub que: String,
    pub puntos: usize,
    pub desde: String,
    pub hasta: String,
    pub primero: f64,
    pub ultimo: f64,
    pub cambio: f64,
    /// El cambio como proporción: 0,05 es «un 5 % más».
    pub relativo: f64,
    pub minimo: Option<Valor<f64>>,
    pub maximo: Option<Valor<f64>>,
}

/// Hacia dónde va algo. **No dice si es bueno o malo**: eso depende de qué se mida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direccion {
    Sube,
    Baja,
    Estable,
}

impl Tendencia {
    /// La dirección, con una **tolerancia**: por debajo de ella, se considera estable.
    ///
    /// Sin tolerancia, cualquier variación mínima parecería una tendencia. Y decir «mejora» por una
    /// centésima es peor que no decir nada.
    pub fn direccion(&self, tolerancia_relativa: f64) -> Direccion {
        if self.relativo.abs() < tolerancia_relativa {
            Direccion::Estable
        } else if self.relativo > 0.0 {
            Direccion::Sube
        } else {
            Direccion::Baja
        }
    }

    /// ¿Bajó y luego volvió a subir? Es la forma que tienen los datos de este cliente,
    /// y **cuenta algo distinto** que una subida recta.
    pub fn bajo_y_subio(&self) -> bool {
        match (&self.minimo, &self.maximo) {
            (Some(min), Some(max)) => {
                // El mínimo NO es el primero, y el máximo SÍ es el último: bajó y remontó.
                min.fecha != self.desde && max.fecha == self.hasta && self.ultimo > self.primero - self.cambio.abs() * 0.0
            }
            _ => false,
        }
    }

    /// Una frase, para leerla sin interfaz.
    pub fn frase(&self, tolerancia_relativa: f64) -> String {
        let d = match self.direccion(tolerancia_relativa) {
            Direccion::Sube => "sube",
            Direccion::Baja => "baja",
            Direccion::Estable => "se mantiene",
        };
        let mut s = format!(
            "{}: {} ({} → {}, {:+.1} %) — {} puntos, de {} a {}",
            self.que,
            d,
            formatea(self.primero),
            formatea(self.ultimo),
            self.relativo * 100.0,
            self.puntos,
            self.desde,
            self.hasta
        );
        if let Some(min) = &self.minimo {
            if min.fecha != self.desde {
                s.push_str(&format!(
                    ". OJO: tocó fondo en {} ({}) y luego remontó",
                    formatea(min.valor),
                    min.fecha
                ));
            }
        }
        s
    }
}

/// Un número, con un decimal como mucho. Lo demás es ruido para quien lee.
fn formatea(v: f64) -> String {
    let r = (v * 10.0).round() / 10.0;
    if (r - r.round()).abs() < f64::EPSILON {
        format!("{}", r.round() as i64)
    } else {
        format!("{r:.1}")
    }
}

/// Tolerancias para decidir si una tendencia es «estable». **Son decisiones, y van a la vista.**
pub mod tolerancia {
    /// Consumo máximo de oxígeno: cambia despacio. Un 2 %.
    pub const VO2: f64 = 0.02;
    /// Eficiencia (vatios por pulsación): cambia aún más despacio. Un 3 %.
    pub const EFICIENCIA: f64 = 0.03;
    /// Volumen de entrenamiento: aquí sí hay variación normal entre meses. Un 15 %.
    pub const VOLUMEN: f64 = 0.15;
}
