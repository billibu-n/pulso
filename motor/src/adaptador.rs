//! El **adaptador**: traduce el formato de fuera a **nuestro** modelo.
//!
//! POR QUÉ ESTA PIEZA EXISTE, y por qué está separada del motor:
//!
//! El motor **no sabe** de Garmin, ni de FIT, ni del JSON que use el cliente. Sabe de **su propio
//! modelo**. Todo lo de fuera entra **por aquí**.
//!
//! La consecuencia práctica: cuando el cliente mande sus datos de verdad, **cambia este fichero, no el
//! motor**. Y si mañana cambia de reloj o de aplicación, **otra vez este fichero**.
//!
//! El primer formato que se implementa es el **sintético** (`datos-sinteticos/`), que existe porque el
//! cliente aún no ha dado los suyos. **Sirve para probar el motor entero sin depender de nadie.**

use crate::datos::{Dato, Fecha, Muestra, Persona, Sesion, Sexo};
use serde::{Deserialize, Serialize};

/// Un dato que **puede venir o no venir** en el fichero.
///
/// Es `Option` y no `Dato` a propósito: en el límite con el mundo exterior se usa lo que el mundo
/// exterior entiende. La traducción a `Dato` la hace este módulo, **en un solo sitio**.
type Quizas<T> = Option<T>;

#[derive(Debug, Deserialize, Serialize)]
pub struct PerfilJson {
    pub persona: PersonaJson,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PersonaJson {
    /// **La fecha de nacimiento, como texto `AAAA-MM-DD`.** Es lo que MANDA sobre la edad.
    ///
    /// POR QUE TEXTO Y NO TRES NUMEROS: así el fichero se puede **leer y editar a mano** —que es lo que
    /// el dueño va a hacer— y una fecha escrita mal se ve de un vistazo. Con tres números sueltos, un
    /// mes 13 pasaria desapercibido hasta que alguien mirara la edad calculada.
    pub fecha_nacimiento: Quizas<String>,
    pub edad: Quizas<u8>,
    pub peso_kg: Quizas<f32>,
    pub pulso_reposo: Quizas<u16>,
    pub pulso_maximo_medido: Quizas<u16>,
    pub sexo: Quizas<String>,
    /// Nuevos: los que aporta una bascula de composicion corporal. Si no vienen, quedan como Falta,
    /// y el motor **calcula igual** -- solo que con un margen mas ancho.
    pub altura_cm: Quizas<f32>,
    pub grasa_corporal_pct: Quizas<f32>,
    pub masa_muscular_kg: Quizas<f32>,
    /// El umbral de lactato y el consumo maximo de oxigeno. **Si no vienen, quedan como Falta** y el
    /// motor calcula igual, solo que con un margen mas ancho.
    pub umbral_lactato: Quizas<u16>,
    pub vo2_maximo: Quizas<f32>,
}

impl PersonaJson {
    /// Traduce al modelo del motor.
    ///
    /// **Lo que NO está, queda como `Falta`.** No se rellena con un valor por defecto, no se pone a
    /// cero, y no se supone. Esa es la regla de todo el proyecto.
    pub fn a_persona(&self) -> Persona {
        Persona {
            fecha_nacimiento: self
                .fecha_nacimiento
                .as_deref()
                .map(fecha_desde_texto)
                .unwrap_or(Dato::Falta),
            edad: segun(self.edad),
            peso_kg: segun(self.peso_kg),
            pulso_reposo: segun(self.pulso_reposo),
            pulso_maximo_medido: segun(self.pulso_maximo_medido),
            altura_cm: segun(self.altura_cm),
            grasa_corporal_pct: segun(self.grasa_corporal_pct),
            masa_muscular_kg: segun(self.masa_muscular_kg),
            umbral_lactato: segun(self.umbral_lactato),
            vo2_maximo: segun(self.vo2_maximo),
            sexo: match self.sexo.as_deref() {
                Some("femenino") | Some("F") => Dato::Hay(Sexo::Femenino),
                Some("masculino") | Some("M") => Dato::Hay(Sexo::Masculino),
                // Un valor que no se entiende NO se adivina: se trata como si faltara.
                _ => Dato::Falta,
            },
        }
    }

    /// Traduce **al revés**: del modelo del motor al formato del fichero.
    ///
    /// **Lo que falta se escribe `null`, nunca un cero.** Es la misma regla en el otro sentido, y
    /// tiene la misma consecuencia: un dato ausente escrito como 0, al volver a leerlo, parecería un
    /// dato de la persona. «No sé su peso» se convertiría en «pesa cero kilos» — y ese cero viajaría
    /// por todas las fórmulas sin que nadie lo notara.
    pub fn de(p: &Persona) -> Self {
        Self {
            // La fecha vuelve al fichero como el MISMO texto que entiende `a_persona`, para que el
            // ciclo leer -> guardar -> leer no cambie el dato por el camino.
            fecha_nacimiento: fecha_a_texto(&p.fecha_nacimiento),
            edad: quizas(p.edad),
            peso_kg: quizas(p.peso_kg),
            pulso_reposo: quizas(p.pulso_reposo),
            pulso_maximo_medido: quizas(p.pulso_maximo_medido),
            // El sexo se guarda con la misma palabra que entiende `a_persona`, para que el ciclo
            // leer → guardar → leer no cambie el dato por el camino.
            sexo: match p.sexo {
                Dato::Hay(Sexo::Femenino) => Some("femenino".to_string()),
                Dato::Hay(Sexo::Masculino) => Some("masculino".to_string()),
                Dato::Falta => None,
            },
            altura_cm: quizas(p.altura_cm),
            grasa_corporal_pct: quizas(p.grasa_corporal_pct),
            masa_muscular_kg: quizas(p.masa_muscular_kg),
            umbral_lactato: quizas(p.umbral_lactato),
            vo2_maximo: quizas(p.vo2_maximo),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SemanaJson {
    pub semana: u32,
    pub sesiones: Vec<SesionJson>,
}

#[derive(Debug, Deserialize)]
pub struct SesionJson {
    pub tipo: String,
    pub duracion_s: u32,
    pub muestras: Vec<MuestraJson>,
}

#[derive(Debug, Deserialize)]
pub struct MuestraJson {
    pub segundo: u32,
    pub pulso: Quizas<u16>,
    pub potencia_w: Quizas<u16>,
    /// La cadencia, **si el formato de fuera la trae**. Los ficheros de ejemplo que ya existen no la
    /// tienen, y por eso es `Quizas`: sin ella la muestra queda en `Falta`, que es la verdad — **no
    /// en 0 rpm**, que sería una bicicleta parada.
    pub cadencia_rpm: Quizas<u8>,
    /// La distancia acumulada y la temperatura, por el mismo motivo: si el fichero no las trae,
    /// quedan en `Falta` y **no en cero**.
    pub distancia_m: Quizas<f64>,
    pub temperatura_c: Quizas<f32>,
}

impl SesionJson {
    /// Traduce una sesión al modelo del motor, **conservando lo que falta como `Falta`**.
    ///
    /// La biomecánica se deja vacía a propósito: el JSON de fuera **todavía no la declara**, y
    /// rellenarla con ceros sería exactamente lo que este motor no hace. El día que ese formato la
    /// traiga, se traduce aquí y se dice.
    pub fn a_sesion(&self) -> Sesion {
        let muestras: Vec<Muestra> = self
            .muestras
            .iter()
            .map(|m| Muestra {
                segundo: m.segundo,
                pulso: segun(m.pulso),
                potencia_w: segun(m.potencia_w),
                cadencia_rpm: segun(m.cadencia_rpm),
                distancia_m: segun(m.distancia_m),
                temperatura_c: segun(m.temperatura_c),
            })
            .collect();
        Sesion::con_duracion(muestras, self.duracion_s)
    }
}

/// Traduce un `Option` del exterior a un `Dato` nuestro. **Un solo sitio para esta regla.**
fn segun<T>(v: Quizas<T>) -> Dato<T> {
    match v {
        Some(x) => Dato::Hay(x),
        None => Dato::Falta,
    }
}

/// Y la vuelta: un `Dato` nuestro a lo que entiende el mundo exterior. **El otro sentido de la misma
/// regla**, y por eso vive aquí al lado: si un dato falta, sale `null` — nunca un valor inventado.
fn quizas<T>(d: Dato<T>) -> Quizas<T> {
    match d {
        Dato::Hay(v) => Some(v),
        Dato::Falta => None,
    }
}

/// Lee el perfil de la persona desde un texto JSON.
pub fn leer_perfil(texto: &str) -> Result<Persona, String> {
    let p: PerfilJson = serde_json::from_str(texto).map_err(|e| format!("el JSON no se entiende: {e}"))?;
    Ok(p.persona.a_persona())
}

/// Escribe el perfil de la persona como texto JSON.
///
/// **INDENTADO A PROPÓSITO**: el fichero vive junto al programa y su dueño va a abrirlo y mirarlo.
/// Un JSON en una sola línea lo lee un programa, no la persona a la que pertenecen los datos.
///
/// No devuelve `Result`: escribir un perfil no puede fallar (son números, textos y `null`), y aquí no
/// hay nada que decidir. Si algún día fallara, sería un fallo del programa y tendría que verse, en
/// vez de devolver un texto a medias que luego no se pudiera leer.
pub fn a_json(p: &Persona) -> String {
    let perfil = PerfilJson { persona: PersonaJson::de(p) };
    serde_json::to_string_pretty(&perfil)
        .expect("un perfil de números, textos y null siempre se puede escribir")
}

/// Lee las semanas de entrenamiento desde un texto JSON.
pub fn leer_semanas(texto: &str) -> Result<Vec<SemanaJson>, String> {
    #[derive(Deserialize)]
    struct Raiz {
        semanas: Vec<SemanaJson>,
    }
    let r: Raiz = serde_json::from_str(texto).map_err(|e| format!("el JSON no se entiende: {e}"))?;
    Ok(r.semanas)
}

/// **Traduce `AAAA-MM-DD` a una fecha.** Y si no se puede, dice `Falta`.
///
/// POR QUE DEVUELVE `Dato` Y NO UN `Result`: quien llama está traduciendo un fichero, y **un fichero
/// con una fecha rara no puede impedir leer el resto**. El perfil entero se perdería por un guion de
/// más. Se pierde **ese dato**, que es lo honesto, y el motor ya sabe trabajar sin él.
///
/// Y SE COMPRUEBA QUE LA FECHA EXISTA: el 31 de febrero se puede escribir y no existe. Una fecha
/// imposible daría una edad calculada con una fecha que no es, **sin que nada avisara**.
pub fn fecha_desde_texto(texto: &str) -> Dato<Fecha> {
    let partes: Vec<&str> = texto.trim().split('-').collect();
    if partes.len() != 3 {
        return Dato::Falta;
    }
    let (Ok(anio), Ok(mes), Ok(dia)) = (
        partes[0].parse::<u16>(),
        partes[1].parse::<u8>(),
        partes[2].parse::<u8>(),
    ) else {
        return Dato::Falta;
    };
    let f = Fecha::nueva(anio, mes, dia);
    if f.es_valida() {
        Dato::Hay(f)
    } else {
        Dato::Falta
    }
}

/// Y el camino de vuelta: la fecha, como texto `AAAA-MM-DD`.
pub fn fecha_a_texto(f: &Dato<Fecha>) -> Option<String> {
    // El formato vive en `Fecha::a_texto`, en el modelo: aqui solo se decide QUE HACER con un campo
    // que falta (`None`, nunca una fecha vacia disfrazada).
    f.valor().map(|f| f.a_texto())
}
