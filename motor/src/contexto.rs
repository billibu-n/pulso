//! **El contexto**: lo que no mide ningún sensor, y lo que mide más de uno.
//!
//! NACE DE UNA PETICIÓN DEL DUEÑO, y tiene tres partes distintas que conviene no mezclar:
//!
//! > *«Los cuadros de contexto son importantes para generar un resultado más completo. Los debe poder
//! > proporcionar el usuario en caso de que no los proporcione una aplicación. Una aplicación los puede
//! > rellenar de manera automática, pero si faltan datos el usuario debería poder llenarlo si tiene la
//! > información, o de lo contrario es nulo.»*
//!
//! Y avisó de **tres escenarios** que hay que contemplar, porque si no **la herramienta daría siempre
//! el mismo resultado**:
//!
//! 1. **La aplicación lo rellena sola** — el reloj da el sueño, la báscula da el peso.
//! 2. **El usuario lo escribe** — porque tiene el dato y la aplicación no.
//! 3. **No lo hay, y queda nulo** — y el motor **calcula igual**, con el margen más ancho.
//!
//! Y AQUÍ HAY UNA COSA QUE HAY QUE DECIR, porque el dueño dijo *«un sensor en teoría no puede mentir a
//! no ser que esté averiado»*. **Es casi verdad, y el matiz importa mucho:**
//!
//! - **El sensor no miente sobre lo que mide. Pero puede estar midiendo OTRA COSA.** Un reloj dice 48
//!   pulsaciones en reposo y una báscula dice 71. **Los dos son verdad**: uno mide el mínimo mientras
//!   duerme, y el otro una medición sentado por la tarde. **No hay avería: hay dos cosas distintas con
//!   el mismo nombre.**
//! - **Y la comida no la mide ningún sensor: la escribe una persona.** Cuando alguien registra lo que
//!   come **se equivoca hacia abajo**, y no por mentir: **es difícil acordarse y estimar raciones**.
//!   Eso está medido en la literatura de nutrición, y significa que **un registro de comidas es un
//!   dato BLANDO**, y hay que tratarlo como tal.
//!
//! POR ESO ESTE MÓDULO EXISTE: para que el motor sepa **de dónde viene cada dato**, **cuánto fiarse**,
//! y **qué hacer cuando faltan**.

use crate::rango::{self, Rango};

/// **De dónde sale un dato de contexto**, y por tanto cuánto se puede apretar con él.
///
/// EL ORDEN NO ES UNA JERARQUÍA DE CALIDAD, y esto es la parte importante: **es una jerarquía de TIPO**.
/// Un peso de báscula es mejor que un peso recordado; pero un sueño del reloj **no es comparable** con
/// un sueño apuntado a mano, porque **miden cosas distintas con el mismo nombre**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fuente {
    /// **Lo rellena una aplicación**, de un aparato que lo mide.
    Automatica { aplicacion: String },
    /// **Lo escribe la persona**, porque tiene el dato y la aplicación no.
    Declarada,
    /// Una báscula: mide el cuerpo, y lo mide bien.
    Bascula,
    /// No está. **Y no es un fallo: es un escenario que hay que contemplar.**
    Ausente,
}

impl Fuente {
    pub fn nombre(&self) -> &'static str {
        match self {
            Fuente::Automatica { .. } => "de la aplicación",
            Fuente::Declarada => "lo escribió el usuario",
            Fuente::Bascula => "de la báscula",
            Fuente::Ausente => "no está",
        }
    }

    /// **El margen que hay que añadir por la FUENTE**, además del margen propio del dato.
    ///
    /// Es la traducción numérica de la diferencia entre un aparato y una persona:
    ///
    /// - **de la aplicación**: cero. Lo midió un aparato.
    /// - **lo escribió el usuario**: un margen, porque recordar tiene error.
    /// - **la báscula**: cero, para el peso. Mide lo que dice medir.
    /// - **ausente**: no se usa el dato, así que no aporta margen.
    pub fn margen_por_la_fuente(&self) -> f64 {
        match self {
            Fuente::Automatica { .. } | Fuente::Bascula => 0.0,
            Fuente::Declarada => 0.15, // un 15 %: lo que se recuerda no es exacto
            Fuente::Ausente => 0.0,
        }
    }
}

/// **Un dato de contexto**: su valor, su margen, y **de dónde salió**.
#[derive(Debug, Clone, PartialEq)]
pub struct Contexto {
    /// Horas de sueño de la noche anterior.
    pub sueno_horas: Option<Rango>,
    /// Cómo se sintió, de 1 a 5. **Es la señal que él ya da a mano hoy, y se pierde.**
    pub sensacion: Option<Rango>,
    /// Qué comió antes de entrenar: proteína, hidratos, grasa, o nada.
    pub comida_antes: Option<Comida>,
    /// Si tomó alcohol.
    pub alcohol: Option<bool>,
    /// **Todos los datos que se pudieron reunir, y de dónde salió cada uno.**
    pub procedencias: Vec<(String, Fuente)>,
}

/// Qué comió antes de entrenar. **Se pregunta así, en cuatro opciones, porque es lo que se contesta.**
///
/// POR QUÉ NO UN DIARIO: porque **nadie registra lo que come de verdad**. Preguntar cada comida **no se
/// contesta**, y un dato que no se contesta es peor que no pedirlo. Estas cuatro opciones **sí se
/// contestan**, y bastan para el **efecto térmico**, que es lo que el cliente notaba.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comida {
    /// Casi todo proteína: es la que más energía cuesta digerir.
    Proteinas,
    /// Hidratos: lo normal antes de entrenar.
    Hidratos,
    /// Grasa: la que menos cuesta.
    Grasa,
    /// Nada.
    Nada,
}

impl Comida {
    /// **La energía que se va en digerir**, como proporción de lo comido.
    ///
    /// DE DÓNDE SALEN LOS NÚMEROS, y no son inventados: el **efecto térmico de los alimentos** está
    /// descrito en la literatura de nutrición, y las cifras habituales son **20-30 % para la proteína,
    /// 5-10 % para los hidratos y 0-3 % para la grasa**. Se usan los centros de cada franja.
    pub fn coste_de_digerir(self) -> f64 {
        match self {
            Comida::Proteinas => 0.25,
            Comida::Hidratos => 0.075,
            Comida::Grasa => 0.015,
            Comida::Nada => 0.0,
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            Comida::Proteinas => "proteínas",
            Comida::Hidratos => "hidratos",
            Comida::Grasa => "grasa",
            Comida::Nada => "nada",
        }
    }
}

impl Contexto {
    /// **Sin nada.** Es el escenario que hay que soportar: el motor calcula igual.
    pub fn vacio() -> Self {
        Self {
            sueno_horas: None,
            sensacion: None,
            comida_antes: None,
            alcohol: None,
            procedencias: Vec::new(),
        }
    }

    /// Cuántos datos de contexto hay, y cuántos se podrían tener.
    pub fn cuantos(&self) -> (usize, usize) {
        let hay = [
            self.sueno_horas.is_some(),
            self.sensacion.is_some(),
            self.comida_antes.is_some(),
            self.alcohol.is_some(),
        ];
        (hay.iter().filter(|x| **x).count(), hay.len())
    }

    /// **Lo que el usuario puede rellenar, dicho en imperativo y en una línea cada cosa.**
    ///
    /// Existe porque la aplicación tiene que **PEDIR** lo que falta — y pedirlo mal es no pedirlo.
    pub fn que_pedir(&self) -> Vec<(&'static str, &'static str)> {
        let mut v = Vec::new();
        if self.sueno_horas.is_none() {
            v.push(("sueño", "un número: cuántas horas dormiste"));
        }
        if self.sensacion.is_none() {
            v.push(("sensación", "una marca del 1 al 5"));
        }
        if self.comida_antes.is_none() {
            v.push(("comida", "una de cuatro: proteínas, hidratos, grasa o nada"));
        }
        if self.alcohol.is_none() {
            v.push(("alcohol", "sí o no"));
        }
        v
    }

    /// **El aviso sobre el sueño y la sensación**, cuando se cruzan con la carga.
    ///
    /// ES LO QUE EL CLIENTE PIDE, en sus palabras: *«mezcla alimentación, horas de sueño, hábitos de
    /// ejercicio»*. Ese cruce **no existía en ningún sitio**, y aquí está.
    ///
    /// Y se hace con una regla prudente: **no se afirma una causa, se señala una coincidencia**. Que
    /// alguien duerma menos y entrene más **no prueba** que una cosa cause la otra.
    pub fn cruce_con_la_carga(&self, carga: &Rango, carga_habitual: &Rango) -> Option<String> {
        let sueno = self.sueno_horas.as_ref()?;
        // Si durmió bien, no hay nada que señalar.
        if sueno.valor >= 7.0 {
            return None;
        }
        // Y si la carga no subió, tampoco.
        if carga.valor <= carga_habitual.valor {
            return None;
        }
        Some(format!(
            "Esta sesión tiene más carga de lo habitual ({:.0} frente a {:.0}) y dormiste {:.1} horas. \
             No se puede afirmar que una cosa cause la otra, pero coincide, y conviene saberlo.",
            carga.valor, carga_habitual.valor, sueno.valor
        ))
    }

    /// **La precisión del contexto**: cuenta los datos, **Y MIRA DE DÓNDE VIENEN**.
    ///
    /// ESTO SE CORRIGIÓ EL 2026-10-13, y el fallo lo cazó una prueba. La primera versión contaba
    /// **solo cuántos datos hay**, así que **una hora de sueño escrita de memoria valía lo mismo que
    /// una medida por el reloj**. Y no valen lo mismo: **recordar tiene error, medir no.**
    ///
    /// Por eso cada dato pesa según su fuente: **un aparato cuenta entero, una persona cuenta la
    /// mitad**. Y «tener los cuatro» puestos a mano **no llega a la misma precisión** que tenerlos
    /// medidos — que es exactamente lo que hay que decirle al usuario para que conecte la aplicación.
    pub fn precision(&self) -> rango::Precision {
        let (hay, total) = self.cuantos();
        if hay == 0 {
            return rango::Precision::Amplia;
        }

        // LA CUENTA, y llego aqui despues de que TRES `assert` mios fallaran. El motor tenia razon
        // las tres veces, y lo que estaba mal era mi forma de contarlo.
        //
        // LA PRIMERA VERSION contaba solo CUANTOS datos hay, y entonces **una hora de sueno escrita de
        // memoria valia lo mismo que una medida por el reloj**. Mal.
        // LA SEGUNDA castigaba POR SALTOS: uno o mas datos declarados bajaban la precision de golpe,
        // asi que **tres escritos y cuatro escritos daban lo mismo**. Tambien mal.
        //
        // LA BUENA es PROPORCIONAL: **cada dato vale segun quien lo puso, y se suman las partes**.
        //   un aparato lo midio ......... vale 1
        //   lo escribio la persona ...... vale 0,5   (recordar tiene error)
        //
        // Y asi los cuatro escenarios salen DISTINTOS, que es lo que el dueno pide: si todos dieran
        // lo mismo, la herramienta seria «constante y repetitiva».
        let peso = |nombre: &str| -> f64 {
            match self
                .procedencias
                .iter()
                .find(|(n, _)| n == nombre)
                .map(|(_, f)| f)
            {
                Some(Fuente::Automatica { .. }) | Some(Fuente::Bascula) => 1.0,
                // Y si no se dice de donde viene, se supone lo peor: es declarado.
                _ => 0.5,
            }
        };
        let puntos = peso("sueño") * f64::from(self.sueno_horas.is_some() as u8)
            + peso("sensación") * f64::from(self.sensacion.is_some() as u8)
            + peso("comida") * f64::from(self.comida_antes.is_some() as u8)
            + peso("alcohol") * f64::from(self.alcohol.is_some() as u8);
        let proporcion = puntos / total as f64;

        if proporcion >= 0.9 {
            // **Nunca «alta»**: ni con los cuatro medidos. Un dato de contexto no es una medicion de
            // laboratorio, y decir «alta» seria mentir.
            rango::Precision::Buena
        } else if proporcion >= 0.6 {
            rango::Precision::Aceptable
        } else {
            rango::Precision::Amplia
        }
    }
}

/// **Los escenarios de relleno**, que es lo que el dueño pidió contemplar.
///
/// Existe para poder **probar los tres**, y para que la aplicación sepa qué hacer en cada uno.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escenario {
    /// La aplicación lo rellena sola, de un aparato.
    Automatico,
    /// El usuario lo escribe, porque lo tiene.
    Manual,
    /// **No está, y queda nulo.** El motor calcula igual.
    Nulo,
}

impl Escenario {
    pub fn nombre(self) -> &'static str {
        match self {
            Escenario::Automatico => "lo rellena la aplicación",
            Escenario::Manual => "lo escribe el usuario",
            Escenario::Nulo => "no está: queda nulo",
        }
    }

    /// Qué hace la aplicación en cada caso. **Y en el nulo NO se inventa: se dice.**
    pub fn que_hace(self) -> &'static str {
        match self {
            Escenario::Automatico => "se lee del aparato y se marca de dónde viene",
            Escenario::Manual => "se pide al usuario, en UNA línea, y se marca como declarado",
            Escenario::Nulo => "se dice que falta, y el resultado sale con el margen más ancho",
        }
    }
}
