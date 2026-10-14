//! **Lo que la aplicacion ENSEÑA, calculado aparte de la ventana.**
//!
//! POR QUE EXISTE ESTE MODULO, y es la critica mas dura que se le hizo al proyecto:
//!
//! > *«Las pruebas comprueban **el motor** y la **geometria**. **Cero pruebas comprueban que un
//! > usuario pueda hacer algo.**»*
//!
//! Y era verdad, y el motivo era de diseño: **lo que la pantalla enseña se calculaba dentro de
//! `ventana.rs`**, asi que comprobarlo exigia **abrir una ventana**. Y abrir una ventana para
//! comprobar algo esta prohibido en este proyecto por un motivo que ya costo caro: una vez se abrio,
//! sin querer, en la pantalla del dueño.
//!
//! LA SEPARACION, dicha en una linea: **aqui se decide QUE se enseña; en `ventana.slint` se decide
//! COMO se ve.** `ventana.rs` solo une las dos cosas.
//!
//! Y `lo_que_ensena` es una **funcion pura**: los mismos datos dan siempre el mismo resultado, no
//! toca el disco, no toca Slint y no abre nada. Eso es lo que la hace comprobable **siempre**.
//!
//! LA REGLA QUE GOBIERNA TODO LO DE AQUI, y es la promesa de este motor: **un numero sin su margen y
//! sin su fuente no se enseña.** Enseñar `169` a secas es una promesa; enseñar `169±3` y decir de
//! donde sale es un dato.

use crate::analisis::Real;
use crate::datos::Persona;
use crate::umbral;

/// **Todo lo que va a la pantalla, en cadenas ya escritas.**
///
/// Es `PartialEq` a proposito, y no es un detalle: la prueba de «volver otro dia» necesita comparar
/// dos estados. Si no se pudiera comparar, habria que comparar campo a campo, **y el dia que se
/// añada un campo nuevo la comparacion se olvidaria** -- y la prueba seguiria pasando sin comprobar
/// nada. Es el fallo que la norma describe (AU-73): lo que existe dos veces acaba divergiendo.
#[derive(Debug, Clone, PartialEq)]
pub struct Ensenado {
    /// **Si no se puede diagnosticar, esto es `false`** y las cifras y las zonas van VACIAS.
    ///
    /// ESTE CAMPO ES EL ARREGLO DEL FALLO MAS GRAVE que tenia la aplicacion: enseñaba `169±3` y
    /// «Precision alta» con un umbral **escrito a mano en el codigo**, y a la vez decía «Sin datos
    /// todavia». **Se contradecia**, y los numeros parecian buenos. Un motor que se inventa un numero
    /// es peor que uno que no arranca: el que no arranca, se nota.
    pub puede_diagnosticar: bool,
    /// Lo que FALTA, en una o dos lineas, **y diciendo donde se rellena**. Vacio si no falta nada.
    pub que_falta: String,
    /// **Las cuatro cifras**, ya escritas. VACIO si no se puede diagnosticar.
    pub cifras: Vec<String>,
    /// Que es cada cifra. Siempre cuatro, si hay cifras.
    pub etiquetas: Vec<String>,
    /// Las zonas: **nombre, TRAMO, y el nivel de la barra** (0 a 100). VACIO si no hay umbral.
    pub zonas: Vec<(String, String, i32)>,
    /// De donde sale el dato: medido, estimado, o de poblacion. Es lo que lo hace creible.
    pub precision: String,
    /// Lo que se deduce de los datos del aparato: la deriva, la eficiencia.
    pub aviso: String,
    /// El recuento de lo que hay: sesiones, lecturas, intervalos.
    pub vacio: String,
}

impl Ensenado {
    /// **Nada que enseñar, y dicho.** Es el estado del primer arranque.
    fn sin_nada(que_falta: String, vacio: String) -> Self {
        Self {
            puede_diagnosticar: false,
            que_falta,
            cifras: Vec::new(),
            etiquetas: Vec::new(),
            zonas: Vec::new(),
            precision: String::new(),
            aviso: String::new(),
            vacio,
        }
    }
}

/// Escribe un numero con coma decimal, como se escribe en español.
///
/// NO se usa el formateo del idioma del equipo a proposito, y esto merece explicacion porque parece
/// lo contrario de lo que dice la norma: **un informe de entrenamiento se lee siempre en español**,
/// y el dia que se traduzca la aplicacion, esto se cambia en UN sitio. Lo que no se puede es dejar
/// que el numero salga distinto en cada equipo **sin que nadie lo haya decidido**.
fn num(v: f64, decimales: usize) -> String {
    format!("{v:.decimales$}").replace('.', ",")
}

/// **EL TRAMO de una zona, no su suelo.**
///
/// ESTE DEFECTO YA SALIO UNA VEZ, y por eso esta aislado en su propia funcion: la primera version
/// devolvia solo el suelo de cada zona, y entonces «recuperacion» y «resistencia» salian con **el
/// mismo numero**, porque las dos empiezan donde acaba la anterior. **Un informe que dice dos veces
/// 137 no se entiende**, y el fallo era de lectura, no de calculo.
fn tramo(desde: f64, hasta: f64) -> String {
    if (hasta - desde).abs() < 1.0 {
        // Una zona que empieza y acaba en el mismo sitio se dice «hasta», que es lo que significa.
        format!("hasta {:.0}", hasta)
    } else {
        format!("{:.0} – {:.0}", desde, hasta)
    }
}

/// **Lo que la ventana va a enseñar.** Ni una cuenta mas: esta es toda.
///
/// `real` es `None` cuando no hay datos del aparato, que **no es lo mismo que haberlos leido y que
/// esten vacios**. La diferencia importa: sin datos no se puede hablar de deriva, y hay que decirlo.
pub fn lo_que_ensena(p: &Persona, real: Option<&Real>) -> Ensenado {
    let u = umbral::umbral(p);
    let zonas = umbral::zonas_del_umbral(p);

    // ¿HAY ALGO QUE DECIR? La pregunta se contesta con el umbral, que es de donde sale todo.
    //
    // Y SE CONTESTA CON EL UMBRAL Y NO CON «hay perfil», y el motivo importa: un perfil con el nombre
    // y el peso pero **sin umbral ni edad** no permite decir nada de las zonas. Lo que decide no es
    // «tengo datos», es «puedo calcular».
    let referencia = u.referencia;
    let puede = referencia != umbral::Referencia::DePoblacion;

    let recuento = match real {
        Some(r) => format!(
            "{} sesiones · {} lecturas · {} intervalos de HRV",
            r.sesiones, r.muestras, r.rr
        ),
        None => "Sin datos de entrenamiento todavía.".to_string(),
    };

    if !puede {
        return Ensenado::sin_nada(que_falta(p), recuento);
    }

    // --- LAS CUATRO CIFRAS ---
    let deriva = real.and_then(|r| r.deriva_media);
    let eficiencia = real.and_then(|r| r.eficiencia_media);
    let las_cifras: Vec<String> = vec![
        format!("{:.0}±{:.0}", u.rango.valor, u.rango.margen),
        format!("{:.0}", p.pulso_maximo_medido.valor().unwrap_or(0)),
        eficiencia.map_or("—".to_string(), |e| num(e, 2)),
        deriva.map_or("—".to_string(), |d| {
            format!("{}{}", if d >= 0.0 { "+" } else { "-" }, num(d.abs(), 1))
        }),
    ];
    let etiquetas: Vec<String> = vec![
        "umbral (pulsaciones)".to_string(),
        "frecuencia máxima".to_string(),
        "vatios por pulso".to_string(),
        "deriva %".to_string(),
    ];

    // --- LAS ZONAS ---
    //
    // El nivel es el ancho de la barra, en proporcion a la zona mas alta. **Esto es DIBUJAR, no
    // calcular**: es la unica aritmetica que se permite aqui, y esta dicho para que nadie la confunda
    // con un resultado del motor.
    let tope = zonas.iter().map(|(_, _, h)| h.valor).fold(f64::MIN, f64::max);
    let las_zonas: Vec<(String, String, i32)> = zonas
        .iter()
        .map(|(z, desde, hasta)| {
            let nivel = if tope > 0.0 {
                (hasta.valor / tope * 100.0).round() as i32
            } else {
                0
            };
            (
                z.nombre().to_string(),
                tramo(desde.valor, hasta.valor),
                nivel,
            )
        })
        .collect();

    // --- LA PRECISION, Y DE DONDE SALE ---
    let texto_precision = match referencia {
        umbral::Referencia::UmbralMedido => format!(
            "Precisión {}: las zonas salen del {}, que es un dato medido.",
            u.rango.como_va().nombre(),
            referencia.nombre()
        ),
        _ => format!(
            "Precisión {}: las zonas salen del {}, que es una estimación.",
            u.rango.como_va().nombre(),
            referencia.nombre()
        ),
    };

    // --- LO QUE SE DEDUCE DE LOS DATOS DEL APARATO ---
    let texto_aviso = match (real, deriva) {
        (Some(r), Some(d)) => format!(
            "{} % de deriva media en {} sesiones: {} se le escapan. Sacado de {} lecturas.",
            num(d, 1),
            r.sesiones,
            r.escapados,
            r.muestras
        ),
        // SIN SESIONES, EL AVISO NO NOMBRA LO QUE NO HAY.
        //
        // Antes decía «Sin datos todavía: la deriva y la eficiencia aparecerán aquí», y estaba mal
        // por dos motivos. El primero, de norma (AU-44 y AU-45): **la pantalla no se explica**, y
        // anunciar lo que aparecerá es explicarse. El segundo, y es el que lo cazó: **nombra dos
        // cosas que no existen** -- sin sesiones no hay deriva ni eficiencia, y un aviso que las
        // menciona hace pensar que hay algo que mirar. Lo encontró `tests/recorrido.rs`.
        _ => "Sin datos de entrenamiento todavía.".to_string(),
    };

    Ensenado {
        puede_diagnosticar: true,
        que_falta: String::new(),
        cifras: las_cifras,
        etiquetas,
        zonas: las_zonas,
        precision: texto_precision,
        aviso: texto_aviso,
        vacio: recuento,
    }
}

/// **Que falta, en una o dos lineas, y donde se rellena.**
///
/// La norma del proyecto (AU-44 y AU-45): la pantalla **no se explica**. Un vacio se dice en UNA
/// linea, y **diciendo que hacer**, no dando pena. La pantalla mide 411 px: un parrafo ahi no se lee,
/// se salta.
///
/// Y se nombran **los datos que de verdad desbloquean el calculo**, no todos los que faltan. Decir
/// «faltan 8 datos» cuando con uno ya funciona es asustar al usuario sin motivo.
fn que_falta(p: &Persona) -> String {
    let tiene_umbral = p.umbral_lactato.valor().is_some();
    let tiene_edad = p.edad.valor().is_some();

    if !tiene_umbral && !tiene_edad {
        return "Faltan tus datos: con la edad y el umbral ya se calculan las zonas. \
                Se rellenan en Datos."
            .to_string();
    }
    if !tiene_umbral {
        return "Falta el umbral de lactato, que es el dato que manda. Se rellena en Datos."
            .to_string();
    }
    // Tiene umbral: entonces SI se puede diagnosticar y esta funcion no se llama. Pero si algun dia
    // se llama, es mejor decir algo cierto que cadena vacia.
    "No hay datos suficientes para calcular las zonas. Se rellenan en Datos.".to_string()
}
