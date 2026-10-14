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
use crate::avisos::fecha_de_epoch;
use crate::datos::Persona;
use crate::umbral;

/// **Una cifra CON SU ETIQUETA DENTRO**, y con la posibilidad de **no existir**.
///
/// CONTRATO (`docs/CONTRATO-cara-datos.md`, contratos 2 y 6). Los dos motivos, porque los dos ya
/// costaron un fallo:
///
/// 1. **`dato` es `Option` a proposito** (AU-88). Antes la cifra que no existia viajaba como la cadena
///    `«—»` (`cara.rs`, version del 2026-10-14), y eso obliga a la pantalla a **interpretar un
///    simbolo**: si el usuario escribe un guion como su deriva, la pantalla no puede distinguirlo.
///    Y una **cadena vacia tampoco vale**: «vacio» y «no medido» se dibujan igual y significan cosas
///    distintas. `None` es «esta cifra no existe», y es lo unico que autoriza a **sacar la tarjeta del
///    arbol**. El guion esta **prohibido** (AU-91 y el anexo A, cambio 7, de la especificacion).
/// 2. **La etiqueta viaja dentro**, y no en una lista paralela (AU-73): con dos listas, el dia que una
///    cifra no existe y se quita, **las etiquetas se desalinean** y la pantalla mentiria sin que
///    ninguna prueba lo viera. Es el mismo fallo que el `#[derive(PartialEq)]` de `Ensenado` evita.
#[derive(Debug, Clone, PartialEq)]
pub struct Cifra {
    /// Que es esta cifra. Son cuatro, siempre las mismas, y en el mismo orden.
    pub etiqueta: String,
    /// El numero, ya escrito. **`None` = esta cifra NO EXISTE**; `Some("")` = existe y esta vacia.
    pub dato: Option<String>,
}

impl Cifra {
    /// ¿Esta cifra existe? Es lo que la pantalla pregunta antes de dibujar la tarjeta.
    pub fn existe(&self) -> bool {
        self.dato.is_some()
    }
}

/// **Una zona del entrenamiento, en pulsaciones y con su margen.**
///
/// CONTRATO 1. Antes esto se convertia a TEXTO dentro de `cara.rs` (`tramo`, mas abajo) y **los numeros
/// se tiraban**, asi que la pantalla no podia pintar una banda sobre el eje: **el dato existia y se
/// perdia al formatearlo**. Ahora viaja crudo, y el texto se sigue montando aparte para la interfaz de
/// hoy.
#[derive(Debug, Clone, PartialEq)]
pub struct ZonaEnsenada {
    /// El nombre, tal como se enseña: «recuperación», «resistencia»…
    pub nombre: String,
    /// Donde empieza la zona, en pulsaciones.
    pub inicio_ppm: f64,
    /// Donde acaba. **Puede ser igual al inicio**, y entonces la zona es un TECHO, no un tramo: es el
    /// caso de la recuperacion, que empieza y acaba en el mismo sitio.
    pub fin_ppm: f64,
    /// **El margen del inicio, CON el inicio.** Sin esto el numero seria una promesa: la regla que
    /// gobierna todo este modulo es que **un numero sin su margen no se enseña**.
    pub margen_inicio_ppm: f64,
    /// El margen del fin, por el mismo motivo.
    pub margen_fin_ppm: f64,
}

/// **El eje del grafico**: de donde a donde van las zonas, en pulsaciones.
///
/// CONTRATO 3. Sale de los DATOS, no de la pantalla, y el motivo es que **la escala es una
/// afirmacion**: si cada pantalla eligiera la suya, la misma persona veria dos graficos distintos y
/// ninguno de los dos seria el suyo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eje {
    /// La minima de las cinco zonas.
    pub inicio_ppm: f64,
    /// La maxima de las cinco zonas.
    pub fin_ppm: f64,
}

impl Eje {
    /// **Un eje que no vale no se pinta.** Sin zonas no hay escala posible, y inventarse una seria
    /// enseñar un grafico que no es de nadie.
    pub fn es_valido(&self) -> bool {
        self.inicio_ppm.is_finite() && self.fin_ppm.is_finite() && self.fin_ppm > self.inicio_ppm
    }
}

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
    ///
    /// SE CONSERVA POR LA INTERFAZ DE HOY. El texto de UNA frase que pide la Fase 2 es
    /// [`que_falta_corto`](Self::que_falta_corto).
    pub que_falta: String,
    /// **El mismo «falta», en UNA frase y UNA linea**, que es lo que la pantalla enseña (AU-44/AU-45:
    /// la pantalla no se explica). Vacio si no falta nada.
    pub que_falta_corto: String,
    /// **Las cuatro cifras**, ya escritas. VACIO si no se puede diagnosticar.
    ///
    /// SE CONSERVA POR LA INTERFAZ DE HOY. Lo que la Fase 2 consume es
    /// [`cifras_atadas`](Self::cifras_atadas).
    pub cifras: Vec<String>,
    /// Que es cada cifra. Siempre cuatro, si hay cifras.
    ///
    /// SE CONSERVA POR LA INTERFAZ DE HOY: son **dos listas paralelas**, y es justo el defecto que
    /// AU-73 describe. En la Fase 2 se borra esta y la de arriba.
    pub etiquetas: Vec<String>,
    /// **Las cuatro cifras CON SU ETIQUETA** (contratos 2 y 6). VACIO si no se puede diagnosticar.
    ///
    /// Es la lista que sustituye a `cifras` + `etiquetas`, y la que permite **sacar del arbol** la
    /// tarjeta de una cifra que no existe.
    pub cifras_atadas: Vec<Cifra>,
    /// Las zonas: **nombre, TRAMO, y el nivel de la barra** (0 a 100). VACIO si no hay umbral.
    ///
    /// SE CONSERVA POR LA INTERFAZ DE HOY, que indexa las tres columnas. **El nivel 0-100 es DIBUJAR,
    /// no calcular**: no entra en ningun contrato. Lo que la Fase 2 consume es
    /// [`zonas_ppm`](Self::zonas_ppm).
    pub zonas: Vec<(String, String, i32)>,
    /// **Las cinco zonas EN PULSACIONES y con su margen** (contrato 1). VACIO si no hay umbral.
    ///
    /// Esto es lo que permite pintar las bandas sobre el eje: **el numero que antes se tiraba al
    /// formatearlo**.
    pub zonas_ppm: Vec<ZonaEnsenada>,
    /// **El rango del eje** (contrato 3): la minima y la maxima de las cinco zonas, en pulsaciones.
    /// Sin zonas, un eje que no vale (y `es_valido()` lo dice).
    pub eje: Eje,
    /// De donde sale el dato: medido, estimado, o de poblacion. Es lo que lo hace creible.
    ///
    /// SE CONSERVA: **el parrafo completo va a la AYUDA** (es donde AU-44/AU-45 lo mandan). La frase
    /// que va a la pantalla es [`precision_corta`](Self::precision_corta).
    pub precision: String,
    /// **La precision en UNA frase y UNA linea** (contrato 4): lo que la pantalla enseña, con el
    /// icono de ayuda al lado que abre el parrafo de arriba.
    pub precision_corta: String,
    /// Lo que se deduce de los datos del aparato: la deriva, la eficiencia.
    pub aviso: String,
    /// El recuento de lo que hay: sesiones, lecturas, intervalos.
    pub vacio: String,
    /// **El periodo que cubre el analisis**, en UNA linea y ya escrita. Vacio si no se sabe.
    ///
    /// POR QUE ES UN CAMPO Y NO SE DEJA PARA LA VENTANA: porque la ventana **no calcula**. Si el
    /// periodo se montara en Slint a partir de dos numeros, el dia que cambie el formato habria que
    /// tocar el .slint, y la pantalla podria mentir sin que una prueba lo viera.
    pub periodo: String,
}

impl Ensenado {
    /// **Nada que enseñar, y dicho.** Es el estado del primer arranque.
    fn sin_nada(que_falta: String, vacio: String, periodo: String) -> Self {
        Self {
            puede_diagnosticar: false,
            que_falta_corto: que_falta_corto(&que_falta),
            que_falta,
            cifras: Vec::new(),
            etiquetas: Vec::new(),
            cifras_atadas: Vec::new(),
            zonas: Vec::new(),
            zonas_ppm: Vec::new(),
            // SIN ZONAS NO HAY EJE. Y un eje de ceros seria peor que ninguno: una escala valida,
            // dibujada, de un grafico que no existe -- por eso `es_valido()` lo rechaza.
            eje: Eje { inicio_ppm: 0.0, fin_ppm: 0.0 },
            precision: String::new(),
            precision_corta: String::new(),
            aviso: String::new(),
            vacio,
            periodo,
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

    // EL PERIODO: la coherencia de las fechas, calculada UNA vez y ya escrita. La ventana no calcula.
    let periodo = periodo_de(real);

    if !puede {
        return Ensenado::sin_nada(que_falta(p), recuento, periodo);
    }

    // --- LAS CUATRO CIFRAS, Y CADA UNA CON SU ETIQUETA DENTRO (contratos 2 y 6) ---
    //
    // SE MONTAN COMO PARES, y las dos listas viejas se DERIVAN de los pares. Al reves --montar las
    // viejas y luego intentar emparejarlas-- seria volver a tener el dato dos veces, que es el defecto
    // que AU-73 describe y el que estas pruebas cazan.
    //
    // Y NINGUNA CIFRA SE RELLENA: la que no se midio va con `None`, que es «esta cifra no existe». NO
    // un cero (seria un dato falso) y NO el guion (prohibido, AU-91).
    let deriva = real.and_then(|r| r.deriva_media);
    let eficiencia = real.and_then(|r| r.eficiencia_media);
    let cifras_atadas: Vec<Cifra> = vec![
        Cifra {
            etiqueta: "umbral (pulsaciones)".to_string(),
            dato: Some(format!("{:.0}±{:.0}", u.rango.valor, u.rango.margen)),
        },
        // SIN MAXIMA MEDIDA, NO EXISTE. NO un cero: lo cazo la captura de pantalla del 2026-10-14 --
        // con umbral de lactato pero sin maxima, la tarjeta ensenaba «0», que es un dato FALSO. Es la
        // misma regla que ya seguian las dos cifras de abajo, y esta se habia quedado atras.
        Cifra {
            etiqueta: "frecuencia máxima".to_string(),
            dato: p.pulso_maximo_medido.valor().map(|v| format!("{v:.0}")),
        },
        Cifra {
            etiqueta: "vatios por pulso".to_string(),
            dato: eficiencia.map(|e| num(e, 2)),
        },
        Cifra {
            etiqueta: "deriva %".to_string(),
            dato: deriva.map(|d| {
                format!("{}{}", if d >= 0.0 { "+" } else { "-" }, num(d.abs(), 1))
            }),
        },
    ];
    // Las dos listas viejas, derivadas. `cifras` lleva el guion porque la UI de hoy lo espera: es la
    // ULTIMA copia del guion en el motor, y desaparece cuando la UI lea `cifras_atadas`.
    let cifras: Vec<String> = cifras_atadas.iter().map(cifra_vieja).collect();
    let etiquetas: Vec<String> = cifras_atadas.iter().map(|c| c.etiqueta.clone()).collect();

    // --- LAS ZONAS: EN PULSACIONES, CON SU MARGEN (contrato 1) ---
    let tope = zonas.iter().map(|(_, _, h)| h.valor).fold(f64::MIN, f64::max);
    let zonas_ppm: Vec<ZonaEnsenada> = zonas
        .iter()
        .map(|(z, desde, hasta)| ZonaEnsenada {
            nombre: z.nombre().to_string(),
            inicio_ppm: desde.valor,
            fin_ppm: hasta.valor,
            margen_inicio_ppm: desde.margen,
            margen_fin_ppm: hasta.margen,
        })
        .collect();

    // --- EL EJE (contrato 3): la escala sale de los DATOS ---
    //
    // Y NO del margen: el eje son los cinco tramos, y el margen viaja DENTRO de cada tramo. Ensanchar
    // el eje con los margenes haria que la escala cambiara al aportar datos que no mueven ninguna
    // zona, y la escala es una afirmacion: no puede bailar por un motivo que el usuario no ve.
    let eje = eje_de(&zonas_ppm);

    // --- LAS ZONAS PARA LA INTERFAZ DE HOY, con el nivel de la barra ---
    //
    // El nivel es el ancho de la barra, en proporcion a la zona mas alta. **Esto es DIBUJAR, no
    // calcular**: es la unica aritmetica que se permite aqui, y esta dicho para que nadie la confunda
    // con un resultado del motor. Y **no entra en ningun contrato**: se conserva por la UI de hoy.
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

    // --- LA PRECISION (contrato 4): UNA frase a la pantalla, el parrafo a la AYUDA ---
    let (precision_corta, texto_precision) = precision_de(referencia, u.rango.como_va());

    // --- LO QUE SE DEDUCE DE LOS DATOS DEL APARATO ---
    let texto_aviso = match (real, deriva) {
        (Some(r), Some(d)) => format!(
            "{} % de deriva media en {} sesiones: {} se le escapan. Sacado de {} lecturas.",
            num(d, 1),
            r.sesiones,
            r.escapados,
            r.muestras
        ),
        // SIN DERIVA, EL AVISO VA VACIO, y la ventana no lo dibuja (AU-07/AU-24).
        //
        // Antes decía «Sin datos todavía: la deriva y la eficiencia aparecerán aquí», y estaba mal
        // por dos motivos. El primero, de norma (AU-44 y AU-45): **la pantalla no se explica**, y
        // anunciar lo que aparecerá es explicarse. El segundo, y es el que lo cazó: **nombra dos
        // cosas que no existen** -- sin sesiones no hay deriva ni eficiencia, y un aviso que las
        // menciona hace pensar que hay algo que mirar. Lo encontró `tests/recorrido.rs`.
        //
        // Y ADEMAS SE REPETIA: con cero sesiones, `vacio` ya dice «Sin datos de entrenamiento
        // todavía.», y el aviso decia LO MISMO. Dos veces el mismo mensaje no es mas claro: es ruido.
        // Lo cazo la captura del 2026-10-14 (ver docs/ESTUDIO-interfaz-android-2026-10-14.md).
        _ => String::new(),
    };

    Ensenado {
        puede_diagnosticar: true,
        // CON SUS DATOS NO FALTA NADA, y las dos formas de decirlo van vacias: si queda una frase
        // suelta en la pantalla, la pantalla esta pidiendo algo que ya tiene.
        que_falta: String::new(),
        que_falta_corto: String::new(),
        cifras,
        etiquetas,
        cifras_atadas,
        zonas: las_zonas,
        zonas_ppm,
        eje,
        precision: texto_precision,
        precision_corta,
        aviso: texto_aviso,
        vacio: recuento,
        periodo,
    }
}

/// **La cifra en el formato VIEJO**, con el guion para la que no existe.
///
/// ES LA ULTIMA COPIA DEL GUION EN EL MOTOR, y esta aislada a proposito: la interfaz de hoy lo
/// consume, y **desaparece en cuanto la interfaz lea `cifras_atadas`** (Fase 2). El guion de 26 px
/// esta prohibido como texto de pantalla (AU-91); lo que queda aqui es un puente, no un formato.
fn cifra_vieja(c: &Cifra) -> String {
    c.dato.clone().unwrap_or_else(|| "\u{2014}".to_string())
}

/// **El eje, de los cinco tramos** (contrato 3): la minima y la maxima, en pulsaciones.
///
/// Sin zonas devuelve un eje que **no vale**, y `Eje::es_valido()` lo dice: un eje de ceros seria una
/// escala valida de un grafico que no existe, y eso es peor que no dibujarlo.
fn eje_de(zonas: &[ZonaEnsenada]) -> Eje {
    if zonas.is_empty() {
        return Eje { inicio_ppm: 0.0, fin_ppm: 0.0 };
    }
    let inicio = zonas.iter().map(|z| z.inicio_ppm).fold(f64::MAX, f64::min);
    let fin = zonas.iter().map(|z| z.fin_ppm).fold(f64::MIN, f64::max);
    Eje { inicio_ppm: inicio, fin_ppm: fin }
}

/// **La precision, de dos formas: la frase de la pantalla y el parrafo de la AYUDA** (contrato 4).
///
/// POR QUE DOS, y no una recortada: AU-44 y AU-45 dicen que **la pantalla no se explica**, y el
/// encargo del dueño lo cierra: **una linea corta con su icono, y el detalle entero a la ayuda**. Si
/// se mandara solo la frase, el detalle se perderia; si se mandara solo el parrafo, la pantalla
/// volveria a tener el defecto 1 de la auditoria.
///
/// Las dos dicen LO MISMO, y por eso salen de la MISMA funcion: una es el resumen de la otra, y no
/// pueden divergir porque no hay dos sitios donde escribirlas.
fn precision_de(referencia: umbral::Referencia, como_va: crate::rango::Precision) -> (String, String) {
    let de_donde = match referencia {
        umbral::Referencia::UmbralMedido => "el umbral está medido",
        _ => "el umbral viene de una estimación",
    };
    let corta = format!("Precisión {}: {}.", como_va.nombre(), de_donde);
    let parrafo = match referencia {
        umbral::Referencia::UmbralMedido => format!(
            "Precisión {}: las zonas salen del {}, que es un dato medido.",
            como_va.nombre(),
            referencia.nombre()
        ),
        _ => format!(
            "Precisión {}: las zonas salen del {}, que es una estimación.",
            como_va.nombre(),
            referencia.nombre()
        ),
    };
    (corta, parrafo)
}

/// **«Lo que falta», recortado a UNA frase** (contrato 5), para la linea de la pantalla.
///
/// SE RECORTA, NO SE REESCRIBE: la frase sale del texto largo quitando las explicaciones, y el texto
/// largo sigue existiendo en `que_falta`. Y **lo que NO se quita nunca es «Se rellenan en Datos»**:
/// un aviso que dice que falta algo y no dice donde se arregla es una queja, no una ayuda.
fn que_falta_corto(largo: &str) -> String {
    match largo {
        // Los tres casos de `que_falta`, uno por uno. Si mañana se añade un cuarto, el `_` lo deja
        // entero y la prueba de UNA linea lo caza -- que es lo que se quiere: que se decida, no que
        // se cuele.
        t if t.starts_with("Faltan tus datos") => {
            "Faltan la edad y el umbral. Se rellenan en Datos.".to_string()
        }
        t if t.starts_with("Falta el umbral") => "Falta el umbral. Se rellena en Datos.".to_string(),
        t if t.is_empty() => String::new(),
        // Y el resto, a UNA linea: se corta en el primer punto y se deja una sola frase.
        t => {
            let primera = t.split(". ").next().unwrap_or(t).trim_end_matches('.');
            format!("{primera}.")
        }
    }
}

/// **El periodo que cubre el analisis, en UNA linea.**
///
/// Recibe lo que ya se sabe (`Real.desde`/`hasta`) y devuelve el texto listo. NO inventa fechas: si hay
/// sesiones pero ninguna trae fecha, lo dice; y si no hay sesiones, no dice nada (el recuento ya habla
/// por si solo). Como todo lo de este modulo, es puro: los mismos datos dan el mismo texto.
fn periodo_de(real: Option<&Real>) -> String {
    let Some(r) = real.filter(|r| r.hay_datos()) else {
        return String::new();
    };
    match (r.desde, r.hasta) {
        (Some(desde), Some(hasta)) => {
            format!("Análisis del {} al {}", fecha_de_epoch(desde), fecha_de_epoch(hasta))
        }
        _ => "Análisis sin fechas en los ficheros".to_string(),
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
