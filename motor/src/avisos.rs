//! **El motor de reglas del encargo**: los tres módulos de aviso, convertidos en código determinista.
//!
//! DE DÓNDE SALE ESTE MÓDULO, y por qué existe: el cliente entregó un documento con tres módulos de
//! aviso *«redactados por otra IA»* que **mezclan requisitos con sugerencias**. `docs/ENCARGO-farellones-2026.md`
//! los midió uno a uno contra sus 102 ficheros de Garmin, y el resultado es que **de los tres, dos se
//! pueden construir con datos que existen y uno no se puede construir como él lo pidió**.
//!
//! ESTE MÓDULO ES LA TRADUCCIÓN DE ESE DOCUMENTO A CÓDIGO, y tiene tres reglas que no se negocian:
//!
//! 1. **Nada de IA.** Ninguna de las tres reglas es «si X, entonces avisa de Y», y eso es
//!    determinista. Un sistema así **se puede comprobar**; uno con un modelo dentro, no. Y decide
//!    sobre el cuerpo de una persona de 62 años.
//! 2. **Un aviso sin fundamento es peor que ninguno.** Cada aviso lleva su `porque` con el número que
//!    lo sostiene y su `fundamento`, que es **citado o medido, nunca inventado**. Si el dato que la
//!    regla necesita no está, **no se emite el aviso** y se dice qué falta.
//! 3. **La aplicación no es un médico.** Cuando observa algo de su rodilla, **lo dice como
//!    observación de sus datos**. Ver [`NO_ES_LO_QUE_PARECE`] y [`Aviso::es_observacion`].
//!
//! ─────────────────────────────────────────────────────────────────────────────────────────────
//!
//! ## Las tres reglas, y qué se puede hacer de verdad con cada una
//!
//! ### A · COMBUSTIBLE — se construye, y son **sus** umbrales
//!
//! El cliente pide: *«si el agua corporal baja de 54 %, añadir un bidón extra con electrolitos y un
//! consomé»* y *«si el entrenamiento supera 100 TSS, subir los carbohidratos de la cena»*.
//!
//! **LOS DOS UMBRALES SON SUYOS, NO HALLAZGOS.** El 54 % y el de 100 TSS salen de su documento, no de
//! una fuente. Por eso la aplicación **le recuerda su propia regla**; no le da una indicación nueva.
//! Y por eso **no se llama «peligro» a nada de esto**: un umbral que él eligió no es un hallazgo
//! clínico, y presentarlo como tal sería darle a su propia pauta una autoridad que no tiene.
//!
//! **El agua corporal la mete él a mano** —viene de su báscula de Huawei, no de los ficheros FIT— así
//! que si no la ha metido, **no hay aviso y se dice que falta**. El TSS sí está en los ficheros.
//!
//! **Y EL CONSOMÉ Y LOS ELECTROLITOS NO SE AFIRMAN.** La aplicación los **recuerda como lo que son
//! —su propia pauta—** y **no dice que sirvan**. No hay ni una fuente que sostenga que un consomé
//! reponga electrolitos mejor que otra cosa, y la norma de este proyecto es que **lo que no se puede
//! citar no se calcula**. Se escriben porque son suyos, no porque funcionen.
//!
//! **Y una cosa que la aplicación NO PUEDE HACER, y hay que decirla**: la regla dice «subir los
//! carbohidratos de la cena», y la aplicación **no sabe qué comió**. No tiene un diario de comidas ni
//! lo va a tener por leer un FIT. Así que **recuerda la pauta y no la comprueba**: no puede felicitarle
//! por haberla cumplido ni avisarle de que no lo hizo. Eso es imposible con estos datos, y callarlo
//! sería fingir una capacidad que no existe.
//!
//! ### B · RODILLA — se construye **sobre otro dato**, y con una regla de honestidad
//!
//! El cliente pide avisar *«si el balance de pedaleo se desvía más allá de 48 %/52 %»*. **ESE DATO NO
//! ESTÁ EN LOS FICHEROS.** El reparto de potencia izquierda/derecha **lo calcula Garmin en su servidor
//! y no viaja en el FIT**; el campo que parece traerlo (`left_right_balance`) trae **el número de serie
//! del sensor**. Está medido y explicado en `docs/ENCARGO-farellones-2026.md` §3. Inventar ese 47/53
//! sería el peor error posible en este proyecto: **un número plausible sobre el cuerpo de una persona**.
//!
//! **LO QUE SÍ EXISTE es mejor para lo que le preocupa**: `avg_left_power_phase` y
//! `avg_right_power_phase`, **la fase del pedaleo de cada pierna, cuatro ángulos** — medido: viene en
//! **83 de sus 102 ficheros**, y las dos piernas **difieren en 80 de esas 83**. Es una medida **de
//! mecánica**, y por tanto más cercana a una compensación de rodilla que un reparto de vatios.
//!
//! **Y el corte no se inventa: sale de sus propios datos.** Ver [`FACTOR_SOBRE_SU_BASE`].
//!
//! ### C · RECUPERACIÓN — se construye **sobre otro dato**
//!
//! El cliente pide avisar *«si el pulso en reposo sube más de un 10 % (78+ ppm) o el sueño es malo»*.
//! **NI EL PULSO EN REPOSO NI EL SUEÑO TIENEN SERIE**: sus 165 ficheros de sueño y sus 165 de pulso
//! **vienen todos vacíos**. Y este cliente **empieza a entrenar con el pulso ya alto** —mediana del
//! mínimo 90 ppm, y **solo 1 de sus 97 sesiones baja de 60**— así que **de un entrenamiento no se saca
//! su pulso en reposo**. Su *«si sube un 10 %»* **no se puede aplicar hoy**, porque no hay serie con la
//! que comparar.
//!
//! **LO QUE SÍ TIENE SERIE Y DICE LO MISMO es la deriva cardíaca**, y ya está calculada en
//! [`crate::patron::deriva`] — no se reescribe aquí, se reutiliza. Medido sobre sus sesiones: **media
//! 9,7 %, mediana 10,8 %, y 43 por encima del 10 %**. La deriva es **el indicador clásico de
//! deshidratación, calor o falta de fondo** ([Friel, *Aerobic Decoupling and Heart Rate Drift
//! Explained*](https://www.trainingpeaks.com/coach-blog/aerobic-endurance-and-decoupling/)), o sea
//! **dice lo mismo que la regla C del cliente, pero con datos que existen**.
//!
//! **Y se compara cada sesión CON SU PROPIA línea base**, no contra las bandas publicadas. El motivo
//! está medido y es contundente: las bandas publicadas —**< 5 % buena, 5-10 % aparece, > 10 % se
//! escapa** ([TrainingPeaks, *Aerobic Decoupling (Pw:Hr and Pa:HR)*](https://help.trainingpeaks.com/hc/en-us/articles/204071724-Aerobic-Decoupling-Pw-Hr-and-Pa-HR-and-Efficiency-Factor-EF))—
//! marcarían **55 de sus 78 sesiones con deriva calculable**, porque **su mediana (10,75 %) ya está
//! por encima del corte publicado del 10 %**. Una regla que señala a siete de cada diez entrenamientos
//! **no informa: hace ruido**, y un aviso que salta siempre se deja de leer. Por eso el corte sale de
//! **su** reparto, y las bandas publicadas se citan para poder decirlas, no para disparar con ellas.
//!
//! ─────────────────────────────────────────────────────────────────────────────────────────────
//!
//! ## Lo que NO se construye, y por qué (escrito aquí para que nadie lo «arregle» mañana)
//!
//! - **Nada de IA.** Ninguna de las tres reglas la necesita, y esto decide sobre el cuerpo de una persona.
//! - **Nada de suplementos como indicación.** CoQ10, Omega-3 y Calcio/D3 se pueden **listar como su
//!   pauta**, nunca afirmar que sirven. No hay fuente que lo sostenga.
//! - **Nada de «recalcular la cena de la noche anterior».** La aplicación **no sabe qué comió**: no hay
//!   diario de comidas en sus ficheros. Es imposible, y por eso la pauta se **recuerda** y no se comprueba.

use crate::datos::{Dato, Persona, Sesion};
use crate::juicio::{Confianza, Fundamento};
use crate::patron;
use std::collections::BTreeMap;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────────────────────
// Las umbrales, cada uno con su fuente o su marca de «pauta del cliente»
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **El umbral de agua corporal del cliente: 54 %.**
///
/// ESTE NÚMERO NO ES UN HALLAZGO. Es **la pauta que él escribió** en su encargo, y la aplicación se
/// limita a **recordársela**. No sale de ningún estudio, y por eso no se presenta como una alarma
/// clínica: **es su regla, no un diagnóstico**.
///
/// La medición de la que habla —**55,3 %** en su captura de Huawei— es **manual**: viene de su báscula,
/// no de un FIT. Si no la ha metido, no hay aviso.
pub const UMBRAL_AGUA_DEL_CLIENTE_PCT: f64 = 54.0;

/// Qué se hace cuando el agua corporal vale **exactamente** el umbral.
///
/// LA DECISIÓN, y va escrita porque el encargo la pide explícitamente: la regla del cliente dice
/// *«si baja de 54 %»*, y **54 no es «por debajo de 54»**. Así que **en 54,0 exacto NO se emite el
/// aviso**. Se elige la lectura LITERAL de su frase por dos motivos: es **su** regla y no se le
/// corrige el sentido al recordársela, y **un valor de corte tiene una sola dirección** — si disparara
/// en 54,00 y también en 53,99, el umbral sería «54 o menos», que es otra regla distinta de la que él
/// escribió.
///
/// Y la consecuencia práctica es mínima, que es lo que hace segura esta elección: **la báscula da un
/// decimal**, así que 54,0 es un valor que aparece, y tratarlo como «aún no toca» es lo conservador.
pub const EN_EL_UMBRAL_DE_AGUA_DISPARA: bool = false;

/// **El umbral de TSS del cliente: 100.**
///
/// TAMPOCO ES UN HALLAZGO: es **su pauta**, y el TSS que la alimenta **sí está en sus ficheros** —el
/// reloj lo calcula y lo guarda en `Session.training_stress_score`—. Medido: viene en **83 de sus 102
/// sesiones**, con mediana **64** y máximo **222**, y **18 sesiones por encima de 100**.
pub const UMBRAL_TSS_DEL_CLIENTE: f64 = 100.0;

/// Qué se hace cuando el TSS vale **exactamente** 100.
///
/// La regla del cliente dice *«si el entrenamiento supera 100 TSS»*, y **100 no supera 100**. Así que
/// **en 100,0 exacto NO se emite el aviso**, por la misma razón que en el agua: se respeta la letra de
/// su regla y **el corte tiene una sola dirección**.
///
/// Y aquí la elección **no cambia su historia**: medido, entre sus 83 sesiones con TSS **ninguna vale
/// exactamente 100** (la más cercana está por encima o por debajo), así que esta decisión **no emite ni
/// silencia ni un solo aviso suyo**. Se escribe igual porque una prueba la fija, y porque el día que
/// una sesión valga 100 clavados tiene que haber una respuesta decidida y no un `>` puesto al azar.
pub const EN_EL_UMBRAL_DE_TSS_DISPARA: bool = false;

/// **Cuánto tiene que desviarse una pierna de SU línea base para que se avise: el doble.**
///
/// ESTE CORTE SÍ ES UN HALLAZGO, y sale de sus datos, no de una tabla. Cómo se llegó a él:
///
/// La línea base es **la mediana de la asimetría de sus propias sesiones** —cuánto se separan las dos
/// fases de pedaleo, en grados, con la diferencia **circular** (357° y 2° están a 5°, no a 355°)—.
/// Medido sobre sus **83 sesiones con fase**: mediana **2,81°**, y la asimetría va de **0° a 43,59°**.
///
/// **Por qué el doble y no «mediana + 2 desviaciones»**: su reparto tiene una cola larga —una sesión
/// de 43,59° y tres más por encima de 25°—, y la desviación típica **se deja arrastrar por la cola**
/// (sale 6,52°, y mediana + 2 desviaciones daría 15,9°, un corte que **no señalaría nada**). El
/// **doble de su mediana** es 5,63°, y **separa 23 de sus 83 sesiones (el 28 %)**. Es además una regla
/// que se puede **decir en una frase**: *«avisa cuando una pierna se desvía el doble de lo que suele
/// desviarse la tuya»* — y una regla que no se puede explicar no se puede revisar.
///
/// **Y es un umbral relativo a propósito**: si su mecánica mejora, su mediana baja y el corte baja con
/// ella. Un corte fijo en grados se quedaría obsoleto en cuanto mejorase, y nadie lo notaría.
pub const FACTOR_SOBRE_SU_BASE: f64 = 2.0;

/// Qué se hace cuando la asimetría vale **exactamente** el doble de su línea base.
///
/// **SE EXIGE SUPERARLO, NO ALCANZARLO**, y esta decisión costó una medición que hay que dejar escrita.
///
/// El motivo de fondo es el de siempre: **un valor de corte tiene UNA sola dirección**. Pero aquí hay
/// además una trampa que solo se ve midiendo, y es esta: **la fase de pedaleo viene CUANTIZADA**. El
/// FIT la guarda en un byte y el lector la escala a grados, así que **los únicos valores posibles son
/// múltiplos de 1,40625°** (360/256). Medido en sus 83 sesiones, aparecen exactamente estos peldaños:
/// 0 · 1,406 · 2,8125 · 4,2188 · 5,625 · 7,031 · 8,4375 · 9,8438 · 11,25 · 12,656 … y luego ya saltos
/// grandes.
///
/// **Y eso significa que el corte cae SIEMPRE sobre un peldaño**, no entre dos: su mediana es
/// **2,8125°** (dos peldaños), así que el doble es **5,625°** (cuatro peldaños) — un valor que
/// **9 de sus 83 sesiones tienen**. La consecuencia, medida:
///
/// | lectura | sesiones que avisa |
/// |---|---:|
/// | «alcanzar el doble» (`>=`) | **28** de 83 |
/// | «superar el doble» (`>`) | **20** de 83 |
///
/// O sea que **la elección de esta constante mueve 8 avisos**, y por eso se decide aquí y no en un `>`
/// puesto al azar. Se elige **superarlo**, por dos razones:
///
/// 1. **La regla del cliente dice «se desvía MÁS ALLÁ de»**, y esa es la misma dirección: más allá del
///    doble, no en el doble.
/// 2. **Su mecánica no empeoró en un peldaño**: un peldaño es **un paso de la escala del aparato**, no
///    una pierna desviándose más. Avisar por estar en el peldaño exacto es avisar **por cómo está
///    guardado el número**, no por lo que hizo con la bicicleta.
///
/// **Y esto se dice aunque el número sea menos redondo**, porque callarlo dejaría un aviso que cambia
/// 5 sesiones según cómo se escriba una comparación — y ese es exactamente el tipo de detalle que nadie
/// vuelve a mirar. El día que se quiera el otro comportamiento, se cambia **esta constante** y el
/// informe dirá otro número, con las dos cifras aquí al lado para poder compararlas.
pub const EN_LA_BASE_DE_FASE_DISPARA: bool = false;

/// **Cuánto tiene que desviarse la deriva de SU línea base para que se avise: una vez y media.**
///
/// TAMBIÉN ES UN HALLAZGO, y sale de sus datos. Medido sobre sus **78 sesiones con deriva calculable**
/// (las 102 menos 19 sin sensor de potencia y 5 con menos de 20 pares pulso/potencia): mediana
/// **10,75 %**, y **MAD 5,72 %**.
///
/// **Por qué la MAD y no la desviación típica**: su deriva va de **−23,6 % a +34,7 %**, y hay **17
/// sesiones con deriva negativa** (mejoró en la segunda mitad). Con esa cola, la desviación típica se
/// infla hasta **9,72 %**, y mediana + 1 desviación daría **19,4 %**, un corte **más alto que el
/// máximo de casi todas sus sesiones**: no señalaría casi nada. La **desviación absoluta mediana**
/// (MAD) mide **cuánto se aparta una sesión típica de su mediana**, y **no la mueven las colas**.
///
/// Medido, los tres candidatos sobre sus datos:
///
/// | corte | valor | sesiones que separa |
/// |---|---:|---:|
/// | mediana + 1,0 × MAD | 16,47 % | 15 de 78 |
/// | **mediana + 1,5 × MAD** | **19,33 %** | **10 de 78** |
/// | mediana + 2,0 × MAD | 22,18 % | 4 de 78 |
///
/// Se elige **1,5** porque es el que deja el aviso en **una de cada ocho sesiones**: con 1,0 saltaría
/// una de cada cinco —demasiado para que se lea—, y con 2,0 solo cuatro veces en todo su histórico, y
/// un aviso que casi nunca sale **no se aprende**. Y hay una comprobación que lo respalda: **las 10
/// sesiones que separa coinciden con las 10 que pasan del 20 % de deriva**, o sea que en la práctica
/// el corte es «su deriva se ha ido al doble de lo normal».
pub const FACTOR_SOBRE_SU_MAD: f64 = 1.5;

/// **Cuántas sesiones hacen falta para poder decir «esta es tu línea base».**
///
/// ES UNA DECISIÓN, y va aquí con su motivo: la línea base es **una mediana**, y **una mediana de dos
/// o tres valores no es una norma: es un accidente**. Con tres sesiones, el valor del medio es
/// cualquiera de las tres, y comparar la cuarta contra él diría «te desvías» por puro azar.
///
/// Se elige **10** porque es el punto en el que la mediana deja de moverse de golpe al añadir una
/// sesión, y porque un aviso sobre el cuerpo de una persona **no debe emitirse el primer día que se
/// abre la aplicación**. Y no es un número que le quite nada: medido, **él tiene 83 sesiones con fase
/// y 78 con deriva**, o sea que en su caso la línea base **siempre se puede calcular**. El mínimo solo
/// actúa cuando los datos son de otro, o cuando alguien acaba de empezar.
pub const MINIMO_PARA_LINEA_BASE: usize = 10;

/// **Cuántos pares (pulso, potencia) hacen falta para que una deriva signifique algo.**
///
/// No es un número nuevo: es **el mismo 20 que ya usa [`crate::patron::deriva`]** y el que usa
/// [`crate::analisis::de_lecturas`] para contar una sesión como «con deriva». Se repite aquí **citando
/// su origen** en vez de inventar otro, porque dos mínimos distintos en el mismo motor serían dos
/// verdades sobre lo mismo.
pub const MINIMO_MUESTRAS_DERIVA: usize = 20;

/// **El paso de la escala de la fase de pedaleo: 360/256 = 1,40625 grados.**
///
/// POR QUÉ ESTE NÚMERO ES IMPORTANTE Y NO UN DETALLE DE IMPLEMENTACIÓN: el FIT guarda la fase de
/// pedaleo en **un byte**, y ese byte reparte la vuelta entera de la biela, así que **los únicos
/// valores que puede tomar son múltiplos de 360/256**. Medido en sus 83 sesiones, aparecen exactamente
/// estos peldaños: **0 · 1,406 · 2,8125 · 4,2188 · 5,625 · 7,031 · 8,4375 · 9,8438 · 11,25 · 12,656 …**
///
/// **Y ESO CAMBIA CÓMO HAY QUE COMPARAR**, que es lo que costó dos intentos en este módulo. Un corte
/// calculado como «el doble de su mediana» **no cae en un peldaño**, mientras que las asimetrías **sí**.
/// La primera versión comparaba `asimetria > corte` con números en coma flotante, y el resultado
/// dependía de diferencias de **3·10⁻¹⁴ grados** — doce órdenes de magnitud por debajo del propio paso
/// de la escala. Tres sesiones que estaban **en el mismo peldaño que el corte** entraban o no según el
/// ruido del redondeo, y el aviso decía 23 donde hay 20.
///
/// La regla que sale de esto, y es la única honesta: **no se puede distinguir nada más fino que el
/// paso de la escala**. Por eso el corte se **alinea al peldaño** con [`alinear_a_la_escala`] antes de
/// comparar, y así la decisión depende de la escala del aparato y no del ruido del ordenador.
pub const PASO_DE_LA_FASE_GRADOS: f64 = 360.0 / 256.0;

/// **Alinea un valor al peldaño más cercano de la escala de la fase.**
///
/// Ver [`PASO_DE_LA_FASE_GRADOS`] para el porqué. Se usa para el corte y **no para los datos**, porque
/// los datos **ya vienen** en la escala: alinearlos otra vez no cambiaría nada y sugeriría que el
/// lector los está corrompiendo cuando el que redondea es el aparato.
pub fn alinear_a_la_escala(valor: f64) -> f64 {
    (valor / PASO_DE_LA_FASE_GRADOS).round() * PASO_DE_LA_FASE_GRADOS
}

/// **Los datos que la aplicación NO tiene, y que hay que decir en voz alta.**
///
/// Existe para que estos tres textos se escriban **una vez** y no se reescriban distinto en cada sitio
/// que los necesite. Y son la mitad del valor de este módulo: **un aviso que no sale porque falta un
/// dato es una respuesta**, y el usuario tiene que poder leerla.
///
/// VA COMO STRUCT DE CONSTANTES Y NO COMO MÓDULO, que sería lo natural, por un motivo medido: Rust
/// exige que los módulos vayan en `snake_case`, y `mod NO_ES_LO_QUE_PARECE` **produce un aviso de
/// compilación** — comprobado: `warning: module NO_ES_LO_QUE_PARECE should have a snake case name`.
/// El encargo exige **cero avisos**, así que el nombre que se lee como una frase se conserva **en el
/// tipo** y las constantes van dentro. Se elige eso antes que escribir `no_es_lo_que_parece`, que
/// perdería justo lo que el nombre quiere decir.
pub struct NoEsLoQueParece;

impl NoEsLoQueParece {
    /// Por qué no hay aviso de balance izquierda/derecha.
    pub const SIN_REPARTO_DE_POTENCIA: &str = "el reparto de potencia izquierda/derecha NO viaja en el fichero FIT: \
lo calcula Garmin en su servidor y lo enseña en su aplicación. El campo que parece traerlo \
(`left_right_balance`) trae el número de serie del sensor. Se puede mirar en Garmin Connect y anotarlo a mano.";

    /// Por qué no hay aviso de pulso en reposo ni de sueño.
    pub const SIN_PULSO_EN_REPOSO: &str = "no hay serie de pulso en reposo: sus ficheros diarios de pulso y de sueño \
vienen vacíos. Y de un entrenamiento no se puede deducir, porque empieza a entrenar con el pulso ya alto \
(mediana del mínimo 90 ppm, y solo 1 de sus 97 sesiones baja de 60).";

    /// Por qué la pauta de la cena se recuerda y no se comprueba.
    pub const SIN_DIARIO_DE_COMIDAS: &str = "la aplicación no sabe qué comió: no hay diario de comidas en sus \
ficheros. La pauta de la cena se recuerda y NO se comprueba — no puede decirle si la cumplió.";

    /// Por qué los suplementos y el consomé no se afirman.
    pub const SIN_FUENTE_DE_SUPLEMENTOS: &str = "no hay ninguna fuente que sostenga que los electrolitos, el consomé \
o los suplementos (CoQ10, Omega-3, Calcio/D3) sirvan para esto. Se recuerdan como pauta suya y no se afirma que funcionen.";
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// Las fuentes citadas
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// La fuente de las **bandas publicadas** de deriva.
///
/// ESTA CITA EXISTE POR UN MOTIVO CONCRETO: el encargo dice *«cita la fuente de la deriva»*, y hasta
/// hoy **no estaba citada en ningún sitio del proyecto** — [`crate::patron::deriva`] declara sus
/// cortes del 5 % y del 10 % como *«una DECISIÓN, no un hallazgo»*, y con razón: los usaba sin fuente.
/// Se buscó, y **sí hay fuente**:
///
/// - **[Friel, *Aerobic Decoupling and Heart Rate Drift Explained*](https://www.trainingpeaks.com/coach-blog/aerobic-endurance-and-decoupling/)**
///   da las tres bandas —**< 5 % buena resistencia aeróbica, 5-10 % limitación moderada o fatiga,
///   > 10 % el esfuerzo estaba por encima del umbral aeróbico o falta fondo**— y **nombra las causas
///   que a este cliente le importan**: *«rising core temperature, dehydration, glycogen depletion, and
///   cardiovascular fatigue»*. Es la cita que sostiene que la deriva **hable de deshidratación y calor**,
///   que es exactamente lo que la regla A del cliente quería vigilar.
/// - **[TrainingPeaks, *Aerobic Decoupling (Pw:Hr and Pa:HR) and Efficiency Factor (EF)*](https://help.trainingpeaks.com/hc/en-us/articles/204071724-Aerobic-Decoupling-Pw-Hr-and-Pa-HR-and-Efficiency-Factor-EF)**
///   da la definición operativa —**compara el factor de eficiencia de la primera mitad con el de la
///   segunda**— que es **la misma cuenta que ya hace [`crate::patron::deriva`]**, y el corte
///   **< 5 % = buena resistencia aeróbica a esa duración y esa potencia**, además de avisar de que
///   **por debajo de 20 minutos el valor no es válido**. Ese aviso de los 20 minutos es,
///   literalmente, el [`MINIMO_MUESTRAS_DERIVA`] que ya usaba el motor.
///
/// **Y una advertencia que va con la cita**: estas bandas son **de poblaciones de deportistas**, y por
/// eso **no son el corte de este módulo**. Su mediana (10,75 %) ya cae por encima de la banda del
/// 10 %, así que aplicarlas dispararía en 55 de sus 78 sesiones. Se citan **para poder decirlas**, no
/// para disparar con ellas.
pub const FUENTE_DERIVA: &str = "Friel, «Aerobic Decoupling and Heart Rate Drift Explained» (TrainingPeaks): \
< 5 % buena resistencia aeróbica, 5-10 % fatiga o limitación, > 10 % por encima del umbral aeróbico o falta de fondo; \
y nombra el calor, la deshidratación y la falta de glucógeno entre las causas. TrainingPeaks da la misma cuenta \
—primera mitad contra segunda— y avisa de que por debajo de 20 minutos el valor no es válido.";

/// La fuente que sostiene que **la fase de pedaleo es una medida por pierna**.
///
/// POR QUÉ HACE FALTA CITAR ESTO: porque el campo de al lado (`avg_power_position`) **parece** decir
/// izquierda y derecha y **no lo dice** —son dos huecos indexados por `rider_position_type`, o sea
/// **sentado y de pie**—. El proyecto ya se equivocó una vez leyéndolo al revés y sacando un 37/63 que
/// no existía. Esta cita es la que distingue los dos campos, y por eso va pegada al aviso.
pub const FUENTE_FASE: &str = "Perfil oficial del formato FIT (Garmin, `Profile.xlsx`): `avg_left_power_phase` y \
`avg_right_power_phase` son cuatro ángulos **por pierna**; `avg_power_position` y `avg_cadence_position` son dos \
huecos **por postura**, indexados por `rider_position_type` (`Seated = 0`, `Standing = 1`).";

/// Cómo se dice que una observación **no es un diagnóstico**, sin usar la palabra prohibida.
///
/// ESTE TEXTO ES LA REGLA DE HONESTIDAD DEL MÓDULO B, y su redacción es una decisión que conviene
/// explicar: el encargo pide que el aviso *«diga que es una observación de sus datos, no un
/// diagnóstico»*. Aquí se dice **lo mismo con otras palabras**, y el motivo es que el propio encargo
/// prohíbe después —en las pruebas que hay que escribir— que el texto de un aviso **contenga** las
/// palabras «diagnóstico» y «lesión». Las dos cosas a la vez solo se pueden cumplir así: **diciendo el
/// sentido y no el término**. Y no se pierde nada: quien lee «esto no es una valoración médica» entiende
/// exactamente lo mismo que quien lee «esto no es un diagnóstico», y además el texto **no se puede
/// confundir con un informe clínico ni recortado fuera de contexto**.
pub const NO_ES_VALORACION_MEDICA: &str = "es una observación de sus propios datos, no una valoración médica: \
describe cómo se reparte el pedaleo entre sus dos piernas, y nada más. No dice qué le pasa ni qué hacer al respecto.";

// ─────────────────────────────────────────────────────────────────────────────────────────────
// El aviso
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **Cuál de los tres módulos del encargo** produjo el aviso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modulo {
    /// **A** · combustible: agua corporal y carga de la sesión.
    Combustible,
    /// **B** · rodilla: la mecánica del pedaleo, medida por la fase de cada pierna.
    Rodilla,
    /// **C** · recuperación: la deriva cardíaca frente a su propia línea base.
    Recuperacion,
}

impl Modulo {
    pub fn nombre(self) -> &'static str {
        match self {
            Modulo::Combustible => "combustible",
            Modulo::Rodilla => "rodilla",
            Modulo::Recuperacion => "recuperación",
        }
    }

    /// **Qué vigila este módulo**, en una línea. Es lo que se enseña al lado del título.
    pub fn vigila(self) -> &'static str {
        match self {
            Modulo::Combustible => "el agua corporal y la carga de la sesión",
            Modulo::Rodilla => "cómo se reparte el pedaleo entre las dos piernas",
            Modulo::Recuperacion => "si el pulso se le escapa a lo largo de la sesión",
        }
    }
}

/// **Cuánto corre prisa**, en tres pasos y ni uno más.
///
/// SE ELIGEN TRES Y NO CINCO a propósito: un semáforo de cinco colores sobre el cuerpo de una persona
/// no se lee, se hojea. Y se llaman así, y no «info/aviso/crítico», porque **ninguno de estos avisos es
/// una urgencia médica**: son cosas que él ya decidió de antemano que quería saber.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Urgencia {
    /// Se lo cuenta, sin más: es una observación de sus datos.
    Informa,
    /// Conviene mirarlo: se ha salido de su norma.
    Atencion,
    /// **Ya**: su propia regla se ha cumplido y toca hacer lo que él decidió.
    Hoy,
}

impl Urgencia {
    pub fn nombre(self) -> &'static str {
        match self {
            Urgencia::Informa => "informa",
            Urgencia::Atencion => "atención",
            Urgencia::Hoy => "hoy",
        }
    }
}

/// **Un aviso**: qué pasa, de dónde sale, y qué toca hacer.
///
/// EL CAMPO QUE LO HACE HONESTO ES `fundamento`: lleva **la afirmación, su fuente y su reserva**, con
/// los tipos que ya usaba [`crate::juicio`]. La reserva **no es opcional cuando hay algo que callar**,
/// y por eso el módulo de la rodilla la usa siempre: un aviso sobre una pierna **tiene que decir que es
/// una observación**, o se convierte en algo que este programa no es.
#[derive(Debug, Clone, PartialEq)]
pub struct Aviso {
    pub modulo: Modulo,
    pub urgencia: Urgencia,
    /// **Una sola línea.** Si hace falta una segunda, el aviso está mal hecho.
    pub titulo: String,
    /// De dónde sale, **con el número y su margen**. Nunca una frase vacía.
    pub porque: String,
    /// **Una sola línea.** Qué toca hacer.
    pub que_hacer: String,
    /// Citado o medido, **nunca inventado**.
    pub fundamento: Fundamento,
}

impl Aviso {
    /// Construye un aviso. Existe para que **no se pueda crear uno sin fundamento**: los cuatro campos
    /// van juntos en la llamada, así que no hay forma de escribir un `Aviso` al que le falte la fuente.
    pub fn nuevo(
        modulo: Modulo,
        urgencia: Urgencia,
        titulo: impl Into<String>,
        porque: impl Into<String>,
        que_hacer: impl Into<String>,
        fuente: impl Into<String>,
        reserva: Option<String>,
    ) -> Self {
        // Se convierte UNA vez y se clona para los dos sitios que lo llevan: `porque` y
        // `fundamento.afirmacion` son **el mismo texto a propósito**, y tenerlo duplicado permitiría
        // que un día dijeran cosas distintas — el aviso contando una cosa y su fundamento otra.
        let porque = porque.into();
        Self {
            modulo,
            urgencia,
            titulo: titulo.into(),
            porque: porque.clone(),
            que_hacer: que_hacer.into(),
            fundamento: Fundamento {
                afirmacion: porque,
                fuente: fuente.into(),
                reserva,
            },
        }
    }

    /// **Todo el texto que verá el usuario**, junto. Es sobre lo que se comprueba que no se cuele una
    /// promesa ni una palabra clínica.
    pub fn texto(&self) -> String {
        let mut s = format!(
            "{} {} {} {} {}",
            self.titulo, self.porque, self.que_hacer, self.fundamento.fuente, self.fundamento.afirmacion
        );
        if let Some(r) = &self.fundamento.reserva {
            s.push(' ');
            s.push_str(r);
        }
        s
    }

    /// **¿Este aviso se presenta como una observación y no como un juicio clínico?**
    ///
    /// Existe para que la regla de honestidad del módulo B **se pueda comprobar** en vez de confiar en
    /// que quien escriba el próximo texto se acuerde. Un aviso de la rodilla **sin reserva** no es un
    /// aviso incompleto: es **un aviso que afirma más de lo que sabe**, y esta función lo delata.
    pub fn es_observacion(&self) -> bool {
        self.modulo != Modulo::Rodilla
            || self
                .fundamento
                .reserva
                .as_ref()
                .map(|r| !r.trim().is_empty())
                .unwrap_or(false)
    }

    /// Cuánta confianza merece, en el vocabulario del motor.
    ///
    /// **Ningún aviso de este módulo es `Seguro` si le falta la línea base**: comparar contra una
    /// mediana que no se pudo calcular daría un aviso inventado. Cuando la reserva dice que hay algo
    /// que el dato no cubre, la confianza **baja a `ConReserva`** — la regla de [`crate::juicio`], que
    /// no se sube por tener muchos datos y sí se baja por faltar uno.
    pub fn confianza(&self) -> Confianza {
        if self.fundamento.reserva.is_some() {
            Confianza::ConReserva
        } else {
            Confianza::Seguro
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// Lo que la aplicación sabe hoy: lo que él mete a mano y la carpeta entera
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **Lo que el cliente aporta hoy, a mano** — no lo que miden los sensores.
///
/// ES UN STRUCT APARTE DE [`Persona`] A PROPÓSITO: `Persona` son los datos **estables** (edad, peso,
/// umbral de lactato) y esto es **el parte del día**: el agua corporal de esta mañana. Mezclarlos haría
/// que el peso de hace tres meses y el agua de hoy viajaran juntos, y **un dato que caduca y nadie lo
/// nota es justo el error que el proyecto ya corrigió una vez** con la edad escrita a mano.
#[derive(Debug, Clone, PartialEq)]
pub struct Hoy {
    /// **Agua corporal, en porcentaje.** Viene de su báscula de Huawei, **la mete él**.
    ///
    /// Es `Dato` y **no `f64` con un cero por defecto**, por la regla del modelo: **`Falta` no es `0`**.
    /// Un «0 % de agua corporal» dispararía la regla de una forma absurda, y es exactamente lo que
    /// pasaría si esto fuera un número sin más.
    pub agua_corporal_pct: Dato<f32>,
    /// **Su umbral de agua corporal**, para que la aplicación **use el suyo y no uno escrito dentro**.
    ///
    /// Existe por la misma razón por la que existe todo este módulo: **el 54 % es su pauta**, y el día
    /// que la cambie a 53 no se tiene que tocar el código. El valor de arranque es
    /// [`UMBRAL_AGUA_DEL_CLIENTE_PCT`], que es el que él escribió.
    pub umbral_agua_pct: f64,
}

impl Default for Hoy {
    fn default() -> Self {
        Self {
            agua_corporal_pct: Dato::Falta,
            umbral_agua_pct: UMBRAL_AGUA_DEL_CLIENTE_PCT,
        }
    }
}

impl Hoy {
    /// Un día del que **no se sabe nada**: sin agua corporal. Es como arranca la aplicación, y es honesto.
    pub fn sin_datos() -> Self {
        Self::default()
    }

    /// Un día con el agua corporal que él acaba de meter, con **su** umbral.
    pub fn con_agua(pct: f32) -> Self {
        Self {
            agua_corporal_pct: Dato::Hay(pct),
            umbral_agua_pct: UMBRAL_AGUA_DEL_CLIENTE_PCT,
        }
    }
}

/// **Una sesión vista como este módulo la necesita**: pocos números, y **cada uno con su `Option`**.
///
/// POR QUÉ NO SE PASA LA `Sesion` ENTERA: porque este módulo **no mira las muestras** —549.842
/// muestras para tres reglas que se deciden con cinco números— y porque **un `Option` en cada campo
/// obliga a decidir qué hacer cuando falta**, que es justo lo que el encargo pide. Con la `Sesion`
/// entera, «no hay fase» y «la fase es cero» se parecerían demasiado.
#[derive(Debug, Clone, PartialEq)]
pub struct SesionVista {
    /// El fichero del que salió, para poder nombrarla.
    pub nombre: String,
    /// **Cuándo empezó**, en segundos desde el 1 de enero de 1970. `None` si el fichero no lo trae.
    pub inicio_s: Option<i64>,
    /// El **TSS** que declara el fichero, tal cual. `None` si esa sesión no lo trae.
    pub tss: Dato<f64>,
    /// **La asimetría de la fase de pedaleo**, en grados: cuánto se separan las dos piernas.
    ///
    /// `None` cuando falta **cualquiera** de las dos fases —comparar una pierna contra un hueco no dice
    /// nada— y también cuando el fichero no trae fase. Ver [`asimetria_de_fase`].
    pub asimetria_grados: Option<f64>,
    /// **La deriva cardíaca**, en porcentaje. Se calcula con [`crate::patron::deriva`], **no aquí**.
    pub deriva_pct: Option<f64>,
}

impl SesionVista {
    /// Una sesión de la que no se sabe nada. Sirve para las pruebas y para el arranque.
    pub fn vacia(nombre: &str) -> Self {
        Self {
            nombre: nombre.to_string(),
            inicio_s: None,
            tss: Dato::Falta,
            asimetria_grados: None,
            deriva_pct: None,
        }
    }

    /// La fecha de la sesión, en `AAAA-MM-DD`, si el fichero la trajo.
    pub fn fecha(&self) -> Option<String> {
        self.inicio_s.map(fecha_de_epoch)
    }
}

/// **Todo el histórico, resumido en lo que estas tres reglas necesitan.**
///
/// Se construye una vez al abrir la aplicación y **se pasa por referencia**: recalcularlo por aviso
/// sería leer la carpeta tres veces.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Historico {
    /// **Cuántas sesiones hay en la carpeta**, se hayan podido leer o no. Es el denominador de todo, y
    /// sin él un «18 de 102» se convertiría en un «18» que parece otra cosa.
    pub total_sesiones: usize,
    /// Los ficheros que **no se pudieron leer**, con su motivo. Un dato que falta hay que decirlo.
    pub no_leidos: Vec<(String, String)>,
    /// Una entrada por sesión, **en el orden en que se leyeron**.
    pub sesiones: Vec<SesionVista>,
}

impl Historico {
    /// **La línea base de la fase de pedaleo**: la **mediana** de la asimetría de sus sesiones.
    ///
    /// Devuelve `None` si hay menos de [`MINIMO_PARA_LINEA_BASE`] sesiones con fase. **No devuelve un
    /// cero**: cero sería una afirmación («sus piernas van iguales») y aquí no se sabe.
    ///
    /// Es la **mediana** y no la media a propósito: con una cola de 43,59°, la media (5,35°) describe
    /// a una sesión que no existe.
    pub fn base_de_fase(&self) -> Option<f64> {
        let mut v: Vec<f64> = self.sesiones.iter().filter_map(|s| s.asimetria_grados).collect();
        if v.len() < MINIMO_PARA_LINEA_BASE {
            return None;
        }
        Some(mediana(&mut v))
    }

    /// **La línea base de la deriva**: la **mediana** de sus derivas, en porcentaje.
    pub fn base_de_deriva(&self) -> Option<f64> {
        let mut v: Vec<f64> = self.sesiones.iter().filter_map(|s| s.deriva_pct).collect();
        if v.len() < MINIMO_PARA_LINEA_BASE {
            return None;
        }
        Some(mediana(&mut v))
    }

    /// **Cuánto se aparta una sesión típica de la deriva de su mediana**: la MAD.
    ///
    /// POR QUÉ LA MAD Y NO LA DESVIACIÓN TÍPICA, medido en sus datos: su deriva va de −23,6 % a
    /// +34,7 % y tiene 17 sesiones negativas, así que la desviación típica (9,72 %) está inflada por
    /// las colas y daría un corte de 19,4 % que no señalaría casi nada. La MAD (5,72 %) mide la
    /// dispersión **de la mitad central** y no la mueven los extremos.
    pub fn mad_de_deriva(&self) -> Option<f64> {
        let base = self.base_de_deriva()?;
        let mut v: Vec<f64> = self
            .sesiones
            .iter()
            .filter_map(|s| s.deriva_pct)
            .map(|d| (d - base).abs())
            .collect();
        Some(mediana(&mut v))
    }

    /// **El corte de la deriva**: su mediana más [`FACTOR_SOBRE_SU_MAD`] veces su MAD.
    ///
    /// Medido sobre sus 78 sesiones con deriva: **19,33 %**, que separa **10 de 78**.
    pub fn corte_de_deriva(&self) -> Option<f64> {
        Some(self.base_de_deriva()? + FACTOR_SOBRE_SU_MAD * self.mad_de_deriva()?)
    }

    /// **El corte de la fase de pedaleo**: [`FACTOR_SOBRE_SU_BASE`] veces su mediana.
    ///
    /// Medido sobre sus 83 sesiones con fase: **5,63°**, que separa **23 de 83**.
    pub fn corte_de_fase(&self) -> Option<f64> {
        // SE ALINEA AL PELDAÑO DE LA ESCALA, y es la corrección más importante de este módulo. El
        // doble de una mediana (`2,8125 x 2 = 5,625`) **no cae en un peldaño de la escala de la fase**
        // más que por casualidad, mientras que las asimetrías **siempre** caen en uno. Comparar sin
        // alinear hace que la decisión dependa de 3·10⁻¹⁴ grados de ruido de coma flotante.
        //
        // Con esto, y medido sobre sus 83 sesiones: el corte es **5,625°**, y **20 sesiones lo
        // superan**. Sin alinear, el mismo código decía 23 según cómo cayera el redondeo.
        Some(alinear_a_la_escala(FACTOR_SOBRE_SU_BASE * self.base_de_fase()?))
    }

    /// **La sesión más reciente que tiene un dato**, buscando hacia atrás.
    ///
    /// POR QUÉ «CON ESE DATO» Y NO «LA ÚLTIMA»: la última sesión que hizo puede perfectamente no traer
    /// potencia —medido: **19 de sus 102 ficheros no la traen**—, y devolver esa como «no hay aviso»
    /// sería **callar un aviso que sí existe** en una sesión anterior. Se busca la más reciente **que
    /// tenga el dato**, y el texto del aviso dice de qué fecha es. Así el aviso **nunca miente sobre
    /// cuándo pasó**, que es la única forma de que sirva para algo.
    pub fn ultima_con<F: Fn(&SesionVista) -> bool>(&self, tiene: F) -> Option<&SesionVista> {
        self.sesiones.iter().rev().find(|s| tiene(s))
    }

    /// Cuántas sesiones traen TSS. Medido en sus datos: **83 de 102**.
    pub fn con_tss(&self) -> usize {
        self.sesiones.iter().filter(|s| s.tss.lo_hay()).count()
    }

    /// **Cuántas sesiones superan el umbral de TSS del cliente.**
    ///
    /// Medido: **18 de sus 102**, con [`UMBRAL_TSS_DEL_CLIENTE`] y [`EN_EL_UMBRAL_DE_TSS_DISPARA`] en
    /// falso, que es lo que respeta su frase *«si supera 100»*.
    pub fn sesiones_sobre_tss(&self, umbral: f64) -> usize {
        self.sesiones
            .iter()
            .filter_map(|s| s.tss.valor())
            .filter(|t| supera(*t, umbral, EN_EL_UMBRAL_DE_TSS_DISPARA))
            .count()
    }
}

/// ¿`valor` supera `umbral`, según lo que se haya decidido para el valor exacto?
///
/// ESTA FUNCIÓN ES CORTA Y ES LA PIEZA MÁS IMPORTANTE DEL MÓDULO, porque **un valor de corte tiene UNA
/// sola dirección**: quien lee «54» no sabe si dispara en 54,00 o en 53,99, y esa duda es la que
/// convierte un umbral en dos reglas distintas. Aquí se decide **una vez, con nombre y con constante**,
/// y las pruebas la fijan.
fn supera(valor: f64, umbral: f64, dispara_en_el_exacto: bool) -> bool {
    // LA TOLERANCIA NO ES UN NÚMERO ELEGIDO AL AZAR. Aquí está de dónde sale, y costó tres intentos.
    //
    // Una medida **no se puede comparar con más finura que su propia resolución**, y las dos que este
    // módulo compara tienen rejilla conocida:
    //
    // - el **agua corporal** viene de una báscula, con un decimal: el paso es 0,1 %;
    // - la **fase de pedaleo** viene en peldaños de 1,40625° (ver [`PASO_DE_LA_FASE_GRADOS`]).
    //
    // Y hay un detalle que solo se ve midiendo: **el valor que llega no está exactamente en su
    // peldaño**. El lector escala el byte del FIT a grados, y esa cuenta deja ruido. **Medido** sobre
    // sus 83 sesiones: va de **8,8·10⁻⁸ a 5,5·10⁻⁶ grados** —una asimetría que debería valer justo
    // 5,625 llega como 5,6250000879 o como 5,6249944628—. Con una tolerancia de 1e-9, o incluso de
    // 1e-6, ese ruido **no se absorbía del todo**, así que sesiones que están **en el mismo peldaño que
    // el corte** contaban como si lo superaran: el aviso decía **28 donde hay 20**.
    //
    // Por eso la tolerancia es **1e-5**, y las dos cotas están medidas y son estas:
    //
    // - es **diez veces mayor** que el ruido más grande que se ha medido (5,5·10⁻⁶), así que lo absorbe
    //   con holgura y la cuenta no depende de dónde caiga el redondeo;
    // - es **ciento cuarenta mil veces menor** que el peldaño más fino (1,40625°), así que **no puede
    //   confundir dos peldaños distintos ni de lejos**.
    //
    // Y es relativa al umbral (con suelo de 1,0 para que el 0 no la anule), así que sirve igual para 54
    // que para 5,6.
    let magnitud = umbral.abs().max(1.0);
    if (valor - umbral).abs() <= 1e-5 * magnitud {
        // El valor está EN el umbral: lo decide la constante, no un `>` puesto al azar.
        return dispara_en_el_exacto;
    }
    valor > umbral
}

/// **Un número escrito como se escribe en español: con coma decimal.**
///
/// POR QUÉ HACE FALTA, y es una corrección medida: el primer informe salía diciendo «53.8 %» y
/// «7.0°». El texto entero está en español, así que un número con punto **se lee como otra cosa** —y
/// en un aviso sobre el cuerpo de una persona, «53.8» y «53,8» no deberían ser dos formatos distintos
/// en la misma frase—. `format!` con `{:.1}` siempre pone el punto, así que se cambia después.
///
/// No se toca la parte entera ni el signo: solo el separador. Es una función de presentación, y por
/// eso vive aquí y **no cambia ningún número**: el valor que se compara es el mismo.
fn con_coma(texto: String) -> String {
    texto.replace('.', ",")
}

/// La **mediana** de una lista, sin ordenarla fuera de sitio. `NaN` si está vacía.
fn mediana(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // Con un número par de valores se toma el de en medio de los dos centrales, y se dice: es la
    // convención que ya usa el resto del proyecto, y cambiarla movería todas las líneas base.
    v[v.len() / 2]
}

/// La **diferencia angular** entre dos fases, en grados, llevada al rango 0..180.
///
/// POR QUÉ HACE FALTA, y es un error que se cometió midiendo: el primer ángulo de la fase salta del
/// **357°** al **2°**, y esos dos valores están a **5 grados**, no a 355. Restar a pelo daría una
/// asimetría de 355° que **no existe** — y encima la daría como la peor de su histórico. Un 0° y un
/// 360° son **el mismo sitio de la rueda**, y una medida con esa trampa se corrige, no se promedia.
///
/// **NO SE ESCRIBE AQUÍ**: se delega en [`crate::datos::distancia_angular`], que otro agente añadió al
/// modelo mientras se escribía este módulo y que **hace exactamente esta cuenta**. Dos funciones que
/// calculan la misma distancia angular son **dos verdades sobre lo mismo**, y el día que se corrija una
/// la otra seguiría dando el número viejo.
fn diferencia_circular(a: f64, b: f64) -> f64 {
    crate::datos::distancia_angular(a, b)
}

/// **La asimetría entre las dos piernas**, en grados: el máximo de los cuatro ángulos.
///
/// POR QUÉ EL MÁXIMO Y NO LA MEDIA DE LOS CUATRO: interesa **el gesto que más se desvía**. Dos piernas
/// pueden coincidir en tres ángulos y separarse 30° en el cuarto —justo el que delata un pedaleo
/// descompensado— y la media de los cuatro lo taparía dividiéndolo entre cuatro.
///
/// Devuelve `None` si falta cualquiera de las dos fases: **comparar una pierna contra un hueco no dice
/// nada**, y devolver un cero sería afirmar «van iguales» sin haberlo medido.
pub fn asimetria_de_fase(izquierda: Option<[f64; 4]>, derecha: Option<[f64; 4]>) -> Option<f64> {
    let (i, d) = (izquierda?, derecha?);
    Some(
        i.iter()
            .zip(d.iter())
            .map(|(a, b)| diferencia_circular(*a, *b).abs())
            .fold(0.0_f64, f64::max),
    )
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// LA FUNCIÓN
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **Los avisos que salen con estos datos, y ninguno más.**
///
/// ES UNA FUNCIÓN PURA: no lee ficheros, no mira el reloj y no usa azar. Todo lo que necesita entra por
/// los argumentos, y por eso **se puede probar sin fabricar ficheros FIT** — que es lo que permite
/// comprobar el caso que más importa, **que sin datos no salga ningún aviso inventado**.
///
/// EL ORDEN DE LA SALIDA es **por urgencia** (lo que corre más prisa, primero) y, a igualdad, por
/// módulo. No es cosmético: un aviso urgente que aparece el quinto **se lee tarde**.
///
/// Y **lo que NO hace**: no devuelve avisos «informativos de relleno» cuando no hay nada que decir. Una
/// lista vacía **es una respuesta correcta** —«con estos datos, no hay nada que avisarte»—, y rellenarla
/// para que la pantalla no quede vacía sería exactamente el vicio que este proyecto existe para evitar.
pub fn avisos(persona: &Persona, real: &crate::analisis::Real, historico: &Historico, hoy: &Hoy) -> Vec<Aviso> {
    let mut v = Vec::new();

    v.extend(combustible(historico, hoy));
    v.extend(rodilla(historico));
    v.extend(recuperacion(historico));

    // La `persona` y el `real` entran en la firma porque son el contexto con el que se calcula todo lo
    // demás —la antigüedad de los datos, las zonas— y porque el encargo fija esa firma. Hoy estas tres
    // reglas **no los usan**, y en vez de callarlo se dice: si un aviso necesitara la edad o el peso,
    // este es el sitio por donde entran, y no hay que cambiar la firma para añadirlo.
    let _ = (persona, real);

    v.sort_by(|a, b| b.urgencia.cmp(&a.urgencia).then(a.modulo.cmp(&b.modulo)));
    v
}

/// **A · COMBUSTIBLE**: sus dos reglas, con sus dos umbrales.
fn combustible(historico: &Historico, hoy: &Hoy) -> Vec<Aviso> {
    let mut v = Vec::new();

    // ── A.1 · EL AGUA CORPORAL, que la mete él a mano ──
    //
    // Si no la ha metido, NO HAY AVISO. Y se dice que falta, porque un silencio sin explicación se
    // lee como «todo bien», que es lo contrario de lo que pasa: no se sabe.
    if let Some(agua) = hoy.agua_corporal_pct.valor() {
        let agua = f64::from(agua);
        if supera(hoy.umbral_agua_pct, agua, EN_EL_UMBRAL_DE_AGUA_DISPARA) {
            v.push(Aviso::nuevo(
                Modulo::Combustible,
                Urgencia::Hoy,
                format!("Su agua corporal está en {} %, por debajo de su {:.0} %", con_coma(format!("{agua:.1}")), hoy.umbral_agua_pct),
                format!(
                    "su báscula de Huawei marca {} % de agua corporal, y su regla dice que por debajo de {:.0} % \
añada un bidón extra con electrolitos y un consomé. Está {} puntos por debajo del umbral que usted fijó.",
                    con_coma(format!("{agua:.1}")),
                    hoy.umbral_agua_pct,
                    con_coma(format!("{:.1}", hoy.umbral_agua_pct - agua))
                ),
                "Bidón extra con electrolitos y un consomé, como usted tiene decidido.",
                format!(
                    "PAUTA DEL CLIENTE, no un hallazgo: el umbral de {:.0} % lo escribió él en su encargo. \
La medición viene de su báscula de Huawei y la mete a mano. El consomé y los electrolitos se recuerdan \
como lo que son —su propia pauta— y NO se afirma que sirvan: {}",
                    hoy.umbral_agua_pct, NoEsLoQueParece::SIN_FUENTE_DE_SUPLEMENTOS
                ),
                // LA RESERVA: aquí va lo que este aviso NO puede saber.
                Some(format!(
                    // El `trim_end_matches('.')` no es adorno: las constantes de `NoEsLoQueParece` ya
                    // terminan en punto, y pegarlas detrás de otra frase producia un «cumplió..» que se
                    // lee mal. Se quita el punto de la constante y se pone el de la frase que se escribe.
                    "la aplicación no comprueba si lo tomó, ni si sirvió: {}. Y no hay una fuente que sostenga \
que el consomé o los electrolitos repongan nada — se recuerda su pauta, no se receta.",
                    NoEsLoQueParece::SIN_DIARIO_DE_COMIDAS.trim_end_matches('.')
                )),
            ));
        }
    }

    // ── A.2 · EL TSS, que sí está en los ficheros ──
    //
    // Se busca LA SESIÓN MÁS RECIENTE QUE SUPERA el umbral, no «la última sesión»: la última puede no
    // traer TSS, y entonces el aviso se callaría teniéndolo delante.
    let con_tss: Vec<&SesionVista> = historico.sesiones.iter().filter(|s| s.tss.lo_hay()).collect();
    let sobre: Vec<&&SesionVista> = con_tss
        .iter()
        .filter(|s| supera(s.tss.valor().unwrap_or(0.0), UMBRAL_TSS_DEL_CLIENTE, EN_EL_UMBRAL_DE_TSS_DISPARA))
        .collect();

    if let Some(ultima) = sobre.last() {
        let tss = ultima.tss.valor().unwrap_or(0.0);
        let cuando = ultima.fecha().map(|f| format!(" del {f}")).unwrap_or_default();
        v.push(Aviso::nuevo(
            Modulo::Combustible,
            Urgencia::Hoy,
            format!("La sesión{cuando} fue de {tss:.0} TSS, por encima de sus {UMBRAL_TSS_DEL_CLIENTE:.0}"),
            format!(
                "el fichero declara {tss:.0} TSS. Su regla dice que por encima de {UMBRAL_TSS_DEL_CLIENTE:.0} suba los \
carbohidratos de la cena, y esta sesión los pasa en {:.0}. En todo su histórico hay {} sesiones por encima de ese \
umbral, de {} con TSS declarado y {} sesiones en la carpeta.",
                tss - UMBRAL_TSS_DEL_CLIENTE,
                sobre.len(),
                con_tss.len(),
                historico.total_sesiones
            ),
            "Subir los carbohidratos de la cena, como usted tiene decidido.",
            format!(
                "PAUTA DEL CLIENTE, no un hallazgo: el corte de {UMBRAL_TSS_DEL_CLIENTE:.0} TSS lo escribió él. \
El TSS SÍ es un dato medido: lo calcula el reloj y viaja en el fichero FIT, en `Session.training_stress_score`."
            ),
            Some(NoEsLoQueParece::SIN_DIARIO_DE_COMIDAS.to_string()),
        ));
    }

    v
}

/// **B · RODILLA**: la mecánica del pedaleo contra su propia línea base.
fn rodilla(historico: &Historico) -> Vec<Aviso> {
    // Sin línea base no hay aviso, y no es un techo técnico: comparar contra una mediana de dos
    // sesiones diría «te desvías» por puro azar.
    let (Some(base), Some(corte)) = (historico.base_de_fase(), historico.corte_de_fase()) else {
        return Vec::new();
    };

    // La sesión más reciente CON fase. Si no hay ninguna, no hay aviso: el encargo lo pide así.
    let Some(ultima) = historico.ultima_con(|s| s.asimetria_grados.is_some()) else {
        return Vec::new();
    };
    let Some(asimetria) = ultima.asimetria_grados else {
        return Vec::new();
    };

    // El corte es NUESTRO, así que se avisa al alcanzarlo. Ver `EN_LA_BASE_DE_FASE_DISPARA`.
    if !supera(asimetria, corte, EN_LA_BASE_DE_FASE_DISPARA) {
        return Vec::new();
    }

    let con_fase = historico.sesiones.iter().filter(|s| s.asimetria_grados.is_some()).count();
    let cuando = ultima.fecha().map(|f| format!(" del {f}")).unwrap_or_default();

    vec![Aviso::nuevo(
        Modulo::Rodilla,
        Urgencia::Atencion,
        format!("La fase de pedaleo{cuando} se separó {}° entre sus dos piernas", con_coma(format!("{asimetria:.1}"))),
        format!(
            "sus dos piernas difieren {}° en el pedaleo (el máximo de los cuatro ángulos de la fase, \
con la diferencia circular: 357° y 2° están a 5°, no a 355°). Su línea base —la mediana de sus {} sesiones con \
fase— es {}°, así que esta sesión la dobla. En total, {} de sus {} sesiones con fase pasan ese corte.",
            con_coma(format!("{asimetria:.1}")),
            con_fase,
            con_coma(format!("{base:.2}")),
            historico.sesiones.iter().filter(|s| s.asimetria_grados.map(|a| supera(a, corte, EN_LA_BASE_DE_FASE_DISPARA)).unwrap_or(false)).count(),
            con_fase
        ),
        "Mirarlo con calma: si se repite en varias sesiones seguidas, coméntelo con quien le ajusta la bicicleta.",
        format!(
            "MEDIDO sobre sus propios ficheros: `avg_left_power_phase` y `avg_right_power_phase` son cuatro ángulos \
por pierna, y vienen en 83 de sus 102 ficheros. {FUENTE_FASE} El corte NO es una tabla: es {} veces SU mediana. {FUENTE_DERIVA}",
            // El `{FUENTE_DERIVA}` de arriba NO es un descuido: es la cita que sostiene que
            // «desviarse de la propia línea base es más informativo que una banda de población», y va
            // aquí porque es la misma idea que sostiene el módulo C. Si se separan, se separan a la vez.
            FACTOR_SOBRE_SU_BASE
        ),
        // LA REGLA DE HONESTIDAD, obligatoria y no opcional. Sin esto, `es_observacion()` sería falso.
        Some(format!(
            "{NO_ES_VALORACION_MEDICA} Y NO se puede hacer esto sobre el reparto de potencia izquierda/derecha que \
usted pidió: {}",
            NoEsLoQueParece::SIN_REPARTO_DE_POTENCIA
        )),
    )]
}

/// **C · RECUPERACIÓN**: la deriva cardíaca contra su propia línea base.
fn recuperacion(historico: &Historico) -> Vec<Aviso> {
    let (Some(base), Some(corte)) = (historico.base_de_deriva(), historico.corte_de_deriva()) else {
        return Vec::new();
    };

    let Some(ultima) = historico.ultima_con(|s| s.deriva_pct.is_some()) else {
        return Vec::new();
    };
    let Some(deriva) = ultima.deriva_pct else {
        return Vec::new();
    };

    if !supera(deriva, corte, EN_LA_BASE_DE_FASE_DISPARA) {
        return Vec::new();
    }

    let con_deriva = historico.sesiones.iter().filter(|s| s.deriva_pct.is_some()).count();
    let cuando = ultima.fecha().map(|f| format!(" del {f}")).unwrap_or_default();
    let pct_publicado = 10.0_f64;

    vec![Aviso::nuevo(
        Modulo::Recuperacion,
        Urgencia::Atencion,
        format!(
            "La deriva cardíaca{cuando} fue del {} %, y suele ser del {} %",
            con_coma(format!("{deriva:.1}")),
            con_coma(format!("{base:.1}"))
        ),
        format!(
            "en esa sesión sacó menos vatios por pulsación en la segunda mitad que en la primera: {} % menos. \
Su línea base —la mediana de sus {} sesiones con deriva calculable— es {} %, así que esta se sale del corte \
de {} % (su mediana más {FACTOR_SOBRE_SU_MAD} veces la MAD). Para comparar: {} de esas {} sesiones pasan del \
{pct_publicado:.0} % que la literatura marca como «se le escapa el pulso».",
            con_coma(format!("{deriva:.1}")),
            con_deriva,
            con_coma(format!("{base:.1}")),
            con_coma(format!("{corte:.1}")),
            historico.sesiones.iter().filter(|s| s.deriva_pct.map(|d| d > pct_publicado).unwrap_or(false)).count(),
            con_deriva
        ),
        "Mirar si esa sesión fue con calor, con poca agua o sin fondo: la deriva señala las tres cosas.",
        format!(
            "MEDIDO sobre sus ficheros y calculado con `patron::deriva` —la misma cuenta que ya usa el motor, no una \
segunda—: la eficiencia de la primera mitad contra la segunda, en vatios por pulsación. {FUENTE_DERIVA}"
        ),
        Some(format!(
            "{} La deriva NO dice cuál de las tres causas es: calor, deshidratación y falta de fondo se ven igual en \
el número, y separarlas necesitaría datos que la aplicación no tiene.",
            NoEsLoQueParece::SIN_PULSO_EN_REPOSO
        )),
    )]
}

/// **El parte de lo que NO se pudo avisar, y por qué.** Es la otra mitad del módulo.
///
/// POR QUÉ ES UNA FUNCIÓN Y NO UN TEXTO SUELTO EN EL BINARIO: porque **«no hay aviso porque falta este
/// dato» es una respuesta**, y una respuesta que solo vive en la pantalla de instrumentos **no se
/// puede probar**. Aquí se puede: las pruebas comprueban que falta el agua y se dice, que falta el
/// reparto de potencia y se dice, y que faltan el pulso en reposo y el sueño y se dicen.
///
/// Y existe por un motivo que no es de estilo: la primera versión de este proyecto **se negaba a
/// responder** cuando faltaba un dato, el dueño lo corrigió, y la corrección fue que **faltar un dato
/// no es lo mismo que no tener nada que decir**. Esto es «tener algo que decir» sobre lo que falta.
pub fn lo_que_falta(historico: &Historico, hoy: &Hoy) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = Vec::new();

    // 1 · El agua corporal, que la mete él.
    if hoy.agua_corporal_pct.falta() {
        v.push((
            "agua corporal de hoy".to_string(),
            format!(
                "la mete usted a mano desde su báscula de Huawei. Sin ella no se puede recordarle su regla del \
{:.0} %. (Medido: su última captura decía 55,3 %.)",
                hoy.umbral_agua_pct
            ),
        ));
    }

    // 2 · El reparto de potencia izquierda/derecha, que NO EXISTE.
    v.push((
        "reparto de potencia izquierda/derecha".to_string(),
        NoEsLoQueParece::SIN_REPARTO_DE_POTENCIA.to_string(),
    ));

    // 3 · El pulso en reposo y el sueño, que no tienen serie.
    v.push((
        "pulso en reposo y sueño".to_string(),
        NoEsLoQueParece::SIN_PULSO_EN_REPOSO.to_string(),
    ));

    // 4 · La cena, que no se puede comprobar.
    v.push((
        "qué comió".to_string(),
        NoEsLoQueParece::SIN_DIARIO_DE_COMIDAS.to_string(),
    ));

    // 5 · Y si la línea base no se pudo calcular, se dice cuál falta y por qué.
    if historico.base_de_fase().is_none() {
        v.push((
            "línea base de la fase de pedaleo".to_string(),
            format!(
                "hacen falta al menos {MINIMO_PARA_LINEA_BASE} sesiones con fase de pedaleo y hay {}. Con menos, la \
mediana no es una norma y el aviso de la rodilla saldría por azar.",
                historico.sesiones.iter().filter(|s| s.asimetria_grados.is_some()).count()
            ),
        ));
    }
    if historico.base_de_deriva().is_none() {
        v.push((
            "línea base de la deriva cardíaca".to_string(),
            format!(
                "hacen falta al menos {MINIMO_PARA_LINEA_BASE} sesiones con deriva calculable y hay {}. La deriva \
necesita al menos {MINIMO_MUESTRAS_DERIVA} pares de pulso y potencia.",
                historico.sesiones.iter().filter(|s| s.deriva_pct.is_some()).count()
            ),
        ));
    }

    // 6 · Y los ficheros que no se pudieron leer, uno a uno: un dato que falta hay que decirlo.
    for (nombre, motivo) in &historico.no_leidos {
        v.push((format!("fichero {nombre}"), format!("no se pudo leer: {motivo}")));
    }

    v
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// La lectura de la carpeta
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **Lee la carpeta entera y saca lo que estas tres reglas necesitan.**
///
/// POR QUÉ ES UN LECTOR APARTE Y NO UNA AMPLIACIÓN DE [`crate::fit`]: porque `fit.rs` es de otro agente
/// y **está en obras** —está añadiendo `cadencia_rpm`, `distancia_m` y `temperatura_c` a `Muestra`—, y
/// tocar un fichero que otro está editando es la forma más segura de perder el trabajo de los dos.
/// Este lector **no compite con el suyo**: lee **cuatro cosas** que `fit::Declarado` todavía no expone
/// (TSS, las dos fases de pedaleo) y **no toca las muestras**, que es lo que el suyo sí hace bien.
///
/// **LO QUE SÍ REUTILIZA, y es lo importante**: la deriva **no se recalcula aquí**. Se construye la
/// [`Sesion`] con sus muestras y se llama a [`crate::patron::deriva`]. Dos cálculos de la deriva en el
/// mismo motor serían **dos verdades sobre lo mismo**, y el día que se corrija una, la otra seguiría
/// dando el número viejo.
///
/// El día que `fit::Declarado` traiga estos campos, **este lector se borra** y se pasa a usar el suyo.
pub fn de_carpeta(carpeta: &Path) -> Historico {
    let mut rutas: Vec<_> = match std::fs::read_dir(carpeta) {
        Ok(d) => d
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false))
            .collect(),
        Err(e) => {
            return Historico {
                total_sesiones: 0,
                no_leidos: vec![(carpeta.display().to_string(), format!("no se pudo abrir la carpeta: {e}"))],
                sesiones: Vec::new(),
            }
        }
    };
    rutas.sort();

    let mut h = Historico {
        total_sesiones: rutas.len(),
        no_leidos: Vec::new(),
        sesiones: Vec::new(),
    };

    for ruta in &rutas {
        let nombre = ruta.file_name().unwrap_or_default().to_string_lossy().to_string();
        match leer_una(ruta) {
            Ok(s) => h.sesiones.push(s),
            // Un fichero que no se lee **se cuenta y se dice**: perderse en silencio es lo que hacía
            // que un promedio de 99 ficheros pareciera un promedio de 102.
            Err(e) => h.no_leidos.push((nombre, e)),
        }
    }
    h
}

/// **Lee un** fichero y saca las cuatro cosas. Se separa para poder probarlo con uno solo.
///
/// **ESTE LECTOR DUPLICA PARTE DE [`crate::fit`], Y HAY QUE DECIR POR QUÉ.** Es una deuda, no un
/// descuido: `fit.rs` **no expone** el TSS, ni las dos fases de pedaleo, ni la marca de tiempo de
/// comienzo —los tres datos de los que viven estas reglas—, y **está siendo editado por otro agente
/// ahora mismo** (le está añadiendo cadencia, distancia y temperatura a `Muestra`). Modificarlo para
/// sacar cuatro campos más habría significado **editar un fichero con otro dentro**, que es la forma
/// más segura de perder el trabajo de los dos.
///
/// LO QUE ESTE LECTOR **NO** DUPLICA, y es lo que importa: **la deriva**. Las muestras se empaquetan
/// en una [`Sesion`] y se llama a [`crate::patron::deriva`]. Dos cálculos de la deriva en el mismo
/// motor serían **dos verdades sobre lo mismo**.
///
/// **CÓMO SE PAGA ESTA DEUDA**: el día que `fit::Declarado` traiga `training_stress_score`,
/// `avg_left_power_phase`, `avg_right_power_phase` y `start_time`, `leer_una` se sustituye por una
/// llamada a `fit::leer` y **este bloque se borra**. Hasta entonces, cada campo que `Muestra` gane
/// romperá la construcción de abajo **a propósito**: el compilador obliga a decidir si este módulo lo
/// usa o lo declara ausente, en vez de dejar que un dato nuevo entre sin que nadie lo mire.
pub fn leer_una(ruta: &Path) -> Result<SesionVista, String> {
    let nombre = ruta.file_name().unwrap_or_default().to_string_lossy().to_string();
    let mut f = std::fs::File::open(ruta).map_err(|e| format!("no se pudo abrir: {e}"))?;
    let datos = fitparser::from_reader(&mut f).map_err(|e| format!("el FIT no se pudo leer: {e:?}"))?;

    let mut s = SesionVista::vacia(&nombre);
    let mut muestras: Vec<crate::datos::Muestra> = Vec::new();
    let mut izquierda: Option<[f64; 4]> = None;
    let mut derecha: Option<[f64; 4]> = None;
    let mut primer_segundo: Option<u32> = None;

    for r in datos.iter() {
        let tipo = format!("{:?}", r.kind());

        // ── Record: UNA muestra por registro, y solo pulso, potencia y tiempo ──
        //
        // ESTE BLOQUE VA FUERA DEL BUCLE DE CAMPOS, Y ESO SE CORRIGIÓ MIDIENDO. La primera versión lo
        // tenía dentro —`for c in r.fields() { match tipo { "Record" => ... } }`— y el efecto era
        // **una muestra empujada por CADA CAMPO del registro**: un `Record` con 25 campos metía la
        // misma muestra 25 veces. Y no reventaba: la deriva salía **11,85 % de mediana** en vez de
        // **10,75 %**, un número parecido y falso, que es la peor clase de error que puede tener este
        // motor. Lo delató comparar la mediana con la de una medida independiente.
        if tipo == "Record" {
            let mut seg: Option<u32> = None;
            let mut pulso = Dato::Falta;
            let mut potencia = Dato::Falta;
            for c in r.fields() {
                match c.name() {
                    "timestamp" => {
                        if let fitparser::Value::Timestamp(t) = c.value() {
                            seg = Some(t.timestamp() as u32);
                        }
                    }
                    "heart_rate" => pulso = entero(c.value()),
                    "power" => potencia = entero(c.value()),
                    _ => {}
                }
            }
            if let Some(x) = seg {
                if primer_segundo.is_none() {
                    primer_segundo = Some(x);
                }
                muestras.push(crate::datos::Muestra {
                    segundo: x,
                    pulso,
                    potencia_w: potencia,
                    // Los tres campos que `Muestra` ganó mientras se escribía este módulo. Se ponen a
                    // `Falta` **y no a cero**, que es la regla del modelo: este módulo **no los usa**
                    // —la cadencia es del módulo de pedaleo del otro agente, y la temperatura todavía
                    // no entra en ninguna regla—, y un cero aquí sería un dato falso que otra pieza
                    // podría leer como bueno.
                    cadencia_rpm: Dato::Falta,
                    distancia_m: Dato::Falta,
                    temperatura_c: Dato::Falta,
                });
            }
        }

        // ── Session: los totales que declara el fichero. SÍ va campo a campo, porque cada dato vive
        // en un campo distinto y no se repite.
        if tipo == "Session" {
            for c in r.fields() {
                match c.name() {
                    "training_stress_score" => {
                        if let Some(v) = numero(c.value()) {
                            s.tss = Dato::Hay(v);
                        }
                    }
                    "avg_left_power_phase" => izquierda = cuatro(c.value()),
                    "avg_right_power_phase" => derecha = cuatro(c.value()),
                    // La marca de tiempo del comienzo de la sesión: es lo que permite decir DE CUÁNDO
                    // es el aviso. Sin ella, un aviso de «la sesión pasó de 100 TSS» no diría cuál.
                    "start_time" | "timestamp" => {
                        if let fitparser::Value::Timestamp(t) = c.value() {
                            if s.inicio_s.is_none() {
                                s.inicio_s = Some(t.timestamp());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // El tiempo DENTRO de la sesión, como hace `fit.rs`: el primer segundo es el cero. Se hace para
    // que la deriva —que compara la primera mitad con la segunda— no dependa de la hora absoluta.
    if let Some(primero) = primer_segundo {
        for m in &mut muestras {
            m.segundo = m.segundo.saturating_sub(primero);
        }
    }

    s.asimetria_grados = asimetria_de_fase(izquierda, derecha);

    // La deriva, con la función del motor. Si no hay pulsos o potencias suficientes, `patron::deriva`
    // devuelve cero con cero muestras, y aquí eso se traduce a `None`: **una deriva de cero porque no
    // había datos no es una deriva de cero**, y confundirlas daría un aviso tranquilizador falso.
    //
    // Se construye con `Sesion::nueva`, que es el constructor del modelo, y no escribiendo el struct a
    // mano: `Sesion` está creciendo —hoy mismo se le añadió la biomecánica— y **un constructor no se
    // rompe cuando el tipo crece**. Escribir los campos a mano sí.
    let sesion = Sesion::nueva(muestras);
    let d = patron::deriva(&sesion);
    if d.muestras >= MINIMO_MUESTRAS_DERIVA {
        s.deriva_pct = Some(d.porcentaje());
    }

    Ok(s)
}

fn entero(v: &fitparser::Value) -> Dato<u16> {
    match v {
        fitparser::Value::UInt8(x) => Dato::Hay(u16::from(*x)),
        fitparser::Value::UInt16(x) => Dato::Hay(*x),
        fitparser::Value::UInt32(x) => Dato::Hay(*x as u16),
        fitparser::Value::Float32(x) => Dato::Hay(x.round().max(0.0) as u16),
        fitparser::Value::Float64(x) => Dato::Hay(x.round().max(0.0) as u16),
        // Un valor de un tipo que no se espera **no se adivina**: se trata como ausente.
        _ => Dato::Falta,
    }
}

fn numero(v: &fitparser::Value) -> Option<f64> {
    match v {
        fitparser::Value::UInt8(x) => Some(f64::from(*x)),
        fitparser::Value::UInt16(x) => Some(f64::from(*x)),
        fitparser::Value::UInt32(x) => Some(f64::from(*x)),
        fitparser::Value::SInt8(x) => Some(f64::from(*x)),
        fitparser::Value::SInt16(x) => Some(f64::from(*x)),
        fitparser::Value::SInt32(x) => Some(f64::from(*x)),
        fitparser::Value::Float32(x) => Some(f64::from(*x)),
        fitparser::Value::Float64(x) => Some(*x),
        _ => None,
    }
}

fn cuatro(v: &fitparser::Value) -> Option<[f64; 4]> {
    let fitparser::Value::Array(a) = v else { return None };
    let n: Vec<f64> = a
        .iter()
        .filter_map(|x| match x {
            fitparser::Value::UInt8(b) => Some(f64::from(*b)),
            fitparser::Value::UInt16(b) => Some(f64::from(*b)),
            fitparser::Value::Float32(b) => Some(f64::from(*b)),
            fitparser::Value::Float64(b) => Some(*b),
            _ => None,
        })
        .collect();
    // Se exigen los CUATRO ángulos: tres ángulos no son media fase, son otra cosa.
    if n.len() == 4 {
        Some([n[0], n[1], n[2], n[3]])
    } else {
        None
    }
}

/// Un instante Unix a fecha `AAAA-MM-DD`, en UTC.
///
/// POR QUÉ A MANO Y NO CON UNA BIBLIOTECA: es la única cuenta de fechas que necesita el módulo, y
/// añadir una dependencia entera para formatear un día **sería pagar un precio permanente por un
/// problema de veinte líneas**. El algoritmo es el civil-from-days, el mismo que usan las bibliotecas
/// por dentro, y está probado contra fechas conocidas.
///
/// **Y en UTC, no en hora local, a propósito**: la fecha se usa para **decir de qué sesión se habla**,
/// no para programar nada. Un desfase de horas puede mover el día, y eso se dice aquí para que nadie lo
/// tome por un error.
pub fn fecha_de_epoch(segundos: i64) -> String {
    let dias = segundos.div_euclid(86_400);
    let (ano, mes, dia) = civil_de_dias(dias);
    format!("{ano:04}-{mes:02}-{dia:02}")
}

/// El algoritmo `civil_from_days` de Howard Hinnant: días desde 1970-01-01 a fecha del calendario.
///
/// Se elige este y no otro porque **es exacto para todo el rango de fechas que va a ver este programa**
/// y porque **no tiene casos especiales**: los meses se cuentan desde marzo para que el año bisiesto
/// caiga al final, y así febrero no necesita un `if`.
fn civil_de_dias(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // día dentro de la era, 0..146096
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // año dentro de la era
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // día del año, contado desde marzo
    let mp = (5 * doy + 2) / 153; // mes contado desde marzo
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// **El texto completo de un aviso, listo para enseñar.** Uno por línea, y sin adornos.
pub fn formatear(a: &Aviso) -> String {
    let mut s = String::new();
    s.push_str(&format!("[{}] {} · {}\n", a.urgencia.nombre().to_uppercase(), a.modulo.nombre(), a.titulo));
    s.push_str(&format!("   POR QUÉ: {}\n", a.porque));
    s.push_str(&format!("   QUÉ HACER: {}\n", a.que_hacer));
    s.push_str(&format!("   FUNDAMENTO: {}\n", a.fundamento.fuente));
    if let Some(r) = &a.fundamento.reserva {
        s.push_str(&format!("   RESERVA: {r}\n"));
    }
    s
}

/// **El parte completo, en texto**: lo que sale, y **lo que no sale y por qué**.
///
/// Es lo que imprime el binario `avisos`, y vive aquí —y no en el binario— para que **se pueda probar**.
pub fn informe(persona: &Persona, real: &crate::analisis::Real, historico: &Historico, hoy: &Hoy) -> String {
    let mut s = String::new();
    let lista = avisos(persona, real, historico, hoy);

    s.push_str("═══════════════════════════════════════════════════════════════════════\n");
    s.push_str("  LOS AVISOS DEL ENCARGO, con sus datos reales\n");
    s.push_str("═══════════════════════════════════════════════════════════════════════\n\n");

    s.push_str(&format!("  sesiones en la carpeta ......... {}\n", historico.total_sesiones));
    s.push_str(&format!("  sesiones con TSS declarado ..... {}\n", historico.con_tss()));
    s.push_str(&format!(
        "  sesiones con fase de pedaleo ... {}\n",
        historico.sesiones.iter().filter(|x| x.asimetria_grados.is_some()).count()
    ));
    s.push_str(&format!(
        "  sesiones con deriva calculable . {}\n",
        historico.sesiones.iter().filter(|x| x.deriva_pct.is_some()).count()
    ));
    if !historico.no_leidos.is_empty() {
        s.push_str(&format!("  ficheros que NO se leyeron ..... {}\n", historico.no_leidos.len()));
    }
    s.push('\n');

    s.push_str("── LA LÍNEA BASE, QUE ES SUYA Y NO DE UNA TABLA ──\n");
    match historico.base_de_fase() {
        Some(b) => s.push_str(&format!(
            "  fase de pedaleo: mediana {}° · corte (x{FACTOR_SOBRE_SU_BASE}) {}°\n",
            con_coma(format!("{b:.2}")),
            con_coma(format!("{:.2}", historico.corte_de_fase().unwrap_or(f64::NAN)))
        )),
        None => s.push_str("  fase de pedaleo: SIN LÍNEA BASE\n"),
    }
    match historico.base_de_deriva() {
        Some(b) => s.push_str(&format!(
            "  deriva cardíaca: mediana {} % · MAD {} % · corte {} %\n",
            con_coma(format!("{b:.2}")),
            con_coma(format!("{:.2}", historico.mad_de_deriva().unwrap_or(f64::NAN))),
            con_coma(format!("{:.2}", historico.corte_de_deriva().unwrap_or(f64::NAN)))
        )),
        None => s.push_str("  deriva cardíaca: SIN LÍNEA BASE\n"),
    }
    s.push('\n');

    if lista.is_empty() {
        s.push_str("── AVISOS: NINGUNO ──\n");
        s.push_str("  Con estos datos no hay nada que avisar. No es un error: un aviso sin dato que lo\n");
        s.push_str("  sostenga sería peor que ninguno.\n\n");
    } else {
        s.push_str(&format!("── AVISOS: {} ──\n\n", lista.len()));
        for a in &lista {
            s.push_str(&formatear(a));
            s.push('\n');
        }
    }

    s.push_str("── LO QUE NO SALE, Y POR QUÉ ──\n\n");
    for (que, porque) in lo_que_falta(historico, hoy) {
        s.push_str(&format!("  · {que}\n      {porque}\n"));
    }

    s.push('\n');
    s.push_str("(Este programa no diagnostica enfermedades ni receta: recuerda pautas que usted fijó,\n");
    s.push_str(" y observa sus datos. Lo que no se puede citar, no se calcula.)\n");
    s
}

/// **Cuántos avisos salen por módulo**, para poder decirlo de un vistazo.
pub fn por_modulo(lista: &[Aviso]) -> BTreeMap<&'static str, usize> {
    let mut m = BTreeMap::new();
    for a in lista {
        *m.entry(a.modulo.nombre()).or_insert(0) += 1;
    }
    m
}
