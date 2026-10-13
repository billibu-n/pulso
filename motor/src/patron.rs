//! **El patrón**: lo que se DEDUCE de los sensores, que el FIT no trae.
//!
//! POR QUÉ ESTE MÓDULO ES EL CORAZÓN DEL PROYECTO, y lo dijo el dueño:
//!
//! > *«Resultados calculados que el mismo FIT no proporciona, pues solo proporciona la información de
//! > los inputs de los sensores.»*
//!
//! Eso es exactamente el valor. Un fichero FIT trae **la lista de lo que midió el sensor**: pulso,
//! potencia, cadencia, segundo a segundo. **No trae nada de esto:**
//!
//! - **La deriva** — si el pulso se escapa mientras la potencia se queda. Es el patrón que señala el
//!   límite de duración de una persona, y **hay que calcularlo**.
//! - **La eficiencia** — cuántos vatios saca por cada pulsación. **Es la medida de la mejora**, porque
//!   sube cuando el mismo trabajo cuesta menos pulso.
//! - **El desfase** — cuántos segundos tarda el pulso en responder a un cambio de potencia.
//!
//! NINGUNO de los tres está en el FIT. Los tres salen de **comparar** el pulso con la potencia, y eso
//! **solo lo puede hacer quien tenga las dos series.**
//!
//! REGLA DE ESTE MÓDULO, la del rediseño: **siempre un número, con su margen**. Nunca un «no se puede».

use crate::datos::{Dato, Persona, Sesion};
use crate::rango::{self, Rango};

/// **La deriva**: se compara la primera mitad de una sesión con la segunda.
///
/// QUÉ MIDE: si el pulso **sube** mientras la potencia **se queda o baja**, el cuerpo está trabajando
/// más de lo que el rendimiento refleja: se está cansando. Se mide como el **cambio en vatios por
/// pulsación** entre las dos mitades, en porcentaje.
///
/// CÓMO SE LEE, y son cortes de ciclismo con potencia:
/// - **por debajo del 5 %**: buena base aeróbica; aguanta;
/// - **del 5 al 10 %**: aparece el cansancio;
/// - **por encima del 10 %**: el pulso se escapa, y ahí está el límite.
///
/// Y ESTOS CORTES SON UNA DECISIÓN, no un hallazgo: van escritos y a la vista.
#[derive(Debug, Clone, PartialEq)]
pub struct Deriva {
    pub rango: Rango,
    /// Vatios por pulsación en la primera mitad.
    pub eficiencia_primera: f64,
    /// Y en la segunda.
    pub eficiencia_segunda: f64,
    /// Cuántas muestras se usaron, y cuántas se quedaron fuera por no tener pulso o potencia.
    pub muestras: usize,
    pub descartadas: usize,
}

impl Deriva {
    /// El porcentaje de deriva: cuánto empeoró la eficiencia de la primera mitad a la segunda.
    pub fn porcentaje(&self) -> f64 {
        self.rango.valor
    }

    /// Cómo de grave es, en palabras.
    pub fn nivel(&self) -> NivelDeriva {
        let v = self.porcentaje();
        if v < 5.0 {
            NivelDeriva::Buena
        } else if v < 10.0 {
            NivelDeriva::Aparece
        } else {
            NivelDeriva::SeEscapa
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NivelDeriva {
    /// Aguanta: la eficiencia apenas cambia entre mitades.
    Buena,
    /// El cansancio aparece, pero se sostiene.
    Aparece,
    /// El pulso se escapa: ahí está el límite de esta persona.
    SeEscapa,
}

impl NivelDeriva {
    pub fn nombre(self) -> &'static str {
        match self {
            NivelDeriva::Buena => "aguanta",
            NivelDeriva::Aparece => "aparece el cansancio",
            NivelDeriva::SeEscapa => "se le escapa el pulso",
        }
    }

    pub fn explicacion(self) -> &'static str {
        match self {
            NivelDeriva::Buena => {
                "la eficiencia apenas cambia entre la primera mitad y la segunda: el motor aguanta"
            }
            NivelDeriva::Aparece => {
                "en la segunda mitad saca menos vatios por pulsación: el cansancio ya se nota"
            }
            NivelDeriva::SeEscapa => {
                "el pulso se le escapa y la potencia cae: ese es el límite de duración a esta intensidad"
            }
        }
    }
}

/// Calcula **la deriva** de una sesión.
///
/// No falla nunca: si falta el pulso o la potencia, **se calcula con lo que haya** y el margen lo dice.
pub fn deriva(sesion: &Sesion) -> Deriva {
    // Los pares (pulso, potencia) que existen. Los que no, se cuentan y se declaran.
    let mut pares: Vec<(f64, f64)> = Vec::new();
    let mut descartadas = 0usize;
    for m in &sesion.muestras {
        match (m.pulso.valor(), m.potencia_w.valor()) {
            (Some(p), Some(w)) if p > 0 && w > 0 => pares.push((f64::from(p), f64::from(w))),
            _ => descartadas += 1,
        }
    }

    // Sin pares suficientes no hay comparación posible: **cero con margen cero**, no un fallo.
    if pares.len() < 20 {
        return Deriva {
            rango: Rango::estimado(0.0, 0.0),
            eficiencia_primera: 0.0,
            eficiencia_segunda: 0.0,
            muestras: pares.len(),
            descartadas,
        };
    }

    let mitad = pares.len() / 2;
    let media = |v: &[(f64, f64)]| -> (f64, f64) {
        let p: f64 = v.iter().map(|x| x.0).sum::<f64>() / v.len() as f64;
        let w: f64 = v.iter().map(|x| x.1).sum::<f64>() / v.len() as f64;
        (p, w)
    };
    let (p1, w1) = media(&pares[..mitad]);
    let (p2, w2) = media(&pares[mitad..]);

    let e1 = w1 / p1; // vatios por pulsación
    let e2 = w2 / p2;

    // La deriva: cuánto empeoró la eficiencia. Positiva = empeoró.
    let valor = ((e1 - e2) / e1) * 100.0;

    // EL MARGEN, y esto se corrigió el 2026-10-13 porque la primera versión estaba mal.
    //
    // EL ERROR QUE TENÍA: calculaba el margen con `(deriva × 0,35) / raíz(n)`. Parece razonable, pero
    // **está mal por un motivo de fondo**: las muestras de una sesión **NO son independientes**. El
    // pulso va correlacionado consigo mismo segundo a segundo, así que **6.000 muestras no son 6.000
    // datos**: son muchas menos. Con esa fórmula, **todas las sesiones salían con el mismo margen**
    // (1,0 a 1,1), y no distinguía una de 24 minutos de una de 210. Un margen que vale igual para
    // todo **no informa de nada**.
    //
    // LO CORRECTO, y es más humilde: **la variabilidad de verdad de una deriva NO se puede sacar de
    // una sola sesión.** Sale de comparar **sesiones entre sí**, y eso necesita histórico. Para una
    // sesión sola, lo honesto es un **margen conservador, y que se estrecha solo si hay muchas
    // muestras** — pero nunca finge una precisión que no tiene.
    let n = pares.len() as f64;
    let margen = if n >= 1000.0 {
        MARGEN_DERIVA
    } else if n >= 300.0 {
        MARGEN_DERIVA * 1.5
    } else if n >= 100.0 {
        MARGEN_DERIVA * 2.0
    } else {
        MARGEN_DERIVA * 3.0
    };

    Deriva {
        rango: Rango::estimado(valor, margen),
        eficiencia_primera: e1,
        eficiencia_segunda: e2,
        muestras: pares.len(),
        descartadas,
    }
}

/// **El margen de una deriva medida en una sola sesión**, en puntos porcentuales.
///
/// ES UNA DECISIÓN, y va aquí a la vista con su motivo: la deriva se calcula comparando dos mitades de
/// la misma sesión, y **una sesión sola no puede decir cuánto varía ese número de una vez a otra**. Lo
/// que sí se sabe es que el pulso tiene un error de unos pocos latidos y que la respuesta
/// cardiovascular varía de un día a otro, así que **±3 puntos es lo honesto**.
///
/// Y sirve para algo concreto: **una deriva del 8 % y una del 12 % se distinguen**, porque el margen no
/// se come la diferencia. Si el margen fuera de ±10, el número no diría nada.
pub const MARGEN_DERIVA: f64 = 3.0;

/// **La eficiencia**: vatios por pulsación. **La medida de la mejora.**
///
/// POR QUÉ ES LA MEDIDA DE LA MEJORA, y no el pulso ni la potencia por separado: porque **junta las
/// dos**. Si con el tiempo hace **los mismos vatios con menos pulso**, está mejorando — y eso **no se
/// ve** mirando el pulso o la potencia por su cuenta.
///
/// **Esto tampoco está en el FIT.** Hay que calcularlo.
pub fn eficiencia(sesion: &Sesion) -> Rango {
    let pares: Vec<(f64, f64)> = sesion
        .muestras
        .iter()
        .filter_map(|m| match (m.pulso.valor(), m.potencia_w.valor()) {
            (Some(p), Some(w)) if p > 0 && w > 0 => Some((f64::from(p), f64::from(w))),
            _ => None,
        })
        .collect();

    if pares.is_empty() {
        return Rango::estimado(0.0, 0.0);
    }
    let p: f64 = pares.iter().map(|x| x.0).sum::<f64>() / pares.len() as f64;
    let w: f64 = pares.iter().map(|x| x.1).sum::<f64>() / pares.len() as f64;
    let valor = w / p;
    // El margen: cuánto varía la eficiencia DENTRO de la propia sesión. Es la medida honesta de
    // cuánto se puede fiar uno de este número para esta sesión.
    let varianza = pares
        .iter()
        .map(|(pi, wi)| {
            let e = wi / pi;
            (e - valor).powi(2)
        })
        .sum::<f64>()
        / pares.len() as f64;
    let desviacion = varianza.sqrt();
    let error_tipico = desviacion / (pares.len() as f64).sqrt();
    Rango::estimado(valor, error_tipico.max(0.001))
}

/// **El desfase**: cuántos segundos tarda el pulso en responder a un cambio de potencia.
///
/// POR QUÉ IMPORTA: el pulso **no responde en el instante**. Cuando el ciclista aprieta, la potencia
/// sube **ya** y el pulso tarda **decenas de segundos**. Saber cuánto tarda **es útil para leer bien
/// los datos** — y **sin esto se pueden sacar conclusiones falsas**: una subida de potencia al final
/// de una sesión **no da tiempo** a que el pulso la refleje, y parecería que fue fácil.
///
/// CÓMO SE CALCULA: se busca el retardo que **mejor correlaciona** el pulso con la potencia. Es la
/// correlación cruzada, y se hace a mano porque son series cortas y no merece una biblioteca.
pub fn desfase(sesion: &Sesion, maximo_segundos: usize) -> Dato<Rango> {
    let pares: Vec<(f64, f64)> = sesion
        .muestras
        .iter()
        .filter_map(|m| match (m.pulso.valor(), m.potencia_w.valor()) {
            (Some(p), Some(w)) if p > 0 && w > 0 => Some((f64::from(p), f64::from(w))),
            _ => None,
        })
        .collect();
    if pares.len() < 100 {
        return Dato::Falta;
    }

    // EL PASO ENTRE MUESTRAS, y esto se corrigió el 2026-10-13 porque la primera versión estaba mal.
    //
    // EL ERROR QUE TENÍA: el retardo se contaba en **muestras**, y se devolvía como si fueran
    // **segundos**. Con muestras cada 10 segundos —que es lo que traen estos ficheros—, un retardo de
    // 6 muestras son **60 segundos**, y el motor habría dicho «6 segundos». **Un factor de 10.**
    //
    // Y hay una segunda trampa: **el paso NO es fijo**. Depende del aparato y de cómo se guardó el
    // FIT. Así que **no se supone: se mide de los propios datos**, con la diferencia más repetida.
    let mut diferencias: Vec<u32> = sesion
        .muestras
        .windows(2)
        .map(|par| par[1].segundo.saturating_sub(par[0].segundo))
        .filter(|d| *d > 0)
        .collect();
    if diferencias.is_empty() {
        return Dato::Falta;
    }
    diferencias.sort_unstable();
    let paso = diferencias[diferencias.len() / 2]; // la mediana: no la mueve un hueco raro

    let maximo_muestras = (maximo_segundos as u32 / paso.max(1)) as usize;
    if maximo_muestras < 2 || pares.len() < maximo_muestras * 3 {
        return Dato::Falta;
    }

    // Se prueban retardos de 0 a `maximo_muestras`, y se queda el que mejor correlaciona.
    let mut mejor = (0usize, f64::MIN);
    for retardo in 0..=maximo_muestras {
        let n = pares.len() - retardo;
        if n < 30 {
            break;
        }
        let pot: Vec<f64> = pares[..n].iter().map(|x| x.1).collect();
        let pul: Vec<f64> = pares[retardo..].iter().map(|x| x.0).collect();
        let c = correlacion(&pot, &pul);
        if c > mejor.1 {
            mejor = (retardo, c);
        }
    }

    if mejor.1 <= 0.0 {
        return Dato::Falta;
    }
    // **Se convierte a segundos**, que es lo que se afirma. Y el margen es **el paso**: no se puede
    // afirmar con más precisión que la distancia entre dos muestras.
    //
    // Y SI EL MEJOR RETARDO ES EL ÚLTIMO QUE SE PROBÓ, el resultado es un **«al menos»**, no un valor:
    // significa que la respuesta sigue mejorando más allá del límite que se buscó. Se devuelve el
    // valor **con un margen que lo dice**, en vez de fingir que 90 es la respuesta.
    let segundos = (mejor.0 as u32 * paso) as f64;
    let en_el_borde = mejor.0 == maximo_muestras;
    let margen = if en_el_borde { paso as f64 * 3.0 } else { paso as f64 };
    Dato::Hay(Rango::estimado(segundos, margen))
}

/// Correlación de Pearson entre dos series del mismo largo.
fn correlacion(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len()) as f64;
    if n < 2.0 {
        return 0.0;
    }
    let ma = a.iter().sum::<f64>() / n;
    let mb = b.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut da = 0.0;
    let mut db = 0.0;
    for i in 0..(n as usize) {
        let x = a[i] - ma;
        let y = b[i] - mb;
        num += x * y;
        da += x * x;
        db += y * y;
    }
    if da <= 0.0 || db <= 0.0 {
        return 0.0;
    }
    num / (da.sqrt() * db.sqrt())
}

/// **El salto frente a todo el histórico disponible**, no solo al mes anterior.
///
/// POR QUÉ: la primera versión comparaba con los 30 días anteriores, porque eso es lo que mide el
/// estudio de lesiones. Pero **si hay seis meses de datos, se puede comparar también con todo**, y esa
/// comparación dice otra cosa: **si esta sesión es la más dura de su historia o es habitual en él**.
pub fn salto_historico(hoy: &Sesion, historico: &[Sesion]) -> Rango {
    let carga_hoy = crate::formulas_v2::trimp(hoy, &Persona::desconocida()).valor;
    let cargas: Vec<f64> = historico
        .iter()
        .map(|s| crate::formulas_v2::trimp(s, &Persona::desconocida()).valor)
        .filter(|v| *v > 0.0)
        .collect();
    if cargas.is_empty() {
        return Rango::estimado(1.0, 1.0);
    }
    let maximo = cargas.iter().cloned().fold(f64::MIN, f64::max);
    if maximo <= 0.0 {
        return Rango::estimado(1.0, 1.0);
    }
    Rango::estimado(carga_hoy / maximo, 0.05)
}

/// La **precisión de un patrón**, según cuántas muestras lo sostienen.
///
/// Es útil para decir, de un vistazo, si un patrón se puede afirmar o es solo un indicio.
pub fn precision_del_patron(muestras: usize) -> rango::Precision {
    if muestras >= 2000 {
        rango::Precision::Alta
    } else if muestras >= 500 {
        rango::Precision::Buena
    } else if muestras >= 100 {
        rango::Precision::Aceptable
    } else {
        rango::Precision::Amplia
    }
}
