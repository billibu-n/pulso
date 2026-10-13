//! El **adaptador**: traduce el formato de fuera a **nuestro** modelo.
//!
//! POR QUÉ ESTA PIEZA EXISTE, y por qué está separada del motor:
//!
//! El motor **no sabe** de Garmin, ni de FIT, ni del JSON que use el cliente. Sabe de **su propio
//! modelo**. Todo lo de fuera entra **por aquí**.
//!
//! La consecuencia práctica: cuando el cliente mande sus datos de verdad, **cambia este fichero, no el
//! motor**. Y si mañana cambia de reloj o de aplicación, **otra vez este fichero**.
//!
//! El primer formato que se implementa es el **sintético** (`datos-sinteticos/`), que existe porque el
//! cliente aún no ha dado los suyos. **Sirve para probar el motor entero sin depender de nadie.**

use crate::datos::{Dato, Muestra, Persona, Sesion, Sexo};
use serde::Deserialize;

/// Un dato que **puede venir o no venir** en el fichero.
///
/// Es `Option` y no `Dato` a propósito: en el límite con el mundo exterior se usa lo que el mundo
/// exterior entiende. La traducción a `Dato` la hace este módulo, **en un solo sitio**.
type Quizas<T> = Option<T>;

#[derive(Debug, Deserialize)]
pub struct PerfilJson {
    pub persona: PersonaJson,
}

#[derive(Debug, Deserialize)]
pub struct PersonaJson {
    pub edad: Quizas<u8>,
    pub peso_kg: Quizas<f32>,
    pub pulso_reposo: Quizas<u16>,
    pub pulso_maximo_medido: Quizas<u16>,
    pub sexo: Quizas<String>,
}

impl PersonaJson {
    /// Traduce al modelo del motor.
    ///
    /// **Lo que NO está, queda como `Falta`.** No se rellena con un valor por defecto, no se pone a
    /// cero, y no se supone. Esa es la regla de todo el proyecto.
    pub fn a_persona(&self) -> Persona {
        Persona {
            edad: segun(self.edad),
            peso_kg: segun(self.peso_kg),
            pulso_reposo: segun(self.pulso_reposo),
            pulso_maximo_medido: segun(self.pulso_maximo_medido),
            sexo: match self.sexo.as_deref() {
                Some("femenino") | Some("F") => Dato::Hay(Sexo::Femenino),
                Some("masculino") | Some("M") => Dato::Hay(Sexo::Masculino),
                // Un valor que no se entiende NO se adivina: se trata como si faltara.
                _ => Dato::Falta,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SemanaJson {
    pub semana: u32,
    pub sesiones: Vec<SesionJson>,
}

#[derive(Debug, Deserialize)]
pub struct SesionJson {
    pub tipo: String,
    pub duracion_s: u32,
    pub muestras: Vec<MuestraJson>,
}

#[derive(Debug, Deserialize)]
pub struct MuestraJson {
    pub segundo: u32,
    pub pulso: Quizas<u16>,
    pub potencia_w: Quizas<u16>,
}

impl SesionJson {
    /// Traduce una sesión al modelo del motor, **conservando lo que falta como `Falta`**.
    pub fn a_sesion(&self) -> Sesion {
        Sesion {
            muestras: self
                .muestras
                .iter()
                .map(|m| Muestra {
                    segundo: m.segundo,
                    pulso: segun(m.pulso),
                    potencia_w: segun(m.potencia_w),
                })
                .collect(),
            duracion_s: Dato::Hay(self.duracion_s),
        }
    }
}

/// Traduce un `Option` del exterior a un `Dato` nuestro. **Un solo sitio para esta regla.**
fn segun<T>(v: Quizas<T>) -> Dato<T> {
    match v {
        Some(x) => Dato::Hay(x),
        None => Dato::Falta,
    }
}

/// Lee el perfil de la persona desde un texto JSON.
pub fn leer_perfil(texto: &str) -> Result<Persona, String> {
    let p: PerfilJson = serde_json::from_str(texto).map_err(|e| format!("el JSON no se entiende: {e}"))?;
    Ok(p.persona.a_persona())
}

/// Lee las semanas de entrenamiento desde un texto JSON.
pub fn leer_semanas(texto: &str) -> Result<Vec<SemanaJson>, String> {
    #[derive(Deserialize)]
    struct Raiz {
        semanas: Vec<SemanaJson>,
    }
    let r: Raiz = serde_json::from_str(texto).map_err(|e| format!("el JSON no se entiende: {e}"))?;
    Ok(r.semanas)
}
