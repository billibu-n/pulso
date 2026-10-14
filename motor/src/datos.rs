//! El modelo: cómo representamos NOSOTROS la sesión y la persona.
//!
//! Es nuestro, y a propósito: **el formato del cliente entra después, con un adaptador**. Así el motor
//! no depende de nadie, y si el cliente cambia de reloj o de aplicación, **cambia una pieza, no el
//! motor**.
//!
//! Y hay una decisión de fondo escrita en los tipos: **un dato puede FALTAR**, y eso no es lo mismo que
//! valer cero. `Dato::Falta` no es cero pulsaciones: es **no saberlo**. Confundir las dos cosas es el
//! error que produce planes con números plausibles y conclusiones falsas.

use std::fmt;

/// Un número que **puede faltar**.
///
/// La distinción es la pieza más importante de todo el modelo: **`Falta` no es `0`**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dato<T> {
    Hay(T),
    Falta,
}

impl<T: Copy> Dato<T> {
    /// El valor, si lo hay. Devuelve `None` cuando falta — y **nunca un cero por defecto**.
    pub fn valor(self) -> Option<T> {
        match self {
            Dato::Hay(v) => Some(v),
            Dato::Falta => None,
        }
    }

    pub fn falta(self) -> bool {
        matches!(self, Dato::Falta)
    }

}

/// Métodos que **NO** exigen que el valor se pueda copiar.
///
/// Van en su propio bloque a propósito: `Tendencia` lleva texto dentro y no es `Copy`, así que con el
/// bloque de arriba —que sí lo exige— no se podía ni preguntar si había valor. Rust no deja relajar una
/// condición dentro del mismo `impl`.
impl<T> Default for Dato<T> {
    /// **El valor por defecto de un dato es que FALTE.**
    ///
    /// Y no es un detalle: la alternativa sería un cero, y **un cero es un dato falso** — «no sé su
    /// peso» no es «pesa cero kilos». Que lo natural sea «falta» significa que **si alguien se olvida
    /// de rellenarlo, el motor lo trata como ausente**, que es lo correcto, en vez de calcular con un
    /// cero que parece un número.
    fn default() -> Self {
        Dato::Falta
    }
}

impl<T> Dato<T> {
    /// El valor, **prestado**.
    pub fn prestado(&self) -> Option<&T> {
        match self {
            Dato::Hay(v) => Some(v),
            Dato::Falta => None,
        }
    }

    /// ¿Lo hay? Sin exigir que se pueda copiar.
    pub fn lo_hay(&self) -> bool {
        matches!(self, Dato::Hay(_))
    }
}

impl<T: fmt::Display> fmt::Display for Dato<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dato::Hay(v) => write!(f, "{}", v),
            Dato::Falta => write!(f, "(falta)"),
        }
    }
}

/// La persona: los datos que **ella declara**, no los que miden los sensores.
///
/// Aquí está el caso que más me interesa, y es real: **la frecuencia cardíaca máxima**.
/// Se puede **medir** (una prueba de esfuerzo) o **estimar** (con una fórmula y la edad). Si se estima,
/// **tiene error**, y el motor tiene que decirlo. Por eso el tipo lo distingue.
#[derive(Debug, Clone, PartialEq)]
pub struct Persona {
    /// **Cuándo nació.** Es lo que MANDA, y sustituye a la edad escrita.
    ///
    /// POR QUE ESTA Y NO SOLO LA EDAD: una edad escrita **caduca**, y nadie se entera. La aplicación
    /// tenía `edad = 60` con el cliente teniendo 62, y de la edad sale la frecuencia máxima: **las zonas
    /// salían corridas**. Guardando la fecha, la edad se calcula cada vez y **nunca caduca**.
    pub fecha_nacimiento: Dato<Fecha>,
    /// Años. **Se conserva para los perfiles que ya la tenían escrita**, pero si hay fecha de
    /// nacimiento, la fecha manda (ver `edad_en`).
    pub edad: Dato<u8>,
    /// Pulsaciones por minuto en reposo. Base del método de Karvonen.
    pub pulso_reposo: Dato<u16>,
    /// Pulsaciones máximas **medidas** en una prueba. Si están, mandan sobre cualquier estimación.
    pub pulso_maximo_medido: Dato<u16>,
    /// Peso en kilogramos.
    pub peso_kg: Dato<f32>,
    /// Sexo, para las fórmulas que lo usan.
    pub sexo: Dato<Sexo>,
    /// **Altura en centímetros.** Sirve para el índice de masa corporal.
    pub altura_cm: Dato<f32>,
    /// **Porcentaje de grasa corporal**, de una báscula de composición.
    ///
    /// ESTE DATO VALE MUCHO, y no es una opinión: hay ecuaciones de frecuencia máxima que lo usan y
    /// publican un error de **±7,5 pulsaciones** frente a las **±12** de las que solo usan la edad.
    /// O sea que **este dato, por sí solo, estrecha el margen casi a la mitad**.
    pub grasa_corporal_pct: Dato<f32>,
    /// Masa muscular en kilogramos, si la báscula la da.
    pub masa_muscular_kg: Dato<f32>,
    /// **El umbral de lactato**, en pulsaciones.
    ///
    /// ESTE ES EL DATO QUE MANDA, y no la frecuencia máxima. En ciclismo las zonas se fijan sobre el
    /// umbral, y es un dato **medido** — mucho más fiable que una estimación por edad. Si está, el
    /// motor lo usa **antes** que cualquier fórmula.
    pub umbral_lactato: Dato<u16>,
    /// **El consumo máximo de oxígeno**, en ml/kg/min. Lo estima el reloj, y **el FIT no lo trae**.
    pub vo2_maximo: Dato<f32>,
}

impl Persona {
    /// Una persona de la que **no sabemos nada**. Es como empieza todo, y es honesto.
    pub fn desconocida() -> Self {
        Self {
            fecha_nacimiento: Dato::Falta,
            edad: Dato::Falta,
            pulso_reposo: Dato::Falta,
            pulso_maximo_medido: Dato::Falta,
            peso_kg: Dato::Falta,
            sexo: Dato::Falta,
            altura_cm: Dato::Falta,
            grasa_corporal_pct: Dato::Falta,
            masa_muscular_kg: Dato::Falta,
            umbral_lactato: Dato::Falta,
            vo2_maximo: Dato::Falta,
        }
    }

    /// Cuántos datos de la persona se conocen, de los que sirven para afinar.
    ///
    /// Existe para poder decir **«con estos datos, el margen es este»** — y para que el usuario vea que
    /// **cada dato que aporta cuenta**.
    pub fn cuantos_datos(&self) -> usize {
        [
            self.edad.lo_hay(),
            self.pulso_reposo.lo_hay(),
            self.pulso_maximo_medido.lo_hay(),
            self.peso_kg.lo_hay(),
            self.altura_cm.lo_hay(),
            self.grasa_corporal_pct.lo_hay(),
            self.masa_muscular_kg.lo_hay(),
        ]
        .iter()
        .filter(|x| **x)
        .count()
    }
}

/// **Una medición de composición corporal**, de una báscula (la de Huawei, hoy).
///
/// POR QUÉ VIVE EN EL MODELO Y NO EN EL ADAPTADOR: es **un dato de la persona**, no un formato. El
/// adaptador (`huawei.rs`) **traduce** su JSON a esto, y el motor **no sabe de Huawei**. Es la misma
/// separación que hace `fit.rs` con las sesiones: fuera el formato, dentro el modelo.
///
/// Y LA REGLA QUE LO DEFINE, la misma que en `Persona`: **un campo que no viene queda en `Dato::Falta`,
/// NUNCA en `Hay(0.0)`.** «No se midió el agua» no es «el agua vale cero»: un cero ahí dispararía la
/// regla de hidratación de una forma absurda.
///
/// UNIDADES: **kilogramos y porcentaje**, siempre. Si la exportación de Huawei trae gramos, la
/// conversión es del adaptador y se prueba allí; no se filtra al motor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Composicion {
    /// El día del pesaje. Fecha de calendario, **sin hora ni zona**.
    pub fecha: Dato<Fecha>,
    pub peso_kg: Dato<f32>,
    pub grasa_corporal_pct: Dato<f32>,
    /// **El dato que desbloquea la regla del agua** sin que nadie teclee.
    pub agua_corporal_pct: Dato<f32>,
    pub masa_muscular_kg: Dato<f32>,
    pub masa_osea_kg: Dato<f32>,
}

impl Composicion {
    /// Una medición de la que **no se sabe nada**. Es honesta: todos los campos en `Falta`.
    pub fn vacia() -> Self {
        Self {
            fecha: Dato::Falta,
            peso_kg: Dato::Falta,
            grasa_corporal_pct: Dato::Falta,
            agua_corporal_pct: Dato::Falta,
            masa_muscular_kg: Dato::Falta,
            masa_osea_kg: Dato::Falta,
        }
    }

    /// **Un valor de báscula solo es dato si es físicamente posible.**
    ///
    /// Un sensor roto devuelve su centinela (0, 255, un número negativo), y ese centinela **no es una
    /// medición**: es una avería. Un sensor que da siempre 0 **no está midiendo**, y ese caso ya está
    /// medido en `docs/ENCARGO-farellones-2026.md` (un campo que valía 0,00 en las 83 sesiones, y por eso
    /// el modelo **no tiene campo** para él). Aquí se filtra en el borde, para que el motor nunca vea un
    /// cero que parece un dato.
    pub fn plausible_kg(v: f32) -> Option<f32> {
        (v > 0.0 && v <= 500.0).then_some(v)
    }

    /// Lo mismo para un porcentaje: fuera de `0..=100` no es un porcentaje.
    pub fn plausible_pct(v: f32) -> Option<f32> {
        (v > 0.0 && v <= 100.0).then_some(v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sexo {
    Femenino,
    Masculino,
}

/// Una muestra dentro de una sesión: un instante con lo que se midió.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Muestra {
    /// Segundos desde el comienzo de la sesión.
    pub segundo: u32,
    /// Pulsaciones por minuto.
    pub pulso: Dato<u16>,
    /// Vatios, si hay medidor de potencia.
    pub potencia_w: Dato<u16>,
    /// **Cadencia, en revoluciones por minuto.**
    ///
    /// ESTE CAMPO SE LEÍA Y SE TIRABA, y por eso está aquí escrito. El lector ya sacaba
    /// `Record.cadence` del fichero y lo descartaba con un «el motor no la usa»; medido sobre sus
    /// datos, son **421.449 muestras en 83 de sus 102 ficheros**, con mediana 83 rpm. Tirar un dato
    /// que se está leyendo no es ahorrar: es **no poder contestar una pregunta que ya tenías
    /// contestada**, y la cadencia es la mitad de «cómo pedalea» — la otra mitad es la fase.
    ///
    /// Va en `u8` porque el FIT la declara de un byte y no llega a 255 ni girando en un descenso.
    /// Y va en `Dato` por la regla de siempre: **una muestra sin cadencia no gira a 0 rpm**, y ese
    /// cero, promediado con las demás, hundiría la media sin que nadie lo notara.
    pub cadencia_rpm: Dato<u8>,
    /// **Distancia acumulada desde el comienzo, en metros.**
    ///
    /// Se guarda por muestra —y no solo el total de la sesión— porque **sin ella no se puede
    /// preguntar dónde**: «la fase de la pierna se desvía a partir del kilómetro 60» es una pregunta
    /// que el total de la sesión no contesta, y la deriva por distancia solo se ve con esto.
    ///
    /// Y no es un número monótono garantizado: si el aparato se pierde o el fichero se corta, puede
    /// bajar. **Aquí no se corrige**: se guarda lo que dice el fichero y el que lo use decide.
    pub distancia_m: Dato<f64>,
    /// **Temperatura, en grados centígrados.**
    ///
    /// Está en **los 102 ficheros** y el motor la ignoraba. Importa porque **la deriva cardíaca que
    /// este motor mide depende del calor**: la misma potencia a 12 °C y a 34 °C no cuesta lo mismo, y
    /// sin este dato una deriva por temperatura se leería como una pérdida de forma física.
    pub temperatura_c: Dato<f32>,
}

/// **La postura sobre la bicicleta**: sentado o de pie.
///
/// EXISTE POR UN ERROR QUE YA SE COMETIÓ, y este enum es la vacuna. El FIT tiene dos campos que
/// parecen decir «izquierda» y «derecha» y **no lo dicen**: `avg_power_position` y
/// `avg_cadence_position` son **potencia y cadencia por POSTURA**, y sus dos huecos los define el
/// enum oficial del formato, `rider_position_type`: **`Seated = 0`, `Standing = 1`**.
///
/// Leído al revés, su ejemplo `[145, 410]` se convierte en «145 W la pierna izquierda y 410 W la
/// derecha»: un 37/63 que **no existe**, y que basta para asustar a una persona de 62 años por una
/// rodilla que está bien. Escribir el enum aquí hace que **el día que alguien lea el campo, lea la
/// palabra «postura»** y no una letra suelta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Postura {
    /// `rider_position_type = 0`.
    Sentado,
    /// `rider_position_type = 1`.
    DePie,
}

/// **Lo que el fichero declara de biomecánica y de carga** — lo que el reloj ya calculó.
///
/// POR QUÉ ES UN STRUCT Y NO CAMPOS SUELTOS DE `Sesion`: son doce datos que **vienen juntos o no
/// vienen**, y ninguno se puede deducir de las muestras. Separados, cualquiera que construya una
/// sesión tendría que escribir doce `Falta`; juntos, se escriben **una vez** con `Biomecanica::vacia()`.
///
/// DOS ADVERTENCIAS QUE VAN ESCRITAS AQUÍ PORQUE COSTARON DOS CONCLUSIONES FALSAS:
///
/// 1. **`potencia_media_sentado` / `potencia_media_de_pie` NO son izquierda y derecha.** Son la
///    potencia media mientras iba sentado y mientras iba de pie (el `avg_power_position` del FIT, dos
///    huecos indexados por `rider_position_type`). Lo mismo la cadencia. **Un nombre con «izquierda»
///    aquí sería una mentira con forma de dato, y por eso no existe ninguno.**
/// 2. **`fase_*` SÍ es por pierna**, y no tiene nada que ver con la postura: son los cuatro ángulos
///    de la fase de pedaleo de cada pierna (`avg_left_power_phase` / `avg_right_power_phase`), en
///    **grados**, ya escalados por el lector. **Esto es lo único de todo el fichero que dice algo de
///    una pierna concreta**, y por eso es lo que sostiene el módulo de la rodilla.
///
/// Y lo que **NO** está aquí, a propósito: el **reparto de potencia** izquierda/derecha. Ese dato
/// **no viaja en el FIT** — lo calcula Garmin en su servidor — y el campo que parece traerlo
/// (`left_right_balance`) trae **el número de serie del sensor**: medido en sus ficheros, 37950,
/// 38087, 38201… Un número de serie leído como porcentaje da un 47/53 que parece un desequilibrio y
/// no es más que el identificador de su potenciómetro. Inventarlo aquí sería el peor de los errores
/// posibles en este proyecto: un número plausible sobre el cuerpo de una persona.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Biomecanica {
    /// Vatios medios **mientras iba sentado** (`avg_power_position[Seated]`).
    pub potencia_media_sentado: Dato<u16>,
    /// Vatios medios **mientras iba de pie** (`avg_power_position[Standing]`).
    pub potencia_media_de_pie: Dato<u16>,
    /// Vatios **máximos** sentado (`max_power_position[Seated]`).
    pub potencia_maxima_sentado: Dato<u16>,
    /// Vatios **máximos** de pie (`max_power_position[Standing]`).
    pub potencia_maxima_de_pie: Dato<u16>,
    /// Revoluciones por minuto **sentado** (`avg_cadence_position[Seated]`).
    pub cadencia_media_sentado: Dato<f64>,
    /// Revoluciones por minuto **de pie** (`avg_cadence_position[Standing]`).
    pub cadencia_media_de_pie: Dato<f64>,
    /// **Fase de pedaleo de la pierna IZQUIERDA**: 4 ángulos en grados.
    ///
    /// Esto sí es una pierna, y es lo único del fichero que lo es. Si falta, falta **la pierna
    /// entera**: media fase no se promedia.
    pub fase_pierna_izquierda: Dato<[f64; 4]>,
    /// **Fase de pedaleo de la pierna DERECHA**: los mismos 4 ángulos.
    pub fase_pierna_derecha: Dato<[f64; 4]>,
    /// Respiraciones por minuto, media de la sesión (`enhanced_avg_respiration_rate`).
    pub respiracion_media: Dato<f64>,
    /// Respiraciones por minuto, **máximo** de la sesión (`enhanced_max_respiration_rate`).
    ///
    /// La respiración **estaba dada por perdida** y no lo está: viene en **97 de sus 102** ficheros.
    /// Va como `u8` en la mitad de los casos y como decimal en la otra mitad, así que se guarda en
    /// `f64` para no perder la segunda mitad.
    pub respiracion_maxima: Dato<f64>,
    /// **Carga anaeróbica** de la sesión (`total_anaerobic_training_effect`).
    ///
    /// Lleva la «A» delante del nombre porque es la del campo del FIT, y son **dos cargas
    /// distintas**: la aeróbica (`total_training_effect`) y la anaeróbica se suman mal.
    pub carga_anaerobica: Dato<f64>,
    /// **Pico de carga** de la sesión (`training_load_peak`), en la escala de carga de Garmin.
    pub pico_de_carga: Dato<f64>,
}

impl Biomecanica {
    /// Una biomecánica de la que **no sabemos nada**. Es como empieza toda sesión, y es honesto.
    pub fn vacia() -> Self {
        Self::default()
    }

    /// ¿Hay **alguna** de las fases? Es lo que se pregunta antes de enseñar nada de la rodilla.
    pub fn hay_fases(&self) -> bool {
        self.fase_pierna_izquierda.lo_hay() && self.fase_pierna_derecha.lo_hay()
    }

    /// Las dos fases juntas, si están las dos. Devuelve `None` si falta **cualquiera** de las dos:
    /// comparar una pierna contra un hueco no dice nada de la asimetría.
    pub fn fases(&self) -> Option<([f64; 4], [f64; 4])> {
        match (self.fase_pierna_izquierda, self.fase_pierna_derecha) {
            (Dato::Hay(i), Dato::Hay(d)) => Some((i, d)),
            _ => None,
        }
    }

    /// **Cuánta asimetría hay entre las dos piernas**, en grados: la diferencia máxima entre los
    /// cuatro ángulos de una y los de la otra.
    ///
    /// POR QUÉ EL MÁXIMO Y NO LA MEDIA: interesa **el gesto que más se desvía**, no el promedio. Dos
    /// piernas pueden coincidir en tres ángulos y separarse 30° en el cuarto —justo el que delata un
    /// pedaleo descompensado— y la media de los cuatro lo taparía dividiéndolo entre cuatro.
    ///
    /// Y LA RESTA VA POR EL CAMINO CORTO, que es un fallo que ya salió al medir: la primera versión
    /// restaba los ángulos tal cual, y una sesión daba **357,2° de asimetría**. No era una pierna
    /// desviada: eran dos piernas **casi iguales** con el cero en distinto lado —una empezaba el
    /// empuje en 357° y la otra en 0°, o sea 3° de diferencia—. Los ángulos son una circunferencia y
    /// **el cero está donde el potenciómetro quiere**: sin plegar la resta al rango −180..180, el
    /// número sale 357 donde la verdad es 3, y en la escala de «peor caso» eso parecería una alarma.
    ///
    /// Y devuelve `None` cuando falta una fase, en vez de un cero: **cero asimetría es una
    /// afirmación** («las dos piernas van iguales») y no se puede hacer sin las dos piernas.
    pub fn asimetria_grados(&self) -> Option<f64> {
        let (i, d) = self.fases()?;
        Some(
            i.iter()
                .zip(d.iter())
                .map(|(a, b)| distancia_angular(*a, *b))
                .fold(0.0_f64, f64::max),
        )
    }
}

/// **La distancia entre dos ángulos, por el camino corto**: siempre entre 0 y 180 grados.
///
/// Existe por una medición, no por elegancia. La primera versión de `asimetria_grados` restaba los
/// ángulos directamente, y sobre sus 83 sesiones dio un «peor caso» de **357,2°** — que en la escala de
/// una pierna desviada parece una barbaridad y en realidad era **una sesión con las dos piernas casi
/// idénticas**: una empezaba el empuje en 357° y la otra en 0°, o sea **3°**. Los ángulos dan la vuelta
/// a los 360 y **el cero lo pone el potenciómetro donde quiere**, así que una resta directa inventa
/// diferencias enormes donde hay coincidencia.
pub fn distancia_angular(a: f64, b: f64) -> f64 {
    let bruta = (a - b).rem_euclid(360.0);
    if bruta > 180.0 {
        360.0 - bruta
    } else {
        bruta
    }
}

/// Una sesión de entrenamiento: la lista de muestras y lo poco que sabemos de su contexto.
#[derive(Debug, Clone, PartialEq)]
pub struct Sesion {
    pub muestras: Vec<Muestra>,
    /// Duración declarada, si el fichero la trae. **No se supone a partir de las muestras**: pueden
    /// estar incompletas, y confundir «no hay muestras» con «no duró nada» sería un error.
    pub duracion_s: Dato<u32>,
    /// **Los intervalos entre latidos**, en milisegundos.
    ///
    /// ESTO NO ES UNA MUESTRA MÁS, y por eso va aparte: no es el pulso en un instante, sino **el tiempo
    /// entre un latido y el siguiente**. De ahí sale el HRV, que dice cómo está el cuerpo **antes** de
    /// entrenar. El FIT lo trae —**617.677 intervalos en sus 102 actividades**— y **Garmin Connect no
    /// lo enseña**.
    pub intervalos_rr_ms: Vec<f64>,
    /// **Lo que el fichero declara de biomecánica y de carga**, ya calculado por el reloj.
    ///
    /// Va en la sesión y no en el fichero leído porque es **de la sesión**: si mañana los datos
    /// entran por otra vía (una API, un JSON), el motor tiene que poder recibir lo mismo por el mismo
    /// sitio. Y va con valor por defecto —`Biomecanica::vacia()`— para que una sesión de la que no se
    /// sabe nada **no se pueda confundir con una de biomecánica cero**.
    pub biomecanica: Biomecanica,
}

impl Sesion {
    pub fn nueva(muestras: Vec<Muestra>) -> Self {
        Self {
            muestras,
            duracion_s: Dato::Falta,
            intervalos_rr_ms: Vec::new(),
            biomecanica: Biomecanica::vacia(),
        }
    }

    /// Una sesión con su duración declarada.
    ///
    /// EXISTE PORQUE CONSTRUIR UNA SESIÓN NO DEBE OBLIGAR A RELLENAR SIETE CAMPOS: cada vez que se
    /// añade un campo al modelo, **todos los sitios que construyen una sesión se rompen**. Con los
    /// constructores, se añade el campo **en un sitio** y el resto sigue igual.
    pub fn con_duracion(muestras: Vec<Muestra>, duracion_s: u32) -> Self {
        Self {
            muestras,
            duracion_s: Dato::Hay(duracion_s),
            intervalos_rr_ms: Vec::new(),
            biomecanica: Biomecanica::vacia(),
        }
    }

    /// Y una con sus intervalos entre latidos, para el HRV.
    pub fn con_rr(mut self, intervalos_rr_ms: Vec<f64>) -> Self {
        self.intervalos_rr_ms = intervalos_rr_ms;
        self
    }

    /// Y una con lo que el fichero **declara** de biomecánica y carga.
    ///
    /// Es el mismo patrón que `con_rr`, y por el mismo motivo: el lector de FIT rellena esto, y una
    /// sesión construida a mano en una prueba **no tiene por qué saber que existe**.
    pub fn con_biomecanica(mut self, biomecanica: Biomecanica) -> Self {
        self.biomecanica = biomecanica;
        self
    }

    /// La duración **que se puede afirmar**: la declarada si está, y si no, la que cubren las muestras.
    ///
    /// Se dice cuál de las dos es, porque `Dato` lo conserva: una duración **declarada** y una
    /// **deducida** no valen lo mismo.
    pub fn duracion(&self) -> Dato<u32> {
        if let Dato::Hay(d) = self.duracion_s {
            return Dato::Hay(d);
        }
        // Se toma el PRIMER y el ÚLTIMO instante de TODAS las muestras, no de las que traen pulso.
        //
        // Esto se corrigió el 2026-10-13, y el fallo era sutil: la primera versión usaba
        // `first()`/`last()` sobre las muestras tal cual, y si la primera no traía pulso, la
        // duración **salía corta sin avisar**. Una duración corta da una carga corta, y el número
        // seguía pareciendo bueno. Lo cazó una prueba que esperaba otra cosa.
        let primero = self.muestras.iter().map(|m| m.segundo).min();
        let ultimo = self.muestras.iter().map(|m| m.segundo).max();
        match (primero, ultimo) {
            (Some(a), Some(b)) if b > a => Dato::Hay(b - a),
            _ => Dato::Falta,
        }
    }

    /// Las pulsaciones de las muestras **que las traen**. Las que no, se saltan — no se rellenan.
    pub fn pulsos(&self) -> Vec<u16> {
        self.muestras.iter().filter_map(|m| m.pulso.valor()).collect()
    }

    /// **Las cadencias con las que de verdad pedaleó**, en rpm.
    ///
    /// **LOS CEROS SE SALTAN, y esto es una medición, no una opinión.** El `cadence` del FIT vale 0
    /// cuando no hay pedaleo, así que un cero no es «giraba a 0 rpm»: es **no saber**, otra vez. Y la
    /// prueba está en uno de sus ficheros: el fichero declara 75 rpm de media, la media de TODAS las
    /// muestras da **67,9** y la de las muestras con pedaleo da **75,4**. Garmin calcula justo eso
    /// —su propio perfil dice *«total_cycles / total_timer_time if nonzero_avg_cadence»*— y el motor
    /// no puede dar un número distinto del que da el aparato **por colar ceros**.
    ///
    /// El día que haya que estudiar el tiempo sin pedalear, ese cero se mira **en la muestra**, que
    /// sigue ahí entera; lo que no puede es entrar en una media disfrazado de cadencia.
    pub fn cadencias(&self) -> Vec<u8> {
        self.muestras
            .iter()
            .filter_map(|m| m.cadencia_rpm.valor())
            .filter(|c| *c > 0)
            .collect()
    }

    /// **La temperatura media de la sesión**, de las muestras que la traen.
    ///
    /// Devuelve `None` —no cero grados— cuando no hay ni una: **cero grados es una temperatura**, y
    /// es además la del hielo, o sea el peor número posible para colar en un cálculo de deriva.
    pub fn temperatura_media(&self) -> Option<f32> {
        let t: Vec<f32> = self.muestras.iter().filter_map(|m| m.temperatura_c.valor()).collect();
        if t.is_empty() {
            return None;
        }
        Some(t.iter().sum::<f32>() / t.len() as f32)
    }
}

// ==================================================================================================
// LA FECHA DE NACIMIENTO, Y POR QUE NO SE GUARDA LA EDAD
// ==================================================================================================
//
// CONTRATO DE ESTA PIEZA, escrito ANTES del codigo (docs/ARQUITECTURA-piezas-y-contrato.md):
//
//   QUE RECIBE:  un año, un mes y un día. Y, para calcular, la fecha de HOY.
//   QUE DEVUELVE: la edad en años cumplidos.
//   QUE **NO** HACE: no lee el reloj del sistema (la fecha de hoy se le PASA, para poder probarla),
//                   no valida rangos de edad, y no formatea nada.
//
// Y EL FALLO QUE ESTA PIEZA ARREGLA, que es de los que no se notan:
//
// La aplicación tenía **`edad = 60` escrito a mano**, y el cliente tiene **62** (su perfil de Garmin
// dice 1964-01-25). Y **no se arregla escribiendo 62**: eso **caduca el 25 de enero de 2027**, y
// entonces vuelve el mismo error, esta vez con un año de retraso. De la edad sale la **frecuencia
// máxima estimada**, así que **las zonas de entrenamiento salen corridas** — y una zona corrida no da
// error: da un entrenamiento equivocado.
//
// **Un dato que caduca y nadie lo nota es peor que un dato que falta**, porque el que falta se ve.

/// Una fecha del calendario, **sin hora y sin zona horaria**.
///
/// POR QUE NO SE USA UNA FECHA DEL SISTEMA: la biblioteca estándar de Rust **no trae** un tipo de
/// fecha, y añadir una dependencia entera para guardar tres números no se justifica. Además, una fecha
/// de nacimiento **no tiene hora ni zona**: nacer un día es nacer ese día en cualquier parte del mundo.
/// Guardarla como marca de tiempo obligaría a elegir una zona horaria, y esa elección sería inventada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fecha {
    pub anio: u16,
    pub mes: u8,
    pub dia: u8,
}

impl Fecha {
    /// Crea una fecha. **No comprueba que exista**, y es a propósito: quien la crea es el lector de un
    /// fichero o un campo de la pantalla, y ahí la comprobación tiene que dar un mensaje al usuario,
    /// no un pánico. Se comprueba en `es_valida`, que sí puede contestar que no.
    pub fn nueva(anio: u16, mes: u8, dia: u8) -> Self {
        Self { anio, mes, dia }
    }

    /// ¿Es una fecha que existe en el calendario?
    ///
    /// Hace falta porque el 31 de febrero **se puede escribir** y no existe. Y un 31 de febrero guardado
    /// en el perfil daría una edad calculada con una fecha imposible, **sin que nada avisara**.
    pub fn es_valida(&self) -> bool {
        if self.mes < 1 || self.mes > 12 || self.dia < 1 {
            return false;
        }
        let bisiesto = (self.anio % 4 == 0 && self.anio % 100 != 0) || self.anio % 400 == 0;
        let dias = match self.mes {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if bisiesto {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };
        self.dia <= dias
    }

    /// **La edad en años cumplidos**, el día `hoy`.
    ///
    /// LA FECHA DE HOY SE PASA COMO ARGUMENTO, y esto no es un capricho de diseño: es lo que hace que
    /// la función **se pueda probar**. Si leyera el reloj por dentro, una prueba solo podría comprobar
    /// la edad de hoy, y **el día del cumpleaños —que es justo el caso que falla— no se podría probar
    /// nunca** sin esperar un año.
    pub fn edad_en(&self, hoy: Fecha) -> u16 {
        let mut edad = hoy.anio.saturating_sub(self.anio);
        // Si todavía no ha llegado su cumpleaños este año, cumple uno menos.
        if (hoy.mes, hoy.dia) < (self.mes, self.dia) {
            edad = edad.saturating_sub(1);
        }
        edad
    }

    /// **La fecha como se escribe en el informe**: `2026-10-14`, sin barras ni nombres de mes.
    ///
    /// Vive en el MODELO y no en cada sitio que la formatea porque el formato es uno solo: el dia que
    /// se cambie, se cambia aqui, y no queda un sitio mintiendo con el formato viejo.
    pub fn a_texto(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.anio, self.mes, self.dia)
    }
}

impl Persona {
    /// **La edad, calculada desde la fecha de nacimiento.**
    ///
    /// Devuelve `Dato::Falta` si no se sabe cuándo nació — y **no un cero ni una edad media**. Un cero
    /// aquí sería un dato falso, igual que en todo el proyecto.
    ///
    /// Y **MANDA LA FECHA SOBRE LA EDAD ESCRITA**: si están las dos, se calcula. Es la misma regla que
    /// con el perfil y el aparato — cuando los dos hablan del mismo dato, gana el que no caduca.
    pub fn edad_en(&self, hoy: Fecha) -> Dato<u16> {
        if let Dato::Hay(f) = self.fecha_nacimiento {
            if f.es_valida() {
                return Dato::Hay(f.edad_en(hoy));
            }
        }
        // Sin fecha, se usa la edad escrita **si la hay**. Es el caso de un perfil viejo, y se acepta
        // para no perder el dato -- pero se prefiere la fecha, porque la fecha no caduca.
        match self.edad.valor() {
            Some(e) => Dato::Hay(u16::from(e)),
            None => Dato::Falta,
        }
    }

    /// **La fecha de hoy**, del reloj del sistema. Es el ÚNICO sitio del proyecto que mira el reloj.
    ///
    /// Está aislado aquí por el mismo motivo que la fecha se pasa como argumento: **que el cálculo sea
    /// comprobable**. Todo lo demás recibe la fecha; solo esto la pide prestada al sistema.
    pub fn hoy() -> Fecha {
        // Los segundos desde 1970 a días, con el algoritmo civil (Howard Hinnant, dominio público).
        // Se implementa en vez de añadir una dependencia: son diez líneas y no hay que mantenerlas.
        let segundos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let dias = segundos.div_euclid(86_400);
        // 1970-01-01 + dias
        let z = dias + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let anio = if m <= 2 { y + 1 } else { y };
        Fecha::nueva(anio as u16, m as u8, d as u8)
    }
}
