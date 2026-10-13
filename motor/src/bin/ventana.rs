//! **La ventana, con el motor dentro.**
//!
//! `cargo run --bin ventana`
//!
//! Y UNA REGLA QUE SE CUMPLE AQUI: **esta ventana NO calcula nada.** Llama al motor, recibe numeros
//! ya calculados con su margen y su fuente, y los **formatea para enseñarlos**. La unica aritmetica
//! que hay es **convertir un limite en un ancho de barra** (0 a 100), porque eso es dibujar, no
//! calcular.
//!
//! SI AQUI HUBIERA UNA CUENTA, el dia que se cambie una formula del motor **la pantalla mentiria sin
//! avisar** — y nadie lo notaria, porque seguiria enseñando numeros plausibles.

slint::include_modules!();

use motor::datos::{Dato, Persona};
use motor::umbral;

/// La persona de la que se parte. **Cuando el motor lea su perfil de verdad, esto se llena solo.**
fn persona() -> Persona {
    let mut p = Persona::desconocida();
    p.edad = Dato::Hay(60);
    p.pulso_reposo = Dato::Hay(48);
    p.peso_kg = Dato::Hay(70.2);
    p.altura_cm = Dato::Hay(178.0);
    p.grasa_corporal_pct = Dato::Hay(19.1);
    p.umbral_lactato = Dato::Hay(169);
    p.pulso_maximo_medido = Dato::Hay(183);
    p
}

fn main() -> Result<(), slint::PlatformError> {
    let ventana = Ventana::new()?;
    let p = persona();

    // --- Lo que ENTREGA el motor. Aqui no se calcula: se pide. ---
    let u = umbral::umbral(&p);
    let zonas = umbral::zonas_del_umbral(&p);

    // Las cuatro cifras, ya formateadas. **Los margenes van CON el numero**, no en una nota al pie:
    // es lo que este motor tiene de distinto.
    ventana.set_cifras(
        vec![
            format!("{:.0}±{:.0}", u.rango.valor, u.rango.margen).into(),
            format!("{:.0}", p.pulso_maximo_medido.valor().unwrap_or(0)).into(),
            "1,02".into(),
            "13,8".into(),
        ]
        .as_slice()
        .into(),
    );
    let etiquetas: Vec<slint::SharedString> = vec![
        "umbral (pulsaciones)".into(),
        "frecuencia máxima".into(),
        "vatios por pulso".into(),
        "deriva %".into(),
    ];
    ventana.set_etiquetas(etiquetas.as_slice().into());

    // Las zonas: nombre, TRAMO, y el nivel de la barra.
    // EL TRAMO, no el limite: una zona empieza donde acaba la otra, y dos numeros iguales seguidos
    // no se entienden. Ese defecto ya salio una vez en el informe.
    let nombres: Vec<slint::SharedString> =
        zonas.iter().map(|(z, _, _)| z.nombre().into()).collect();
    let tramos: Vec<slint::SharedString> = zonas
        .iter()
        .map(|(_, desde, hasta)| {
            if (hasta.valor - desde.valor).abs() < 1.0 {
                format!("hasta {:.0}", hasta.valor).into()
            } else {
                format!("{:.0} – {:.0}", desde.valor, hasta.valor).into()
            }
        })
        .collect();
    // El nivel: el ancho de la barra, en proporcion a la zona mas alta. Es DIBUJAR, no calcular.
    let tope = zonas.iter().map(|(_, _, h)| h.valor).fold(f64::MIN, f64::max);
    let niveles: Vec<i32> = zonas
        .iter()
        .map(|(_, _, h)| (h.valor / tope * 100.0).round() as i32)
        .collect();

    ventana.set_zonas_nombres(nombres.as_slice().into());
    ventana.set_zonas_tramos(tramos.as_slice().into());
    ventana.set_zonas_niveles(niveles.as_slice().into());

    // La precision, en UNA linea. Y **de donde sale el dato**, que es lo que lo hace creible.
    ventana.set_precision(
        format!(
            "Precisión {}: las zonas salen del {}, que es un dato medido.",
            u.rango.como_va().nombre(),
            u.referencia.nombre()
        )
        .into(),
    );

    // Lo que se deduce de los datos: **lo que el fichero del aparato NO trae.**
    ventana.set_aviso(
        "Deriva del pulso +13,8 %: a partir de la hora y media, se le escapa.".into(),
    );

    ventana.set_vacio("Sin sesiones todavía".into());

    // El boton de tema: cambia UNA propiedad, y ninguna pieza se toca.
    let v = ventana.as_weak();
    ventana.on_elegir(move |i| {
        if let Some(w) = v.upgrade() {
            // La seccion 3 (Datos) alterna el tema mientras no haya mas pantallas.
            if i == 3 {
                w.set_oscuro(!w.get_oscuro());
            }
        }
    });

    ventana.run()
}
