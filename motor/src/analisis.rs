//! **El analisis de la carpeta entera, en un solo sitio.**
//!
//! POR QUE ESTA AQUI Y NO DENTRO DEL BINARIO DE LA VENTANA: estaba dentro, y tenia dos consecuencias
//! malas. La primera es que **la memoria entre arranques no podia guardarlo**, porque no habia forma
//! de nombrarlo desde fuera. La segunda es que **no se podia probar sin abrir la ventana**.
//!
//! Un dato que no se puede nombrar no se puede guardar, y un dato que no se puede probar no se puede
//! creer. Las dos cosas se arreglan moviendolo aqui.
//!
//! Y LO QUE ESTE MODULO **NO** HACE: no dibuja, no formatea numeros y no decide que enseñar. Devuelve
//! numeros; quien los enseñe decide como se escriben. Mezclar las dos cosas fue el error de la primera
//! version, donde el motor formateaba con comas para la pantalla.

use crate::datos::Dato;
use crate::fit::{self, Lectura};
use crate::{hrv, patron};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Lo que se saca de la carpeta entera: **nueve numeros**, ni uno mas.
///
/// Es `Serialize`/`Deserialize` porque es lo que se guarda entre arranques: guardar **esto** y no los
/// 102 ficheros es lo que hace que la segunda apertura sea instantanea.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Real {
    /// Cuantas sesiones se leyeron.
    pub sesiones: usize,
    /// Cuantas lecturas (muestras) hay en total.
    pub muestras: usize,
    /// Cuantos intervalos entre latidos hay en total.
    pub rr: usize,
    /// La deriva media, en porcentaje. `None` si no habia ninguna sesion con muestras suficientes.
    pub deriva_media: Option<f64>,
    /// Los vatios por pulso, de media.
    pub eficiencia_media: Option<f64>,
    /// Cuantas sesiones se les escapa la deriva por encima del 10 %.
    pub escapados: usize,
    /// La media del RMSSD. **OJO**: medido durante el ejercicio, no en reposo (ver `docs/`).
    pub hrv_medio: Option<f64>,
    /// La frecuencia mas alta vista **en los datos**.
    pub maxima_vista: Option<u16>,
    /// **Los ficheros que NO se pudieron leer**, con su motivo.
    ///
    /// ESTE CAMPO ES LA HONESTIDAD DEL MODULO, y sin el estaria mintiendo: si de 102 ficheros se leen
    /// 99, todos los promedios salen de 99, y **nada en la pantalla lo diria**. Un dato que falta hay
    /// que decirlo. Antes esto se tiraba a la basura en el binario de la ventana.
    pub no_leidos: Vec<(String, String)>,
}

impl Real {
    /// ¿Hay algo que analizar?
    pub fn hay_datos(&self) -> bool {
        self.sesiones > 0
    }

    /// ¿Se leyo TODO lo que habia?
    pub fn esta_completo(&self) -> bool {
        self.no_leidos.is_empty()
    }
}

/// Calcula el resultado a partir de las sesiones ya leidas.
///
/// Se separa de `de_carpeta` a proposito: asi **se puede probar sin tocar el disco**, que es lo que
/// permite comprobar los casos raros (una carpeta vacia, una sesion sin pulsos) sin fabricar ficheros.
pub fn de_lecturas(ficheros: &[(String, Lectura)], no_leidos: Vec<(String, String)>) -> Real {
    let mut r = Real {
        sesiones: ficheros.len(),
        no_leidos,
        ..Real::default()
    };
    if ficheros.is_empty() {
        return r;
    }

    let mut derivas: Vec<f64> = Vec::new();
    let mut efs: Vec<f64> = Vec::new();
    let mut hrvs: Vec<f64> = Vec::new();

    for (_, l) in ficheros {
        r.muestras += l.sesion.muestras.len();
        r.rr += l.sesion.intervalos_rr_ms.len();

        if let Some(m) = l.declarado.pulso_maximo.valor() {
            r.maxima_vista = Some(r.maxima_vista.map_or(m, |x: u16| x.max(m)));
        }

        // La deriva: solo cuenta si hay muestras bastantes para que signifique algo. El 20 no es un
        // numero elegido aqui: es el que ya usaba el motor, y se respeta.
        let d = patron::deriva(&l.sesion);
        if d.muestras >= 20 {
            derivas.push(d.porcentaje());
            if d.porcentaje() > 10.0 {
                r.escapados += 1;
            }
        }

        let e = patron::eficiencia(&l.sesion);
        if e.valor > 0.0 {
            efs.push(e.valor);
        }

        // El HRV: solo si hay latidos bastantes. Un RMSSD con pocos latidos no es un dato, es ruido.
        let h = hrv::calcular(hrv::intervalos_de(&l.sesion));
        if h.latidos >= 120 {
            hrvs.push(h.rmssd.valor);
        }
    }

    let media = |v: &[f64]| -> Option<f64> {
        if v.is_empty() {
            None
        } else {
            Some(v.iter().sum::<f64>() / v.len() as f64)
        }
    };
    r.deriva_media = media(&derivas);
    r.eficiencia_media = media(&efs);
    r.hrv_medio = media(&hrvs);
    r
}

/// Lee la carpeta entera y calcula. **Esto es lo que tarda 5,50 s**, y por eso hay memoria.
pub fn de_carpeta(carpeta: &Path) -> Real {
    let (ficheros, no_leidos) = fit::leer_carpeta(carpeta);
    de_lecturas(&ficheros, no_leidos)
}

/// La frecuencia maxima que se puede defender: la del aparato **solo si el usuario no la declaro**.
///
/// POR QUE ESTA REGLA ES UNA FUNCION Y NO UN `if` SUELTO: es una decision del dueño (2026-10-13) y
/// tiene que estar **en un solo sitio**, dicha una vez. El aparato aporta lo que el usuario no puede
/// saber; el usuario aporta lo que el aparato no puede saber. **Cuando los dos hablan del mismo dato,
/// manda el que lo declaro a proposito.**
pub fn maxima_que_manda(declarada: &Dato<u16>, vista: Option<u16>) -> Dato<u16> {
    match (declarada.valor(), vista) {
        (Some(d), _) => Dato::Hay(d),
        (None, Some(v)) => Dato::Hay(v),
        (None, None) => Dato::Falta,
    }
}
