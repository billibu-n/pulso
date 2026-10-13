//! El juez de datos: **qué se puede afirmar, qué falta, y cuánto afecta**.
//!
//! Esta es la pieza que casi nadie hace, y la que el cliente pidió sin saber que la pedía:
//! *«un resultado al que le falta data puede arrojar márgenes de error no menores»*.
//!
//! Un modelo de inteligencia artificial al que le falta un dato **lo inventa con seguridad**. Este
//! módulo existe para lo contrario: **mira lo que hay, dice lo que se puede, y dice lo que falta**.
//!
//! Las tres respuestas posibles, y **no hay una cuarta** (inventar no es una respuesta):
//!
//! | estado | qué se le dice al usuario |
//! |---|---|
//! | **se puede** | la afirmación, con su fundamento |
//! | **con reserva** | la afirmación, con su margen y qué falta |
//! | **no se puede** | «no se puede, y por esto» — y qué dato haría falta |

use crate::datos::{Persona, Sesion};
use std::fmt::Display;
use crate::formulas::{self, Resultado};

/// Cuánto se puede afirmar algo. Es el vocabulario del motor entero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confianza {
    /// Están los datos que la afirmación necesita.
    Seguro,
    /// Falta algo que **cambia el número**, pero se puede acotar.
    ConReserva,
    /// Falta algo **sin lo cual la afirmación no tiene sentido**. No se afirma.
    NoSePuede,
}

/// Un dato que falta, y **para qué hacía falta**.
#[derive(Debug, Clone, PartialEq)]
pub struct Falta {
    pub que: String,
    pub para_que: String,
}

/// Una afirmación con su fundamento. **No hay afirmaciones sin fuente.**
#[derive(Debug, Clone, PartialEq)]
pub struct Fundamento {
    pub afirmacion: String,
    pub fuente: String,
    pub reserva: Option<String>,
}

/// Deja un número con un decimal como mucho. Lo demás es ruido para quien lee.
///
/// "70.9432" no le dice nada a nadie; "70,9" sí.
fn redondear(t: &str) -> String {
    match t.parse::<f64>() {
        // Solo se redondea lo que es un número. Un texto se deja como está.
        Ok(n) => {
            let r = (n * 10.0).round() / 10.0;
            if (r - r.round()).abs() < f64::EPSILON {
                format!("{}", r.round() as i64)
            } else {
                format!("{:.1}", r)
            }
        }
        Err(_) => t.to_string(),
    }
}

/// El informe del juez: **todo lo que se puede decir**, y **todo lo que falta**.
#[derive(Debug, Clone, PartialEq)]
pub struct Juicio {
    pub confianza: Confianza,
    pub fundamentos: Vec<Fundamento>,
    pub faltantes: Vec<Falta>,
}

impl Juicio {
    /// Construye el juicio a partir de los resultados de las fórmulas.
    ///
    /// **La regla**: si **cualquier** cálculo necesario no se puede hacer, el juicio **baja de
    /// confianza**. No se sube nunca por tener muchos datos: se baja por faltar uno.
    pub fn juzgar(persona: &Persona, sesion: &Sesion) -> Self {
        let mut fundamentos = Vec::new();
        let mut faltantes = Vec::new();

        // Un ayudante genérico: vale para cualquier `Resultado<T>`, porque cada cálculo devuelve un
        // tipo distinto (u16, f32, una lista de zonas). Rust no deja meterlos en la misma lista, y
        // tiene razón: son cosas distintas.
        fn recoger<T: Display>(
            nombre: &str,
            r: &Resultado<T>,
            fundamentos: &mut Vec<Fundamento>,
            faltantes: &mut Vec<Falta>,
        ) {
            match &r.valor {
                Some(v) => fundamentos.push(Fundamento {
                    // Redondeado a un decimal: "70.9432" no dice nada a nadie.
                    afirmacion: format!("{}: {}", nombre, redondear(&v.to_string())),
                    fuente: r.fuente.clone(),
                    reserva: r.reserva.clone(),
                }),
                None => faltantes.push(Falta {
                    que: r.fuente.clone(),
                    para_que: nombre.to_string(),
                }),
            }
            for f in &r.falta {
                if !faltantes.iter().any(|x| &x.que == f) {
                    faltantes.push(Falta { que: f.clone(), para_que: nombre.to_string() });
                }
            }
        }

        let maximo = formulas::pulso_maximo(persona);
        recoger("frecuencia cardíaca máxima", &maximo, &mut fundamentos, &mut faltantes);

        let reserva = formulas::reserva_cardiaca(persona);
        recoger("reserva cardíaca", &reserva, &mut fundamentos, &mut faltantes);

        let zonas = formulas::zonas(persona);
        match &zonas.valor {
            Some(zs) => {
                let resumen = zs
                    .iter()
                    .map(|(z, a, b)| format!("{} {}-{}", z.nombre(), a, b))
                    .collect::<Vec<_>>()
                    .join(" · ");
                fundamentos.push(Fundamento {
                    afirmacion: format!("zonas de intensidad: {}", resumen),
                    fuente: zonas.fuente.clone(),
                    reserva: zonas.reserva.clone(),
                });
            }
            None => faltantes.push(Falta {
                que: zonas.fuente.clone(),
                para_que: "las zonas de intensidad".to_string(),
            }),
        }
        for f in &zonas.falta {
            if !faltantes.iter().any(|x| &x.que == f) {
                faltantes.push(Falta { que: f.clone(), para_que: "las zonas de intensidad".to_string() });
            }
        }

        let carga = formulas::trimp(sesion, persona);
        recoger("carga de la sesión (TRIMP)", &carga, &mut fundamentos, &mut faltantes);

        // La confianza: se baja si algo no se pudo, y también si algo salió con reserva.
        let hay_imposible =
            !(maximo.se_puede() && reserva.se_puede() && zonas.se_puede() && carga.se_puede());
        // Cada resultado tiene su propio tipo (u16, f32, una lista de zonas), asi que la reserva se
        // mira uno a uno. Rust no deja meterlos en la misma lista, y tiene razon: son cosas distintas.
        let hay_reserva = maximo.reserva.is_some()
            || reserva.reserva.is_some()
            || zonas.reserva.is_some()
            || carga.reserva.is_some();

        let confianza = if hay_imposible {
            Confianza::NoSePuede
        } else if hay_reserva {
            Confianza::ConReserva
        } else {
            Confianza::Seguro
        };

        Self { confianza, fundamentos, faltantes }
    }

    /// Un informe en texto, para leerlo sin interfaz. **Esto es lo que se puede enseñar hoy.**
    pub fn informe(&self) -> String {
        let mut s = String::new();

        s.push_str(match self.confianza {
            Confianza::Seguro => "SE PUEDE AFIRMAR\n",
            Confianza::ConReserva => "SE PUEDE AFIRMAR, CON RESERVA\n",
            Confianza::NoSePuede => "NO SE PUEDE AFIRMAR TODO\n",
        });
        s.push('\n');

        if !self.fundamentos.is_empty() {
            s.push_str("LO QUE SE PUEDE DECIR, Y DE DÓNDE SALE\n");
            for f in &self.fundamentos {
                s.push_str(&format!("  · {}\n", f.afirmacion));
                s.push_str(&format!("      fuente: {}\n", f.fuente));
                if let Some(r) = &f.reserva {
                    s.push_str(&format!("      reserva: {}\n", r));
                }
            }
            s.push('\n');
        }

        if !self.faltantes.is_empty() {
            s.push_str("LO QUE FALTA, Y PARA QUÉ HACÍA FALTA\n");
            for f in &self.faltantes {
                s.push_str(&format!("  · {}  →  hacía falta para {}\n", f.que, f.para_que));
            }
            s.push('\n');
        }

        s.push_str("(Este informe no diagnostica enfermedades: da estado de forma y carga de entrenamiento.)\n");
        s
    }
}
