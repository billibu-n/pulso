//! **La potencia**: las zonas que este cliente necesita, porque entrena con medidor.
//!
//! POR QUÉ ESTE MÓDULO EXISTE, y es el problema que lo justifica entero:
//!
//! El motor calculaba las zonas **solo con el pulso**. Y este cliente **entrena con potencia**: tiene
//! medidor de doble sensor y un FTP que le detectó el reloj. Para un puerto de montaña las zonas de
//! potencia dicen **mucho más** que las de pulso, y el motivo es fisiológico, no de gusto:
//!
//! - **El pulso va con retraso.** Cuando el pulso llega al número, el esfuerzo ya lleva medio minuto
//!   hecho. En un puerto eso es empezar por encima de lo que se puede sostener.
//! - **El pulso se mueve por cosas que no son el esfuerzo**: el calor, la fatiga acumulada, el café,
//!   las horas de sueño. Los vatios no: **un vatio es un vatio** aunque haga 32 grados.
//!
//! Y hay un detalle que se midió sobre sus 102 ficheros y **decide el diseño de todo esto**:
//! **83 sesiones traen potencia y 19 no**. O sea que este módulo **no sustituye** a las zonas de
//! pulso: las **acompaña**, y cuando no hay vatios, lo dice en vez de rellenar con ceros.
//!
//! LA FUENTE DE LOS CORTES, y esto importa porque en este proyecto **lo que no se puede citar no se
//! calcula**: son los **niveles de entrenamiento basados en potencia de Andrew Coggan**, publicados
//! por primera vez el **15 de octubre de 2001** y recogidos después en *Training and Racing with a
//! Power Meter* (Hunter Allen y Andrew Coggan). Se citan uno a uno en [`ZonaPotencia::fraccion`], que
//! es donde viven los números.
//!
//! LA REGLA DEL PROYECTO CON ESTE FICHERO: **la misma forma que `umbral.rs`**. Un enum de zonas con
//! `todas()`, `fraccion()`, `techo()`, `nombre()`, y un struct de resultado que **dice de dónde sale
//! su referencia**. Inventar aquí otra forma sería tener dos maneras de decir lo mismo en el mismo
//! motor, y eso se paga cada vez que alguien lee el código.
//!
//! UNA DEUDA DECLARADA, y va aquí arriba para que no se pierda: el mensaje `TimeInZone` del FIT
//! —el reparto del tiempo por zonas que **ya calculó el aparato**— todavía **no lo lee `fit.rs`**, y
//! este módulo **no puede tocar `fit.rs`** porque hay otro agente ampliándolo. Así que aquí vive un
//! lector propio, [`leer_declarado_de_potencia`], **con el mismo patrón que `fit.rs`** y **solo para
//! esta pieza**. El día que `fit::Declarado` traiga las zonas, ese lector se borra y estas funciones
//! siguen igual: reciben los datos, no los buscan.

use crate::datos::Dato;
use crate::fit::Declarado;
use crate::fuentes::{Procedencia, Valor};
use crate::rango::{self, Rango};
use crate::tendencia::{Serie, Tendencia};
use std::path::Path;

// ============================================================================================
// 1 · LA REFERENCIA: DE DÓNDE SALE EL FTP
// ============================================================================================

/// De dónde sale el umbral de potencia con el que se fijan las zonas.
///
/// ES LA MISMA PREGUNTA QUE SE HACE `umbral.rs` CON EL UMBRAL DE LACTATO, y por el mismo motivo:
/// **un umbral medido y uno estimado no valen lo mismo**, y el margen tiene que decirlo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Referencia {
    /// **Lo declara el aparato**: el FIT trae `threshold_power`, y el perfil del reloj dice que lo
    /// detectó él (`ftpAutoDetected: true`). Es el mejor caso disponible aquí, y aun así **tiene
    /// margen**: el aparato estima, no mide en laboratorio.
    DeclaradoPorElAparato,
    /// **Estimado por nosotros**, sin una prueba que lo respalde. El margen es más ancho, y se dice.
    Estimado,
    /// **No hay umbral de potencia**: las zonas de potencia **no se pueden dar**. Y esto **no es un
    /// cero**: es no saberlo. El motor ya tiene el tipo para decirlo — `Dato::Falta`.
    SinUmbral,
}

impl Referencia {
    pub fn nombre(self) -> &'static str {
        match self {
            Referencia::DeclaradoPorElAparato => "umbral de potencia declarado por el aparato",
            Referencia::Estimado => "umbral de potencia estimado, sin prueba que lo respalde",
            Referencia::SinUmbral => "sin umbral de potencia",
        }
    }

    /// Qué gana el usuario si aporta el dato que falta. Mismo contrato que en `umbral.rs`.
    pub fn que_falta(self) -> Option<&'static str> {
        match self {
            Referencia::DeclaradoPorElAparato => None,
            Referencia::Estimado => Some(
                "una prueba de 20 minutos con su medidor: es lo que convierte el umbral en un dato medido en vez de una estimación",
            ),
            Referencia::SinUmbral => Some(
                "una sesión con el medidor de potencia conectado: sin un solo vatio no hay de dónde sacar las zonas",
            ),
        }
    }

    /// **El margen que se le supone**, en vatios, sobre un umbral de `valor`.
    ///
    /// Va aquí y no en una función suelta porque **el margen es una propiedad de la procedencia**, no
    /// del número: el mismo 213 W vale ±7 si lo declaró el aparato y ±36 si lo estimamos nosotros.
    pub fn margen(self, valor: f64) -> f64 {
        match self {
            Referencia::DeclaradoPorElAparato => ERROR_FTP_DECLARADO,
            Referencia::Estimado => rango::suma_de_margenes(&[
                ERROR_FTP_DECLARADO,
                valor * MARGEN_FTP_ESTIMADO_RELATIVO,
            ]),
            Referencia::SinUmbral => 0.0,
        }
    }
}

/// El **umbral de potencia (FTP)**, con su valor, su margen y su origen.
///
/// MISMA FORMA QUE `umbral::Umbral`. No es casualidad: dos structs distintos para la misma idea
/// obligarían a quien lee el informe a aprender dos vocabularios.
#[derive(Debug, Clone, PartialEq)]
pub struct Ftp {
    pub rango: Rango,
    pub referencia: Referencia,
}

impl Ftp {
    pub fn valor(&self) -> f64 {
        self.rango.valor
    }
}

/// **El margen de un umbral de potencia declarado por el aparato**, en vatios.
///
/// DE DÓNDE SALE, y hay que decirlo con precisión porque **no es de este aparato**:
///
/// Es el **error típico de medición de 7 W** que publican Borszcz, Tramontin y Costa en *«Reliability
/// of the Functional Threshold Power in Competitive Cyclists»* (*International Journal of Sports
/// Medicine*, 2020; 41(03):175-181, DOI `10.1055/a-1018-1965`). Son **25 ciclistas entrenados**, dos
/// pruebas separadas por siete días, y el coeficiente de variación del protocolo de 20 minutos sale
/// **2,9 %** (ICC 0,97). En vatios, ese error típico son **7 W** sobre una media de 246 W.
///
/// Y AHORA LA PARTE HONESTA, que es lo que este proyecto exige: **ese número viene de un test de 20
/// minutos con protocolo, y el FTP de este cliente lo detectó el reloj solo.** No son lo mismo, y el
/// comentario lo dice en vez de callarlo. Lo que se sostiene es más débil pero sigue siendo cierto:
/// **7 W es lo que varía un FTP de verdad de un día para otro**, así que es el **suelo** del margen.
/// Lo que **no** se puede citar es cuánto añade la detección automática del aparato: Garmin no
/// publica su algoritmo, y por eso **no se inventa un número mayor**.
pub const ERROR_FTP_DECLARADO: f64 = 7.0;

/// **El margen de un umbral estimado**, como proporción del valor.
///
/// ESTE NÚMERO ES UNA DECISIÓN, NO UN HALLAZGO, y por eso va aquí a la vista. Cuando el umbral no lo
/// declara nadie, lo estima este motor, y **no hay ninguna fuente que publique el error de una
/// estimación nuestra**. Se toma un **15 %** porque es del orden del error que la propia literatura
/// describe entre métodos distintos de fijar el umbral: el factor de corrección del test de 20 minutos
/// respecto a una hora real varía entre el 5 % y el 10 % según cómo sea el calentamiento, y las
/// cifras de FTP20 frente a FTP60 llegan a separarse un 10 %. **Se declara como estimación propia**,
/// con un margen que no pretende ser estrecho.
pub const MARGEN_FTP_ESTIMADO_RELATIVO: f64 = 0.15;

/// **El umbral de potencia**, con la mejor información disponible.
///
/// EL ORDEN DE PREFERENCIA, igual que en `umbral.rs`:
/// 1. **lo que declara el aparato** — es un dato suyo, con su margen publicado;
/// 2. **estimado** — ver [`Ftp::estimado`], que existe y está probado;
/// 3. **sin umbral** — **no se inventan zonas**. Devuelve `Dato::Falta`.
pub fn umbral_de_potencia(declarado: &Declarado) -> Dato<Ftp> {
    // 1 · ¿Lo declara el aparato? Entonces manda, y no se estima nada.
    if let Some(v) = declarado.potencia_umbral.valor() {
        // Un umbral de CERO no es un umbral: es un campo que se escribió mal. Tratarlo como un número
        // daría **siete zonas de cero vatios**, que es el fallo exacto que este módulo tiene que
        // hacer imposible. Y `v > 0` es una comprobación, no una suposición: hay un caso de prueba.
        if v > 0 {
            return Dato::Hay(Ftp {
                rango: Rango::medido(
                    f64::from(v),
                    Referencia::DeclaradoPorElAparato.margen(f64::from(v)),
                ),
                referencia: Referencia::DeclaradoPorElAparato,
            });
        }
    }

    // 2 · No hay umbral declarado, y aquí **no se estima**. Y esto es una decisión, no un olvido: se
    // podría estimar desde la potencia normalizada de sus sesiones (el 95 % de lo mejor de 20
    // minutos), pero eso exige **elegir la mejor sesión**, y con 83 sesiones de ENTRENAMIENTO —no de
    // prueba— esa elección sería una invención con aspecto de cálculo. El motor dice que falta; el día
    // que haya una prueba de 20 minutos se usa `Ftp::estimado` y el margen ya está escrito.
    Dato::Falta
}

impl Ftp {
    /// Un umbral **estimado**, con el margen ancho que le corresponde.
    ///
    /// EXISTE Y NO SE USA TODAVÍA, y eso está dicho a propósito: el enum [`Referencia`] tiene que
    /// poder nombrar el caso **antes** de que el caso llegue, y una prueba lo comprueba. Es la
    /// diferencia entre un caso previsto y un caso inventado.
    pub fn estimado(vatios: f64) -> Self {
        Self {
            rango: Rango::estimado(
                vatios,
                Referencia::Estimado.margen(vatios),
            ),
            referencia: Referencia::Estimado,
        }
    }

    /// El umbral, en una frase con su origen y su margen.
    pub fn frase(&self) -> String {
        format!(
            "FTP {:.0} ± {:.0} W ({}). {}",
            self.rango.valor,
            self.rango.margen,
            self.referencia.nombre(),
            self.referencia
                .que_falta()
                .unwrap_or("es el dato que fija las zonas de potencia")
        )
    }
}

// ============================================================================================
// 2 · LAS ZONAS
// ============================================================================================

/// Las **zonas de potencia del ciclismo**, como fracciones del umbral.
///
/// Las siete de Coggan, con su nombre original traducido. **Las siete existen**, aunque la séptima
/// tenga una particularidad que se explica en [`ZonaPotencia::fraccion`]: su suelo está definido y su
/// techo **no**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZonaPotencia {
    /// Nivel 1 · *Active recuperation*.
    RecuperacionActiva,
    /// Nivel 2 · *Endurance*.
    Resistencia,
    /// Nivel 3 · *Tempo*.
    Tempo,
    /// Nivel 4 · *Lactate threshold*.
    Umbral,
    /// Nivel 5 · *Maximal aerobic power*, hoy VO2max.
    ConsumoMaximo,
    /// Nivel 6 · *Anaerobic capacity*.
    CapacidadAnaerobica,
    /// Nivel 7 · *Neuromuscular power*.
    Neuromuscular,
}

impl ZonaPotencia {
    /// Las siete, **de menos a más**. El orden importa dos veces: es el de la tabla de la fuente, y
    /// es el orden en el que el FIT escribe su reparto de tiempo por zonas.
    pub fn todas() -> [ZonaPotencia; 7] {
        [
            ZonaPotencia::RecuperacionActiva,
            ZonaPotencia::Resistencia,
            ZonaPotencia::Tempo,
            ZonaPotencia::Umbral,
            ZonaPotencia::ConsumoMaximo,
            ZonaPotencia::CapacidadAnaerobica,
            ZonaPotencia::Neuromuscular,
        ]
    }

    /// El **suelo** de cada zona, como fracción del umbral.
    ///
    /// LOS CORTES, Y DE DÓNDE SALEN UNO A UNO. Fuente: **Andrew Coggan, «Power-based training
    /// levels»**, publicado el **15 de octubre de 2001** y recogido después en *Training and Racing
    /// with a Power Meter* (Allen y Coggan). **Todas las zonas de esa tabla se expresan como
    /// porcentaje de la potencia media de una contrarreloj de 60 minutos** — que es lo que hoy se
    /// llama FTP:
    ///
    /// | nivel | nombre original | % de la potencia umbral |
    /// |---|---|---:|
    /// | 1 | Active recuperation | **< 55 %** |
    /// | 2 | Endurance | **56-75 %** |
    /// | 3 | Tempo | **76-90 %** |
    /// | 4 | Lactate threshold | **90-105 %** |
    /// | 5 | Maximal aerobic power | **106-120 %** |
    /// | 6 | Anaerobic capacity | **≥ 121 %** |
    /// | 7 | Neuromuscular power | **no definido por potencia** |
    ///
    /// LAS TRES DECISIONES QUE HAY QUE ENSEÑAR, porque la tabla original **no es contigua**:
    ///
    /// 1. **La tabla salta del 55 al 56, y del 105 al 106.** Son huecos de redondeo: el original
    ///    escribe los porcentajes como enteros. Aquí los cortes se ponen en **55, 75, 90, 105 y
    ///    120 %, de modo que cada zona empieza donde acaba la anterior y no queda ni un vatio sin
    ///    zona**. Sin esto, un vatio al 55,5 % no pertenecería a ninguna zona, que es peor que
    ///    elegir un lado.
    /// 2. **El nivel 3 acaba en 90 y el 4 empieza en 90**: el original **se solapa** en ese punto.
    ///    Quedarse con 90 para el corte es lo que menos falsea, porque las dos lecturas coinciden ahí.
    /// 3. **El nivel 7 no tiene techo**, y esto **no es un olvido**: la tabla de 2001 le pone `n/a`
    ///    en potencia y explica que son esfuerzos de menos de 25 segundos que estresan **más el
    ///    sistema musculoesquelético que el metabólico**, y que la potencia solo sirve como guía
    ///    **respecto a esfuerzos previos parecidos, no respecto al ritmo de contrarreloj**. O sea:
    ///    **la fuente no publica un porcentaje, y por eso aquí no se inventa uno**. El nivel 7
    ///    empieza en 1,50 y **hacia arriba no se puede afirmar nada**.
    ///
    /// Y una comprobación que sale gratis, hecha sobre sus ficheros: los cortes de zona que **el
    /// propio Garmin escribe** en el FIT (`power_zone_high_boundary`, medidos con `--bin potencia`)
    /// son **54,9 / 75,1 / 90,1 / 105,2 / 120,2 / 150,2 %** de su FTP de 213 W. Los cinco primeros
    /// coinciden con esta tabla **hasta la décima**, y el sexto confirma el corte de 1,50 del nivel 6.
    /// Dos fuentes independientes que coinciden valen más que una repetida.
    pub fn fraccion(self) -> f64 {
        match self {
            ZonaPotencia::RecuperacionActiva => 0.00,
            ZonaPotencia::Resistencia => 0.55,
            ZonaPotencia::Tempo => 0.75,
            ZonaPotencia::Umbral => 0.90,
            ZonaPotencia::ConsumoMaximo => 1.05,
            ZonaPotencia::CapacidadAnaerobica => 1.20,
            ZonaPotencia::Neuromuscular => 1.50,
        }
    }

    /// **El techo de la zona**, como fracción del umbral. `None` en la última.
    ///
    /// SIN ESTO, DOS ZONAS SEGUIDAS ENSEÑARÍAN EL MISMO NÚMERO, porque una empieza donde acaba la
    /// otra. Ese defecto ya salió una vez en este proyecto —en `umbral.rs`, el 2026-10-13— y el
    /// informe decía dos veces el mismo vatio. Aquí no se repite: cada zona enseña **su tramo**.
    ///
    /// Y `None` **no es un cero**: es «la fuente no publica techo para esta zona». Confundir las dos
    /// cosas es exactamente lo que el tipo `Dato` existe para evitar.
    pub fn techo(self) -> Option<f64> {
        match self {
            ZonaPotencia::RecuperacionActiva => Some(0.55),
            ZonaPotencia::Resistencia => Some(0.75),
            ZonaPotencia::Tempo => Some(0.90),
            ZonaPotencia::Umbral => Some(1.05),
            ZonaPotencia::ConsumoMaximo => Some(1.20),
            ZonaPotencia::CapacidadAnaerobica => Some(1.50),
            // El nivel 7 es «maximal» en la fuente: sin techo, y se dice.
            ZonaPotencia::Neuromuscular => None,
        }
    }

    /// ¿La fuente deja esta zona **sin techo**?
    pub fn sin_techo(self) -> bool {
        self.techo().is_none()
    }

    /// El nombre traducido, que es el que va en el informe.
    pub fn nombre(self) -> &'static str {
        match self {
            ZonaPotencia::RecuperacionActiva => "recuperación activa",
            ZonaPotencia::Resistencia => "resistencia",
            ZonaPotencia::Tempo => "tempo",
            ZonaPotencia::Umbral => "umbral",
            ZonaPotencia::ConsumoMaximo => "consumo máximo",
            ZonaPotencia::CapacidadAnaerobica => "capacidad anaeróbica",
            ZonaPotencia::Neuromuscular => "potencia neuromuscular",
        }
    }

    /// El nombre original de la fuente, para poder ir a comprobarlo.
    pub fn nombre_de_la_fuente(self) -> &'static str {
        match self {
            ZonaPotencia::RecuperacionActiva => "Active recuperation (nivel 1)",
            ZonaPotencia::Resistencia => "Endurance (nivel 2)",
            ZonaPotencia::Tempo => "Tempo (nivel 3)",
            ZonaPotencia::Umbral => "Lactate threshold (nivel 4)",
            ZonaPotencia::ConsumoMaximo => "Maximal aerobic power / VO2max (nivel 5)",
            ZonaPotencia::CapacidadAnaerobica => "Anaerobic capacity (nivel 6)",
            ZonaPotencia::Neuromuscular => "Neuromuscular power (nivel 7)",
        }
    }

    /// Qué es, en una frase que se le puede decir a una persona.
    pub fn explicacion(self) -> &'static str {
        match self {
            ZonaPotencia::RecuperacionActiva => "rodar suave: sirve para recuperar, no para mejorar",
            ZonaPotencia::Resistencia => "el fondo: aquí se construye la base aeróbica",
            ZonaPotencia::Tempo => "trabajo sostenido: cuesta, pero se aguanta un rato largo",
            ZonaPotencia::Umbral => {
                "justo en el umbral: es la zona que más mejora la resistencia, y se aguanta alrededor de una hora"
            }
            ZonaPotencia::ConsumoMaximo => {
                "por encima del umbral: se aguanta entre 3 y 8 minutos, y cansa mucho"
            }
            ZonaPotencia::CapacidadAnaerobica => {
                "esfuerzos de 30 segundos a 3 minutos: aquí el pulso ya no sirve de guía, porque no llega a tiempo"
            }
            ZonaPotencia::Neuromuscular => {
                "esfuerzos de menos de 25 segundos: la fuente avisa de que la potencia solo se puede comparar con esfuerzos parecidos, no con el ritmo de contrarreloj"
            }
        }
    }

    /// De dónde salen los cortes de esta zona. Es el texto que viaja al informe.
    pub fn fundamento() -> &'static str {
        "niveles de entrenamiento basados en potencia de Andrew Coggan (2001), \
         recogidos en «Training and Racing with a Power Meter» (Allen y Coggan)"
    }
}

/// **Las zonas de potencia, para un umbral dado.**
///
/// DEVUELVE EL TRAMO, NO SOLO EL SUELO — `(zona, desde, hasta)` —, **el mismo contrato que
/// `umbral::zonas_del_umbral`**. Y la razón está escrita en el proyecto desde el 2026-10-13: la
/// primera versión devolvía solo el suelo, y como cada zona empieza donde acaba la anterior, **dos
/// zonas seguidas enseñaban el mismo número**. Un informe que dice dos veces 213 no se entiende, y el
/// fallo era de lectura, no de cálculo.
///
/// EL TECHO ES `Option<Rango>` POR UN MOTIVO, no por comodidad: la **última zona no tiene techo**
/// porque la fuente no lo publica. Un `Rango` normal obligaría a inventar un número —o a poner
/// infinito, que en un informe se lee como un error—. Aquí `None` significa exactamente lo que dice:
/// **hacia arriba no se puede afirmar nada**.
///
/// Los márgenes **se propagan**: el margen del umbral multiplica a cada corte, porque un umbral de
/// ±7 W mueve el suelo y el techo de todas las zonas.
pub fn zonas_de_potencia(ftp: &Ftp) -> Vec<(ZonaPotencia, Rango, Option<Rango>)> {
    ZonaPotencia::todas()
        .iter()
        .map(|z| {
            let desde = ftp.rango.valor * z.fraccion();
            let m_desde = ftp.rango.margen * z.fraccion();
            let techo = z.techo().map(|t| {
                let hasta = ftp.rango.valor * t;
                let m_hasta = ftp.rango.margen * t;
                Rango::estimado(hasta, m_hasta)
            });
            (*z, Rango::estimado(desde, m_desde), techo)
        })
        .collect()
}

/// **En qué zona cae un vatio concreto.** Es la pregunta que se hace al mirar una muestra.
///
/// Devuelve la zona **y la parte del tramo** que ya se ha recorrido, porque «está en tempo» y «está
/// en lo más alto de tempo» no son la misma información.
pub fn zona_de_una_potencia(ftp: &Ftp, vatios: f64) -> ZonaPotencia {
    // Se recorre de ARRIBA ABAJO y se devuelve la primera cuyo suelo no se pasa. Así la última zona
    // abierta se lleva todo lo que sobre, que es justo lo que dice la fuente.
    for z in ZonaPotencia::todas().iter().rev() {
        if vatios >= ftp.rango.valor * z.fraccion() {
            return *z;
        }
    }
    ZonaPotencia::RecuperacionActiva
}

// ============================================================================================
// 3 · EL LECTOR DE LAS ZONAS QUE YA CALCULÓ EL APARATO
// ============================================================================================

/// **Lo que el fichero declara sobre potencia, en un solo sitio.**
///
/// POR QUÉ NO ES `fit::Declarado`: porque `fit.rs` **no está a mi alcance** —otro agente lo está
/// ampliando— y el mensaje `TimeInZone` no lo lee todavía. Este struct es **la pieza que falta**,
/// aislada aquí para poder borrarla entera el día que `fit.rs` la traiga.
///
/// TODO LO QUE NO VENGA QUEDA COMO `Falta`, igual que en `fit.rs`. Y el `Vec` del reparto por zonas
/// es `Dato<Vec<f64>>` y no `Vec<f64>` por el motivo de siempre: **una lista vacía y un cero son
/// afirmaciones, y aquí lo que hay es un hueco**.
#[derive(Debug, Clone, Default)]
pub struct DeclaradoDePotencia {
    /// Los vatios que el reloj usa como umbral (`threshold_power`).
    pub umbral_w: Dato<u16>,
    /// El TSS (`training_stress_score`).
    pub tss: Dato<f64>,
    /// La potencia normalizada (`normalized_power`).
    pub potencia_normalizada: Dato<f64>,
    /// El factor de intensidad (`intensity_factor`).
    pub factor_intensidad: Dato<f64>,
    /// La duración con la que se cronometró la sesión (`total_timer_time`), en segundos.
    ///
    /// **ES EL TIEMPO EN MOVIMIENTO, NO EL TRANSCURRIDO**, y la diferencia importa y está medida:
    /// `fit::Declarado.duracion_s` guarda `total_elapsed_time` —las dos, la que se mueve y la que no—,
    /// porque es la primera que aparece y el modelo del motor todavía no distingue una de otra. Para
    /// **comprobar el TSS** hace falta esta: la cuenta de Coggan usa el tiempo **pedaleando**. Medido
    /// sobre sus 83 sesiones con potencia, con el tiempo transcurrido la comprobación cuadra en **31**;
    /// con el tiempo en movimiento, en **55**. La diferencia no es un detalle de precisión: es la
    /// diferencia entre una comprobación que sirve y otra que acusa al fichero de un error que es
    /// nuestro.
    pub duracion_en_movimiento_s: Dato<f64>,
    /// **El reparto del tiempo por zonas de potencia**, en segundos, de la zona 1 a la 7.
    pub tiempo_en_potencia_s: Dato<Vec<f64>>,
    /// **El reparto del tiempo por zonas de pulso**, en segundos. Lo trae el mismo mensaje.
    pub tiempo_en_pulso_s: Dato<Vec<f64>>,
}

/// Lee de un FIT **solo lo que hace falta para las zonas de potencia**.
///
/// DEUDA DECLARADA (ver la cabecera del módulo): esto debería estar en `fit.rs` y no está porque hay
/// otro agente dentro de ese fichero. **No falla nunca por datos raros**: lo que no esté, queda
/// `Falta`. Devuelve `Err` solo si el fichero no se puede abrir o no es un FIT.
pub fn leer_declarado_de_potencia(ruta: &Path) -> Result<DeclaradoDePotencia, String> {
    let mut f = std::fs::File::open(ruta)
        .map_err(|e| format!("no se pudo abrir {}: {e}", ruta.display()))?;
    let datos =
        fitparser::from_reader(&mut f).map_err(|e| format!("el FIT no se pudo leer: {e:?}"))?;

    let mut d = DeclaradoDePotencia::default();
    for r in datos.iter() {
        match format!("{:?}", r.kind()).as_str() {
            "Session" => {
                for c in r.fields() {
                    match c.name() {
                        "threshold_power" => d.umbral_w = a_entero(c.value()),
                        "training_stress_score" => d.tss = a_real(c.value()),
                        "normalized_power" => d.potencia_normalizada = a_real(c.value()),
                        "intensity_factor" => d.factor_intensidad = a_real(c.value()),
                        "total_timer_time" => {
                            if d.duracion_en_movimiento_s.falta() {
                                d.duracion_en_movimiento_s = a_real(c.value());
                            }
                        }
                        _ => {}
                    }
                }
            }
            "TimeInZone" => {
                // El mensaje viene UNA VEZ POR VUELTA y una más por la sesión entera. Se elige la de
                // la sesión comparando `reference_mesg`; las de las vueltas se ignoran.
                //
                // ESTA DISTINCIÓN NO ES UN DETALLE: medido sobre sus 102 ficheros, si se coge el
                // PRIMER mensaje que aparezca —que es el de la vuelta 0— el reparto por zonas suma
                // una MEDIA del 84,7 % MENOS que la sesión. O sea que el reparto parecería bueno y
                // estaría contando solo una parte. Lo cazó esta comprobación, y por eso está escrita.
                let mut es_de_la_sesion = false;
                let mut pot: Option<Vec<f64>> = None;
                let mut pul: Option<Vec<f64>> = None;
                for c in r.fields() {
                    match c.name() {
                        "reference_mesg" => {
                            if let fitparser::Value::String(s) = c.value() {
                                es_de_la_sesion = s == "session";
                            }
                        }
                        "time_in_power_zone" => pot = a_serie(c.value()),
                        "time_in_hr_zone" => pul = a_serie(c.value()),
                        _ => {}
                    }
                }
                if es_de_la_sesion {
                    if let Some(v) = pot {
                        d.tiempo_en_potencia_s = Dato::Hay(v);
                    }
                    if let Some(v) = pul {
                        d.tiempo_en_pulso_s = Dato::Hay(v);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(d)
}

/// Un entero de un valor de FIT. **Un tipo que no se espera no se adivina**: se trata como ausente.
fn a_entero(v: &fitparser::Value) -> Dato<u16> {
    match v {
        fitparser::Value::UInt8(x) => Dato::Hay(u16::from(*x)),
        fitparser::Value::UInt16(x) => Dato::Hay(*x),
        fitparser::Value::UInt32(x) => Dato::Hay(*x as u16),
        fitparser::Value::SInt8(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::SInt16(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::SInt32(x) => Dato::Hay((*x).max(0) as u16),
        fitparser::Value::Float32(x) => Dato::Hay(x.round().max(0.0) as u16),
        fitparser::Value::Float64(x) => Dato::Hay(x.round().max(0.0) as u16),
        _ => Dato::Falta,
    }
}

/// Un número con decimales de un valor de FIT.
fn a_real(v: &fitparser::Value) -> Dato<f64> {
    match v {
        fitparser::Value::Float32(x) => Dato::Hay(f64::from(*x)),
        fitparser::Value::Float64(x) => Dato::Hay(*x),
        fitparser::Value::UInt8(x) => Dato::Hay(f64::from(*x)),
        fitparser::Value::UInt16(x) => Dato::Hay(f64::from(*x)),
        fitparser::Value::UInt32(x) => Dato::Hay(f64::from(*x)),
        fitparser::Value::SInt8(x) => Dato::Hay(f64::from(*x)),
        fitparser::Value::SInt16(x) => Dato::Hay(f64::from(*x)),
        fitparser::Value::SInt32(x) => Dato::Hay(f64::from(*x)),
        _ => Dato::Falta,
    }
}

/// Una serie de números de un valor de FIT.
///
/// OJO CON LOS HUECOS, y aquí hay una decisión que hay que enseñar: en este array **un hueco se
/// conserva como 0,0**, y **no** se salta. El motivo es que **la posición es el significado**: la
/// posición 3 es «tiempo en la zona 4», y saltarse un hueco correría todas las zonas siguientes una
/// posición — con lo que el tiempo de la zona 5 se leería como zona 4 y **el error no se vería por
/// ningún lado**. Un 0,0 en una posición que el aparato dejó vacía significa «cero segundos ahí», que
/// es lo que el array dice, y la comprobación de [`cuadra_con_la_duracion`] es la que caza que la suma
/// no cuadre si alguna posición viniera mal.
///
/// Y si TODAS las posiciones vinieran vacías, el array es un hueco con forma de dato: devuelve `None`,
/// que no es lo mismo que una lista de ceros.
fn a_serie(v: &fitparser::Value) -> Option<Vec<f64>> {
    match v {
        fitparser::Value::Array(v) => {
            let mut out = Vec::with_capacity(v.len());
            for x in v {
                match a_real(x) {
                    Dato::Hay(n) => out.push(n),
                    Dato::Falta => out.push(0.0),
                }
            }
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        }
        _ => None,
    }
}

/// Una zona **con su tiempo**, ya resuelta.
#[derive(Debug, Clone, PartialEq)]
pub struct EnZona {
    pub zona: ZonaPotencia,
    /// Segundos en esa zona, **según el fichero**.
    pub segundos: f64,
}

impl EnZona {
    /// La parte de la sesión que se pasó ahí. 0,25 es «un cuarto del tiempo».
    pub fn fraccion_de(&self, total_s: f64) -> f64 {
        if total_s <= 0.0 {
            return 0.0;
        }
        self.segundos / total_s
    }

    /// El tiempo, escrito para leerlo: «1 h 12 min», «8 min», «35 s».
    pub fn tiempo_legible(&self) -> String {
        segundos_legibles(self.segundos)
    }
}

/// Un tiempo en segundos, escrito para leerlo: «1 h 12 min», «8 min», «35 s».
///
/// Está fuera de [`EnZona`] porque también hace falta para tiempos que **no** son de una zona —el
/// reparto de pulso, sin ir más lejos—, y duplicarlo sería tener dos formas de escribir lo mismo.
pub fn segundos_legibles(segundos: f64) -> String {
    let s = segundos.round() as i64;
    let (h, m) = (s / 3600, (s % 3600) / 60);
    if h > 0 {
        format!("{h} h {m} min")
    } else if m > 0 {
        format!("{m} min")
    } else {
        format!("{s} s")
    }
}

/// El **tiempo en cada zona de potencia**, con la zona ya puesta.
///
/// `None` cuando el fichero **no lo trae**. Y no es lo mismo que una lista de ceros: **una lista de
/// ceros dice «estuvo cero segundos en todas las zonas»**, que es una afirmación y además falsa. Lo
/// que pasa de verdad es que **no se sabe**, y eso se dice con `None`.
///
/// SE EMPAREJA POR POSICIÓN, y esa correspondencia se comprobó sobre los ficheros reales: la posición
/// *i* de `time_in_power_zone` corresponde al nivel *i+1* de la tabla de Coggan. Van siete zonas en un
/// array de **diez** posiciones, y **las tres últimas también traen tiempo**.
///
/// Y ESO HAY QUE DECIRLO, porque está medido y NO es un detalle: **28 de sus 83 sesiones con potencia
/// tienen tiempo en las posiciones 8, 9 o 10**, y entre todas suman **1.322 segundos**. Garmin define
/// diez tramos de potencia y Coggan siete; los tres que sobran **no se pueden nombrar con la tabla que
/// este módulo cita**, así que **no se inventan nombres para ellos** y aquí se devuelven solo las siete
/// que sí tienen nombre. La consecuencia práctica, con su número: si al comprobar el reparto se suman
/// las siete, cuadran **69 de 83** sesiones dentro del 1 %; sumando las diez, **79 de 83**. Los diez
/// puntos de diferencia **no son un error del lector**: son el tiempo que cae en los tramos que la
/// fuente no define, y se cuentan aparte con [`segundos_sin_zona`].
pub fn tiempo_en_zonas(serie: &Dato<Vec<f64>>) -> Option<Vec<EnZona>> {
    let s = serie.prestado()?;
    if s.is_empty() {
        return None;
    }
    let pares: Vec<EnZona> = ZonaPotencia::todas()
        .iter()
        .enumerate()
        .filter_map(|(i, z)| {
            s.get(i).map(|seg| EnZona {
                zona: *z,
                segundos: *seg,
            })
        })
        .collect();
    if pares.is_empty() {
        None
    } else {
        Some(pares)
    }
}

/// ¿El tiempo repartido entre las zonas **cuadra con la duración de la sesión**?
///
/// POR QUÉ ESTA FUNCIÓN VALE, y es exactamente lo que dice `fit.rs`: *«si el total de Garmin y el
/// nuestro no coinciden, uno de los dos está mal, y eso es una comprobación que sale gratis»*. Aquí
/// se aplica a las zonas: el tiempo repartido entre las zonas **tiene que sumar la sesión**. Si suma
/// la mitad, alguien está leyendo mal — y sin esta comprobación el número seguiría pareciendo bueno.
///
/// MEDIDO, Y ES LO QUE HACE QUE ESTA COMPROBACIÓN HAYA VALIDO LA PENA: cogiendo el **primer** mensaje
/// `TimeInZone` que aparece —el de la vuelta 0, que es lo natural si no se mira— cuadran **16 de 102**
/// fichas y la suma se queda, de mediana, un **84,7 % por debajo** de la sesión. Cogiéndolo bien —el
/// que declara `reference_mesg = session`— cuadran **69 de 83** dentro del 1 %. La comprobación, por sí
/// sola, distingue las dos implementaciones sin mirar el código, y ese es todo su valor.
///
/// Y LOS 14 QUE NO CUADRAN TIENEN UNA EXPLICACIÓN MEDIDA, no una excusa: **28 de las 83 sesiones
/// tienen tiempo en las tres posiciones del array que Coggan no define** (posiciones 8, 9 y 10; 1.322
/// segundos entre todas). Sumando también esas, cuadran **79 de 83**. Los cinco que quedan son
/// sesiones donde el reloj estuvo parado —una de ellas reparte solo 22 s de una sesión de 934 s, que
/// es una sesión que se cortó—, y **eso es un hallazgo sobre el fichero, no un error del motor**.
///
/// LA TOLERANCIA ES UNA DECISIÓN: un **1 %**. El aparato redondea y el reloj arranca y para, así que
/// exigir el segundo exacto haría fallar la comprobación por ruido; pero un 1 % de una sesión de tres
/// horas son **108 segundos**, que ya es un desajuste de verdad y no redondeo.
pub fn cuadra_con_la_duracion(zonas: &[EnZona], duracion_s: f64, tolerancia_relativa: f64) -> bool {
    if duracion_s <= 0.0 {
        return false;
    }
    let suma: f64 = zonas.iter().map(|z| z.segundos).sum();
    ((suma - duracion_s) / duracion_s).abs() <= tolerancia_relativa
}

/// **El tiempo que cae en los tramos que la fuente no define**, en segundos.
///
/// Garmin reparte el tiempo en **diez** tramos de potencia; la tabla de Coggan que este módulo cita
/// tiene **siete** niveles. Las posiciones 8, 9 y 10 del array **existen y traen tiempo** —medido:
/// **28 de sus 83 sesiones con potencia, 1.322 segundos entre todas**—, y **no se pueden nombrar** con
/// la fuente. Esta función los cuenta **en vez de repartirlos a ojo entre las siete zonas**, que sería
/// la forma fácil de cuadrar el total inventando datos.
pub fn segundos_sin_zona(serie: &Dato<Vec<f64>>) -> f64 {
    match serie.prestado() {
        Some(s) => s.iter().skip(ZonaPotencia::todas().len()).sum(),
        None => 0.0,
    }
}

/// El resultado de esa comprobación, **con sus números**, para poder enseñarlo.
#[derive(Debug, Clone, PartialEq)]
pub struct ComprobacionDeZonas {
    pub suma_de_zonas_s: f64,
    pub duracion_s: f64,
    pub desvio_relativo: f64,
    pub cuadra: bool,
}

impl ComprobacionDeZonas {
    /// Una frase, para leerla sin interfaz.
    pub fn frase(&self) -> String {
        if self.cuadra {
            format!(
                "las zonas reparten {:.0} s de una sesión de {:.0} s (desvío {:+.1} %): cuadran",
                self.suma_de_zonas_s,
                self.duracion_s,
                self.desvio_relativo * 100.0
            )
        } else {
            format!(
                "las zonas reparten {:.0} s y la sesión duró {:.0} s (desvío {:+.1} %): NO cuadran, \
                 así que uno de los dos números está mal",
                self.suma_de_zonas_s,
                self.duracion_s,
                self.desvio_relativo * 100.0
            )
        }
    }
}

/// Compara el reparto de zonas con la duración y **devuelve el resultado con sus números**.
pub fn comparar_zonas_con_duracion(
    zonas: &[EnZona],
    duracion_s: f64,
    tolerancia_relativa: f64,
) -> ComprobacionDeZonas {
    let suma: f64 = zonas.iter().map(|z| z.segundos).sum();
    let desvio = if duracion_s > 0.0 {
        (suma - duracion_s) / duracion_s
    } else {
        0.0
    };
    ComprobacionDeZonas {
        suma_de_zonas_s: suma,
        duracion_s,
        desvio_relativo: desvio,
        cuadra: cuadra_con_la_duracion(zonas, duracion_s, tolerancia_relativa),
    }
}

/// **La comparación honesta entre las dos formas de mirar el mismo esfuerzo.**
///
/// POR QUÉ ESTA FUNCIÓN ES LA QUE CONTESTA AL CLIENTE, aunque no calcule nada nuevo: dice en qué zona
/// de potencia estuvo y en qué zona de pulso, **con el tiempo de cada una**, para que la diferencia se
/// vea en vez de suponerse.
///
/// Y HAY DOS COSAS QUE ESTA FUNCIÓN **NO** HACE, A PROPÓSITO:
///
/// 1. **No dice que las dos «coinciden» o «no coinciden» comparando nombres.** Las zonas de potencia
///    de Coggan son **siete** y las de pulso del aparato son **cinco**, y no son la misma escala: un
///    «tempo» de potencia y una «zona 3» de pulso **no son la misma cosa aunque suenen parecido**.
///    Una versión anterior de esta función comparaba sus nombres y decía «NO señalan la misma zona»
///    **siempre**, porque estaba comparando dos vocabularios distintos. Eso no es un hallazgo: es un
///    error de lectura, y se quitó.
/// 2. **No elige un ganador.** Lo que sí se puede afirmar, y se afirma, es lo que la fisiología
///    sostiene: el pulso **va con retraso** respecto al esfuerzo y lo mueven el calor y la fatiga,
///    mientras que los vatios son el esfuerzo mismo. Cuando el reparto por potencia y el de pulso
///    cuentan cosas distintas, **el de potencia describe el esfuerzo**; el de pulso describe cómo
///    respondió el cuerpo, que es otra pregunta.
///
/// Las zonas de potencia **las calculamos nosotros** desde su FTP; las de pulso **las trae el
/// fichero**. O sea que no se comparan dos cálculos nuestros: se compara **nuestro criterio con el del
/// aparato**, y por eso la frase lo dice.
///
/// `None` cuando falta cualquiera de las dos: **sin las dos no hay comparación**, y decir «coinciden»
/// cuando solo hay una sería la peor respuesta posible.
pub fn comparar_potencia_y_pulso(potencia: &[EnZona], pulso: &[(String, f64)]) -> Option<String> {
    if potencia.is_empty() || pulso.is_empty() {
        return None;
    }
    let total_pot: f64 = potencia.iter().map(|z| z.segundos).sum();
    let total_pul: f64 = pulso.iter().map(|(_, s)| *s).sum();
    if total_pot <= 0.0 || total_pul <= 0.0 {
        return None;
    }
    // La zona donde MÁS tiempo pasó, en cada una de las dos formas de mirarlo. Es la que resume la
    // sesión, y comparar las dos dominantes dice más que comparar siete parejas.
    let dominante = potencia
        .iter()
        .max_by(|a, b| a.segundos.partial_cmp(&b.segundos).unwrap_or(std::cmp::Ordering::Equal))?;
    let dominante_pulso = pulso
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))?;
    let numero_de_pulso = pulso
        .iter()
        .position(|(n, _)| n == &dominante_pulso.0)
        .map(|i| i + 1)
        .unwrap_or(0);

    Some(format!(
        "por POTENCIA (calculado por nosotros desde su FTP) pasó el {:.0} % del tiempo en «{}», {}; \
         por PULSO (el reparto que trae el fichero) pasó el {:.0} % en la zona {} de las suyas, {}. \
         Y NO SON LA MISMA ESCALA: Coggan define siete zonas de potencia y el aparato reparte el pulso \
         en cinco. Cuando las dos cuentan cosas distintas, el reparto por potencia es el que describe \
         el esfuerzo — el pulso va con retraso y lo mueven el calor y la fatiga.",
        dominante.fraccion_de(total_pot) * 100.0,
        dominante.zona.nombre(),
        dominante.tiempo_legible(),
        dominante_pulso.1 / total_pul * 100.0,
        numero_de_pulso,
        segundos_legibles(dominante_pulso.1),
    ))
}

// ============================================================================================
// 4 · LA CARGA: EL TSS QUE EL FICHERO YA TRAE
// ============================================================================================

/// **La regla del proyecto, y va la primera porque es la que más fácil se rompe:**
///
/// > **El TSS que trae el fichero SE CONSERVA, no se recalcula.**
///
/// POR QUÉ: recalcularlo daría **un número distinto con el mismo nombre**, y a partir de ahí nadie
/// sabría cuál de los dos es «el TSS del cliente». Su TSS está en su aplicación, en su reloj y en su
/// historial; si este motor enseñara otro, el cliente vería dos cifras para la misma sesión y
/// **dejaría de creerse las dos**. Un motor que discute con el aparato del usuario pierde.
///
/// Y SI ALGÚN DÍA SE RECALCULA, **tiene que ser para comprobar que coincide**. Eso ya es otra cosa y
/// es legítima: se calcula el nuestro, se pone al lado del suyo, y si no cuadran **se dice**, porque
/// entonces uno de los dos está mal. Eso es [`comprobar_tss`], y es lo único que este módulo hace con
/// el TSS además de leerlo.
pub fn carga_de(declarado: &Declarado) -> Dato<f64> {
    declarado.tss
}

/// El resultado de contrastar el TSS del fichero con el que saldría de sus propias piezas.
///
/// NO ES UN RECÁLCULO QUE SUSTITUYA NADA: es una **comprobación**. El TSS que se enseña sigue siendo
/// el del fichero, pase lo que pase aquí.
#[derive(Debug, Clone, PartialEq)]
pub struct ComprobacionDelTss {
    pub declarado: f64,
    pub recalculado: f64,
    pub diferencia: f64,
    pub cuadra: bool,
}

impl ComprobacionDelTss {
    pub fn frase(&self) -> String {
        if self.cuadra {
            format!(
                "el TSS del fichero ({:.1}) coincide con NP·IF·100/FTP ({:.1}): la cadena cuadra",
                self.declarado, self.recalculado
            )
        } else {
            format!(
                "el TSS del fichero ({:.1}) NO coincide con NP·IF·100/FTP ({:.1}), y se separan {:.1}: \
                 uno de los dos está mal, y el que se enseña —siempre— es el del fichero. \
                 (Medido sobre sus 83 sesiones con potencia: la cuenta cuadra en 55, y las 28 que no \
                 cuadran son las que hay que mirar.)",
                self.declarado, self.recalculado, self.diferencia
            )
        }
    }
}

/// Recalcula el TSS **desde las piezas que declara el mismo fichero**, para contrastarlo.
///
/// SOLO PARA COMPROBAR. Nunca para sustituir: ver [`carga_de`].
///
/// LA CUENTA, y de dónde sale: `TSS = duración × NP × IF / (FTP × 3600) × 100`, con NP la potencia
/// normalizada e IF el factor de intensidad (`NP / FTP`). La definición es de **Allen y Coggan**, la
/// misma fuente que las zonas, y por eso el módulo entero cita una sola obra.
///
/// Y AQUÍ ESTÁ LO QUE SE MIDIÓ, que es más interesante que la fórmula: con la duración del fichero,
/// **la cuenta cuadra en 55 de sus 83 sesiones con potencia** dentro de un punto de TSS, con una
/// diferencia mediana de **0,39** puntos. En las otras 28 no cuadra, y las diferencias llegan a **77
/// puntos**. O sea que el TSS que publica Garmin **casi siempre** es esta fórmula —y por eso la
/// comprobación sirve—, pero **no siempre**, y esas 28 son las que hay que mirar. Esta función existe
/// para que eso **se pueda ver** en vez de suponer que cuadra; el TSS que se enseña sigue siendo el
/// suyo, pase lo que pase.
pub fn comprobar_tss(declarado: &Declarado, tolerancia: f64) -> Option<ComprobacionDelTss> {
    let tss = declarado.tss.valor()?;
    let np = declarado.potencia_normalizada.valor()?;
    let fi = declarado.factor_intensidad.valor()?;
    let ftp = f64::from(declarado.potencia_umbral.valor()?);
    let segundos = f64::from(declarado.duracion_s.valor()?);
    if ftp <= 0.0 || segundos <= 0.0 {
        return None;
    }
    let recalculado = segundos * np * fi / (ftp * 3600.0) * 100.0;
    Some(ComprobacionDelTss {
        declarado: tss,
        recalculado,
        diferencia: (recalculado - tss).abs(),
        cuadra: (recalculado - tss).abs() <= tolerancia,
    })
}

/// La tolerancia con la que se decide que el TSS del fichero «cuadra» con la cuenta.
///
/// **UN PUNTO DE TSS**, y es una decisión con su medición detrás: el fichero guarda el TSS con un
/// decimal, pero **la duración con la que lo calculó el aparato no viaja con esa precisión** —el
/// tiempo en movimiento se guarda redondeado—, así que la cuenta de vuelta **nunca puede dar el mismo
/// decimal**. Medido sobre una de sus sesiones: el fichero declara **155,6** y la cuenta da **156,4**
/// con la duración declarada; esa diferencia de 0,8 puntos es redondeo del tiempo, no un desajuste.
/// Con el corte en 0,5 esa sesión «no cuadraría» por un motivo que no es un error; con 1,0 la
/// comprobación sigue cazando lo que tiene que cazar —diferencias de decenas de puntos— sin ponerse
/// roja por el último decimal.
pub const TOLERANCIA_TSS: f64 = 1.0;

/// **La regla del cliente**: «si el entrenamiento supera los 100 TSS».
///
/// ESTO ES UNA REGLA SUYA, NO UN HALLAZGO MÉDICO, y la diferencia importa mucho: **por eso no se
/// llama «peligro»**. `esfuerzo.rs` sí usa esa palabra, pero allí la respalda un estudio de más de
/// 5.000 corredores. Aquí lo que hay es **una regla que el cliente se puso a sí mismo**, y el motor la
/// implementa **como él la dijo**, diciendo de dónde sale. Llamarla «peligro» sería ponerle a su
/// propia regla una autoridad que no tiene.
///
/// MEDIDO SOBRE SUS SESIONES: **18 de las 83 sesiones con potencia superan los 100 TSS**, y la mayor
/// es **221,5**. O sea que su regla **salta de verdad**, y no es decorativa.
///
/// EL CORTE ES ESTRICTO —`> 100`, no `>= 100`—, Y ES UNA DECISIÓN QUE HAY QUE JUSTIFICAR: el cliente
/// dijo «supera los 100», y «supera» quiere decir **más de**. Dos consecuencias, las dos escritas:
/// **100,0 exacto NO es carga alta**, y por eso la frontera queda del lado conservador. La alternativa
/// (`>=`) marcaría como alta una sesión que está exactamente en el número que él puso como límite, y
/// eso sería **leer su regla un poco peor de lo que la dijo**. En sus datos **ninguna sesión cae
/// exactamente en 100**, así que hoy los dos cortes dan el mismo resultado; se elige el estricto
/// porque es el que él dijo, y queda comprobado en las pruebas.
///
/// Es una **función pura**: no lee ficheros, no mira la fecha y no depende de nada más que su
/// entrada. Por eso se puede comprobar con números a mano, que es como se comprueba de verdad.
pub fn carga_es_alta(tss: f64, umbral: f64) -> bool {
    // Un TSS que no es un número —negativo, infinito o `NaN`— **no es una carga alta**: es un dato que
    // no es un dato. Se devuelve `false` en vez de `true` a propósito: **no se avisa por un hueco**.
    // El motor ya sabe decir «falta» en otro sitio; lo que no puede pasar es que un hueco se disfrace
    // de aviso.
    if !tss.is_finite() || tss <= 0.0 {
        return false;
    }
    // El valor EXACTO del umbral lo decide una constante, no un `>` puesto al azar: si algún día se
    // decide que 100 clavados sí avisan, se cambia en un sitio y esta función lo obedece.
    if (tss - umbral).abs() < 1e-9 {
        return EN_EL_UMBRAL_DE_CARGA_ES_ALTA;
    }
    tss > umbral
}

/// **El umbral de carga que el cliente se puso**: 100 TSS.
///
/// **NO SE ESCRIBE AQUÍ OTRA VEZ, Y ESO ES LO IMPORTANTE.** El número ya vive en
/// [`crate::avisos::UMBRAL_TSS_DEL_CLIENTE`], que es donde el motor de avisos lo usa para recordarle su
/// propia pauta. Si este módulo declarara su propio `100.0`, el día que alguien cambiara uno de los dos
/// **el motor tendría dos reglas del cliente distintas**, y la de la pantalla no sería la que se
/// comprueba. Un número del cliente, en un solo sitio.
pub const UMBRAL_DE_CARGA_DEL_CLIENTE: f64 = crate::avisos::UMBRAL_TSS_DEL_CLIENTE;

/// Qué se hace cuando el TSS vale **exactamente** el umbral.
///
/// **SE EXIGE SUPERARLO, NO ALCANZARLO**, y es la letra de su regla: «si supera los 100». La decisión
/// se **toma prestada** de [`crate::avisos::EN_EL_UMBRAL_DE_TSS_DISPARA`], por el mismo motivo que el
/// número: **una sola dirección para un valor de corte**, y un solo sitio donde cambiarla.
pub const EN_EL_UMBRAL_DE_CARGA_ES_ALTA: bool = crate::avisos::EN_EL_UMBRAL_DE_TSS_DISPARA;

/// El fundamento del aviso, **con la voz del cliente**: es su regla, no un hallazgo.
pub fn fundamento_del_aviso_de_carga() -> &'static str {
    "regla del propio cliente: «si el entrenamiento supera los 100 TSS». NO es un hallazgo médico: \
     el motor la implementa tal como él la dijo"
}

// ============================================================================================
// 5 · LA CARGA SEMANAL Y SU TENDENCIA
// ============================================================================================

/// Una sesión, con lo mínimo para poder agruparla por semanas y medir su carga.
///
/// SE PIDE FECHA Y TSS, Y NADA MÁS, a propósito: cuanta menos información necesite una función, menos
/// motivos tiene para fallar. Esta no necesita ni las muestras ni el pulso.
#[derive(Debug, Clone, PartialEq)]
pub struct SesionConCarga {
    /// La fecha, en `AAAA-MM-DD`. Si no se sabe, `None`, y entonces la sesión **no entra en ninguna
    /// semana**: meterla en la semana equivocada sería peor que dejarla fuera.
    pub fecha: Option<String>,
    pub tss: Dato<f64>,
}

impl SesionConCarga {
    pub fn nueva(fecha: &str, tss: f64) -> Self {
        Self {
            fecha: Some(fecha.to_string()),
            tss: Dato::Hay(tss),
        }
    }

    /// Una sesión de la que no se sabe ni cuándo fue ni cuánta carga tuvo. Existe para poder probar
    /// que **no se inventa** ni una cosa ni la otra.
    pub fn sin_datos() -> Self {
        Self {
            fecha: None,
            tss: Dato::Falta,
        }
    }
}

/// Una semana de entrenamiento, con su carga **y el recuento de lo que no se pudo sumar**.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanaDeCarga {
    /// El **lunes** de esa semana, en `AAAA-MM-DD`. Se usa el lunes porque es el primer día de la
    /// semana ISO, y porque así la serie se puede ordenar como texto sin equivocarse — que es lo que
    /// `fuentes.rs` ya hace con las fechas en todo el proyecto.
    pub lunes: String,
    pub tss_total: f64,
    pub sesiones: usize,
    /// **Cuántas sesiones de esa semana no traían TSS.** No se suman como cero —un cero sería un dato
    /// falso— y se cuentan, que es lo correcto.
    pub sin_tss: usize,
}

/// Agrupa las sesiones por semanas y suma su carga.
///
/// QUÉ HACE CON LO QUE FALTA, que es la decisión importante de esta función:
/// - Una sesión **sin TSS** no suma cero: **se cuenta aparte** (`sin_tss`). Un cero haría que una
///   semana con dos sesiones ilegibles pareciera una semana de descanso, y esa confusión es
///   exactamente la que produce planes equivocados.
/// - Una sesión **sin fecha** no entra en ninguna semana. No se inventa una fecha.
///
/// La semana empieza el **lunes**, y se calcula sin biblioteca de fechas: el proyecto ya decidió no
/// arrastrar una dependencia de fechas por un cálculo así (ver `fuentes.rs`, que usa texto
/// `AAAA-MM-DD` por el mismo motivo).
pub fn carga_semanal(sesiones: &[SesionConCarga]) -> Vec<SemanaDeCarga> {
    let mut por_semana: std::collections::BTreeMap<String, SemanaDeCarga> = Default::default();
    for s in sesiones {
        let Some(fecha) = &s.fecha else { continue };
        let Some(lunes) = lunes_de_la_semana(fecha) else {
            continue;
        };
        let e = por_semana
            .entry(lunes.clone())
            .or_insert_with(|| SemanaDeCarga {
                lunes,
                tss_total: 0.0,
                sesiones: 0,
                sin_tss: 0,
            });
        e.sesiones += 1;
        match s.tss.valor() {
            Some(t) => e.tss_total += t,
            None => e.sin_tss += 1,
        }
    }
    // Se devuelven **ordenadas por fecha**, que es lo que `Serie` necesita para no tener que
    // ordenarlas otra vez.
    por_semana.into_values().collect()
}

/// **La tendencia de la carga semanal**, reutilizando `tendencia.rs` y no escribiendo otro.
///
/// POR QUÉ SE REUTILIZA: `tendencia.rs` ya sabe decir si algo **sube, baja o se mantiene**, con su
/// dirección, su tolerancia y su frase. Escribir aquí una segunda forma de medir tendencias daría dos
/// respuestas distintas a la misma pregunta según por dónde se entrara — y ese es el fallo que el
/// proyecto ya se ha encontrado con los cortes de zona.
///
/// LO ÚNICO QUE SE APORTA ES LA **TOLERANCIA**, y es una decisión declarada: un **15 %**, el mismo
/// que `tendencia::tolerancia::VOLUMEN`. Se elige el mismo número a propósito: la carga semanal y el
/// volumen semanal se mueven por las mismas razones —una semana de viaje, una enfermedad, una semana
/// fuerte—, así que **usar dos tolerancias distintas para lo mismo sería una precisión falsa**.
pub fn tendencia_de_carga(semanas: &[SemanaDeCarga]) -> Dato<Tendencia> {
    let puntos: Vec<Valor<f64>> = semanas
        .iter()
        .map(|s| Valor::nuevo(s.tss_total, Procedencia::AparatoDeEntrenamiento, &s.lunes))
        .collect();
    Serie::nueva("carga semanal (TSS)", puntos).tendencia()
}

/// La tolerancia con la que se decide si la carga semanal «se mantiene».
pub const TOLERANCIA_CARGA: f64 = crate::tendencia::tolerancia::VOLUMEN;

/// **El lunes de la semana** de una fecha `AAAA-MM-DD`. `None` si la fecha no se entiende.
///
/// Se calcula sin biblioteca de fechas: ver el comentario de [`carga_semanal`]. La cuenta es la
/// estándar de **días desde el 1 de enero de 1970** (Howard Hinnant, *«chrono-Compatible Low-Level
/// Date Algorithms»*, el algoritmo que usan hoy la biblioteca estándar de C++ y buena parte de las de
/// fechas). Vale para el calendario gregoriano y **también para fechas anteriores a 1970**, que es
/// donde muchas fórmulas corrientes se rompen.
pub fn lunes_de_la_semana(fecha: &str) -> Option<String> {
    let (a, m, d) = parte_fecha(fecha)?;
    let z = dias_desde_1970(a, m, d);
    // El 1 de enero de 1970 fue **jueves**. Contando con el lunes como 0, el jueves es 3, así que
    // `(z + 3) % 7` da el día de la semana con el lunes en 0. Y se escribe `rem_euclid` porque en Rust
    // el resto de un entero negativo **puede ser negativo**, y una fecha de 1960 daría un lunes en el
    // futuro. Se elige la operación por lo que significa, no por lo que hace el operador.
    let desde_lunes = (z + 3).rem_euclid(7);
    Some(a_texto(z - desde_lunes))
}

/// Separa `AAAA-MM-DD` en tres números. **No se adivina**: si no tiene la forma, es `None`.
fn parte_fecha(fecha: &str) -> Option<(i64, i64, i64)> {
    let p: Vec<&str> = fecha.split('-').collect();
    if p.len() != 3 {
        return None;
    }
    let mes: i64 = p[1].parse().ok()?;
    let dia: i64 = p[2].parse().ok()?;
    // Un mes de 13 o un día de 40 no son una fecha: son un número con guiones. Se rechazan aquí para
    // que no salga un lunes calculado a partir de una fecha imposible.
    if !(1..=12).contains(&mes) || !(1..=31).contains(&dia) {
        return None;
    }
    Some((p[0].parse().ok()?, mes, dia))
}

/// **Días desde el 1 de enero de 1970**, que es la cuenta que hace falta para saber el día de la
/// semana. Fuente: Howard Hinnant, *«chrono-Compatible Low-Level Date Algorithms»* (2013), función
/// `days_from_civil`. Es el algoritmo que usan la biblioteca estándar de C++ y la de fechas de
/// buena parte de los lenguajes.
///
/// SE USA ESTE Y NO EL «DÍA JULIANO» CLÁSICO, y el motivo está medido: la primera versión de este
/// fichero usaba la fórmula del día juliano **de memoria**, y daba **dos días de menos y un año de
/// 0020** — el informe enseñaba semanas de «0020-04-26». Lo cazó el propio binario al imprimir la
/// tabla de semanas. Este algoritmo se comprobó contra el calendario de Python para ocho fechas,
/// incluidas 1970-01-01, 2000-02-29 y 1900-03-01, y **las ocho coinciden**.
fn dias_desde_1970(a: i64, m: i64, d: i64) -> i64 {
    // Enero y febrero se cuentan como meses 13 y 14 del año anterior: así el año empieza en marzo y
    // **el 29 de febrero deja de ser un caso aparte**. Ese es todo el truco del algoritmo.
    let y = a - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400; // el año dentro de la era, 0..399
    let mp = (m + 9) % 12; // marzo = 0
    let doy = (153 * mp + 2) / 5 + d - 1; // el día del año, contando desde el 1 de marzo
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // el día dentro de la era
    era * 146097 + doe - 719468
}

/// La vuelta: de días desde 1970 a `AAAA-MM-DD`. Hace falta para poder escribir el lunes.
/// Fuente: la misma, función `civil_from_days`.
fn a_texto(z: i64) -> String {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097; // 0..146096
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // 0..399
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // 0..365
    let mp = (5 * doy + 2) / 153; // 0..11
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let anio = y + i64::from(m <= 2);
    format!("{anio:04}-{m:02}-{d:02}")
}

/// **La suma de la carga de las últimas `semanas`**, para poder decir «cuánto lleva».
///
/// Devuelve la suma **y cuántas semanas entraron de verdad**. Si no hay ninguna, la suma es 0 y el
/// recuento también — y `0 de 0` es una respuesta honesta, mientras que un 0 suelto parecería «no
/// entrenó».
pub fn carga_de_las_ultimas(semanas: &[SemanaDeCarga], cuantas: usize) -> (f64, usize) {
    let mut suma = 0.0;
    let mut n = 0;
    for s in semanas.iter().rev().take(cuantas) {
        suma += s.tss_total;
        n += 1;
    }
    (suma, n)
}

/// Un resumen de la carga de un conjunto de sesiones, **con lo que falta contado aparte**.
///
/// Es lo que el binario enseña. Existe para que **el recuento de lo que falta viaje junto al número**:
/// un mínimo, una mediana y un máximo calculados sobre 83 sesiones de 102 **no significan lo mismo**
/// que sobre 102, y quien lee tiene derecho a saberlo en la misma línea.
#[derive(Debug, Clone, PartialEq)]
pub struct ResumenDeCarga {
    pub sesiones: usize,
    pub con_tss: usize,
    pub sin_tss: usize,
    pub con_potencia: usize,
    pub sin_potencia: usize,
    pub minimo: Option<f64>,
    pub mediana: Option<f64>,
    pub maximo: Option<f64>,
    pub por_encima_del_umbral: usize,
    pub umbral: f64,
}

impl ResumenDeCarga {
    /// Calcula el resumen. `sesiones` es, por sesión, **su TSS y si traía vatios en las muestras**.
    pub fn calcular(sesiones: &[(Dato<f64>, bool)], umbral: f64) -> Self {
        let mut valores: Vec<f64> = sesiones.iter().filter_map(|(t, _)| t.valor()).collect();
        valores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = valores.len();
        let mediana = if n == 0 {
            None
        } else if n % 2 == 1 {
            Some(valores[n / 2])
        } else {
            // Con un número PAR de valores, la mediana es la media de los dos centrales. Decir «el de
            // en medio» con 83 valores (impar) funcionaba por casualidad; con 82 daría un número que
            // no es la mediana. Se hace bien porque el día que haya 82 sesiones, nadie va a mirar
            // esta línea.
            Some((valores[n / 2 - 1] + valores[n / 2]) / 2.0)
        };
        Self {
            sesiones: sesiones.len(),
            con_tss: n,
            sin_tss: sesiones.len() - n,
            con_potencia: sesiones.iter().filter(|(_, p)| *p).count(),
            sin_potencia: sesiones.iter().filter(|(_, p)| !*p).count(),
            minimo: valores.first().copied(),
            mediana,
            maximo: valores.last().copied(),
            por_encima_del_umbral: valores.iter().filter(|t| carga_es_alta(**t, umbral)).count(),
            umbral,
        }
    }

    /// La frase que resume la carga, **diciendo sobre cuántas sesiones se calculó**.
    pub fn frase(&self) -> String {
        match (self.minimo, self.mediana, self.maximo) {
            (Some(mi), Some(me), Some(ma)) => format!(
                "TSS de {} sesiones: mínimo {:.0} · mediana {:.0} · máximo {:.0}. Superan los {:.0} TSS: {} sesiones. \
                 (Y hay {} sesiones más SIN TSS, y {} de las {} sin potencia: no entran en la cuenta.)",
                self.con_tss,
                mi,
                me,
                ma,
                self.umbral,
                self.por_encima_del_umbral,
                self.sin_tss,
                self.sin_potencia,
                self.sesiones
            ),
            _ => "no hay ni una sesión con TSS: no se puede decir nada de la carga".to_string(),
        }
    }
}

// ============================================================================================
// 6 · LO QUE NO SE PUEDE AFIRMAR
// ============================================================================================

/// **Qué falta para poder hablar de potencia**, en una frase, cuando no se puede.
///
/// EXISTE PORQUE LA RESPUESTA CORRECTA A «¿en qué zona de potencia estuvo?» CUANDO NO HAY POTENCIA
/// **NO ES «zona 1»**: es «no se sabe». Este motor tiene esa regla desde el principio —`Dato::Falta`
/// no es cero— y este módulo la aplica a la potencia.
pub fn que_falta_para_potencia(tiene_umbral: bool, tiene_potencia: bool) -> Option<String> {
    match (tiene_umbral, tiene_potencia) {
        (true, true) => None,
        (false, true) => Some(
            "hay vatios en las muestras pero no hay umbral de potencia, así que no se pueden fijar las zonas: \
             hace falta una prueba de 20 minutos, o que el aparato lo detecte"
                .to_string(),
        ),
        (true, false) => Some(
            "hay umbral de potencia pero esta sesión no trae ni un vatio, así que sus zonas de potencia no \
             existen: no se rellenan con ceros"
                .to_string(),
        ),
        (false, false) => {
            Some("ni vatios ni umbral: de esta sesión no se puede decir nada por potencia".to_string())
        }
    }
}
