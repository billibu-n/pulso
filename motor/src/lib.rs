//! El motor de análisis de entrenamiento.
//!
//! QUÉ ES: el programa que **piensa**. Recibe datos, calcula, **decide qué se puede afirmar**, y
//! **produce un resultado justificado**. No dibuja nada.
//!
//! LA REGLA QUE LO DEFINE, y está en el tipo `Confianza`:
//! **este motor NO inventa un dato que falta**. Cuando no se puede afirmar algo, **lo dice**,
//! **dice qué falta**, y **dice qué haría falta para poder afirmarlo**. Un plan de entrenamiento
//! afecta al cuerpo de una persona: una afirmación sin fundamento, aquí, no es un error de cálculo.
//!
//! DE DÓNDE SALE CADA FÓRMULA: todas están citadas en `docs/contexto.md`, y cada una lleva su
//! comentario al lado. **Si no se puede citar, no se calcula.**

#![forbid(unsafe_code)]

pub mod adaptador;
pub mod analisis;
pub mod avisos;
pub mod cara;
pub mod carpeta;
pub mod contexto;
pub mod datos;
pub mod esfuerzo;
pub mod fit;
pub mod fuentes;
pub mod formulas;
pub mod hrv;
pub mod huawei;
pub mod informe;
pub mod formulas_v2;
pub mod juicio;
pub mod memoria;
pub mod patron;
pub mod potencia;
pub mod profile;
pub mod rango;
pub mod sincronizacion;
pub mod sincronizar;
pub mod tendencia;
pub mod umbral;

pub use contexto::{Comida, Contexto, Escenario, Fuente};
pub use analisis::Real;
pub use avisos::{Aviso, Historico, Hoy, Modulo, SesionVista, Urgencia};
pub use datos::{Composicion, Dato, Persona, Sesion};
pub use esfuerzo::{NivelSalto, Salto};
pub use fuentes::{Consenso, Procedencia, Valor};
pub use hrv::Hrv;
pub use informe::{generar, hechos, Hechos};
pub use patron::{Deriva, NivelDeriva};
pub use rango::{Precision, Rango};
pub use tendencia::{Direccion, Serie, Tendencia};
pub use umbral::{Referencia, Umbral};
pub use juicio::{Confianza, Falta, Fundamento, Juicio};

/// Atajo para escribir `Dato::Hay`. Existe para que el código de fuera se lea.
pub fn hay<T>(v: T) -> Dato<T> {
    Dato::Hay(v)
}
