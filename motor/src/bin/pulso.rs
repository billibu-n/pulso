//! **La ventana, con el motor dentro.**
//!
//! `cargo run --bin pulso`
//!
//! Y UNA REGLA QUE SE CUMPLE AQUI, y es la mas importante de este fichero: **aqui NO se calcula nada,
//! y aqui NO se decide que se enseña.** Las dos cosas las hace `cara::lo_que_ensena`, que es una funcion
//! pura. Este fichero **une**: lee los argumentos, carga los datos, pide lo que hay que enseñar, y lo
//! escribe en la ventana. Nada mas.
//!
//! POR QUE SE SEPARO, y costo un rato darse cuenta: antes esto calculaba dentro de `main`, y tenia dos
//! consecuencias malas. La primera, que **lo que la pantalla enseña no se podia comprobar sin abrir una
//! ventana** -- y abrir ventanas en el equipo del dueño esta prohibido por un motivo que ya costo caro.
//! La segunda, que **la memoria entre arranques no podia guardar el resultado**, porque no habia forma
//! de nombrarlo desde fuera. Las dos se arreglaron moviendolo a `cara.rs`.
//!
//! Y LAS TRES PIEZAS QUE ESTE FICHERO CONECTA, que estaban construidas y sin conectar:
//!
//! | pieza | que hace | medido |
//! |---|---|---|
//! | `profile` | lee y guarda los datos de la persona, **junto al programa** | -- |
//! | `memoria` | recuerda el analisis entre arranques | **5,40 s -> 0,001 s** |
//! | `cara` | saca lo que va a la pantalla, sin abrir ventana | 11 pruebas del recorrido |

slint::include_modules!();

use motor::analisis::{self, Real};
use motor::cara::{self, Ensenado};
use motor::datos::{Dato, Persona, Sexo};
use motor::avisos::Hoy;
use motor::{carpeta, memoria, profile};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Donde se guarda la memoria del analisis. **Junto al perfil**, para que las dos cosas viajen juntas.
///
/// POR QUE AL LADO DEL PERFIL Y NO EN OTRO SITIO: si el usuario copia su carpeta a otro equipo, tienen
/// que irse las dos. Una memoria que se queda atras **no es un problema** (se recalcula), pero un
/// perfil que se queda atras **si**: sus datos son suyos y no se regeneran solos.
fn ruta_de_la_memoria(junto_al_perfil: &Path) -> PathBuf {
    junto_al_perfil.with_file_name("analisis.json")
}

/// El texto de un `Dato` ENTERO para un campo de la pantalla.
///
/// **`Falta` se escribe VACIO, nunca como cero.** Es la regla que sostiene la honestidad del motor: «no
/// lo se» y «vale cero» son cosas distintas, y un campo con un 0 dentro dice lo segundo. El dia que el
/// usuario pulse guardar sin tocar ese campo, tiene que volver a quedar en `Falta`.
///
/// Y HAY DOS FUNCIONES, UNA PARA ENTEROS Y OTRA PARA DECIMALES, por un motivo que no se ve: formatear
/// un entero con decimales (`format!("{:.1}", 60u8)`) **entra en panico en tiempo de ejecucion**. Es un
/// error que el compilador no puede ver porque el formato es una cadena, asi que se separa por tipo y se
/// quita el riesgo de raiz.
fn texto_entero<T: std::fmt::Display + Copy>(d: Dato<T>) -> String {
    match d.valor() {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

/// Y lo mismo para los decimales, **con COMA**: el informe esta en español, y un «70.2» se lee como
/// otra cosa. La coma se pone aqui, en la vista, y no en el motor.
fn texto_decimal<T: std::fmt::Display + Copy>(d: Dato<T>, decimales: usize) -> String {
    match d.valor() {
        Some(v) => format!("{v:.decimales$}").replace('.', ","),
        None => String::new(),
    }
}

/// Los datos de la persona, tal como van a los campos de la pantalla.
fn campos_de(p: &Persona) -> DatosPersona {
    DatosPersona {
        edad: texto_entero(p.edad).into(),
        pulso_reposo: texto_entero(p.pulso_reposo).into(),
        pulso_maximo_medido: texto_entero(p.pulso_maximo_medido).into(),
        peso_kg: texto_decimal(p.peso_kg, 1).into(),
        altura_cm: texto_decimal(p.altura_cm, 1).into(),
        grasa_corporal_pct: texto_decimal(p.grasa_corporal_pct, 1).into(),
        masa_muscular_kg: texto_decimal(p.masa_muscular_kg, 1).into(),
        umbral_lactato: texto_entero(p.umbral_lactato).into(),
        vo2_maximo: texto_decimal(p.vo2_maximo, 1).into(),
        // 0 femenino, 1 masculino, 2 «no lo sé». **El 2 no es un adorno**: un desplegable no se puede
        // desmarcar, y sin esa opcion el que se equivoca al elegir no puede volver a decir que no lo
        // sabe. O sea: el «no lo sé» dejaria de existir por un accidente de la interfaz.
        sexo: match p.sexo.valor() {
            Some(Sexo::Femenino) => 0,
            Some(Sexo::Masculino) => 1,
            None => 2,
        },
        // (`sexo` es un INDICE, no un texto: 0 femenino, 1 masculino, 2 no lo sé)
    }
}

/// Los campos de la pantalla, tal como vuelven para guardarse.
///
/// **El orden es el que espera `profile::aplicar`**, y por eso se construye con la constante
/// `profile::CAMPOS` en vez de escribir los nombres otra vez: si un dia se añade un campo, **se añade en
/// un sitio y no en dos**. Es la misma regla que evito que el proyecto tuviera dos listas de tareas.
fn lista_de_campos(c: &DatosPersona) -> Vec<(&'static str, String)> {
    let valores = [
        c.edad.clone(),
        c.pulso_reposo.clone(),
        c.pulso_maximo_medido.clone(),
        c.peso_kg.clone(),
        c.altura_cm.clone(),
        c.grasa_corporal_pct.clone(),
        c.masa_muscular_kg.clone(),
        c.umbral_lactato.clone(),
        c.vo2_maximo.clone(),
        match c.sexo {
            0 => "femenino".into(),
            1 => "masculino".into(),
            // Vacio, no una palabra: `profile::sexo` traduce vacio a `Falta`, que es «no lo sé».
            _ => String::new().into(),
        },
    ];
    // Devuelve `String` y NO prestados, y esto merece explicacion porque el primer intento fue al
    // reves: prestar desde `c` obliga a tocar tambien el sitio de la llamada, y alli `v.as_str()`
    // sobre un `&&str` cae en `str::as_str()`, que en este toolchain es INESTABLE (E0658).
    // El prestamo de verdad ocurre despues, al llamar a `aplicar`: ver el manejador del boton.
    profile::CAMPOS
        .iter()
        .copied()
        .zip(valores.iter().map(|s| s.to_string()))
        .collect()
}

/// El analisis de la carpeta: **lo recordado, y SOLO lo nuevo o lo cambiado**.
///
/// ESTA FUNCION ES LA QUE HACE QUE LA APLICACION ABRA AL INSTANTE. Y las cifras van MEDIDAS, con el
/// comando que las mide, porque una cifra redonda en un comentario acaba siendo una creencia:
///
/// - leer los 102 ficheros .... **5,57 s**  (`cargo run -q --release --bin medir-carga`)
/// - con la memoria .......... **0,032 s**  (el proceso entero, no solo la lectura)
///   -> y un control con la cache de disco ya caliente, 5,570 s -> 0,032 s, que prueba que lo que
///      acelera es la MEMORIA y no la cache del sistema
/// - el fichero de memoria ... **9.473 bytes**, en vez de releer los **34 MB** de FIT que hay hoy
///
/// OJO CON EL TAMANO: al principio se comparo contra «1.003 MB», y era un error mio -- esa cifra es
/// **todo** `datos/`, con la base de datos de Garmin dentro. Los 102 FIT pesan **34 MB**.
fn el_analisis(carpeta: &Path, ruta_memoria: &Path) -> Option<Real> {
    // La carpeta no existe: no se lee nada. No es un error, es «no hay datos», y la pantalla ya sabe
    // decirlo.
    if !carpeta.exists() {
        return None;
    }
    // LO RECORDADO, Y SOLO LO NUEVO. Esto es lo que hace que añadir la sesion de hoy no obligue a
    // releer los tres meses de historico: cada fichero ya leido deja su ficha, y aqui solo se leen las
    // fichas que faltan o que cambiaron de tamaño.
    let previas = memoria::leer_fichas(ruta_memoria);
    let (real, fichas, cuantas) = analisis::actualizar(carpeta, &previas);
    if real.hay_datos() {
        memoria::guardar(ruta_memoria, &fichas);
        eprintln!(
            "  {} sesiones - {} recordadas - {} leidas",
            real.sesiones, cuantas.recordadas, cuantas.leidas
        );
        Some(real)
    } else {
        None
    }
}

/// **Pinta el selector**: la lista de carpetas, el numero de FIT y el aviso.
///
/// No decide nada: carpeta::abrir ya calculo la vista, con sus pruebas. Aqui solo se traduce a las
/// propiedades que dibuja el componente SelectorCarpeta.
fn pintar_selector(ventana: &Ventana, dir: &Path) {
    let v = carpeta::abrir(dir);
    let ruta = v
        .ruta
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    ventana.set_carpeta_ruta(ruta.into());
    ventana.set_carpeta_fit(v.fit_aqui as i32);
    ventana.set_carpeta_puede_subir(v.subir_a.is_some());
    ventana.set_carpeta_aviso(v.aviso.clone().unwrap_or_default().into());
    let entradas: Vec<EntradaCarpeta> = v
        .entradas
        .iter()
        .map(|e| EntradaCarpeta {
            nombre: slint::SharedString::from(e.nombre.as_str()),
            cuantos: e.cuantos_fit as i32,
        })
        .collect();
    let modelo = slint::VecModel::from_slice(&entradas);
    ventana.set_carpetas(slint::ModelRc::new(modelo));
}

/// Escribe en la ventana lo que hay que enseñar. **No decide nada**: recibe y escribe.
fn pintar(ventana: &Ventana, e: &Ensenado) {
    let cifras: Vec<slint::SharedString> = e
        .cifras
        .iter()
        .map(|s| slint::SharedString::from(s.as_str()))
        .collect();
    let etiquetas: Vec<slint::SharedString> = e
        .etiquetas
        .iter()
        .map(|s| slint::SharedString::from(s.as_str()))
        .collect();
    ventana.set_cifras(cifras.as_slice().into());
    ventana.set_etiquetas(etiquetas.as_slice().into());
    let nombres: Vec<slint::SharedString> = e
        .zonas
        .iter()
        .map(|(n, _, _)| slint::SharedString::from(n.as_str()))
        .collect();
    let tramos: Vec<slint::SharedString> = e
        .zonas
        .iter()
        .map(|(_, t, _)| slint::SharedString::from(t.as_str()))
        .collect();
    // (`cifras` y `etiquetas` son `Vec<String>` y NO se convierten solas con `as_slice().into()`;
    //  estas dos necesitan el tipo dicho, porque el `into()` de dentro no sabe a que convertir.)
    let niveles: Vec<i32> = e.zonas.iter().map(|(_, _, n)| *n).collect();
    ventana.set_zonas_nombres(nombres.as_slice().into());
    ventana.set_zonas_tramos(tramos.as_slice().into());
    ventana.set_zonas_niveles(niveles.as_slice().into());
    ventana.set_precision(e.precision.as_str().into());
    ventana.set_periodo(e.periodo.as_str().into());
    // ¿Hay algo que diagnosticar? Si no, la ventana NO dibuja cifras ni zonas (nada de tarjetas vacias).
    ventana.set_puede(e.puede_diagnosticar);
    // El aviso: si no se puede diagnosticar, **lo que falta, y donde se rellena**.
    let aviso = if e.puede_diagnosticar { &e.aviso } else { &e.que_falta };
    ventana.set_aviso(aviso.as_str().into());
    ventana.set_vacio(e.vacio.as_str().into());
}

/// **¿El usuario pide que se le explique?** `--ayuda`, `-h` o `--help`, y nada mas.
///
/// Es una funcion PURA sobre una lista de argumentos, y a proposito: una ayuda que no se puede
/// comprobar es una ayuda que un dia deja de salir — y entonces `--ayuda` vuelve a abrir una ventana
/// en la pantalla de quien lo escribio.
fn pide_ayuda(argumentos: &[String]) -> bool {
    argumentos
        .iter()
        .any(|a| a == "--ayuda" || a == "-h" || a == "--help")
}

/// **La carpeta que se quiere mirar con el selector**, si se pidio el modo. La ruta va DESPUES de
/// --elegir; sin ruta, se empieza por donde se lanzo el programa. No abre nada: solo devuelve que hay
/// que pintar.
fn carpeta_a_elegir(argumentos: &[String]) -> Option<PathBuf> {
    let i = argumentos.iter().position(|a| a == "--elegir")?;
    Some(
        argumentos
            .get(i + 1)
            .filter(|a| !a.starts_with("--"))
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    )
}

/// **La exportacion de Huawei** (`--huawei <ruta>`), si se pidio. El ZIP tal cual sale del correo, o
/// una carpeta ya descomprimida: lo decide `huawei::leer`. Sin valor no hay ruta, y una opcion no
/// puede valer otra opcion.
fn ruta_huawei(argumentos: &[String]) -> Option<PathBuf> {
    let i = argumentos.iter().position(|a| a == "--huawei")?;
    argumentos
        .get(i + 1)
        .filter(|a| !a.starts_with("--"))
        .map(PathBuf::from)
}

/// **El selector, impreso.** Es la misma vista que dibujara el panel, para poder comprobarla hoy.
fn imprimir_selector(dir: &Path) {
    let v = carpeta::abrir(dir);
    println!("--- EL SELECTOR DE CARPETA (sin abrir ventana) ---");
    match &v.ruta {
        Some(r) => println!("  estas en ... {}", r.display()),
        None => println!("  estas en ... (no se pudo leer)"),
    }
    if let Some(p) = &v.subir_a {
        println!("  subir a .... {}", p.display());
    }
    println!("  FIT aqui ... {}", v.fit_aqui);
    if let Some(a) = &v.aviso {
        println!("  aviso ...... {a}");
    }
    for e in &v.entradas {
        println!("  {:<28} {} FIT", e.nombre, e.cuantos_fit);
    }
    println!("  ==> las MISMAS carpetas y los MISMOS numeros que dibujara el panel.");
}

/// **Lo que el programa hace y como se usa**, SIN abrir nada.
///
/// El nombre del programa sale de `CARGO_BIN_NAME` y no escrito a mano: el dia que la puerta de
/// entrada se llame de otra forma, **la ayuda se renombra sola** y no queda mintiendo.
fn imprimir_ayuda() {
    let programa = env!("CARGO_BIN_NAME");
    println!("{programa}: analiza tu entrenamiento y dice que se puede afirmar, y que no.");
    println!();
    println!("USO:");
    println!("  {programa} [CARPETA] [--huawei RUTA] [--texto] [--elegir [CARPETA]]");
    println!();
    println!("  CARPETA    la carpeta con tus ficheros (FIT de Garmin). Si no se indica, se usa la");
    println!("             de por defecto.");
    println!("  --huawei   tu exportacion de Huawei (el ZIP del correo, o la carpeta ya descomprimida):");
    println!("             de ahi se lee el agua corporal CON SU FECHA, y no la teclea nadie.");
    println!("  --texto    imprime lo MISMO que va a la pantalla y NO abre ninguna ventana.");
    println!("  --elegir   el SELECTOR de carpeta, impreso y SIN abrir ventana: enseña las");
    println!("             subcarpetas y cuantos FIT tiene cada una. Es la intuicion.");
    println!("  --ayuda    esto.");
    println!();
    println!("SIN TUS DATOS (edad, peso, umbral) NO SE ENSEÑA NINGUN NUMERO: lo dice y para.");
    println!("Tus datos se guardan junto al programa, en un fichero que viaja con la aplicacion.");
}

fn main() -> Result<(), slint::PlatformError> {
    // ---- LA AYUDA, LO PRIMERO DE TODO ----
    //
    // POR QUE VA AQUI Y NO AL FINAL: `--ayuda` abria una ventana, y es un defecto que cazo el dueño.
    // Un modo que solo EXPLICA no puede tener un efecto que se vea, y menos en la pantalla de otro.
    // Se comprueba ANTES de crear la interfaz: si se pide ayuda, no se construye nada y no se toca
    // ningun dato.
    let argumentos: Vec<String> = std::env::args().collect();
    if pide_ayuda(&argumentos) {
        imprimir_ayuda();
        return Ok(());
    }
    // EL SELECTOR, EN TEXTO, SIN ABRIR VENTANA. Es la misma vista que dibujara el panel (carpeta::abrir),
    // asi que lo que se comprueba aqui es exactamente lo que el cliente vera. Un modo que SOLO MIRA no
    // puede tener un efecto que se vea.
    if let Some(dir) = carpeta_a_elegir(&argumentos) {
        imprimir_selector(&dir);
        return Ok(());
    }
    let ventana = Ventana::new()?;

    // ---- EL PERFIL: SUS DATOS, Y DE DONDE SE LEYERON ----
    let (ruta_perfil, junto_al_programa) = profile::ruta_del_perfil();
    let persona = profile::leer(&ruta_perfil).unwrap_or_else(|e| {
        eprintln!("El perfil no se pudo leer, y se empieza de cero: {e}");
        Persona::desconocida()
    });
    ventana.set_ruta_del_perfil(ruta_perfil.display().to_string().into());
    ventana.set_campos(campos_de(&persona));
    if junto_al_programa {
        eprintln!("  sus datos se guardan en {}", ruta_perfil.display());
    } else {
        eprintln!(
            "  la carpeta del programa no se puede escribir, asi que sus datos van a {}",
            ruta_perfil.display()
        );
    }

    // ---- LA CARPETA DE DATOS ----
    // La de arranque entra por linea de ordenes, y ademas **se puede cambiar sin reiniciar** desde el
    // selector (el boton «Buscar mis ficheros»). Por eso va compartida: los manejadores la leen y la
    // cambian, y el analisis siguiente usa la nueva.
    let carpeta = PathBuf::from(
        std::env::args()
            .nth(1)
            .filter(|a| !a.starts_with("--"))
            .unwrap_or_else(|| "../datos/garmin_files/storage".to_string()),
    );
    let carpeta = Rc::new(RefCell::new(carpeta));

    // ---- EL AGUA CORPORAL, DE LA EXPORTACION DE HUAWEI (no la teclea nadie) ----
    //
    // El agua NO sale del FIT: sale de su bascula. La lee el adaptador (`huawei::leer`, que abre el ZIP
    // tal cual) y `Hoy::de_composiciones` se queda con la medicion MAS RECIENTE **y su fecha**.
    let hoy = match ruta_huawei(&argumentos) {
        Some(ruta) => match motor::huawei::leer(&ruta) {
            Ok(composiciones) => {
                let h = Hoy::de_composiciones(&composiciones);
                match (h.agua_corporal_pct.valor(), h.fecha_agua.valor()) {
                    (Some(a), Some(f)) => eprintln!(
                        "  agua corporal {} % (medida el {})",
                        texto_decimal(Dato::Hay(a), 1),
                        f.a_texto()
                    ),
                    (Some(a), None) => eprintln!(
                        "  agua corporal {} % (sin fecha en la exportacion)",
                        texto_decimal(Dato::Hay(a), 1)
                    ),
                    _ => eprintln!("  la exportacion de Huawei no traia agua corporal"),
                }
                h
            }
            Err(e) => {
                eprintln!(
                    "  no se pudo leer la exportacion de Huawei ({}): {e}. Se sigue SIN agua.",
                    ruta.display()
                );
                Hoy::sin_datos()
            }
        },
        None => Hoy::sin_datos(),
    };

    // ---- Y EL ANALISIS, CON MEMORIA ----
    let ruta_memoria = ruta_de_la_memoria(&ruta_perfil);
    let real = el_analisis(carpeta.borrow().as_path(), &ruta_memoria);

    // ---- LO QUE SE ENSEÑA: LO DECIDE `cara`, NO ESTE FICHERO ----
    let ensenado = cara::lo_que_ensena(&persona, real.as_ref());
    pintar(&ventana, &ensenado);

    // ---- Y SI SE PIDE, SE IMPRIME LO MISMO QUE VA A LA PANTALLA ----
    //
    // POR QUE EXISTE: la ventana se ve, pero **no se puede comprobar desde fuera**. Imprimir los
    // MISMOS valores que recibe permite verificar que enseña los datos de verdad.
    //
    // Y AQUI NO SE ABRE NADA. Este fallo ya se corrigio una vez, y lo cazo el dueño: `--texto`
    // imprimia los valores y DESPUES seguia hasta `run()`, asi que **abria una ventana en su
    // pantalla**. Un modo que SOLO COMPRUEBA no puede tener efectos que se vean.
    if std::env::args().any(|a| a == "--texto") {
        println!("--- LO QUE LA VENTANA RECIBE Y ENSEÑA ---");
        if !ensenado.puede_diagnosticar {
            println!("  NO SE PUEDE DIAGNOSTICAR TODAVIA");
            println!("  {}", ensenado.que_falta);
        } else {
            for i in 0..ensenado.cifras.len() {
                println!("  {:<12} {}", ensenado.cifras[i], ensenado.etiquetas[i]);
            }
            println!();
            for (n, t, nivel) in &ensenado.zonas {
                println!("  {n:<16} {t:<14} barra {nivel}%");
            }
            println!();
            println!("  {}", ensenado.precision);
            println!("  {}", ensenado.aviso);
        }
        println!("  {}", ensenado.vacio);
        println!();
        // EL PERIODO QUE CUBRE: la coherencia de las fechas, dicha sin abrir ventana. La MISMA cadena
        // que va a la pantalla (una sola fuente: si se formateara dos veces, un dia divergirian).
        if !ensenado.periodo.is_empty() {
            println!("  {}", ensenado.periodo);
        }
        // EL AGUA DEL DIA, con su fecha: el dato que antes se tecleaba y ahora sale del ZIP.
        match (hoy.agua_corporal_pct.valor(), hoy.fecha_agua.valor()) {
            (Some(a), Some(f)) => println!(
                "  agua ...... {} % (medida el {})",
                texto_decimal(Dato::Hay(a), 1),
                f.a_texto()
            ),
            (Some(a), None) => println!(
                "  agua ...... {} % (sin fecha conocida)",
                texto_decimal(Dato::Hay(a), 1)
            ),
            _ => println!("  agua ...... sin dato (se lee de la exportacion de Huawei)"),
        }
        println!("  perfil .... {}", ruta_perfil.display());
        println!("  memoria ... {}", ruta_memoria.display());
        println!("  ==> los MISMOS valores que salen en pantalla, no un resumen aparte.");
        return Ok(());
    }

    // ---- GUARDAR SUS DATOS ----
    //
    // LA INTERFAZ ENTREGA, RUST COMPRUEBA Y GUARDA. Y **despues de guardar se vuelve a calcular con los
    // datos NUEVOS, sin reiniciar**: eso es lo que hace que los resultados sean suyos.
    let v = ventana.as_weak();
    let ruta_perfil_guardar = ruta_perfil.clone();
    let ruta_memoria_guardar = ruta_memoria.clone();
    let carpeta_guardar = carpeta.clone();
    ventana.on_guardar_datos(move |campos| {
        let Some(w) = v.upgrade() else { return };

        let lista = lista_de_campos(&campos);
        // Los textos se PRESTAN: `aplicar` los lee y los traduce, no los guarda. El `Vec` de `String`
        // vive hasta que termina la llamada.
        let prestados: Vec<(&str, &str)> = lista
            .iter()
            .map(|(n, v)| (*n, v.as_str()))
            .collect();
        // `aplicar` es quien traduce texto a `Dato`, y quien dice que un numero mal escrito **no se
        // traga en silencio**. Aqui no se valida nada por segunda vez: la regla vive en un solo sitio.
        let mut nueva =
            profile::leer(&ruta_perfil_guardar).unwrap_or_else(|_| Persona::desconocida());
        match profile::aplicar(&mut nueva, &prestados) {
            Err(e) => {
                w.set_mensaje_guardado(e.as_str().into());
                w.set_error_al_guardar(true);
            }
            Ok(()) => match profile::guardar(&nueva, &ruta_perfil_guardar) {
                Err(e) => {
                    w.set_mensaje_guardado(e.as_str().into());
                    w.set_error_al_guardar(true);
                }
                Ok(()) => {
                    w.set_mensaje_guardado("Guardado.".into());
                    w.set_error_al_guardar(false);
                    // Y SE VUELVE A CALCULAR CON SUS DATOS NUEVOS.
                    let real = el_analisis(carpeta_guardar.borrow().as_path(), &ruta_memoria_guardar);
                    let e = cara::lo_que_ensena(&nueva, real.as_ref());
                    pintar(&w, &e);
                    w.set_campos(campos_de(&nueva));
                }
            },
        }
    });

    // ---- EL SELECTOR DE CARPETA ----
    // La capa superpuesta se dibuja en la ventana; aqui se le da lo que tiene que enseñar y se atiende
    // lo que el cliente pulsa. Al USAR una carpeta se vuelve a analizar SIN reiniciar: sus resultados
    // pasan a ser los de la carpeta que acaba de elegir.
    pintar_selector(&ventana, carpeta.borrow().as_path());
    let debil = ventana.as_weak();

    ventana.on_abrir_selector({
        let w = debil.clone();
        let d = carpeta.clone();
        move || {
            if let Some(v) = w.upgrade() {
                pintar_selector(&v, d.borrow().as_path());
                v.set_eligiendo(true);
            }
        }
    });
    ventana.on_cerrar_selector({
        let w = debil.clone();
        move || {
            if let Some(v) = w.upgrade() {
                v.set_eligiendo(false);
            }
        }
    });
    ventana.on_subir_carpeta({
        let w = debil.clone();
        let d = carpeta.clone();
        move || {
            if let Some(v) = w.upgrade() {
                // Se calcula el padre ANTES de tocarlo: un borrow y un borrow_mut a la vez seria un panic.
                let padre = carpeta::ruta_padre(d.borrow().as_path());
                if let Some(p) = padre {
                    *d.borrow_mut() = p;
                }
                pintar_selector(&v, d.borrow().as_path());
            }
        }
    });
    ventana.on_elegir_subcarpeta({
        let w = debil.clone();
        let d = carpeta.clone();
        move |i| {
            if let Some(v) = w.upgrade() {
                let actual = d.borrow().clone();
                if let Some(nombre) = carpeta::subcarpetas(&actual).get(i as usize).cloned() {
                    *d.borrow_mut() = actual.join(nombre);
                    pintar_selector(&v, d.borrow().as_path());
                }
            }
        }
    });
    ventana.on_usar_carpeta({
        let w = debil.clone();
        let d = carpeta.clone();
        let mem = ruta_memoria.clone();
        let perfil = ruta_perfil.clone();
        move || {
            if let Some(v) = w.upgrade() {
                v.set_eligiendo(false);
                let real = el_analisis(d.borrow().as_path(), &mem);
                let persona = profile::leer(&perfil).unwrap_or_else(|_| Persona::desconocida());
                let e = cara::lo_que_ensena(&persona, real.as_ref());
                pintar(&v, &e);
            }
        }
    });

    ventana.run()
}

#[cfg(test)]
mod pruebas {
    use super::pide_ayuda;

    #[test]
    fn la_ayuda_se_reconoce_en_sus_tres_formas() {
        for forma in ["--ayuda", "-h", "--help"] {
            assert!(pide_ayuda(&[forma.to_string()]), "no reconoce {forma}");
        }
    }

    #[test]
    fn una_carpeta_o_texto_no_son_ayuda() {
        assert!(!pide_ayuda(&[]));
        assert!(!pide_ayuda(&[
            "../datos".to_string(),
            "--texto".to_string()
        ]));
    }

    #[test]
    fn la_ruta_de_huawei_se_lee_y_no_se_confunde_con_la_siguiente_opcion() {
        use super::ruta_huawei;
        assert_eq!(ruta_huawei(&[]), None);
        assert_eq!(
            ruta_huawei(&["--huawei".to_string(), "/tmp/salud.zip".to_string()]),
            Some(std::path::PathBuf::from("/tmp/salud.zip"))
        );
        // Sin valor no hay ruta, y una opcion no puede valer otra opcion.
        assert_eq!(ruta_huawei(&["--huawei".to_string()]), None);
        assert_eq!(
            ruta_huawei(&["--huawei".to_string(), "--texto".to_string()]),
            None
        );
    }
}
