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
    /// Años. Hace falta para estimar la frecuencia máxima.
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
}

impl Sesion {
    pub fn nueva(muestras: Vec<Muestra>) -> Self {
        Self { muestras, duracion_s: Dato::Falta, intervalos_rr_ms: Vec::new() }
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
        }
    }

    /// Y una con sus intervalos entre latidos, para el HRV.
    pub fn con_rr(mut self, intervalos_rr_ms: Vec<f64>) -> Self {
        self.intervalos_rr_ms = intervalos_rr_ms;
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
}
