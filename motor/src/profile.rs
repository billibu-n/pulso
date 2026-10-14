//! **Dónde viven los datos de la persona, y cómo se leen y se guardan.**
//!
//! POR QUÉ ESTE MÓDULO EXISTE, y es el fallo más grave que tenía la aplicación: los datos de la
//! persona estaban **escritos en el código** (`edad = 60`, `umbral = 169`...). Eso significa que
//! **los resultados que enseñaba no eran del cliente, eran de quien compiló el programa**, y no se
//! notaba: la pantalla enseñaba números con buena cara.
//!
//! LA DECISIÓN DEL DUEÑO (2026-10-13), y manda: **el fichero se guarda JUNTO AL PROGRAMA**, no en la
//! carpeta de configuración del sistema. El motivo es suyo y es bueno: así **el fichero viaja con la
//! aplicación** — si copia la carpeta a otro equipo, o al teléfono, **sus datos van con ella**.
//!
//! Y EL AVISO QUE HAY QUE RESOLVER, porque una decisión sin su caso malo no está terminada: si el
//! programa se instala en una carpeta de **solo lectura** (`/usr/bin`, por ejemplo, que es de root),
//! **ahí no se puede escribir**. La salida es caer a la carpeta de configuración del usuario — y
//! **decirlo**, devolviendo esa información para que la pantalla pueda enseñar dónde quedaron los
//! datos. Un dato guardado en un sitio que el usuario no conoce es un dato que no encuentra.

use crate::adaptador;
use crate::datos::{Dato, Persona, Sexo};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// El nombre del fichero. **Uno solo, y aquí**: el resto del programa lo nombra a través de este
/// módulo, para que no haya dos ideas distintas de dónde están los datos.
pub const NOMBRE: &str = "perfil.json";

/// La carpeta que se usa cuando junto al programa no se puede escribir.
///
/// Es la convención de siempre en Linux (`$HOME/.config/<programa>/`). **No se inventa una ruta
/// propia**: si el usuario tiene que buscar el fichero a mano, la convención es lo que se lo
/// encuentra.
const CARPETA_DE_RESERVA: &str = ".config/pulso";

/// **Dónde se guardan los datos de la persona**, y si es el sitio decidido o la reserva.
///
/// Devuelve la ruta y `true` si es **junto al programa** (lo decidido) o `false` si es la **reserva**
/// (la carpeta de configuración, porque junto al programa no se puede escribir).
///
/// El segundo valor no es un detalle: es lo que permite que la pantalla diga **dónde quedaron sus
/// datos**, en vez de que el usuario los busque.
pub fn ruta_del_perfil() -> (PathBuf, bool) {
    let junto_al_programa = std::env::current_exe()
        .ok()
        // `current_exe` da el ejecutable; sus datos van en su carpeta padre.
        .and_then(|exe| exe.parent().map(|carpeta| carpeta.join(NOMBRE)))
        // SI NO SE PUEDE SABER DÓNDE ESTÁ EL PROGRAMA, se cae a `./perfil.json`, que es relativo al
        // directorio desde el que se lanza. Es peor -- dos lanzamientos desde carpetas distintas
        // serían dos perfiles distintos -- pero es preferible a no guardar nada. Y se dice, porque
        // `elegir_ruta` devolverá `false` y la pantalla lo enseñará.
        .unwrap_or_else(|| PathBuf::from(NOMBRE));
    elegir_ruta(junto_al_programa)
}

/// Elige entre la ruta decidida y la de reserva, **comprobando la escritura de verdad**.
///
/// Está separada de `ruta_del_perfil` para poder **probarla**: una carpeta de solo lectura se puede
/// fabricar en una prueba, pero no se puede hacer que `current_exe` apunte a donde uno quiera.
pub fn elegir_ruta(junto_al_programa: PathBuf) -> (PathBuf, bool) {
    if se_puede_escribir(&junto_al_programa) {
        return (junto_al_programa, true);
    }
    // LA RESERVA, y por qué: una instalación del sistema (`/usr/bin`) es de solo lectura para quien
    // no sea root, y ahí el programa no puede guardar. Sin esta salida, el primer guardado fallaría
    // y el usuario se quedaría sin poder usar la aplicación. **Se sigue guardando; se guarda en otro
    // sitio, y se dice cuál.**
    (ruta_de_reserva(), false)
}

/// La ruta de reserva: `$HOME/.config/pulso/perfil.json`.
pub fn ruta_de_reserva() -> PathBuf {
    let casa = std::env::var_os("HOME")
        .map(PathBuf::from)
        // Sin `HOME` no hay carpeta de configuración que valga; el directorio temporal es lo único
        // que queda. Es malo (el sistema lo puede borrar), y por eso la pantalla enseña la ruta:
        // así el usuario lo ve en vez de perder los datos sin enterarse.
        .unwrap_or_else(std::env::temp_dir);
    casa.join(CARPETA_DE_RESERVA).join(NOMBRE)
}

/// **¿Se puede escribir ahí? Comprobado intentándolo, no preguntando.**
///
/// POR QUÉ NO VALE MIRAR LOS PERMISOS: en Unix el permiso del fichero no dice si se puede escribir
/// (depende de quién sea el usuario, de su grupo, y de si el sistema de ficheros está montado de
/// solo lectura). La única comprobación que no miente es intentarlo.
///
/// Y SE HACE SIN ROMPER NADA: si el fichero ya existe se abre para escribir **sin truncarlo** (eso
/// no destruye su contenido); si no existe, se prueba la CARPETA con un fichero de prueba que se
/// borra en el acto. **No se crea el `perfil.json` vacío**, y esto importa: un fichero vacío no es
/// «no hay perfil», es un JSON roto, y la aplicación diría que los datos no se entienden.
pub fn se_puede_escribir(ruta: &Path) -> bool {
    if ruta.exists() {
        return std::fs::OpenOptions::new().write(true).open(ruta).is_ok();
    }
    let carpeta = carpeta_de(ruta);
    let prueba = carpeta.join(format!("{NOMBRE}.prueba-{}", std::process::id()));
    match std::fs::File::create(&prueba) {
        Ok(_) => {
            let _ = std::fs::remove_file(&prueba);
            true
        }
        Err(_) => false,
    }
}

/// La carpeta donde vive una ruta. Para `./perfil.json` (sin carpeta) es la carpeta actual.
fn carpeta_de(ruta: &Path) -> PathBuf {
    match ruta.parent() {
        Some(c) if !c.as_os_str().is_empty() => c.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// **Lee el perfil.** Si el fichero no existe, devuelve una persona de la que no se sabe nada — y eso
/// **no es un error**: es el primer arranque, que es el caso más normal de todos.
///
/// Si el fichero existe y está mal, **sí es un error, y se dice el motivo**: un fichero roto no se
/// puede confundir con «no hay datos», porque uno se arregla rellenándolo y el otro releyéndolo.
pub fn leer(ruta: &Path) -> Result<Persona, String> {
    match std::fs::read_to_string(ruta) {
        Ok(texto) => adaptador::leer_perfil(&texto).map_err(|e| format!("{}: {e}", ruta.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Persona::desconocida()),
        Err(e) => Err(format!("no se pudo leer {}: {e}", ruta.display())),
    }
}

/// **Guarda el perfil**, y crea la carpeta si hace falta.
///
/// Se crea la carpeta porque la de reserva (`$HOME/.config/pulso`) no existe en un equipo nuevo, y
/// fallar por eso sería fallar por un detalle que el programa puede resolver solo.
pub fn guardar(p: &Persona, ruta: &Path) -> Result<(), String> {
    let carpeta = carpeta_de(ruta);
    if !carpeta.exists() {
        std::fs::create_dir_all(&carpeta)
            .map_err(|e| format!("no se pudo crear {}: {e}", carpeta.display()))?;
    }
    std::fs::write(ruta, adaptador::a_json(p))
        .map_err(|e| format!("no se pudo escribir {}: {e}", ruta.display()))
}

/// **Traduce un campo de TEXTO a un dato del motor.** Es la regla que sostiene toda la honestidad
/// del proyecto, escrita una sola vez:
///
/// - **Vacío es «no lo sé»** (`Dato::Falta`), **no cero**. «No sé su peso» no es «pesa cero kilos».
/// - **La coma decimal se acepta**, porque el proyecto escribe los números en español (`70,2`) y
///   rechazar el formato del propio usuario sería absurdo. El fichero se guarda con punto, que es lo
///   que entiende el JSON.
/// - **Un número mal escrito es un error, y dice qué campo**: no se traga en silencio. Guardar un
///   cero porque el usuario escribió mal sería el peor de los finales: un dato falso, con buena cara.
pub fn numero<T: FromStr>(campo: &str, texto: &str) -> Result<Dato<T>, String> {
    let limpio = texto.trim();
    if limpio.is_empty() {
        return Ok(Dato::Falta);
    }
    limpio
        .replace(',', ".")
        .parse::<T>()
        .map(Dato::Hay)
        .map_err(|_| format!("«{limpio}» no se entiende como número, en «{campo}»"))
}

/// El sexo, en texto. Se aceptan las mismas formas que el lector del JSON, para que lo que se
/// escribe y lo que se lee sean lo mismo. **Lo que no se entiende es un error**, no un «Falta»
/// silencioso: si el usuario escribe «mujer», tiene que saber que no se ha guardado.
pub fn sexo(texto: &str) -> Result<Dato<Sexo>, String> {
    match texto.trim().to_lowercase().as_str() {
        "" => Ok(Dato::Falta),
        "femenino" | "f" => Ok(Dato::Hay(Sexo::Femenino)),
        "masculino" | "m" => Ok(Dato::Hay(Sexo::Masculino)),
        otro => Err(format!("«{otro}» no es un sexo: es «femenino» o «masculino»")),
    }
}

/// **Los nombres de los campos, y en un solo sitio.** Los usan la línea de órdenes
/// (`--guardar-perfil edad=60`) y la pantalla, y por eso están aquí: si cada uno tuviera su lista,
/// un día dirían cosas distintas.
pub const CAMPOS: [&str; 10] = [
    "edad",
    "pulso_reposo",
    "pulso_maximo_medido",
    "peso_kg",
    "altura_cm",
    "grasa_corporal_pct",
    "masa_muscular_kg",
    "umbral_lactato",
    "vo2_maximo",
    "sexo",
];

/// Aplica unos campos de texto **sobre una persona que ya existe**.
///
/// Es lo que necesita `--guardar-perfil`: lo que se escribe es **el perfil que ya había más lo que
/// se indique**, para que probar el guardado no borre lo que el usuario ya tenía.
pub fn aplicar(p: &mut Persona, campos: &[(&str, &str)]) -> Result<(), String> {
    for (nombre, texto) in campos {
        match *nombre {
            "edad" => p.edad = numero("edad", texto)?,
            "pulso_reposo" => p.pulso_reposo = numero("pulso_reposo", texto)?,
            "pulso_maximo_medido" => {
                p.pulso_maximo_medido = numero("pulso_maximo_medido", texto)?
            }
            "peso_kg" => p.peso_kg = numero("peso_kg", texto)?,
            "altura_cm" => p.altura_cm = numero("altura_cm", texto)?,
            "grasa_corporal_pct" => p.grasa_corporal_pct = numero("grasa_corporal_pct", texto)?,
            "masa_muscular_kg" => p.masa_muscular_kg = numero("masa_muscular_kg", texto)?,
            "umbral_lactato" => p.umbral_lactato = numero("umbral_lactato", texto)?,
            "vo2_maximo" => p.vo2_maximo = numero("vo2_maximo", texto)?,
            "sexo" => p.sexo = sexo(texto)?,
            otro => {
                return Err(format!(
                    "«{otro}» no es un dato de la persona; los que hay son: {}",
                    CAMPOS.join(", ")
                ))
            }
        }
    }
    Ok(())
}

/// Una persona **a partir de los campos de texto**, todos a la vez. Es el camino de la pantalla:
/// la interfaz entrega los diez campos y aquí se traducen.
pub fn de_campos(campos: &[(&str, &str)]) -> Result<Persona, String> {
    let mut p = Persona::desconocida();
    aplicar(&mut p, campos)?;
    Ok(p)
}
