//! Las **fuentes**: el mismo dato dicho por dos sitios que no coinciden.
//!
//! POR QUÉ EXISTE ESTE MÓDULO, y es un problema real de este cliente:
//!
//! Tiene **dos ecosistemas**. **Garmin** mide el entrenamiento (pulso, potencia, umbral). **Huawei** mide
//! el cuerpo (peso, grasa, y también un pulso en reposo). Y **no coinciden**: Garmin dice que su pulso en
//! reposo es **48** y la báscula de Huawei dijo **71**, la misma semana.
//!
//! **Los dos números pueden ser correctos y significar cosas distintas** (48 = el mínimo mientras duerme;
//! 71 = sentado por la tarde). Pero **un motor que los mezcle sin decirlo da zonas equivocadas por hasta
//! 12 pulsaciones**, y el usuario no tiene forma de saber por qué.
//!
//! LA REGLA, y es la de todo el proyecto: **no se elige un valor y se calla**. Se **elige diciendo cuál**,
//! y **se guarda el otro**. Un dato descartado que se tira no se puede revisar después.
//!
//! Y esto es lo mismo que se aprendió con `power` y `accumulated_power`: **dos campos parecidos con
//! significados distintos** es la trampa clásica, y **lo que la caza es compararlos**, no elegir uno.

use std::fmt;

/// De dónde viene un dato. **Se declara siempre**, porque no todas las fuentes valen lo mismo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Procedencia {
    /// Medido en el propio aparato durante el entrenamiento.
    AparatoDeEntrenamiento,
    /// Medido por una báscula o un aparato de composición corporal.
    Bascula,
    /// El usuario lo escribió a mano.
    DeclaradoPorElUsuario,
    /// Estimado con una fórmula.
    Estimado,
}

impl Procedencia {
    pub fn nombre(&self) -> &'static str {
        match self {
            Procedencia::AparatoDeEntrenamiento => "aparato de entrenamiento",
            Procedencia::Bascula => "báscula",
            Procedencia::DeclaradoPorElUsuario => "declarado por el usuario",
            Procedencia::Estimado => "estimado por fórmula",
        }
    }

    /// La procedencia que **manda** para cada tipo de dato, y **por qué**.
    ///
    /// Esto está escrito a propósito como funciones con nombre, y no como una tabla, para que **el
    /// motivo** viaje con la decisión. **No hay una jerarquía universal de fuentes**: hay **una fuente
    /// buena para cada cosa**.
    pub fn para_pulso_de_reposo() -> Self {
        // El aparato que lo mide MIENTRAS DUERME da el mínimo real, que es el que usa Karvonen.
        // Una báscula lo mide sentado y despierto: eso es otra cosa, y mueve las zonas hasta 12 ppm.
        Procedencia::AparatoDeEntrenamiento
    }

    pub fn para_composicion_corporal() -> Self {
        // Aquí es al revés: **la báscula es la que pesa**. El aparato de entrenamiento no.
        Procedencia::Bascula
    }

    pub fn para_datos_de_entrenamiento() -> Self {
        Procedencia::AparatoDeEntrenamiento
    }
}

/// Un valor, con **de dónde viene** y **de cuándo es**.
#[derive(Debug, Clone, PartialEq)]
pub struct Valor<T> {
    pub valor: T,
    pub procedencia: Procedencia,
    /// Fecha en formato `AAAA-MM-DD`. Se usa texto para no arrastrar una biblioteca de fechas.
    pub fecha: String,
    /// Una nota de quien lo aporta: «mínimo mientras duerme», «sentado por la tarde».
    pub nota: Option<String>,
}

impl<T> Valor<T> {
    pub fn nuevo(valor: T, procedencia: Procedencia, fecha: &str) -> Self {
        Self { valor, procedencia, fecha: fecha.to_string(), nota: None }
    }

    pub fn con_nota(mut self, nota: &str) -> Self {
        self.nota = Some(nota.to_string());
        self
    }
}

/// El resultado de juntar varios valores del mismo dato.
#[derive(Debug, Clone, PartialEq)]
pub struct Consenso<T> {
    /// El valor que el motor va a usar. `None` si no había ninguno.
    pub elegido: Option<Valor<T>>,
    /// **Los demás, conservados.** No se tiran: se enseñan.
    pub discrepantes: Vec<Valor<T>>,
    /// Si los valores no coinciden lo bastante como para dar igual, aquí se dice.
    pub aviso: Option<String>,
}

impl<T: fmt::Display + PartialEq + Copy> Consenso<T> {
    /// Junta varios valores del mismo dato y decide **cuál usar**, diciéndolo.
    ///
    /// `preferida` es la procedencia que manda **para este dato concreto**. Lo que decide quién gana
    /// **no es una jerarquía universal de fuentes**: es **qué fuente es buena para qué cosa**.
    pub fn juntar(valores: Vec<Valor<T>>, preferida: Procedencia, tolerancia_relativa: f64) -> Self
    where
        T: Into<f64>,
    {
        if valores.is_empty() {
            return Self { elegido: None, discrepantes: Vec::new(), aviso: None };
        }

        // Se ordena por: primero la procedencia preferida, y después lo más reciente.
        let mut ordenados = valores;
        ordenados.sort_by(|a, b| {
            let pa = a.procedencia == preferida;
            let pb = b.procedencia == preferida;
            pb.cmp(&pa).then(b.fecha.cmp(&a.fecha))
        });

        let elegido = ordenados.first().cloned();
        let discrepantes: Vec<Valor<T>> = ordenados.iter().skip(1).cloned().collect();

        // Y ahora, el aviso: ¿se diferencian lo bastante como para que importe?
        let mut aviso = None;
        if let (Some(e), Some(o)) = (&elegido, discrepantes.first()) {
            let ve: f64 = e.valor.into();
            let vo: f64 = o.valor.into();
            if ve.abs() > f64::EPSILON {
                let dif = ((ve - vo) / ve).abs();
                if dif > tolerancia_relativa {
                    aviso = Some(format!(
                        "Hay dos valores distintos para el mismo dato: {} ({}, {}) frente a {} ({}). \
                         Se usa el primero, pero conviene saber de dónde sale cada uno.",
                        e.valor,
                        e.procedencia.nombre(),
                        e.fecha,
                        o.valor,
                        o.procedencia.nombre()
                    ));
                }
            }
        }

        Self { elegido, discrepantes, aviso }
    }

    pub fn hay(&self) -> bool {
        self.elegido.is_some()
    }
}


/// Tolerancias: **a partir de cuánto se avisa** de que dos fuentes no coinciden.
///
/// Son **decisiones**, no hallazgos, y por eso van aquí a la vista y con su motivo.
pub mod tolerancia {
    /// Peso: dos básculas distintas se diferencian legitimamente en un 2 %. Por debajo, no se avisa.
    pub const PESO: f64 = 0.02;
    /// Pulso de reposo: aquí **sí importa poco cambio**, porque mueve las zonas enteras. Un 5 %.
    pub const PULSO_REPOSO: f64 = 0.05;
    /// Grasa corporal: los métodos discrepan bastante entre sí. Un 10 %.
    pub const GRASA: f64 = 0.10;
}
