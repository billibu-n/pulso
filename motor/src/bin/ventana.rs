//! **La primera ventana del motor.**
//!
//! `cargo run --bin ventana`
//!
//! QUÉ ES, y qué no es: **es el motor enseñado, no la aplicación**. La ventana no calcula nada — recibe
//! los números que el motor produce y los coloca. La regla del proyecto es que **el motor piensa y la
//! interfaz enseña**, y si aquí hubiera una sola cuenta, el día que se cambie una fórmula **la pantalla
//! mentiría sin avisar**.
//!
//! LO QUE ENSEÑA, y es lo que este motor tiene de distinto: no solo **el número**, sino **su margen**,
//! **de dónde sale**, y **qué haría falta para afinarlo**. Los cuatro botones cambian **los datos**,
//! no el cálculo — y el margen se ensancha o se estrecha **a la vista**.

slint::include_modules!();

use motor::datos::{Persona, Sexo};
use motor::formulas_v2;
use motor::hay;

/// Los cuatro escalones de datos, de menos a más. **El motor responde en los cuatro.**
fn personas() -> [(Persona, &'static str); 4] {
    let sin_nada = Persona::desconocida();

    let mut con_edad = Persona::desconocida();
    con_edad.edad = hay(60u8);

    let mut con_composicion = con_edad.clone();
    con_composicion.peso_kg = hay(70.2f32);
    con_composicion.grasa_corporal_pct = hay(19.1f32);
    con_composicion.altura_cm = hay(178.0f32);
    con_composicion.sexo = hay(Sexo::Masculino);

    let mut con_medida = con_composicion.clone();
    con_medida.pulso_maximo_medido = hay(179u16);
    con_medida.pulso_reposo = hay(48u16);

    [
        (sin_nada, "Sin ningún dato: el motor parte de una persona media y lo dice"),
        (con_edad, "Con la edad: la estimación más común, y la de margen más ancho"),
        (con_composicion, "Con la edad y la composición corporal de una báscula"),
        (con_medida, "Con la frecuencia máxima medida en una prueba de esfuerzo"),
    ]
}

fn main() -> Result<(), slint::PlatformError> {
    let ventana = Ventana::new()?;
    let casos = personas();

    let pintar = {
        let ventana = ventana.as_weak();
        move |i: usize| {
            let ventana = ventana.unwrap();
            let (p, subtitulo) = &casos[i];
            let maxima = formulas_v2::pulso_maximo(p);
            let zonas_motor = formulas_v2::zonas(p);

            ventana.set_titulo("Pulso — análisis de entrenamiento".into());
            ventana.set_subtitulo((*subtitulo).into());
            ventana.set_maxima(
                format!("{:.0} ± {:.0} ppm", maxima.rango.valor, maxima.rango.margen).into(),
            );
            ventana.set_origen(format!("De dónde sale: {}", maxima.origen.nombre()).into());
            ventana.set_precision(maxima.rango.como_va().nombre().into());
            ventana.set_significado(maxima.rango.como_va().explicacion().into());
            ventana.set_datos(
                format!("datos que tiene: {} de 7", p.cuantos_datos()).into(),
            );
            ventana.set_falta(
                maxima
                    .origen
                    .que_falta()
                    .map(|f| {
                        let mut s = f.to_string();
                        if let Some(c) = s.get_mut(0..1) {
                            c.make_ascii_uppercase();
                        }
                        s
                    })
                    .unwrap_or_else(|| {
                        "Nada: el número está medido, y su margen es el de la propia medición."
                            .to_string()
                    })
                    .into(),
            );

            // Las zonas, traducidas a lo que la ventana sabe pintar.
            // El ancho de la barra es PROPORCIONAL al límite: si sube, la barra crece.
            let maximo = zonas_motor
                .iter()
                .map(|(_, r)| r.valor)
                .fold(f64::MIN, f64::max);
            let zonas: Vec<Zona> = zonas_motor
                .iter()
                .map(|(z, r)| Zona {
                    nombre: z.nombre().into(),
                    limite: format!("{:.0}", r.valor).into(),
                    margen: format!("{:.0}", r.margen).into(),
                    desde: format!("{:.0}", r.minimo()).into(),
                    hasta: format!("{:.0}", r.maximo()).into(),
                    // El nivel (0-100) es para la barra: proporcional al limite.
                    nivel: ((r.valor / maximo) * 100.0).round() as i32,
                })
                .collect();
            ventana.set_zonas(zonas.as_slice().into());
            ventana.set_elegido(i as i32);
        }
    };

    pintar(2); // Se abre con el caso del cliente: edad y composición, sin prueba de esfuerzo.

    let v = ventana.as_weak();
    ventana.on_cambiar(move |i| {
        let i = i.clamp(0, 3) as usize;
        if let Some(w) = v.upgrade() {
            w.set_elegido(i as i32);
        }
        pintar(i);
    });

    ventana.run()
}
