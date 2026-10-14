// LOS AVISOS DEL ENCARGO, CON SUS DATOS REALES — y los que NO salen, con el motivo.
//
// POR QUE ESTE PROGRAMA SE QUEDA, y no es un programa de un solo uso: la mitad de lo que hay que
// contar de este encargo es **lo que NO se puede avisar y por que**. Un aviso que no sale porque falta
// un dato **es una respuesta**, y esa respuesta no cabe en una prueba: hay que poder leerla entera,
// con sus numeros, cada vez que se cambien los datos.
//
// USO:
//   cargo run -q --release --bin avisos -- ../datos/garmin_files/storage
//   cargo run -q --release --bin avisos -- ../datos/garmin_files/storage 53.8   <- con el agua de hoy
//
// EL AGUA CORPORAL ES UN ARGUMENTO Y NO UN DATO DEL FICHERO, y eso es lo correcto: viene de su bascula
// de Huawei y **la mete el a mano**. Si no se pasa, el aviso del agua NO SALE y el programa **dice que
// falta** — que es exactamente lo que tiene que hacer la aplicacion cuando no ha metido el dato.
use motor::analisis::{self, Real};
use motor::avisos::{self, Historico, Hoy};
use motor::datos::Persona;

fn main() {
    let carpeta = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../datos/garmin_files/storage".into());

    // El segundo argumento, si viene, es el agua corporal de hoy. Se admite la coma decimal porque
    // quien lo va a escribir es una persona en espanol, y "53,8" es como lo escribe.
    let hoy = match std::env::args().nth(2) {
        Some(t) => match t.replace(',', ".").parse::<f32>() {
            Ok(v) => {
                if (0.0..=100.0).contains(&v) {
                    Hoy::con_agua(v)
                } else {
                    eprintln!("El agua corporal tiene que ir entre 0 y 100 (llegó «{t}»). Se sigue SIN ese dato.");
                    Hoy::sin_datos()
                }
            }
            Err(_) => {
                eprintln!("No se entiende «{t}» como un porcentaje de agua corporal. Se sigue SIN ese dato.");
                Hoy::sin_datos()
            }
        },
        None => Hoy::sin_datos(),
    };

    // La persona: del perfil si existe, y de la nada si no. **No se inventa un perfil**: si el fichero
    // no esta, se dice, y el motor arranca con `Persona::desconocida()`, que es lo honesto.
    let (ruta, _) = motor::profile::ruta_del_perfil();
    let persona = match motor::profile::leer(&ruta) {
        Ok(p) => {
            println!("(perfil leído de {})", ruta.display());
            p
        }
        Err(e) => {
            println!("(sin perfil en {}: {e})", ruta.display());
            println!("(estas tres reglas NO usan la edad ni el peso: los umbrales son suyos y la línea base es suya)");
            Persona::desconocida()
        }
    };

    // El `Real` es lo que el resto del motor ya sabe de la carpeta. Se calcula **una vez** y se pasa:
    // recalcularlo por aviso serian tres lecturas de 102 ficheros.
    eprintln!("Leyendo {} ...", carpeta);
    let real: Real = analisis::de_carpeta(std::path::Path::new(&carpeta));
    let historico: Historico = avisos::de_carpeta(std::path::Path::new(&carpeta));

    print!("{}", avisos::informe(&persona, &real, &historico, &hoy));

    // Y una linea de cierre con lo que el resto del motor aporta, para que se vea que las dos lecturas
    // de la carpeta **cuadran**: si `analisis` y `avisos` contaran sesiones distintas, una de las dos
    // estaria midiendo otra cosa, y eso hay que verlo aqui y no dentro de tres semanas.
    println!("── CONTROL CRUZADO DE LAS DOS LECTURAS ──");
    println!("  analisis::de_carpeta dice ....... {} sesiones", real.sesiones);
    println!("  avisos::de_carpeta dice ......... {} sesiones", historico.total_sesiones);
    match real.deriva_media {
        Some(d) => println!("  deriva media (analisis) ......... {} %", format!("{d:.2}").replace('.', ",")),
        None => println!("  deriva media (analisis) ......... no se pudo calcular"),
    }
    match historico.base_de_deriva() {
        Some(d) => println!("  deriva mediana (avisos) ......... {} %   <- la linea base del modulo C", format!("{d:.2}").replace('.', ",")),
        None => println!("  deriva mediana (avisos) ......... SIN LINEA BASE"),
    }
    println!(
        "  (la media y la mediana NO tienen por que coincidir: la media la mueven las colas, y este\n   cliente tiene una deriva de -23,6 % y otra de +34,7 %)",
    );
    if !historico.no_leidos.is_empty() {
        println!("\n  FICHEROS QUE NO SE LEYERON: {}", historico.no_leidos.len());
        for (n, m) in historico.no_leidos.iter().take(10) {
            println!("    · {n}: {m}");
        }
    }
}
