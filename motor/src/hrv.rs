//! **El HRV**: la variabilidad entre latidos, que dice cómo está el cuerpo **antes** de entrenar.
//!
//! POR QUÉ ESTE MÓDULO EXISTE, y lo dijo el dueño al verlo: el HRV está **dentro del fichero FIT** y
//! **no está en Garmin Connect**. Son **617.677 intervalos latido a latido** en sus 102 actividades, y
//! **hoy nadie les saca partido**.
//!
//! QUÉ MIDE: el tiempo entre un latido y el siguiente varía. **Cuanto más varía, más recuperado está el
//! sistema** — y cuanto menos, más cargado. Es el dato que decide **si hoy toca apretar o descansar**, y
//! es justo **la pieza que le falta a un plan que solo mira el entrenamiento**.
//!
//! CÓMO SE SACA, y los nombres son estándar:
//!
//! - **RR medio** — el tiempo medio entre latidos, en milisegundos. Su inverso es el pulso.
//! - **SDNN** — la desviación de todos los intervalos. Mide la variabilidad **total**.
//! - **RMSSD** — la raíz de la media de las diferencias **entre latidos consecutivos**. Es **el que más
//!   se usa** para estado y recuperación, porque refleja la parte del sistema nervioso que se recupera
//!   rápido.
//! - **Coeficiente de variación** — el RMSSD dividido por el RR medio, en porcentaje. Sirve para
//!   comparar **una persona consigo misma**, que es lo que importa aquí.
//!
//! REGLA, la del rediseño: **siempre un número, con su margen**. Y una advertencia que va por delante:
//! **estos números son para comparar a la misma persona consigo misma día a día**, no para compararla
//! con una tabla de población.

use crate::datos::Sesion;
use crate::rango::{self, Rango};

/// Lo que se saca del HRV de una sesión.
#[derive(Debug, Clone, PartialEq)]
pub struct Hrv {
    /// Cuántos intervalos entre latidos se pudieron usar.
    pub latidos: usize,
    /// Cuántos se descartaron por no ser fisiológicamente posibles.
    pub descartados: usize,
    /// El tiempo medio entre latidos, en milisegundos.
    pub rr_medio: Rango,
    /// La variabilidad total.
    pub sdnn: Rango,
    /// **El número que más se usa** para estado y recuperación.
    pub rmssd: Rango,
    /// El RMSSD sobre el RR medio, en porcentaje: **para comparar a la misma persona**.
    pub coeficiente: Rango,
}

impl Hrv {
    /// El pulso medio, deducido de los intervalos. **Es una comprobación**: tiene que cuadrar con el
    /// pulso que midió el aparato.
    pub fn pulso_deducido(&self) -> f64 {
        if self.rr_medio.valor > 0.0 {
            60_000.0 / self.rr_medio.valor
        } else {
            0.0
        }
    }

    /// ¿Cuánta confianza merece este HRV, según cuántos latidos lo sostienen?
    pub fn fiabilidad(&self) -> rango::Precision {
        // Para el RMSSD se recomienda un mínimo de un par de minutos de latidos seguidos. A 60 ppm,
        // dos minutos son 120 latidos. Y cuantos más, mejor, hasta cierto punto.
        if self.latidos >= 1000 {
            rango::Precision::Alta
        } else if self.latidos >= 300 {
            rango::Precision::Buena
        } else if self.latidos >= 120 {
            rango::Precision::Aceptable
        } else {
            rango::Precision::Amplia
        }
    }
}

/// **Los límites de lo que puede ser un intervalo entre latidos**, en milisegundos.
///
/// POR QUÉ HACEN FALTA: los aparatos de pecho **pierden latidos y meten artefactos**. Un intervalo de
/// 12 000 ms no es un latido: es un sensor que se soltó. Y **si no se quitan, el RMSSD sale enorme y
/// falso** — porque el RMSSD se calcula sobre diferencias, y **un solo artefacto lo dispara**.
///
/// LOS LÍMITES, y de dónde salen: 300 ms son **200 pulsaciones por minuto**, y 2000 ms son **30**. Por
/// fuera de ahí, en una persona entrenando, **casi seguro es un error de medida y no un latido**.
pub const RR_MINIMO_MS: f64 = 300.0;
pub const RR_MAXIMO_MS: f64 = 2000.0;

/// **Un intervalo que se salta mucho respecto al anterior** también es sospechoso.
///
/// Se descarta si cambia más de un 50 % de golpe: el pulso no puede doblarse ni caer a la mitad entre
/// dos latidos. **Es el filtro que de verdad limpia los artefactos**, porque un sensor que se suelta
/// suele dar **un valor absurdo seguido de otro normal**.
pub const SALTO_SOSPECHOSO: f64 = 0.5;

/// Calcula el HRV a partir de los intervalos entre latidos de una sesión.
///
/// **Nunca falla**: si no hay intervalos, devuelve todo a cero con margen cero, y quien lo lea sabe que
/// no vale nada.
pub fn calcular(intervalos_ms: &[f64]) -> Hrv {
    // --- La limpieza, y es la parte que decide si el número sirve o no ---
    let mut limpios: Vec<f64> = Vec::new();
    let mut descartados = 0usize;
    for (i, v) in intervalos_ms.iter().enumerate() {
        if !v.is_finite() || *v < RR_MINIMO_MS || *v > RR_MAXIMO_MS {
            descartados += 1;
            continue;
        }
        // El salto brusco respecto al anterior ACEPTADO: el pulso no se dobla entre dos latidos.
        if let Some(previo) = limpios.last() {
            if ((v - previo) / previo).abs() > SALTO_SOSPECHOSO {
                descartados += 1;
                continue;
            }
        } else if i > 0 {
            // El primero de la serie tras descartar: se acepta, no hay con qué compararlo.
        }
        limpios.push(*v);
    }

    let n = limpios.len();
    if n < 10 {
        let cero = Rango::estimado(0.0, 0.0);
        return Hrv {
            latidos: n,
            descartados,
            rr_medio: cero,
            sdnn: cero,
            rmssd: cero,
            coeficiente: cero,
        };
    }

    let nf = n as f64;
    let media = limpios.iter().sum::<f64>() / nf;

    // SDNN: la desviación de todos los intervalos.
    let varianza = limpios.iter().map(|v| (v - media).powi(2)).sum::<f64>() / (nf - 1.0);
    let sdnn = varianza.sqrt();

    // RMSSD: la raíz de la media de las diferencias entre CONSECUTIVOS. Es el número que más se usa.
    let sumadif: f64 = limpios.windows(2).map(|p| (p[1] - p[0]).powi(2)).sum();
    let rmssd = (sumadif / (nf - 1.0)).sqrt();

    let coeficiente = if media > 0.0 { (rmssd / media) * 100.0 } else { 0.0 };

    // --- LOS MARGENES, y de dónde salen ---
    //
    // Aquí hay que ser humilde, y se aprendió de la deriva: **las muestras de una serie de latidos NO
    // son independientes**. Un RMSSD calculado sobre 500 latidos seguidos **no tiene la precisión de
    // 500 medidas sueltas**. Por eso el margen **no baja con la raíz de n** hasta valores absurdos:
    // se usa el error típico con un **tope inferior**, que reconoce que la propia medida tiene ruido.
    let error_tipico = |valor: f64| -> f64 {
        let por_muestras = valor * 0.5 / nf.sqrt();
        // El tope: nunca por debajo del 4 % del valor. Una sola sesión no puede dar más precisión.
        por_muestras.max(valor.abs() * 0.04).max(0.5)
    };

    Hrv {
        latidos: n,
        descartados,
        rr_medio: Rango::estimado(media, error_tipico(media)),
        sdnn: Rango::estimado(sdnn, error_tipico(sdnn)),
        rmssd: Rango::estimado(rmssd, error_tipico(rmssd)),
        coeficiente: Rango::estimado(coeficiente, error_tipico(coeficiente)),
    }
}

/// Lee los intervalos entre latidos de una sesión, **si los trae**.
///
/// El motor guarda los intervalos dentro de la sesión, porque son parte de lo que se midió.
pub fn intervalos_de(sesion: &Sesion) -> &[f64] {
    &sesion.intervalos_rr_ms
}

/// **Compara el HRV de hoy con el de las sesiones anteriores**, que es lo único que se puede afirmar.
///
/// POR QUÉ ASÍ, y es la decisión importante de este módulo: **los valores de HRV no se comparan con una
/// tabla**, porque dependen de la edad, del deporte, del aparato y de la persona. **Se comparan consigo
/// mismos**: la pregunta útil es *«¿cómo está hoy respecto a su propia normalidad?»*.
///
/// Devuelve cuánto se aparta de su media, en desviaciones. Y si no hay histórico suficiente, lo dice.
pub fn respecto_a_su_normalidad(hoy: &Hrv, historicas: &[Hrv]) -> Option<Rango> {
    let rmssds: Vec<f64> = historicas
        .iter()
        .filter(|h| h.latidos >= 120 && h.rmssd.valor > 0.0)
        .map(|h| h.rmssd.valor)
        .collect();
    if rmssds.len() < 5 || hoy.rmssd.valor <= 0.0 {
        return None;
    }
    let n = rmssds.len() as f64;
    let media = rmssds.iter().sum::<f64>() / n;
    let varianza = rmssds.iter().map(|v| (v - media).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let desviacion = varianza.sqrt();
    if desviacion <= 0.0 {
        return None;
    }
    // Cuántas desviaciones se aparta hoy, en porcentaje de su propia media.
    let apartamiento = ((hoy.rmssd.valor - media) / desviacion) * 100.0;
    Some(Rango::estimado(apartamiento, 20.0))
}
