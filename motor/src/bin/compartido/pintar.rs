//! **EL UNICO SITIO donde lo que el motor decidio se convierte en la ventana** (Fase 2).
//!
//! POR QUE EXISTE ESTE FICHERO, y es una leccion registrada: habia **DOS `fn pintar`**, uno en
//! `pulso.rs` y otro en `capturar-ui.rs`, con el mismo cuerpo copiado. Eso significa que **la captura
//! podia mentir**: el dia que la aplicacion pintara una cosa y la captura otra, nadie lo veria, y la
//! captura es justo lo que se mira para juzgar el diseno. Aqui hay **un solo camino**, y lo usan los
//! tres: la aplicacion (`pulso`), la captura (`capturar-ui`) y **la prueba de geometria**
//! (`tests/medir-interfaz.rs`), que asi mide **lo que la aplicacion pinta de verdad** en vez de unos
//! valores de prueba escritos a mano que un dia divergen sin que nadie se entere.
//!
//! Y AQUI NO SE DECIDE NADA: el que decide es `cara::lo_que_ensena`, que es una funcion pura. Esto
//! solo **traduce**: de `Ensenado` a las propiedades del `.slint`. Ni una cuenta de significado: la
//! unica aritmetica es escribir numeros con su unidad, que es lo que el contrato manda a la interfaz
//! («el redondeo y la coma decimal son de la interfaz», `docs/CONTRATO-cara-datos.md:15-16`).
//!
//! COMO SE USA: no es un binario (vive en un subdirectorio de `src/bin`, y Cargo no lo mira), se
//! incluye con `#[path = ".../compartido/pintar.rs"] mod pintar;` desde cada uno de los tres sitios.

// `Ventana` y las estructuras las genera Slint con `include_modules!()` en la raiz de CADA crate que
// incluye este modulo (los dos binarios y la prueba), y por eso se piden a `crate` y no a un `use`.
use crate::{BandaZona, TarjetaCifra, Ventana};
use motor::cara::Ensenado;

/// **EL TRAMO de una zona, CON SU UNIDAD.**
///
/// EL DEFECTO QUE ARREGLA, y es el 4 de la auditoria: hoy la pantalla enseñaba «hasta 114» y
/// «114 – 127», **mezclando dos notaciones y sin decir la unidad**. Un numero de pulsaciones sin
/// «ppm» al lado no dice de que habla (`doc:299`).
///
/// Y la regla de las dos notaciones se conserva, porque es cierta: una zona cuyo inicio y su fin
/// **coinciden** no es un tramo, es un TECHO, y se dice «hasta 137 ppm» (la recuperacion es asi).
fn tramo_ppm(inicio: f64, fin: f64) -> String {
    if (fin - inicio).abs() < 1.0 {
        format!("hasta {fin:.0} ppm")
    } else {
        format!("{inicio:.0} – {fin:.0} ppm")
    }
}

/// Un numero de pulsaciones, sin decimales: el eje y sus extremos se leen en pulsaciones enteras.
fn ppm(v: f64) -> String {
    format!("{v:.0}")
}

/// **Escribe en la ventana lo que hay que enseñar.** No decide nada: traduce y escribe.
pub fn pintar(v: &Ventana, e: &Ensenado) {
    // ---- §3.5 · LAS TARJETAS, Y SOLO LAS QUE EXISTEN ----
    //
    // `dato: None` significa «esta cifra no existe» (contrato 2), y es **lo unico que autoriza a la
    // vista a sacar la tarjeta del arbol** (AU-91). Por eso lo que se manda es la lista FILTRADA: la
    // cifra que no existe **no llega a la ventana**, y asi no puede quedar un marco vacio ni un «—»
    // de 26 px, que es el defecto 5 de la auditoria y esta prohibido.
    //
    // Y el filtro se hace AQUI, en el unico camino de pintado, y no en cada consumidor: con dos
    // sitios donde filtrar, uno de los dos se olvidaria.
    let tarjetas: Vec<TarjetaCifra> = e
        .cifras_atadas
        .iter()
        .filter_map(|c| {
            let numero = c.dato.as_ref()?; // `None` = la cifra no existe: la tarjeta no entra
            Some(TarjetaCifra {
                numero: numero.as_str().into(),
                etiqueta: c.etiqueta.as_str().into(),
            })
        })
        .collect();
    v.set_tarjetas(tarjetas.as_slice().into());

    // ---- §3.6 · LAS BANDAS Y EL EJE, EN PULSACIONES ----
    //
    // Las bandas viajan **en ppm, sin formatear** (contrato 1): es lo que permite pintarlas EN SU
    // SITIO del eje. El nivel 0-100 que habia antes (el techo de la zona dividido por el techo mayor)
    // **no se manda**: no representaba nada y esta prohibido (`doc:298`).
    //
    // El TEXTO del tramo se monta aqui, de los MISMOS numeros que pintan la banda: asi el texto y el
    // dibujo no pueden decir cosas distintas (AU-73).
    let bandas: Vec<BandaZona> = e
        .zonas_ppm
        .iter()
        .map(|z| BandaZona {
            nombre: z.nombre.as_str().into(),
            tramo: tramo_ppm(z.inicio_ppm, z.fin_ppm).into(),
            inicio_ppm: z.inicio_ppm as f32,
            fin_ppm: z.fin_ppm as f32,
        })
        .collect();
    v.set_bandas(bandas.as_slice().into());

    // EL EJE: los dos numeros (para situar cada banda) y los dos textos (para que se lea la escala).
    // Los dos salen del MISMO campo `eje` en esta misma funcion: no son dos fuentes, son dos formas
    // de enseñar el mismo dato.
    v.set_eje_inicio_ppm(e.eje.inicio_ppm as f32);
    v.set_eje_fin_ppm(e.eje.fin_ppm as f32);
    v.set_eje_inicio_texto(ppm(e.eje.inicio_ppm).into());
    v.set_eje_fin_texto(ppm(e.eje.fin_ppm).into());
    // SIN ZONAS NO HAY GRAFICO, y sin un eje valido tampoco: `Eje::es_valido()` es del motor, y la
    // vista NO lo recalcula (contrato 3). El `length > 0` se exige ademas porque es lo que impide que
    // una banda se pinte con un eje de ceros.
    v.set_eje_valido(e.eje.es_valido() && !e.zonas_ppm.is_empty());

    // ---- §3.7 · LAS DOS FRASES DE UNA LINEA ----
    //
    // `precision_corta` es lo que va a la pantalla (el parrafo entero, `precision`, se queda en el
    // motor para la AYUDA); `que_falta_corto` es el vacio dicho en UNA frase, y **vacio si no falta
    // nada** (contrato 5). La vista dibuja el bloque solo si el texto no esta vacio (AU-07/AU-24).
    v.set_precision_corta(e.precision_corta.as_str().into());
    v.set_que_falta_corto(e.que_falta_corto.as_str().into());

    // ---- El estado de la pantalla ----
    // ¿Hay algo que diagnosticar? Si no, la vista NO dibuja tarjetas ni grafico (nada de marcos
    // vacios), y en su lugar enseña la linea de lo que falta.
    v.set_puede(e.puede_diagnosticar);
    v.set_aviso(e.aviso.as_str().into());
    v.set_vacio(e.vacio.as_str().into());
    v.set_periodo(e.periodo.as_str().into());
}
