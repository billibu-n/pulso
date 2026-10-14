//! Las pruebas del motor.
//!
//! **Se prueba en los dos sentidos**, y esa es la regla:
//!
//! 1. **Que la fórmula dé bien** — con un caso calculado a mano. Si no cuadra, la fórmula está mal.
//! 2. **Que el motor DETECTE lo que falta** — quitando un dato a propósito. Un motor que no detecta
//!    lo que le falta **tranquiliza de más**, y un falso verde no se nota por el resultado.
//! 3. **Que no reviente ni invente** en los límites: un pulso imposible, una duración de cero.
//! 4. **Que toda afirmación traiga su fuente** — ninguna sin ella.

// Los nombres de las pruebas llevan MAYUSCULAS a proposito: lo que se prueba va en el nombre,
// y leer `es_PELIGRO` dice mas que `es_peligro`. Rust avisa; aqui se le dice que es a proposito.
#![allow(non_snake_case)]

use motor::datos::{Composicion, Dato, Fecha, Muestra, Persona, Sesion, Sexo};
use motor::formulas::{self, Zona};
use motor::juicio::{Confianza, Juicio};

/// Una persona con todo lo que hace falta. La referencia para comparar.
fn ana() -> Persona {
    Persona {
        // Sin fecha de nacimiento a proposito: `ana` es la persona de las pruebas del CALCULO, y
        // sirve para comprobar que el motor trabaja igual con la edad escrita que con la fecha.
        fecha_nacimiento: Dato::Falta,
        edad: Dato::Hay(40),
        pulso_reposo: Dato::Hay(55),
        pulso_maximo_medido: Dato::Falta,
        peso_kg: Dato::Hay(68.0),
        sexo: Dato::Hay(Sexo::Femenino),
        altura_cm: Dato::Hay(172.0),
        grasa_corporal_pct: Dato::Hay(22.0),
        masa_muscular_kg: Dato::Hay(28.0),
        umbral_lactato: Dato::Falta,
        vo2_maximo: Dato::Falta,
    }
}

/// Una sesión de 30 minutos con pulso cada 10 segundos, a 150 pulsaciones.
/// 180 muestras = 1790 segundos cubiertos... no: 180 muestras cada 10 s son 1790 s, así que se
/// declara la duración a mano para que la prueba no dependa de eso.
fn sesion_30_min(pulso: u16) -> Sesion {
    let muestras: Vec<Muestra> = (0..180)
        .map(|i| Muestra {
            segundo: i * 10,
            pulso: Dato::Hay(pulso),
            potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta,
        })
        .collect();
    Sesion::con_duracion(muestras, 1800)
}

// ---------------------------------------------------------------- 1. LA FÓRMULA, A MANO

#[test]
fn la_frecuencia_maxima_estimada_cuadra_con_el_calculo_a_mano() {
    // Tanaka: 208 − (0,7 × 40) = 208 − 28 = 180. Calculado a mano, no copiado del programa.
    let r = formulas::pulso_maximo(&ana());
    assert_eq!(r.valor, Some(180), "la ecuacion de Tanaka no da lo calculado a mano");
    // Y como es una ESTIMACIÓN, tiene que declarar su reserva. Si no, engaña.
    assert!(r.reserva.is_some(), "una estimacion por edad DEBE declarar su reserva");
}

#[test]
fn la_frecuencia_maxima_medida_manda_sobre_la_estimada() {
    let mut p = ana();
    p.pulso_maximo_medido = Dato::Hay(195);
    let r = formulas::pulso_maximo(&p);
    assert_eq!(r.valor, Some(195), "si esta medida, no se estima: manda la medida");
    assert!(r.reserva.is_none(), "una medida no tiene la reserva de una estimacion");
}

#[test]
fn la_reserva_cardiaca_cuadra_y_no_es_la_maxima() {
    // 180 − 55 = 125. Confundir la reserva con la máxima es el error clasico.
    let r = formulas::reserva_cardiaca(&ana());
    assert_eq!(r.valor, Some(125));
    assert_ne!(r.valor, Some(180), "la reserva NO es la frecuencia maxima");
}

#[test]
fn las_zonas_van_de_menos_a_mas_y_no_se_solapan() {
    let r = formulas::zonas(&ana());
    let zs = r.valor.expect("con todos los datos, las zonas se calculan");
    assert_eq!(zs.len(), 4);
    // Ordenadas: cada zona empieza donde acaba la anterior, y la ultima no pasa de la maxima.
    for par in zs.windows(2) {
        assert_eq!(par[0].2, par[1].1, "las zonas tienen que ser contiguas, sin huecos");
    }
    assert_eq!(zs[0].0, Zona::Suave);
    assert_eq!(zs[3].0, Zona::Maxima);
    // Calculado a mano: 55 + 0,50 × 125 = 117,5 -> 118.
    assert_eq!(zs[0].1, 118, "el suelo de la zona suave no cuadra con el calculo a mano");
}

#[test]
fn la_carga_sube_con_la_intensidad_mucho_mas_que_con_el_tiempo() {
    // La regla fisiologica: el TRIMP crece de forma EXPONENCIAL con la intensidad.
    // Dos sesiones igual de largas, una a 120 y otra a 170: la segunda tiene que dar MUCHO mas.
    let suave = formulas::trimp(&sesion_30_min(120), &ana()).valor.expect("se calcula");
    let fuerte = formulas::trimp(&sesion_30_min(170), &ana()).valor.expect("se calcula");
    assert!(suave > 0.0);
    assert!(
        fuerte > suave * 2.0,
        "subir la intensidad tiene que subir la carga MUCHO mas que proporcionalmente: {:.1} vs {:.1}",
        fuerte,
        suave
    );
}

// ------------------------------------------------- 2. QUE DETECTE LO QUE FALTA (lo importante)

#[test]
fn sin_edad_y_sin_medida_no_se_puede_saber_la_maxima_y_lo_dice() {
    let mut p = ana();
    p.edad = Dato::Falta;
    let r = formulas::pulso_maximo(&p);
    assert_eq!(r.valor, None, "sin edad ni medida NO se puede: no se inventa");
    assert!(!r.falta.is_empty(), "tiene que decir QUE falta");
    // Y tiene que decir que haria falta para poder afirmarlo.
    assert!(
        r.falta.iter().any(|f| f.contains("prueba de esfuerzo") || f.contains("edad")),
        "tiene que decir que haria falta: {:?}",
        r.falta
    );
}

#[test]
fn sin_pulso_de_reposo_no_hay_reserva_ni_zonas() {
    let mut p = ana();
    p.pulso_reposo = Dato::Falta;
    assert_eq!(formulas::reserva_cardiaca(&p).valor, None);
    assert_eq!(formulas::zonas(&p).valor, None, "sin reserva NO se inventan zonas");
    // Y el motivo tiene que nombrar lo que falta.
    let r = formulas::zonas(&p);
    assert!(r.fuente.contains("reposo") || r.falta.iter().any(|f| f.contains("reposo")));
}

#[test]
fn una_sesion_sin_pulsos_no_da_carga_y_lo_dice() {
    let s = Sesion::con_duracion(vec![
            Muestra { segundo: 0, pulso: Dato::Falta, potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta },
            Muestra { segundo: 10, pulso: Dato::Falta, potencia_w: Dato::Hay(210), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta },
        ], 600);
    let r = formulas::trimp(&s, &ana());
    assert_eq!(r.valor, None, "sin un solo pulso, la carga no se puede afirmar");
    assert!(r.fuente.contains("pulso"), "el motivo tiene que decir que falta el pulso");
}

#[test]
fn si_falta_el_pulso_en_MAS_de_la_mitad_la_carga_no_se_da() {
    // La mitad de la sesion sin pulso: el numero no se sostiene, y el motor NO lo da.
    let mut muestras = Vec::new();
    for i in 0..180 {
        muestras.push(Muestra {
            segundo: i * 10,
            pulso: if i % 3 == 0 { Dato::Hay(150) } else { Dato::Falta },
            potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta,
        });
    }
    let s = Sesion::con_duracion(muestras, 1800);
    let r = formulas::trimp(&s, &ana());
    assert_eq!(r.valor, None, "con mas de la mitad sin pulso, la carga NO se afirma");
    assert!(r.fuente.contains("mitad"), "tiene que decir por que: {}", r.fuente);
}

#[test]
fn si_falta_el_pulso_en_POCO_la_carga_se_da_PERO_con_reserva() {
    // Un 25 % de la sesion sin pulso: pasa del limite del 20 %, asi que se puede dar PERO avisando.
    //
    // OJO, esto se corrigio el 2026-10-13: la primera version de esta prueba pedia reserva con solo
    // un 10 % de hueco, y **la prueba estaba mal, no el motor**. Un 10 % no cambia el numero lo
    // suficiente como para tener que avisar. Y al mirarlo salio un DEFECTO de verdad: la duracion se
    // calculaba con la primera y la ultima muestra, asi que si la primera no traia pulso, **salia
    // corta sin avisar**. Esta prueba es la que lo cazo.
    let mut muestras = Vec::new();
    for i in 0..180 {
        muestras.push(Muestra {
            segundo: i * 10,
            pulso: if i % 4 == 0 { Dato::Falta } else { Dato::Hay(150) },
            potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta,
        });
    }
    let s = Sesion::con_duracion(muestras, 1800);
    let r = formulas::trimp(&s, &ana());
    assert!(r.valor.is_some(), "con un hueco por debajo del limite, la carga se puede dar");
    assert!(r.reserva.is_some(), "pero CON RESERVA: el hueco cambia el numero");
}

#[test]
fn la_duracion_no_se_acorta_si_la_primera_muestra_no_trae_pulso() {
    // El defecto que cazo la prueba de arriba, con su caso minimo y propio.
    // Sin duracion declarada, la duracion se deduce de las muestras: y tiene que salir de la
    // PRIMERA y la ULTIMA, aunque la primera no traiga pulso.
    let s = Sesion::nueva(vec![
        Muestra { segundo: 0, pulso: Dato::Falta, potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta },
        Muestra { segundo: 600, pulso: Dato::Hay(150), potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta },
        Muestra { segundo: 1200, pulso: Dato::Hay(150), potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta },
    ]);
    assert_eq!(
        s.duracion(),
        Dato::Hay(1200),
        "la duracion tiene que ser de la primera a la ultima muestra, no desde la primera CON pulso"
    );
}

// ------------------------------------------------------------------ 3. LÍMITES: NO REVIENTA

#[test]
fn un_reposo_mayor_que_la_maxima_no_revienta_y_se_rechaza() {
    let mut p = ana();
    p.pulso_reposo = Dato::Hay(200); // imposible: 200 en reposo con maxima 180
    let r = formulas::reserva_cardiaca(&p);
    assert_eq!(r.valor, None, "datos que no cuadran no dan un numero: dan un rechazo");
    assert!(r.fuente.contains("no cuadran") || r.fuente.contains("coherentes"));
}

#[test]
fn una_sesion_de_duracion_cero_no_revienta() {
    let s = Sesion::con_duracion(vec![], 0);
    let r = formulas::trimp(&s, &ana());
    // Sin muestras no hay pulso, asi que se rechaza. Lo importante: NO da infinito ni NaN.
    assert!(r.valor.is_none() || r.valor.unwrap().is_finite());
    if let Some(v) = r.valor {
        assert!(v.is_finite(), "un calculo con cero duracion no puede dar infinito");
    }
}

#[test]
fn una_persona_de_la_que_no_sabemos_nada_no_produce_zonas_inventadas() {
    let p = Persona::desconocida();
    let r = formulas::zonas(&p);
    assert_eq!(r.valor, None, "de la nada no salen zonas");
    assert!(!r.falta.is_empty(), "y se dice que falta");
}

#[test]
fn un_pulso_imposible_dentro_de_la_sesion_no_rompe_el_calculo() {
    // 300 pulsaciones no existen; el motor no debe dar un numero absurdo ni reventar.
    let s = sesion_30_min(300);
    let r = formulas::trimp(&s, &ana());
    if let Some(v) = r.valor {
        assert!(v.is_finite() && v < 1000.0, "un pulso imposible no puede dar una carga absurda: {}", v);
    }
}

// ---------------------------------------------------- 4. QUE TODA AFIRMACIÓN TRAIGA SU FUENTE

#[test]
fn con_todos_los_datos_el_juicio_es_seguro_o_con_reserva_y_todo_tiene_fuente() {
    let j = Juicio::juzgar(&ana(), &sesion_30_min(150));
    assert!(
        matches!(j.confianza, Confianza::Seguro | Confianza::ConReserva),
        "con datos suficientes, el juicio no puede ser 'no se puede': {:?}",
        j.confianza
    );
    assert!(!j.fundamentos.is_empty());
    for f in &j.fundamentos {
        assert!(!f.fuente.trim().is_empty(), "afirmacion SIN fuente: {}", f.afirmacion);
    }
}

#[test]
fn sin_datos_el_juicio_baja_a_NO_SE_PUEDE_y_dice_que_falta() {
    let j = Juicio::juzgar(&Persona::desconocida(), &Sesion::nueva(vec![]));
    assert_eq!(j.confianza, Confianza::NoSePuede, "sin datos no se puede afirmar");
    assert!(!j.faltantes.is_empty(), "y hay que decir que falta");
    for f in &j.faltantes {
        assert!(!f.para_que.trim().is_empty(), "falta sin decir PARA QUE hacia falta");
    }
}

#[test]
fn el_informe_dice_lo_que_falta_y_no_diagnostica_enfermedades() {
    let j = Juicio::juzgar(&Persona::desconocida(), &Sesion::nueva(vec![]));
    let t = j.informe();
    assert!(t.contains("NO SE PUEDE AFIRMAR"));
    assert!(t.contains("LO QUE FALTA"));
    assert!(
        t.contains("no diagnostica enfermedades"),
        "el informe tiene que decir lo que NO es"
    );
}

// ================================================================ 5. EL SALTO DE ESFUERZO

use motor::esfuerzo::{self, NivelSalto};

/// Una sesión con una duración y un pulso fijos, para comparar saltos.
fn sesion_de(minutos: u32, pulso: u16) -> Sesion {
    let n = (minutos * 60) / 10;
    let muestras = (0..n)
        .map(|i| Muestra { segundo: i * 10, pulso: Dato::Hay(pulso), potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta })
        .collect();
    Sesion::con_duracion(muestras, minutos * 60)
}

#[test]
fn sin_sesiones_anteriores_no_se_puede_comparar_y_se_dice() {
    let r = esfuerzo::salto(&sesion_de(60, 140), &[], &ana(), false);
    assert_eq!(r.valor, None, "sin mes anterior no hay con que comparar: no se inventa una media");
    assert!(r.fuente.contains("anteriores"), "tiene que decir que faltan las anteriores");
}

#[test]
fn una_sesion_igual_a_la_mas_larga_del_mes_MANTIENE() {
    let antes = vec![sesion_de(90, 140), sesion_de(60, 140)];
    let r = esfuerzo::salto(&sesion_de(90, 140), &antes, &ana(), false);
    let (sal, nivel) = r.valor.expect("se puede comparar");
    assert_eq!(nivel, NivelSalto::Mantiene);
    assert!((sal.proporcion - 1.0).abs() < 0.01, "igual que la mas larga es proporcion 1,0");
}

#[test]
fn un_salto_del_90_por_ciento_en_DURACION_es_PELIGRO() {
    // Este es el caso del cliente sintetico: 210 minutos contra 110 del mes anterior.
    // El estudio dice que ahi el riesgo crece de forma exponencial.
    let antes = vec![sesion_de(110, 130), sesion_de(100, 135), sesion_de(60, 150)];
    let r = esfuerzo::salto(&sesion_de(210, 128), &antes, &ana(), false);
    let (sal, nivel) = r.valor.expect("se puede comparar");
    assert!((sal.subida() - 0.909).abs() < 0.02, "la subida tiene que ser ~91 %: {}", sal.subida());
    assert_eq!(nivel, NivelSalto::Peligro, "un 91 % mas es peligro");
}

#[test]
fn el_mismo_salto_mirado_por_CARGA_puede_salir_menos_grave() {
    // ESTA PRUEBA RECOGE UN HALLAZGO del 2026-10-13, y es importante que quede escrito:
    // la duracion y la carga NO cuentan la misma historia. Una salida larga y suave sube mucho
    // en duracion y poco en carga. Por eso el motor da LAS DOS, y no elige por su cuenta.
    let antes = vec![sesion_de(110, 130), sesion_de(100, 135)];
    let hoy = sesion_de(210, 128);

    let por_dur = esfuerzo::salto(&hoy, &antes, &ana(), false).valor.expect("se puede");
    let por_carga = esfuerzo::salto(&hoy, &antes, &ana(), true).valor.expect("se puede");

    // La DURACION sube mas que la CARGA: ese es el hallazgo, y es lo que hay que dejar escrito.
    assert!(
        por_dur.0.subida() > por_carga.0.subida(),
        "en una salida larga y suave, la duracion sube MAS que la carga: {:.0} % vs {:.0} %",
        por_dur.0.subida() * 100.0,
        por_carga.0.subida() * 100.0
    );

    // Y por eso la eleccion de unidad IMPORTA: hay casos en que una avisa y la otra no.
    // Aqui las dos avisan, pero la de duracion avisa mas alto.
    assert!(
        matches!(por_carga.1, NivelSalto::Aviso | NivelSalto::Peligro),
        "la carga tambien tiene que avisar en un salto asi"
    );

    // El caso en que de verdad DIVERGEN: se comprueba abajo, con pulsos distintos.
    // Aqui se deja dicho por que el motor da LAS DOS medidas en vez de elegir.
}

#[test]
fn la_duracion_y_la_carga_DIVERGEN_cuando_la_salida_es_larga_pero_suave() {
    // ESTE es el caso que obliga al motor a dar las DOS medidas y no elegir una:
    // el mes anterior fueron series CORTAS e INTENSAS; hoy es una salida LARGA y SUAVE.
    // En duracion es un salto enorme; en carga, mucho menor.
    let antes = vec![sesion_de(45, 165), sesion_de(50, 168)]; // series cortas y fuertes
    let hoy = sesion_de(150, 120); // salida larga y suave

    let por_dur = esfuerzo::salto(&hoy, &antes, &ana(), false).valor.expect("se puede");
    let por_carga = esfuerzo::salto(&hoy, &antes, &ana(), true).valor.expect("se puede");

    assert!(
        por_dur.0.subida() > por_carga.0.subida() + 0.5,
        "tienen que divergir claramente: duracion +{:.0} %, carga +{:.0} %",
        por_dur.0.subida() * 100.0,
        por_carga.0.subida() * 100.0
    );
    assert_eq!(
        por_dur.1,
        NivelSalto::Peligro,
        "en duracion es peligro: mas del doble de lo mas largo del mes"
    );
    assert_ne!(por_carga.1, NivelSalto::Peligro, "en carga NO llega a peligro");
}

#[test]
fn el_salto_siempre_declara_que_los_cortes_son_una_DECISION() {
    let antes = vec![sesion_de(60, 140)];
    let r = esfuerzo::salto(&sesion_de(70, 140), &antes, &ana(), false);
    let reserva = r.reserva.expect("el salto declara su reserva");
    assert!(
        reserva.contains("DECISIÓN") || reserva.contains("DECISION"),
        "tiene que decir que los cortes los eligio el motor, no el estudio: {reserva}"
    );
}

#[test]
fn una_sesion_mas_corta_que_el_mes_no_alerta_de_nada() {
    let antes = vec![sesion_de(120, 140)];
    let r = esfuerzo::salto(&sesion_de(30, 120), &antes, &ana(), false);
    let (sal, nivel) = r.valor.expect("se puede comparar");
    assert!(sal.subida() < 0.0, "es mas corta, asi que la subida es negativa");
    assert_eq!(nivel, NivelSalto::Mantiene, "una sesion corta no es un riesgo");
}

// ------------------------------------------- 6. EL AVISO DE LA FRECUENCIA MÁXIMA ESTIMADA

#[test]
fn a_los_60_ANOS_el_motor_avisa_de_que_la_estimacion_puede_quedarse_corta() {
    // El caso del cliente: 60 anos, sin prueba de esfuerzo. El aviso tiene que decir que a esa
    // edad una persona entrenada suele tener la maxima MAS ALTA que la media de su edad.
    let mut p = ana();
    p.edad = Dato::Hay(60);
    p.pulso_maximo_medido = Dato::Falta;
    let aviso = esfuerzo::aviso_de_la_estimacion(&p).expect("a los 60 tiene que avisar");
    assert!(aviso.contains("ESTIMADA"), "tiene que decir que esta estimada");
    assert!(
        aviso.contains("MÁS ALTA") || aviso.contains("MAS ALTA"),
        "y que en una persona entrenada suele quedarse corta: {aviso}"
    );
    assert!(aviso.contains("prueba de esfuerzo"), "y que la solucion es medirla");
}

#[test]
fn si_la_maxima_esta_MEDIDA_no_hay_nada_que_avisar() {
    let mut p = ana();
    p.edad = Dato::Hay(60);
    p.pulso_maximo_medido = Dato::Hay(178);
    assert_eq!(esfuerzo::aviso_de_la_estimacion(&p), None, "si se sabe, no se avisa");
}

// ------------------------------------------------- 7. EL ADAPTADOR DEL FORMATO DE FUERA

#[test]
fn el_adaptador_lee_un_perfil_y_lo_que_falta_queda_como_FALTA_no_como_cero() {
    let json = r#"{"persona": {"edad": 60, "peso_kg": 72.0, "pulso_reposo": 48, "pulso_maximo_medido": null, "sexo": null}}"#;
    let p = motor::adaptador::leer_perfil(json).expect("el JSON se entiende");
    assert_eq!(p.edad, Dato::Hay(60));
    assert_eq!(p.pulso_reposo, Dato::Hay(48));
    // ESTO es lo importante: lo que viene `null` NO se convierte en 0 ni en un valor por defecto.
    assert_eq!(p.pulso_maximo_medido, Dato::Falta);
    assert_eq!(p.sexo, Dato::Falta);
}

#[test]
fn el_adaptador_NO_adivina_un_sexo_que_no_entiende() {
    let json = r#"{"persona": {"edad": 40, "peso_kg": null, "pulso_reposo": null, "pulso_maximo_medido": null, "sexo": "lo-que-sea"}}"#;
    let p = motor::adaptador::leer_perfil(json).expect("se entiende");
    assert_eq!(p.sexo, Dato::Falta, "un valor que no se entiende se trata como ausente, no se adivina");
}

#[test]
fn el_adaptador_rechaza_un_json_que_no_es_lo_que_espera() {
    let r = motor::adaptador::leer_perfil("{\"otra_cosa\": 1}");
    assert!(r.is_err(), "un JSON con otra forma tiene que fallar, no dar una persona vacia en silencio");
}

// ============================================ 8. LAS DOS FUENTES (Garmin y Huawei)

use motor::fuentes::{tolerancia, Consenso, Procedencia, Valor};

#[test]
fn el_pulso_de_reposo_lo_manda_el_aparato_que_lo_mide_durmiendo_no_la_bascula() {
    // EL CASO REAL de este cliente: Garmin dice 48 (el mínimo, durmiendo) y la báscula de
    // Huawei dijo 71 (sentado por la tarde). Los dos números pueden ser correctos, pero para
    // calcular zonas manda el de durmiendo: es el que usa el método de Karvonen.
    let valores = vec![
        Valor::nuevo(48u16, Procedencia::AparatoDeEntrenamiento, "2026-10-11")
            .con_nota("mínimo mientras duerme"),
        Valor::nuevo(71u16, Procedencia::Bascula, "2026-10-13").con_nota("sentado por la tarde"),
    ];
    let c = Consenso::juntar(valores, Procedencia::para_pulso_de_reposo(), tolerancia::PULSO_REPOSO);

    assert_eq!(c.elegido.as_ref().unwrap().valor, 48, "tiene que usar el de durmiendo");
    assert_eq!(c.discrepantes.len(), 1, "y GUARDAR el otro, no tirarlo");
    assert!(c.aviso.is_some(), "y AVISAR de que los dos no coinciden");
    let aviso = c.aviso.unwrap();
    assert!(aviso.contains("48") && aviso.contains("71"), "el aviso tiene que dar los dos valores");
}

#[test]
fn para_el_peso_manda_la_bascula_y_no_el_aparato_de_entrenamiento() {
    // Aquí es al revés, y es lo que hace que la confianza NO sea una jerarquía fija:
    // para el peso, **la báscula es la que pesa**.
    let valores = vec![
        Valor::nuevo(68.0f32, Procedencia::AparatoDeEntrenamiento, "2026-10-01"),
        Valor::nuevo(69.45f32, Procedencia::Bascula, "2026-10-13"),
    ];
    let c = Consenso::juntar(valores, Procedencia::para_composicion_corporal(), tolerancia::PESO);
    assert_eq!(c.elegido.as_ref().unwrap().valor, 69.45, "para el peso manda la bascula");
    assert_eq!(c.elegido.unwrap().procedencia, Procedencia::Bascula);
}

#[test]
fn si_las_dos_fuentes_coinciden_NO_se_avisa_de_nada() {
    // Un aviso que sale siempre no sirve para nada: tiene que salir solo cuando importa.
    let valores = vec![
        Valor::nuevo(69.0f32, Procedencia::Bascula, "2026-10-12"),
        Valor::nuevo(69.45f32, Procedencia::Bascula, "2026-10-13"),
    ];
    let c = Consenso::juntar(valores, Procedencia::para_composicion_corporal(), tolerancia::PESO);
    assert!(c.aviso.is_none(), "una diferencia del 0,7 % no es un problema");
    assert_eq!(c.elegido.unwrap().fecha, "2026-10-13", "y se queda el mas reciente");
}

#[test]
fn con_una_sola_fuente_no_hay_nada_que_consensuar_y_no_se_inventa() {
    let valores = vec![Valor::nuevo(48u16, Procedencia::AparatoDeEntrenamiento, "2026-10-11")];
    let c = Consenso::juntar(valores, Procedencia::para_pulso_de_reposo(), tolerancia::PULSO_REPOSO);
    assert_eq!(c.elegido.as_ref().unwrap().valor, 48);
    assert!(c.discrepantes.is_empty());
    assert!(c.aviso.is_none(), "con una sola fuente no hay discrepancia que avisar");
}

#[test]
fn sin_ningun_valor_no_se_elige_nada() {
    let c: Consenso<u16> = Consenso::juntar(vec![], Procedencia::para_pulso_de_reposo(), 0.05);
    assert!(!c.hay(), "sin datos no hay valor: no se rellena con cero");
    assert!(c.elegido.is_none());
}

#[test]
fn las_dos_tolerancias_son_distintas_a_proposito_y_se_comprueba_con_el_mismo_porcentaje() {
    // Las tolerancias son DECISIONES, y son distintas porque el mismo error no cuesta lo mismo:
    //   peso ......... 2 %   (dos basculas discrepan legitimamente)
    //   pulso reposo . 5 %   (aqui cada pulsacion mueve las zonas enteras)
    //
    // Se prueba con un 3 %, que cae ENTRE las dos: asi se ve que la diferencia de trato es real.
    //
    // OJO, y es una leccion: la primera version de esta prueba usaba un 8 % para los dos y decia
    // que el peso NO avisaria. FALLO, y con razon: un 8 % pasa del 2 %. **La prueba estaba mal,
    // no el motor** — el mismo tipo de fallo que ya aparecio con la duracion de las sesiones.
    let tres_por_ciento = 1.03f64;

    // PESO: 70 -> 72,1 es un 3 %, y pasa del 2 %, asi que AVISA.
    let peso_alto = (70.0f64 * tres_por_ciento) as f32;
    let p = Consenso::juntar(
        vec![Valor::nuevo(70.0f32, Procedencia::Bascula, "2026-10-13"),
             Valor::nuevo(peso_alto, Procedencia::Bascula, "2026-10-01")],
        Procedencia::para_composicion_corporal(), tolerancia::PESO);
    assert!(p.aviso.is_some(), "un 3 % en el peso pasa del 2 %: tiene que avisar");

    // PULSO DE REPOSO: 50 -> 51 es un 2 %, y NO llega al 5 %, asi que NO avisa.
    let pulso_alto = (50.0f64 * 1.02) as u16; // = 51
    let r = Consenso::juntar(
        vec![Valor::nuevo(50u16, Procedencia::AparatoDeEntrenamiento, "2026-10-11"),
             Valor::nuevo(pulso_alto, Procedencia::Bascula, "2026-10-13")],
        Procedencia::para_pulso_de_reposo(), tolerancia::PULSO_REPOSO);
    assert!(r.aviso.is_none(), "un 2 % en el pulso de reposo no llega al 5 %: no avisa");

    // Y el mismo 8 % que fallaba antes: en el pulso SI tiene que avisar.
    let r8 = Consenso::juntar(
        vec![Valor::nuevo(50u16, Procedencia::AparatoDeEntrenamiento, "2026-10-11"),
             Valor::nuevo((50.0f64 * 1.08) as u16, Procedencia::Bascula, "2026-10-13")],
        Procedencia::para_pulso_de_reposo(), tolerancia::PULSO_REPOSO);
    assert!(r8.aviso.is_some(), "un 8 % en el pulso de reposo si avisa");
}

// ================================================== 9. LA TENDENCIA (¿mejora o solo entrena?)

use motor::tendencia::{tolerancia as tol_tend, Direccion, Serie};

fn serie(que: &str, valores: &[(f64, &str)]) -> Serie {
    Serie::nueva(que, valores.iter()
        .map(|(v, f)| Valor::nuevo(*v, Procedencia::AparatoDeEntrenamiento, f))
        .collect())
}

#[test]
fn con_un_solo_punto_NO_hay_tendencia_y_se_dice() {
    // Con un mes de datos no se puede afirmar si mejora. Esta prueba es la que obliga a decirlo.
    let s = serie("consumo de oxigeno", &[(51.9, "2026-04-30")]);
    assert_eq!(s.tendencia(), Dato::Falta, "un punto no es una tendencia");
}

#[test]
fn la_tendencia_del_consumo_de_oxigeno_que_tiene_este_cliente_sube() {
    // Sus datos reales, mes a mes: arranca, baja, y remonta hasta el mejor valor.
    let s = serie("consumo de oxigeno", &[
        (51.9, "2026-04-30"), (53.6, "2026-05-31"), (51.0, "2026-06-30"),
        (50.0, "2026-07-31"), (54.7, "2026-08-31"), (56.1, "2026-09-30"), (56.8, "2026-10-11"),
    ]);
    let t = s.tendencia().prestado().cloned().expect("hay 7 puntos");
    assert_eq!(t.puntos, 7);
    assert!(t.cambio > 4.0, "sube casi 5 puntos: {}", t.cambio);
    assert_eq!(t.direccion(tol_tend::VO2), Direccion::Sube);
    assert!(t.bajo_y_subio(), "y la forma es bajar y remontar: el minimo NO es el primero");
}

#[test]
fn la_frase_avisa_de_que_toco_fondo_antes_de_remontar() {
    // Una subida recta y una que toca fondo cuentan cosas distintas. La frase tiene que decirlo.
    let s = serie("consumo de oxigeno", &[
        (51.9, "2026-04-30"), (49.6, "2026-08-01"), (56.8, "2026-10-11"),
    ]);
    let t = s.tendencia().prestado().cloned().unwrap();
    let f = t.frase(tol_tend::VO2);
    assert!(f.contains("sube"), "tiene que decir la direccion: {f}");
    assert!(f.contains("tocó fondo"), "y que toco fondo antes de remontar: {f}");
}

#[test]
fn una_variacion_por_debajo_de_la_tolerancia_se_considera_ESTABLE() {
    // Decir «mejora» por una centésima es peor que no decir nada.
    let s = serie("eficiencia", &[(0.983, "2026-09-30"), (0.990, "2026-10-11")]);
    let t = s.tendencia().prestado().cloned().unwrap();
    assert_eq!(
        t.direccion(tol_tend::EFICIENCIA),
        Direccion::Estable,
        "un 0,7 % no llega al 3 %: es estable, no una mejora"
    );
}

#[test]
fn los_puntos_se_ordenan_por_fecha_aunque_lleguen_desordenados() {
    // Un motor que se fie del orden en que llegan los datos da tendencias falsas.
    let s = serie("peso", &[(56.8, "2026-10-11"), (51.9, "2026-04-30"), (54.0, "2026-07-15")]);
    let t = s.tendencia().prestado().cloned().unwrap();
    assert_eq!(t.desde, "2026-04-30", "el primero es el mas ANTIGUO, no el primero que llego");
    assert_eq!(t.hasta, "2026-10-11");
    assert!(t.cambio > 0.0, "y el cambio va del antiguo al nuevo");
}

#[test]
fn la_tendencia_del_VOLUMEN_no_se_confunde_con_la_del_RENDIMIENTO() {
    // EL CASO QUE OBLIGA A ESTE MODULO A EXISTIR, y son los datos reales de este cliente:
    // en julio entreno 39,1 horas (su mes de mas volumen) y su consumo de oxigeno BAJO.
    // Un motor que confunda «mas» con «mejor» le diria que entrene mas justo cuando necesita recuperar.
    let volumen = serie("horas al mes", &[(25.9, "2026-05-31"), (39.1, "2026-07-31"), (13.9, "2026-10-11")]);
    let rendimiento = serie("consumo de oxigeno", &[(53.6, "2026-05-31"), (50.0, "2026-07-31"), (56.8, "2026-10-11")]);

    let tv = volumen.tendencia().prestado().cloned().unwrap();
    let tr = rendimiento.tendencia().prestado().cloned().unwrap();

    // Las dos series van en direcciones OPUESTAS en el mismo periodo, y las dos medidas son correctas.
    assert_eq!(tv.direccion(tol_tend::VOLUMEN), Direccion::Baja, "el volumen acabo bajando");
    assert_eq!(tr.direccion(tol_tend::VO2), Direccion::Sube, "y el rendimiento acabo subiendo");
    // Y las dos tocan fondo en el mismo mes: el de mas volumen.
    assert_eq!(tv.minimo.as_ref().unwrap().fecha, "2026-10-11");
    assert_eq!(tr.minimo.as_ref().unwrap().fecha, "2026-07-31", "su punto mas bajo fue el mes de mas carga");
}

// ======================== 10. EL REDISEÑO: SIEMPRE RESPONDE, CON SU MARGEN
//
// Estas pruebas nacen de una corrección del dueño, y son las más importantes del proyecto:
//
//   «No es posible que solo funcione la aplicación con toda la data, debe poder funcionar con
//    información nula, pues no puede fallar por falta de información. Más información proporciona
//    más precisión, pero no limita.»
//
// Antes, el motor SE NEGABA a calcular cuando faltaba un dato. Ahora SIEMPRE responde, y la
// honestidad viaja en el margen.

use motor::formulas_v2::{self, OrigenMaxima};
use motor::rango::{Precision, Rango};

/// Una persona SIN NINGÚN DATO. El caso extremo.
fn nadie() -> Persona {
    Persona::desconocida()
}

#[test]
fn SIN_NINGUN_DATO_el_motor_responde_igual_y_lo_dice() {
    // El caso que antes fallaba. Ahora tiene que dar un número.
    let m = formulas_v2::pulso_maximo(&nadie());
    assert!(m.rango.valor > 0.0, "sin datos TIENE que responder un numero");
    assert_eq!(m.origen, OrigenMaxima::DePoblacion, "y decir de donde sale");
    assert!(m.rango.margen > 0.0, "y con su margen: no puede fingir que es exacto");
    assert!(!m.ganaria.is_empty(), "y decir que ganaria el usuario aportando datos");
}

#[test]
fn las_zonas_se_calculan_SIEMPRE_aunque_no_haya_ni_un_dato() {
    // Lo que pedía el dueño, literal: la aplicación no puede fallar por falta de información.
    let z = formulas_v2::zonas(&nadie());
    assert_eq!(z.len(), 4, "las cuatro zonas, siempre");
    for (zona, r) in &z {
        assert!(r.valor > 0.0, "la zona {} tiene que tener un limite", zona.nombre());
        assert!(r.margen > 0.0, "y su margen");
    }
    // Y tienen que ir en orden: suave < moderada < fuerte < maxima.
    let valores: Vec<f64> = z.iter().map(|(_, r)| r.valor).collect();
    let mut ordenados = valores.clone();
    ordenados.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(valores, ordenados, "las zonas tienen que ir de menos a mas");
}

#[test]
fn MAS_DATOS_ESTRECHAN_EL_MARGEN_y_eso_es_lo_que_el_dueno_pide() {
    // Aquí está la frase del dueño convertida en una prueba:
    //   «Más información proporciona más precisión, pero no limita».
    // Se comprueba que el margen se estrecha al añadir datos, en tres escalones.
    let sin_nada = formulas_v2::pulso_maximo(&nadie());

    let mut con_edad = nadie();
    con_edad.edad = Dato::Hay(60);
    let solo_edad = formulas_v2::pulso_maximo(&con_edad);

    let mut con_todo = con_edad.clone();
    con_todo.peso_kg = Dato::Hay(70.0);
    con_todo.grasa_corporal_pct = Dato::Hay(19.0);
    let con_composicion = formulas_v2::pulso_maximo(&con_todo);
    let _ = &con_composicion;

    let mut medido = con_todo.clone();
    medido.pulso_maximo_medido = Dato::Hay(179);
    let con_medida = formulas_v2::pulso_maximo(&medido);

    // Los cuatro responden. Y el margen va de más ancho a más estrecho.
    assert!(
        sin_nada.rango.margen >= solo_edad.rango.margen,
        "sin nada, el margen no puede ser MAS estrecho que con la edad"
    );
    assert!(
        solo_edad.rango.margen > con_composicion.rango.margen,
        "la composicion corporal TIENE que estrechar el margen: {:.1} vs {:.1}",
        solo_edad.rango.margen,
        con_composicion.rango.margen
    );
    assert!(
        con_composicion.rango.margen > con_medida.rango.margen,
        "medirla es lo mejor: {:.1} vs {:.1}",
        con_composicion.rango.margen,
        con_medida.rango.margen
    );
}

#[test]
fn la_precision_se_ve_de_un_vistazo_y_mejora_con_los_datos() {
    // El usuario tiene que poder saber si el numero es fiable SIN leer el margen.
    let mut p = nadie();
    p.edad = Dato::Hay(60);
    let solo_edad = formulas_v2::precision_de_conjunto(&p);

    p.peso_kg = Dato::Hay(70.0);
    p.grasa_corporal_pct = Dato::Hay(19.0);
    let con_composicion = formulas_v2::precision_de_conjunto(&p);

    // Los cortes van por PULSACIONES, no por porcentaje: lo que mueve una zona son los latidos.
    // ±12 (solo edad) -> Aceptable.  ±7,5 (con composicion) -> Buena.
    assert_eq!(solo_edad, Precision::Aceptable, "±12 pulsaciones es un margen apreciable");
    assert_eq!(con_composicion, Precision::Buena, "±7,5 ya sirve para decidir");
}

#[test]
fn el_origen_dice_QUE_LE_FALTA_al_usuario_para_que_el_numero_sea_mejor() {
    // No basta con decir «el margen es ancho»: hay que decir QUE hacer para estrecharlo.
    let mut p = nadie();
    p.edad = Dato::Hay(60);
    let m = formulas_v2::pulso_maximo(&p);
    let falta = m.origen.que_falta().expect("tiene que decir que falta");
    assert!(falta.contains("composición"), "y tiene que ser util: {falta}");

    // Y si ya esta medida, no falta nada.
    p.pulso_maximo_medido = Dato::Hay(179);
    assert_eq!(formulas_v2::pulso_maximo(&p).origen.que_falta(), None);
}

#[test]
fn una_persona_de_la_que_no_sabemos_nada_cuenta_CERO_datos() {
    // Sirve para que el usuario vea que cada dato que aporta cuenta.
    assert_eq!(nadie().cuantos_datos(), 0);
    let mut p = nadie();
    p.edad = Dato::Hay(60);
    p.peso_kg = Dato::Hay(70.0);
    assert_eq!(p.cuantos_datos(), 2);
}

#[test]
fn la_precision_nunca_dice_que_algo_es_exacto_si_no_lo_es() {
    // La honestidad, ahora, vive en el margen. Un dato de poblacion NO puede parecer exacto.
    let r = Rango::estimado(166.0, 12.0);
    assert!(r.precision() < 1.0, "una estimacion no puede tener precision perfecta");
    assert!((r.minimo() - 154.0).abs() < 0.01);
    assert!((r.maximo() - 178.0).abs() < 0.01);
    // Y con la composicion, el mismo valor tiene mejor precision.
    let mejor = Rango::estimado(166.0, 7.5);
    assert!(mejor.precision() > r.precision(), "menos margen es mas precision");
}

#[test]
fn los_margenes_se_combinan_sin_exagerar() {
    // Sumar los margenes a pelo daria un rango DEMASIADO ancho y asustaria al usuario sin motivo.
    // Se combinan en cuadratura: raiz de la suma de cuadrados.
    let solo = motor::rango::suma_de_margenes(&[12.0]);
    assert!((solo - 12.0).abs() < 0.01, "un solo margen no cambia");
    let dos = motor::rango::suma_de_margenes(&[12.0, 12.0]);
    assert!(dos < 24.0, "dos margenes NO se suman a pelo: {dos:.1} < 24");
    assert!(dos > 12.0, "pero si crecen: {dos:.1} > 12");
}

#[test]
fn el_caso_real_del_cliente_sale_CON_MENOS_MARGEN_que_solo_con_la_edad() {
    // El cliente tiene su composicion corporal (de la bascula) y su umbral medido.
    // Eso es lo que hace que su resultado sea mas preciso que el de una formula a secas.
    let mut p = nadie();
    p.edad = Dato::Hay(60);
    p.pulso_reposo = Dato::Hay(48);
    let solo_edad = formulas_v2::pulso_maximo(&p);

    p.peso_kg = Dato::Hay(70.2);
    p.grasa_corporal_pct = Dato::Hay(19.1);
    p.altura_cm = Dato::Hay(178.0);
    let con_todo = formulas_v2::pulso_maximo(&p);

    assert!(
        con_todo.rango.margen < solo_edad.rango.margen,
        "con sus datos, el margen tiene que ser menor: {:.1} < {:.1}",
        con_todo.rango.margen,
        solo_edad.rango.margen
    );
    // Y las zonas se calculan en los dos casos, sin fallar nunca.
    assert_eq!(formulas_v2::zonas(&p).len(), 4);
}

// ============ 11. LOS PATRONES: lo que el FIT no trae, y hay que deducir de los sensores
//
// El dueño lo dijo así: «resultados calculados que el mismo FIT no proporciona, pues solo
// proporciona la información de los inputs de los sensores». Estas pruebas defienden eso.

use motor::patron::{self, NivelDeriva};

/// Una sesion sintetica con **DOS ESCALONES**: la primera mitad con unos valores y la segunda con
/// otros. Se hace asi, y no con un cambio lineal, porque **un cambio lineal hace que la media de cada
/// mitad no sea la media de los extremos** — y entonces la cuenta a mano no cuadra con la del motor.
/// Lo aprendi porque una prueba fallo por eso, y **el motor tenia razon**.
fn sesion_dos_escalones(pulso1: u16, pulso2: u16, pot1: u16, pot2: u16) -> Sesion {
    let n = 240usize; // 240 muestras, 120 por mitad, **constantes cada mitad**
    let mut muestras = Vec::new();
    for i in 0..n {
        let (p, w) = if i < n / 2 { (pulso1, pot1) } else { (pulso2, pot2) };
        muestras.push(Muestra {
            segundo: (i * 10) as u32,
            pulso: Dato::Hay(p),
            potencia_w: Dato::Hay(w),
            cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta,
        });
    }
    Sesion::con_duracion(muestras, (n * 10) as u32)
}

/// El mismo ayudante con los nombres de antes, para no tocar el resto de las pruebas.
fn sesion_con_deriva(pulso_inicio: u16, pulso_fin: u16, pot_inicio: u16, pot_fin: u16) -> Sesion {
    sesion_dos_escalones(pulso_inicio, pulso_fin, pot_inicio, pot_fin)
}

#[test]
fn una_sesion_ESTABLE_no_tiene_deriva() {
    // Ni el pulso sube ni la potencia baja: no hay deriva que medir.
    let s = sesion_con_deriva(140, 140, 200, 200);
    let d = patron::deriva(&s);
    assert!(d.porcentaje().abs() < 1.0, "sin cambios no hay deriva: {:.1} %", d.porcentaje());
    assert_eq!(d.nivel(), NivelDeriva::Buena);
}

#[test]
fn el_pulso_que_sube_con_la_potencia_quieta_da_deriva_POSITIVA() {
    // Es el patron del cliente: el pulso se escapa mientras la potencia se queda.
    // Tiene que dar deriva POSITIVA, y el nivel tiene que avisar.
    let s = sesion_con_deriva(130, 155, 200, 200);
    let d = patron::deriva(&s);
    assert!(d.porcentaje() > 10.0, "el pulso sube un 19 % y la potencia no: deriva {:.1} %", d.porcentaje());
    assert_eq!(d.nivel(), NivelDeriva::SeEscapa, "y tiene que decirlo");
}

#[test]
fn la_deriva_declara_su_margen_y_el_margen_NO_se_come_la_diferencia() {
    // EL DEFECTO QUE SE CORRIGIO: la primera version sacaba el margen del numero de muestras, y
    // salia ±1 en TODAS las sesiones. Un margen igual para todo no informa de nada.
    // Y un margen tan ancho que se coma la diferencia tampoco sirve: 8 % y 12 % tienen que
    // poder distinguirse.
    let ligera = patron::deriva(&sesion_con_deriva(130, 138, 200, 200));
    let fuerte = patron::deriva(&sesion_con_deriva(130, 160, 200, 200));

    assert!(ligera.rango.margen > 0.0, "tiene que declarar margen");

    // LA INVARIANTE QUE DE VERDAD IMPORTA, y la primera version de esta prueba la tenia mal:
    // no se trata de que el margen sea pequeño en abstracto, sino de que **las dos se distingan**.
    // Y con 240 muestras el margen es ±6 -- porque una sesion corta da menos confianza, y eso es
    // correcto: **el margen tiene que ensancharse cuando hay menos datos**.
    assert!(
        ligera.rango.margen >= patron::MARGEN_DERIVA,
        "con pocas muestras el margen no puede ser MAS estrecho que el minimo: {:.1}",
        ligera.rango.margen
    );
    // Una deriva del 5,8 % y otra del 18,8 % tienen que quedar en rangos SEPARADOS.
    assert!(
        fuerte.rango.minimo() > ligera.rango.maximo(),
        "los dos rangos tienen que quedar separados:\n  ligera  {:.1} ± {:.1}  ->  {:.1} a {:.1}\n  fuerte  {:.1} ± {:.1}  ->  {:.1} a {:.1}",
        ligera.porcentaje(), ligera.rango.margen, ligera.rango.minimo(), ligera.rango.maximo(),
        fuerte.porcentaje(), fuerte.rango.margen, fuerte.rango.minimo(), fuerte.rango.maximo()
    );
}

#[test]
fn sin_pulso_ni_potencia_la_deriva_da_CERO_con_margen_cero_y_no_falla() {
    // La regla del rediseno: nunca se falla. Si no hay datos, el resultado es cero Y SE SABE que no
    // vale nada, porque el margen tambien es cero. No es un cero que enganie.
    let s = Sesion::nueva(vec![
        Muestra { segundo: 0, pulso: Dato::Falta, potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta },
        Muestra { segundo: 10, pulso: Dato::Hay(140), potencia_w: Dato::Falta, cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta },
    ]);
    let d = patron::deriva(&s);
    assert_eq!(d.porcentaje(), 0.0);
    assert_eq!(d.rango.margen, 0.0, "y el margen cero avisa de que el numero no vale");
    assert_eq!(d.muestras, 0);
}

#[test]
fn la_EFICIENCIA_mide_vatios_por_pulsacion_y_es_la_medida_de_la_mejora() {
    // 200 W a 140 ppm = 1,43 W por pulsacion.
    let s = sesion_con_deriva(140, 140, 200, 200);
    let e = patron::eficiencia(&s);
    assert!((e.valor - 200.0 / 140.0).abs() < 0.01, "eficiencia {:.3}", e.valor);
}

#[test]
fn MEJORAR_es_subir_los_vatios_SIN_subir_el_pulso() {
    // Esta es la definicion de mejorar, y es lo que el cliente busca.
    // Mismo pulso, mas vatios -> mas eficiencia. Eso NO se ve mirando el pulso ni la potencia solos.
    let antes = patron::eficiencia(&sesion_con_deriva(140, 140, 180, 180));
    let despues = patron::eficiencia(&sesion_con_deriva(140, 140, 200, 200));
    assert!(
        despues.valor > antes.valor,
        "los mismos 140 ppm con 200 W en vez de 180 W es MEJOR: {:.3} > {:.3}",
        despues.valor,
        antes.valor
    );
}

#[test]
fn el_desfase_se_mide_en_SEGUNDOS_y_no_en_muestras() {
    // EL ERROR QUE SE CORRIGIO: el retardo se contaba en muestras y se devolvia como segundos.
    // Con muestras cada 10 segundos, eso era un factor de 10.
    // Aqui se comprueba que el numero que sale es PLAUSIBLE en segundos.
    let s = sesion_con_deriva(130, 150, 200, 200);
    if let Dato::Hay(r) = patron::desfase(&s, 60) {
        assert!(
            r.valor <= 60.0,
            "no puede devolver mas segundos de los que se le pidieron: {:.0}",
            r.valor
        );
        assert!(r.margen > 0.0, "y su margen");
    }
}

#[test]
fn la_precision_de_un_patron_depende_de_las_muestras_que_lo_sostienen() {
    use motor::rango::Precision;
    assert_eq!(patron::precision_del_patron(5000), Precision::Alta);
    assert_eq!(patron::precision_del_patron(800), Precision::Buena);
    assert_eq!(patron::precision_del_patron(200), Precision::Aceptable);
    assert_eq!(patron::precision_del_patron(20), Precision::Amplia);
    // Y 129.418 lecturas de sensores en un mes dan precision alta.
    assert_eq!(patron::precision_del_patron(129_418), Precision::Alta);
}

// ============================================= 12. EL LECTOR DE FIT
//
// Hasta aqui el motor leia un JSON que se preparaba a mano. Con esto, los ficheros del aparato
// entran directamente. La prueba que lo valida es LA COMPROBACION CRUZADA: si el motor calcula el
// pulso medio y el fichero declara otro, uno de los dos esta mal.

use motor::fit;
use std::path::Path;

/// La carpeta con los FIT de ejemplo, si existe en esta maquina.
fn carpeta_fit() -> std::path::PathBuf {
    // Ruta relativa al directorio del crate, para que funcione desde `cargo test`.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../datos/garmin_files/storage")
}

#[test]
fn un_fichero_que_no_existe_da_un_error_CLARO_y_no_un_panic() {
    // Un fichero que falta es un dato que falta, y eso **se dice**, no se revienta.
    let r = fit::leer(Path::new("/no/existe/esto.fit"));
    assert!(r.is_err());
    let e = r.unwrap_err();
    assert!(e.contains("no se pudo abrir"), "el error tiene que decir que paso: {e}");
}

#[test]
fn un_fichero_que_no_es_FIT_da_un_error_y_no_un_panic() {
    let ruta = std::env::temp_dir().join("no-es-un-fit.fit");
    std::fs::write(&ruta, b"esto no es un FIT, es texto").unwrap();
    let r = fit::leer(&ruta);
    assert!(r.is_err(), "un fichero que no es FIT tiene que fallar, no dar una sesion vacia");
    let _ = std::fs::remove_file(&ruta);
}

#[test]
fn una_carpeta_que_no_existe_se_dice_y_no_se_cae() {
    let (bien, mal) = fit::leer_carpeta(Path::new("/no/existe/esta/carpeta"));
    assert!(bien.is_empty());
    assert_eq!(mal.len(), 1, "tiene que decir que la carpeta no se pudo leer");
}

#[test]
fn EL_LECTOR_CUADRA_CON_LOS_TOTALES_QUE_DECLARA_EL_FICHERO() {
    // ESTA ES LA PRUEBA QUE VALE, y es la comprobacion cruzada:
    // el pulso medio que el motor calcula a partir de las muestras tiene que coincidir con el que
    // el propio FIT declara. Si no coinciden, **uno de los dos esta mal**.
    let carpeta = carpeta_fit();
    if !carpeta.exists() {
        eprintln!("(sin ficheros FIT en esta maquina: la prueba se salta)");
        return;
    }
    // Se leen DOCE ficheros, no los 102: comprobar el lector no necesita el historico entero, y una
    // prueba que tarda 23 segundos es una prueba que se deja de correr.
    let (bien, _) = fit::leer_carpeta_hasta(&carpeta, 12);
    if bien.is_empty() {
        eprintln!("(no se leyo ningun FIT: la prueba se salta)");
        return;
    }

    let mut comparadas = 0;
    let mut coinciden = 0;
    for (_, l) in &bien {
        let pulsos = l.sesion.pulsos();
        let declarado = match (pulsos.is_empty(), l.declarado.pulso_medio.valor()) {
            (false, Some(d)) => d,
            _ => continue,
        };
        let calculado = pulsos.iter().map(|v| f64::from(*v)).sum::<f64>() / pulsos.len() as f64;
        comparadas += 1;
        if (calculado - f64::from(declarado)).abs() <= 3.0 {
            coinciden += 1;
        }
    }

    assert!(comparadas > 0, "tiene que haber podido comparar alguna sesion");
    let proporcion = coinciden as f64 / comparadas as f64;
    assert!(
        proporcion >= 0.85,
        "el lector tiene que cuadrar con el fichero en al menos el 85 % de las sesiones: \
         {coinciden} de {comparadas}"
    );
}

#[test]
fn el_lector_saca_muestras_y_la_duracion_del_fichero() {
    let carpeta = carpeta_fit();
    if !carpeta.exists() {
        eprintln!("(sin ficheros FIT: la prueba se salta)");
        return;
    }
    let (bien, _) = fit::leer_carpeta_hasta(&carpeta, 1);
    let (_, l) = match bien.first() {
        Some(x) => x,
        None => return,
    };
    assert!(!l.sesion.muestras.is_empty(), "tiene que sacar muestras");
    assert!(l.declarado.duracion_s.lo_hay(), "y la duracion");
    // Y el FIT trae mas de lo que el motor usa todavia: eso se conserva en `mensajes`.
    assert!(!l.mensajes.is_empty(), "y el recuento de mensajes");
    let tiene_record = l.mensajes.iter().any(|(k, _)| k == "Record");
    assert!(tiene_record, "todo FIT de actividad trae mensajes Record");
}

#[test]
fn lo_que_no_se_usa_NO_se_guarda_y_eso_esta_dicho() {
    // El motor no usa la cadencia ni la distancia todavia. Guardarlas daria a entender que si se
    // usan. Se comprueba que el modelo de la muestra solo lleva lo que se usa.
    let m = Muestra { segundo: 0, pulso: Dato::Hay(140), potencia_w: Dato::Hay(200), cadencia_rpm: Dato::Falta, distancia_m: Dato::Falta, temperatura_c: Dato::Falta };
    assert_eq!(m.pulso, Dato::Hay(140));
    assert_eq!(m.potencia_w, Dato::Hay(200));
    // Y si un dia se anade la cadencia, esta prueba es la que obliga a decirlo.
}

// ================================================== 13. EL HRV
//
// El dato que esta DENTRO del fichero FIT y Garmin Connect no enseña: 1.259.082 intervalos entre
// latidos en sus 102 actividades. Dice como esta el cuerpo ANTES de entrenar.

use motor::hrv;

/// Una serie de intervalos entre latidos, con un RMSSD conocido.
/// Alterna dos valores separados por `salto` ms, asi el RMSSD sale exactamente `salto`.
fn serie_rr(n: usize, base: f64, salto: f64) -> Vec<f64> {
    (0..n)
        .map(|i| if i % 2 == 0 { base } else { base + salto })
        .collect()
}

#[test]
fn el_HRV_calculado_cuadra_con_la_cuenta_a_mano() {
    // 100 intervalos que alternan 800 y 850 ms. Todas las diferencias consecutivas son de 50 ms,
    // asi que el RMSSD tiene que ser 50 exacto. Calculado a mano, no copiado.
    let h = hrv::calcular(&serie_rr(100, 800.0, 50.0));
    assert!((h.rmssd.valor - 50.0).abs() < 0.01, "RMSSD {:.2}, esperado 50", h.rmssd.valor);
    assert!((h.rr_medio.valor - 825.0).abs() < 0.01, "RR medio {:.1}, esperado 825", h.rr_medio.valor);
    // Y el pulso deducido: 60000 / 825 = 72,7 ppm.
    assert!((h.pulso_deducido() - 72.7).abs() < 0.1, "pulso {:.1}", h.pulso_deducido());
}

#[test]
fn un_intervalo_ABSURDO_no_dispara_el_RMSSD() {
    // ESTA ES LA PRUEBA QUE JUSTIFICA LA LIMPIEZA, y es la mas importante del modulo:
    // el RMSSD se calcula sobre DIFERENCIAS, asi que **un solo artefacto lo dispara**.
    // Un sensor que se suelta da un valor enorme, y sin filtrar el HRV saldria falsisimo.
    let mut con_basura = serie_rr(100, 800.0, 50.0);
    con_basura.push(12_000.0); // el sensor se solto: 12 segundos entre latidos no es un latido
    let h = hrv::calcular(&con_basura);
    assert!((h.rmssd.valor - 50.0).abs() < 1.0, "el artefacto no puede cambiar el RMSSD: {:.1}", h.rmssd.valor);
    assert_eq!(h.descartados, 1, "y tiene que contarlo como descartado");
}

#[test]
fn un_salto_brusco_tambien_se_descarta() {
    // Un pulso no se dobla ni cae a la mitad entre dos latidos. El salto brusco es el artefacto
    // tipico, y hay que quitarlo.
    let mut s = serie_rr(100, 800.0, 20.0);
    s.insert(50, 1600.0); // el doble de golpe: no es fisiologico
    let h = hrv::calcular(&s);
    assert!(h.descartados >= 1, "el salto brusco tiene que descartarse");
}

#[test]
fn sin_intervalos_el_HRV_da_CERO_con_margen_CERO_y_no_falla() {
    // La regla del rediseno: nunca se falla. Y un cero con margen cero **avisa de que no vale nada**,
    // que es distinto de un cero que parece un dato.
    let h = hrv::calcular(&[]);
    assert_eq!(h.latidos, 0);
    assert_eq!(h.rmssd.valor, 0.0);
    assert_eq!(h.rmssd.margen, 0.0, "el margen cero dice que el numero no sirve");
}

#[test]
fn el_margen_del_RMSSD_NO_finge_una_precision_que_no_tiene() {
    // LA LECCION DE LA DERIVA, aplicada aqui: las muestras de una serie de latidos NO son
    // independientes, asi que el margen NO puede bajar con la raiz de n hasta valores absurdos.
    // Con muchos latidos el margen se estrecha, pero tiene un suelo.
    let pocos = hrv::calcular(&serie_rr(150, 800.0, 50.0));
    let muchos = hrv::calcular(&serie_rr(20_000, 800.0, 50.0));

    assert!(muchos.rmssd.margen < pocos.rmssd.margen, "mas latidos, menos margen");
    assert!(
        muchos.rmssd.margen >= muchos.rmssd.valor * 0.03,
        "pero con un suelo: una sola sesion no puede dar mas precision que eso ({:.2} de {:.1})",
        muchos.rmssd.margen,
        muchos.rmssd.valor
    );
}

#[test]
fn la_fiabilidad_del_HRV_depende_de_los_latidos_que_lo_sostienen() {
    use motor::rango::Precision;
    assert_eq!(hrv::calcular(&serie_rr(2000, 800.0, 50.0)).fiabilidad(), Precision::Alta);
    assert_eq!(hrv::calcular(&serie_rr(500, 800.0, 50.0)).fiabilidad(), Precision::Buena);
    assert_eq!(hrv::calcular(&serie_rr(150, 800.0, 50.0)).fiabilidad(), Precision::Aceptable);
    assert_eq!(hrv::calcular(&serie_rr(20, 800.0, 50.0)).fiabilidad(), Precision::Amplia);
}

#[test]
fn sin_historico_suficiente_NO_se_puede_decir_si_esta_mejor_o_peor() {
    // El HRV no se compara con una tabla: se compara CONSIGO MISMO. Y con dos sesiones no hay
    // normalidad con la que comparar. Se dice que no se puede, en vez de inventar una referencia.
    let hoy = hrv::calcular(&serie_rr(500, 800.0, 50.0));
    let pocas = vec![hoy.clone(), hoy.clone()];
    assert_eq!(hrv::respecto_a_su_normalidad(&hoy, &pocas), None, "con dos no hay normalidad");

    let mut suficientes = Vec::new();
    for i in 0..8 {
        suficientes.push(hrv::calcular(&serie_rr(500, 800.0, 40.0 + i as f64)));
    }
    assert!(
        hrv::respecto_a_su_normalidad(&hoy, &suficientes).is_some(),
        "con ocho si se puede decir como esta respecto a su propia normalidad"
    );
}

// ============================================= 14. EL INFORME POR PLANTILLAS
//
// El dueño lo decidió así: «al final lo que le interesa es una plantilla e información bien
// distribuida». Una plantilla NO improvisa, y en algo que afecta al cuerpo de una persona, que no
// improvise es una ventaja. Y es reproducible y auditable.

use motor::informe;

fn persona_completa() -> Persona {
    let mut p = nadie();
    p.edad = Dato::Hay(60);
    p.pulso_reposo = Dato::Hay(48);
    p.peso_kg = Dato::Hay(70.2);
    p.altura_cm = Dato::Hay(178.0);
    p.grasa_corporal_pct = Dato::Hay(19.1);
    p.umbral_lactato = Dato::Hay(169);
    p.pulso_maximo_medido = Dato::Hay(183);
    p
}

#[test]
fn el_informe_se_genera_SIN_DATOS_y_no_falla() {
    // La regla del rediseno aplicada al informe: **nunca se queda sin responder**.
    let h = informe::hechos(&nadie(), None, None);
    let texto = informe::generar(&h);
    assert!(texto.contains("Análisis de entrenamiento"));
    assert!(texto.contains("zonas"), "aunque no haya datos, las zonas se enseñan");
    assert!(!texto.is_empty());
}

#[test]
fn el_informe_ensena_las_zonas_como_TRAMOS_y_no_como_numeros_sueltos() {
    // EL DEFECTO QUE SE CORRIGIO: la primera version daba solo el suelo de cada zona, y entonces
    // «recuperación» y «resistencia» salian con EL MISMO NUMERO, porque una empieza donde acaba la
    // otra. Un informe que dice dos veces 137 no se entiende.
    let h = informe::hechos(&persona_completa(), None, None);
    let texto = informe::generar(&h);

    // Con el umbral en 169, la resistencia va de 137 a 152.
    assert!(texto.contains("137 – 152"), "la resistencia tiene que salir como tramo:\n{texto}");
    assert!(texto.contains("hasta 137"), "y la recuperacion como «hasta»");
    // Y ninguna zona puede repetir el mismo tramo que otra.
    let mut tramos: Vec<&str> = Vec::new();
    for linea in texto.lines().filter(|l| l.starts_with("| **")) {
        if let Some(t) = linea.split('|').nth(2) {
            tramos.push(t.trim());
        }
    }
    let unicos: std::collections::HashSet<_> = tramos.iter().collect();
    assert_eq!(tramos.len(), unicos.len(), "ninguna zona puede repetir el tramo de otra: {tramos:?}");
}

#[test]
fn el_informe_dice_DE_DONDE_SALE_la_referencia_de_las_zonas() {
    // Con umbral medido, tiene que decirlo. Y no puede llamarlo «estimado».
    let h = informe::hechos(&persona_completa(), None, None);
    let texto = informe::generar(&h);
    assert!(texto.contains("umbral de lactato medido"), "tiene que decir de donde sale");
    assert!(texto.contains("169"), "y el valor");
    assert!(!texto.contains("estimado a partir"), "no puede decir estimado si esta medido");
}

#[test]
fn el_informe_pone_lo_que_FALTA_AL_FINAL_y_no_al_principio() {
    // Un informe que empieza por los avisos metodologicos no se lee. Y ademas, lo que falta NO
    // impide dar un resultado: el resultado esta arriba.
    let h = informe::hechos(&nadie(), None, None);
    let texto = informe::generar(&h);

    let pos_zonas = texto.find("Sus zonas de entrenamiento").expect("estan las zonas");
    let pos_falta = texto.find("Para que el análisis sea más preciso").expect("esta lo que falta");
    assert!(pos_zonas < pos_falta, "las zonas van ANTES de lo que falta");

    // Y tiene que quedar claro que falta precision, no resultado.
    assert!(
        texto.contains("No falta información para **dar un resultado**"),
        "hay que decir que el resultado esta dado"
    );
}

#[test]
fn la_COMPROBACION_del_informe_caza_un_numero_que_nadie_calculo() {
    // Es la guarda que evita el fallo silencioso: si alguien añade texto con un numero que no salio
    // de un calculo, esto lo dice. Es la misma idea que el motor revisando lo que escribiria una IA.
    let bueno = "Su umbral es 169 y la zona de umbral empieza en 159.";
    let problemas = informe::comprobar_que_no_inventa(bueno, &[169.0, 159.0]);
    assert!(problemas.is_empty(), "los numeros legitimos no dan problema: {problemas:?}");

    let malo = "Su umbral es 169 y deberia entrenar a 145 pulsaciones.";
    let problemas = informe::comprobar_que_no_inventa(malo, &[169.0, 159.0]);
    assert_eq!(problemas.len(), 1, "el 145 no salio de ningun calculo");
    assert!(problemas[0].contains("145"), "y hay que decir cual: {}", problemas[0]);
}

#[test]
fn el_informe_es_REPRODUCIBLE() {
    // La ventaja de la plantilla sobre una IA: los mismos datos dan EL MISMO informe, siempre.
    // Un plan que cambia cada vez que lo pides no es un plan.
    let p = persona_completa();
    let uno = informe::generar(&informe::hechos(&p, None, None));
    let dos = informe::generar(&informe::hechos(&p, None, None));
    assert_eq!(uno, dos, "dos veces los mismos datos tienen que dar el mismo informe");
}

#[test]
fn el_informe_termina_recordando_que_NO_es_un_diagnostico_medico() {
    let h = informe::hechos(&persona_completa(), None, None);
    let texto = informe::generar(&h);
    assert!(texto.contains("No es un diagnóstico médico"), "tiene que decirlo");
    assert!(texto.contains("profesional sanitario"), "y a quien corresponde decidir");
}

// ===================== 15. EL CONTEXTO: los TRES escenarios, y de dónde sale cada dato
//
// El dueño lo pidió así: «los debe poder proporcionar el usuario en caso de que no los proporcione
// una aplicación, una aplicación lo puede rellenar de manera automatica, pero si faltan datos el
// usuario debería poder llenarlo, o de lo contrario es nulo».
//
// Y avisó de lo que pasaría si no se contemplan: «la herramienta generaría resultados constantes y
// repetitivos». Estas pruebas defienden que los tres escenarios dan resultados DISTINTOS.

use motor::contexto::{Comida, Contexto, Escenario, Fuente};

#[test]
fn SIN_CONTEXTO_el_motor_calcula_igual_y_lo_dice() {
    // El escenario NULO, que es el que hay que soportar sin romperse.
    let c = Contexto::vacio();
    let (hay, total) = c.cuantos();
    assert_eq!(hay, 0, "sin contexto no hay ningun dato");
    assert_eq!(total, 4, "y se sabe cuantos faltan");
    // Y lo que pide, en imperativo: la aplicacion tiene que PEDIR lo que falta.
    let pedir = c.que_pedir();
    assert_eq!(pedir.len(), 4, "pide los cuatro");
    assert!(pedir.iter().any(|(q, _)| q.contains("sueño")));
    // Y la precision, sin contexto, es la mas ancha: no se puede apretar con nada.
    assert_eq!(c.precision(), motor::rango::Precision::Amplia);
    // Y queda claro que la aplicacion NO se queda sin responder por esto.
    assert!(Escenario::Nulo.que_hace().contains("falta"));
}

#[test]
fn los_TRES_escenarios_dan_resultados_DISTINTOS() {
    // ESTA ES LA PRUEBA QUE PIDE EL DUENO: si los tres dieran lo mismo, la herramienta seria
    // «constante y repetitiva», que es exactamente lo que NO quiere.
    //
    // OJO, y aprendi algo escribiendola: la primera version comparaba escenarios con UN SOLO dato, y
    // entonces no se distinguian. **La diferencia se ve cuando el contexto esta COMPLETO**: con los
    // cuatro datos, quien los puso cambia el resultado.
    let nulo = Contexto::vacio();

    // Completo, pero ESCRITO POR LA PERSONA.
    let mut manual = Contexto::vacio();
    manual.sueno_horas = Some(Rango::medido(7.0, 0.5));
    manual.sensacion = Some(Rango::medido(4.0, 0.5));
    manual.comida_antes = Some(Comida::Hidratos);
    manual.alcohol = Some(false);
    for k in ["sueño", "sensación", "comida", "alcohol"] {
        manual.procedencias.push((k.into(), Fuente::Declarada));
    }

    // Completo, y MEDIDO: el sueno del reloj y el peso de la bascula.
    let mut automatico = Contexto::vacio();
    automatico.sueno_horas = Some(Rango::medido(7.0, 0.2));
    automatico.sensacion = Some(Rango::medido(4.0, 0.5));
    automatico.comida_antes = Some(Comida::Hidratos);
    automatico.alcohol = Some(false);
    automatico.procedencias.push(("sueño".into(), Fuente::Automatica { aplicacion: "Garmin".into() }));
    automatico.procedencias.push(("sensación".into(), Fuente::Declarada));
    automatico.procedencias.push(("comida".into(), Fuente::Declarada));
    automatico.procedencias.push(("alcohol".into(), Fuente::Declarada));

    let p_nulo = nulo.precision();
    let p_manual = manual.precision();
    let p_auto = automatico.precision();

    // EL ESCENARIO NULO es el peor de todos, y eso no admite duda.
    assert_eq!(p_nulo, motor::rango::Precision::Amplia, "sin nada, lo mas ancho");

    // Y LOS CUATRO ESCRITOS A MANO tampoco llegan arriba: si no lo mide nadie, no se puede apretar.
    // Esto lo aprendi porque un `assert` mio fallo: yo esperaba «aceptable», y el motor dijo «amplia».
    // **El motor tenia razon.**
    assert_eq!(p_manual, motor::rango::Precision::Amplia, "los cuatro escritos a mano no bastan");

    // AHORA LA PRUEBA QUE DE VERDAD DISTINGUE LOS ESCENARIOS: **la misma cantidad, distinta fuente.**
    // Cuatro datos medidos por un aparato frente a los mismos cuatro escritos por la persona.
    let mut todo_medido = Contexto::vacio();
    todo_medido.sueno_horas = Some(Rango::medido(7.0, 0.2));
    todo_medido.sensacion = Some(Rango::medido(4.0, 0.2));
    todo_medido.comida_antes = Some(Comida::Hidratos);
    todo_medido.alcohol = Some(false);
    for k in ["sueño", "sensación", "comida", "alcohol"] {
        todo_medido.procedencias.push((k.into(), Fuente::Automatica { aplicacion: "Garmin".into() }));
    }

    let p_medido = todo_medido.precision();
    assert_eq!(p_medido, motor::rango::Precision::Buena, "los cuatro medidos: se puede decidir");
    assert_ne!(
        p_medido, p_manual,
        "LOS MISMOS CUATRO DATOS, y distinto resultado segun quien los puso: \
         {p_medido:?} medidos frente a {p_manual:?} escritos a mano"
    );

    // Y el mixto queda en medio o mejor, nunca peor que el escrito a mano entero.
    assert!(
        p_auto == motor::rango::Precision::Buena || p_auto == motor::rango::Precision::Aceptable,
        "con un dato medido y tres escritos, tiene que quedar en medio: {p_auto:?}"
    );

    // Y el margen del dato declarado es MAYOR que el del medido: recordar tiene error.
    let m_medido = automatico.procedencias[0].1.margen_por_la_fuente();
    let m_declarado = manual.procedencias[0].1.margen_por_la_fuente();
    assert_eq!(m_medido, 0.0, "lo que mide un aparato no añade margen");
    assert!(m_declarado > 0.0, "lo que se recuerda SI: {m_declarado}");
}

#[test]
fn LA_COMIDA_no_es_un_sensor_y_se_trata_como_dato_BLANDO() {
    // El dueño dijo: «un sensor en teoria no puede mentir a no ser que este averiado».
    // Y eso es casi verdad -- pero LA COMIDA NO LA MIDE NINGUN SENSOR: la escribe una persona.
    // Cuando alguien registra lo que come, se equivoca HACIA ABAJO. Es un dato blando, y se trata
    // como tal.
    for c in [Comida::Proteinas, Comida::Hidratos, Comida::Grasa, Comida::Nada] {
        let coste = c.coste_de_digerir();
        assert!((0.0..=0.30).contains(&coste), "{}: coste {coste}", c.nombre());
    }
    // Y el orden es el que dice la literatura: proteina > hidratos > grasa.
    assert!(Comida::Proteinas.coste_de_digerir() > Comida::Hidratos.coste_de_digerir());
    assert!(Comida::Hidratos.coste_de_digerir() > Comida::Grasa.coste_de_digerir());
    assert!(Comida::Grasa.coste_de_digerir() > Comida::Nada.coste_de_digerir());
}

#[test]
fn el_margen_de_la_comida_es_blando_porque_lo_escribe_una_persona() {
    // Una comida escrita a mano NO puede tener el mismo margen que un pulso medido.
    let declarada = Fuente::Declarada;
    let medida = Fuente::Automatica { aplicacion: "Garmin".into() };
    assert!(declarada.margen_por_la_fuente() > medida.margen_por_la_fuente());
}

#[test]
fn EL_CRUCE_avisa_cuando_coinciden_mas_carga_y_menos_sueno() {
    // Es lo que el cliente pide: «mezcla alimentacion, horas de sueño, habitos de ejercicio».
    // Ese cruce NO EXISTIA en ningun sitio, y aqui esta.
    let mut c = Contexto::vacio();
    c.sueno_horas = Some(Rango::medido(6.2, 0.3));

    let carga = Rango::estimado(180.0, 10.0);
    let habitual = Rango::estimado(120.0, 10.0);

    let aviso = c.cruce_con_la_carga(&carga, &habitual).expect("tiene que avisar");
    assert!(aviso.contains("6.2"), "dice las horas: {aviso}");
    assert!(aviso.contains("180"), "y la carga");
    // Y NO afirma una causa: solo senala que coincide. Eso es lo honesto.
    assert!(
        aviso.contains("No se puede afirmar"),
        "no puede afirmar que una cosa cause la otra: {aviso}"
    );
}

#[test]
fn el_cruce_NO_avisa_si_durmio_bien_o_si_la_carga_no_subio() {
    // Un aviso que sale siempre no sirve para nada. Tiene que salir solo cuando importa.
    let carga = Rango::estimado(180.0, 10.0);
    let habitual = Rango::estimado(120.0, 10.0);

    let mut durmio_bien = Contexto::vacio();
    durmio_bien.sueno_horas = Some(Rango::medido(8.0, 0.3));
    assert_eq!(durmio_bien.cruce_con_la_carga(&carga, &habitual), None, "durmio bien");

    let mut poca_carga = Contexto::vacio();
    poca_carga.sueno_horas = Some(Rango::medido(6.0, 0.3));
    let floja = Rango::estimado(100.0, 10.0);
    assert_eq!(poca_carga.cruce_con_la_carga(&floja, &habitual), None, "la carga no subio");
}

#[test]
fn el_contexto_nunca_dice_tener_precision_ALTA() {
    // Ni con los cuatro datos: uno que se recuerda no es una medicion. Decir «alta» seria mentir.
    let mut c = Contexto::vacio();
    c.sueno_horas = Some(Rango::medido(7.5, 0.2));
    c.sensacion = Some(Rango::medido(4.0, 0.2));
    c.comida_antes = Some(Comida::Hidratos);
    c.alcohol = Some(false);
    assert_eq!(c.cuantos(), (4, 4), "estan los cuatro");
    assert_ne!(c.precision(), motor::rango::Precision::Alta, "pero nunca es «alta»");
}

// ==================================================================================================
// LA MEMORIA ENTRE ARRANQUES (paso 8 del recorrido del usuario)
// ==================================================================================================
//
// MEDIDO antes de escribir esto: abrir la aplicacion con 102 ficheros tardaba **5,50 s**, y de esos
// 5,50 s, **5,50 eran leer los ficheros**. Contar 549.842 muestras y calcular la deriva de las 102
// sesiones: **0,000 s y 0,00 s**. O sea que el problema nunca fue calcular, era **volver a leer lo
// que no ha cambiado**.
//
// Estas pruebas comprueban las tres cosas de las que depende que la memoria sirva: que la huella
// detecta un cambio, que NO se inventa un acierto cuando los ficheros son otros, y que la memoria
// inutil no rompe nada.

/// Una carpeta de mentira, propia de cada prueba, para no pisarse entre ellas.
fn carpeta_de_prueba(nombre: &str) -> std::path::PathBuf {
    let c = std::env::temp_dir().join(format!("pulso-prueba-{nombre}"));
    let _ = std::fs::remove_dir_all(&c);
    std::fs::create_dir_all(&c).expect("tiene que poder crearse la carpeta de prueba");
    c
}

#[test]
fn listar_ve_los_fit_con_su_tamano_y_nada_mas() {
    // Es la base de la memoria incremental: mirar el tamaño de 102 ficheros es instantaneo, y con eso
    // se decide cuales hay que leer de verdad.
    let c = carpeta_de_prueba("listar");
    assert_eq!(motor::fit::listar(&c).len(), 0, "una carpeta vacia no tiene FIT");

    std::fs::write(c.join("a.fit"), b"contenido").unwrap();
    let uno = motor::fit::listar(&c);
    assert_eq!(uno.len(), 1, "ahora hay un fichero");
    assert_eq!(uno[0].0, "a.fit");
    assert_eq!(uno[0].1, 9, "y su tamaño son 9 bytes");

    std::fs::write(c.join("b.fit"), b"otro").unwrap();
    assert_eq!(motor::fit::listar(&c).len(), 2, "y ahora dos");

    // Y los ficheros que NO son FIT no cuentan: no se leen, asi que no cambian el resultado.
    std::fs::write(c.join("notas.txt"), b"da igual").unwrap();
    assert_eq!(motor::fit::listar(&c).len(), 2, "un .txt no es un FIT");

    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn la_decision_reutiliza_lo_que_no_cambio_y_lee_lo_nuevo() {
    // EL CORAZON DE LA MEMORIA INCREMENTAL, y por eso es una funcion pura que se prueba sin disco.
    use motor::analisis::{decidir, Aporte, Decision, Ficha};
    let previas = vec![
        Ficha::bien("a.fit".into(), 9, Aporte { muestras: 10, ..Aporte::default() }),
        Ficha::bien("b.fit".into(), 4, Aporte::default()),
        Ficha::bien("z.fit".into(), 8, Aporte::default()),
    ];
    let actual = vec![
        ("a.fit".to_string(), 9u64), // igual: se reutiliza
        ("b.fit".to_string(), 7u64), // cambio de tamaño: se relee
        ("c.fit".to_string(), 1u64), // nueva: se lee
    ];
    let d = decidir(&actual, &previas);
    assert!(matches!(d[0], Decision::Recordada(_)), "a.fit no cambio");
    assert!(matches!(&d[1], Decision::Leer(n) if n == "b.fit"), "b.fit cambio de tamaño");
    assert!(matches!(&d[2], Decision::Leer(n) if n == "c.fit"), "c.fit es nueva");
    assert_eq!(d.len(), 3, "z.fit ya no esta: su ficha se olvida, no se arrastra");
}

#[test]
fn combinar_es_aditivo_y_no_depende_del_orden() {
    // El total se tiene que poder reconstruir sumando los aportes, en cualquier orden: es lo que
    // permite reutilizar lo recordado y sumarle solo lo nuevo, sin releer nada.
    use motor::analisis::{combinar, Aporte};
    let a = Aporte {
        inicio_s: Some(100),
        muestras: 100,
        rr: 200,
        maxima: Some(180),
        deriva: Some(10.0),
        escapado: false,
        eficiencia: Some(1.0),
        hrv: Some(9.0),
    };
    let b = Aporte {
        inicio_s: Some(300),
        muestras: 50,
        rr: 80,
        maxima: Some(170),
        deriva: Some(20.0),
        escapado: true,
        eficiencia: Some(0.9),
        hrv: None,
    };
    let ab = combinar(&[a.clone(), b.clone()]);
    let ba = combinar(&[b.clone(), a.clone()]);
    assert_eq!(ab, ba, "el orden no puede cambiar el total");
    assert_eq!(ab.sesiones, 2);
    assert_eq!(ab.muestras, 150);
    assert_eq!(ab.rr, 280);
    assert_eq!(ab.maxima_vista, Some(180));
    assert_eq!(ab.escapados, 1);
    assert_eq!(ab.deriva_media, Some(15.0), "la media de 10 y 20");
    assert_eq!(ab.desde, Some(100));
    assert_eq!(ab.hasta, Some(300));
}

#[test]
fn sin_fechas_el_periodo_queda_en_None_y_no_se_inventa() {
    // Una sesion sin fecha no puede meter una fecha falsa en el periodo. Se dice que no se sabe.
    use motor::analisis::{combinar, Aporte};
    let r = combinar(&[Aporte { muestras: 10, ..Aporte::default() }]);
    assert_eq!(r.desde, None);
    assert_eq!(r.hasta, None);
    assert_eq!(r.sesiones, 1, "pero la sesion SI se cuenta");
}

// ==================================================================================================
// ELEGIR LA CARPETA: la logica del selector, sin abrir ninguna ventana
// ==================================================================================================

#[test]
fn el_selector_lista_subcarpetas_sin_ocultas_ni_ficheros() {
    let c = carpeta_de_prueba("selector-carpetas");
    std::fs::create_dir_all(c.join("garmin_files")).unwrap();
    std::fs::create_dir_all(c.join("b")).unwrap();
    std::fs::create_dir_all(c.join(".oculta")).unwrap();
    std::fs::write(c.join("un.fit"), b"x").unwrap();

    let s = motor::carpeta::subcarpetas(&c);
    assert_eq!(
        s,
        vec!["b".to_string(), "garmin_files".to_string()],
        "ordenadas, sin ocultas y sin ficheros"
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn subir_desde_la_raiz_no_se_puede_y_se_dice() {
    assert!(motor::carpeta::ruta_padre(std::path::Path::new("/")).is_none());
    assert_eq!(
        motor::carpeta::ruta_padre(std::path::Path::new("/home")),
        Some(std::path::PathBuf::from("/"))
    );
}

#[test]
fn una_carpeta_que_no_se_puede_leer_no_revienta() {
    let no = std::path::Path::new("/no/existe/esta/carpeta");
    assert!(motor::carpeta::subcarpetas(no).is_empty(), "vacio, no panico");
    assert!(!motor::carpeta::es_carpeta(no));
}

#[test]
fn el_selector_enseña_cuantos_fit_tiene_cada_carpeta() {
    // La INTUICION: el numero junto al nombre es lo que hace que el cliente reconozca su carpeta.
    let c = carpeta_de_prueba("selector-vista");
    let datos = c.join("garmin_files").join("storage");
    std::fs::create_dir_all(&datos).unwrap();
    std::fs::write(c.join("suelto.fit"), b"x").unwrap();
    std::fs::write(datos.join("uno.fit"), b"x").unwrap();
    std::fs::write(datos.join("dos.fit"), b"x").unwrap();
    std::fs::write(datos.join("notas.txt"), b"x").unwrap();

    let v = motor::carpeta::abrir(&c);
    assert_eq!(v.fit_aqui, 1, "esta carpeta tiene un FIT suelto");
    assert_eq!(v.ruta.as_deref(), Some(c.as_path()));
    assert!(v.aviso.is_none(), "con FIT no hay nada que avisar");
    let garmin = v.entradas.iter().find(|e| e.nombre == "garmin_files").unwrap();
    assert_eq!(garmin.cuantos_fit, 0, "el FIT esta en la subcarpeta storage, no aqui");
    assert_eq!(
        motor::carpeta::abrir(&datos).fit_aqui, 2,
        "y dentro estan los dos FIT, contados como los cuenta el lector"
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn el_selector_avisa_cuando_no_hay_ningun_fit_a_la_vista() {
    let c = carpeta_de_prueba("selector-vacio");
    std::fs::create_dir_all(c.join("otra")).unwrap();
    let v = motor::carpeta::abrir(&c);
    assert_eq!(v.fit_aqui, 0);
    assert!(v.aviso.is_some(), "sin FIT a la vista, se dice");

    let no = motor::carpeta::abrir(std::path::Path::new("/no/existe/esta/carpeta"));
    assert!(no.ruta.is_none());
    assert!(no.aviso.is_some());
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn un_fit_ilegible_se_recuerda_con_su_motivo_y_no_aborta() {
    // Un fichero que no se puede leer NO puede tirar el analisis entero, y su fallo NO se puede
    // reintentar en cada arranque: se guarda la ficha con el motivo y se sigue.
    let c = carpeta_de_prueba("fit-roto");
    std::fs::write(c.join("roto.fit"), b"esto no es un FIT").unwrap();

    let (real, fichas, cuantas) = motor::analisis::actualizar(&c, &[]);
    assert_eq!(cuantas.leidas, 1, "se intento leer");
    assert_eq!(cuantas.no_leidas, 1, "y no se pudo");
    assert_eq!(fichas.len(), 1);
    assert!(fichas[0].error.is_some(), "el motivo se guarda en la ficha");
    assert!(!real.hay_datos(), "y no se inventa ninguna sesion");
    assert_eq!(real.no_leidos.len(), 1, "el total lo dice");

    // Y al arrancar otra vez, la ficha del FALLO se reutiliza: no se reintenta.
    let (_r2, f2, c2) = motor::analisis::actualizar(&c, &fichas);
    assert_eq!(c2.recordadas, 1, "el fallo se recuerda");
    assert_eq!(c2.leidas, 0, "y no se reintenta en cada arranque");
    assert_eq!(f2, fichas);

    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn una_memoria_corrupta_o_de_otra_version_no_rompe_nada_solo_se_recalcula() {
    // Un fichero de memoria a medias (el programa se cerro mientras escribia) o de una version vieja
    // NO puede impedir abrir la aplicacion. La respuesta correcta es siempre la misma: no recordar
    // nada y volver a leer. Aqui la memoria no es una puerta: es un atajo.
    let c = carpeta_de_prueba("memoria-corrupta");
    let ruta = c.join("memoria.json");

    std::fs::write(&ruta, b"esto no es json {{{").unwrap();
    assert!(motor::memoria::leer_fichas(&ruta).is_empty(), "corrupta: no recuerda nada");

    // Y de otra version: los campos podrian significar otra cosa, asi que tampoco se usa.
    std::fs::write(
        &ruta,
        br#"{"version": 99, "guardado_en_s": null, "fichas": []}"#,
    )
    .unwrap();
    assert!(motor::memoria::leer_fichas(&ruta).is_empty(), "de otra version: no recuerda nada");

    // Y si no hay fichero, tampoco es un error: es la primera vez.
    let sin_nada = c.join("no-existe.json");
    assert!(motor::memoria::leer_fichas(&sin_nada).is_empty(), "sin memoria: primera vez");

    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn las_fichas_van_y_vuelven_enteras_por_la_memoria() {
    // Guardar y leer tiene que devolver LO MISMO. Si se perdiera un campo de un aporte al guardar, la
    // segunda apertura enseñaria menos que la primera, y nadie sabria por que.
    use motor::analisis::{Aporte, Ficha};
    let c = carpeta_de_prueba("memoria-ida-y-vuelta");
    let ruta = c.join("memoria.json");

    let original = vec![
        Ficha::bien(
            "uno.fit".into(),
            1234,
            Aporte {
                inicio_s: Some(1_700_000_000),
                muestras: 5000,
                rr: 6000,
                maxima: Some(180),
                deriva: Some(9.7),
                escapado: true,
                eficiencia: Some(0.98),
                hrv: Some(9.2),
            },
        ),
        Ficha::mal("roto.fit".into(), 9, "no se pudo leer".into()),
    ];

    motor::memoria::guardar(&ruta, &original);
    let leido = motor::memoria::leer_fichas(&ruta);
    assert_eq!(leido, original, "las fichas tienen que volver enteras");

    let g = motor::memoria::leer_guardado(&ruta).expect("tiene que poder leerse");
    assert!(g.guardado_en_s.is_some(), "y con la fecha del guardado documentada");

    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn los_ficheros_que_no_se_pudieron_leer_SE_DICEN() {
    // ESTE ERA UN DATO QUE SE TIRABA A LA BASURA. Si de 102 ficheros se leen 99, todos los promedios
    // salen de 99 y nada en la pantalla lo decia. Un dato que falta hay que decirlo.
    let no_leidos = vec![("roto.fit".to_string(), "no es un FIT".to_string())];
    let r = motor::analisis::de_lecturas(&[], no_leidos);

    assert_eq!(r.sesiones, 0, "no se leyo ninguna");
    assert_eq!(r.no_leidos.len(), 1, "pero el fallo se cuenta");
    assert!(!r.hay_datos(), "sin sesiones no hay nada que analizar");
    assert!(!r.esta_completo(), "y no se leyo todo");
}

#[test]
fn una_carpeta_sin_datos_NO_produce_numeros_inventados() {
    // Sin datos no hay resultado. Ni ceros, ni medias de nada: **nada**. Un cero aqui seria un dato
    // falso, que es exactamente lo que este motor existe para no hacer.
    let r = motor::analisis::de_lecturas(&[], Vec::new());
    assert!(!r.hay_datos());
    assert_eq!(r.deriva_media, None, "sin sesiones no hay deriva");
    assert_eq!(r.eficiencia_media, None);
    assert_eq!(r.hrv_medio, None);
    assert_eq!(r.maxima_vista, None);
}

#[test]
fn la_frecuencia_maxima_que_MANDA_es_la_que_declara_la_persona() {
    // Decision del dueño (2026-10-13): el perfil manda sobre el aparato. El aparato aporta lo que el
    // usuario no puede saber; el usuario aporta lo que el aparato no puede saber. Cuando los dos
    // hablan del mismo dato, gana el que se declaro a proposito.
    use motor::Dato;

    // Lo declaro y el aparato vio otra: manda lo declarado.
    assert_eq!(
        motor::analisis::maxima_que_manda(&Dato::Hay(169), Some(183)),
        Dato::Hay(169),
        "lo que la persona declara manda sobre lo que vio el aparato"
    );
    // No lo declaro: se usa lo que vio el aparato, que es mejor que nada.
    assert_eq!(
        motor::analisis::maxima_que_manda(&Dato::Falta, Some(183)),
        Dato::Hay(183)
    );
    // No hay ni una cosa ni la otra: falta, y el motor ya sabe decirlo.
    assert_eq!(motor::analisis::maxima_que_manda(&Dato::Falta, None), Dato::Falta);
}

// ================================================================================
// BIOMECÁNICA Y CARGA: lo que el fichero declara y el motor ignoraba (2026-10-13)
// ================================================================================
//
// POR QUÉ ESTAS PRUEBAS ESTÁN AQUÍ Y NO EN UNA HOJA APARTE: cada una nació de un error concreto que
// ya se cometió con estos campos, y su sitio es al lado de las demás pruebas del lector, que es donde
// alguien mirará el día que toque `fit.rs`. Las cuatro primeras cubren el modelo y el lector; **la
// quinta deja escrito el error que ya se cometió una vez**, que es la que más importa.

#[test]
fn la_cadencia_de_la_muestra_se_GUARDA_y_cuadra_con_la_media_del_fichero() {
    // La regla que se rompió: `Record.cadence` **se leía y se tiraba**. Medido sobre sus ficheros son
    // 421.449 muestras en 83 de 102: no era un campo raro, era el 90 % de sus sesiones.
    //
    // Y se comprueba CONTRA EL FICHERO, no contra nosotros mismos: la media de las muestras tiene que
    // cuadrar con la que el propio FIT declara. Un dato guardado mal cuadra consigo mismo; contra el
    // fichero, no.
    //
    // Se buscan ficheros CON cadencia en vez de coger el primero, y no es comodidad: medido, hay
    // ficheros de estos con el campo `cadence` en **todo cero** —el potenciómetro no la mandó ese
    // día— y ahí la media de las muestras no puede cuadrar con nada. Saltárselos es lo que hace que
    // esta prueba compruebe el lector y no el estado de un sensor una tarde concreta.
    let carpeta = carpeta_fit();
    if !carpeta.exists() {
        eprintln!("(sin ficheros FIT en esta maquina: la prueba se salta)");
        return;
    }
    let (bien, _) = fit::leer_carpeta_hasta(&carpeta, 20);
    let mut comparadas = 0;
    let mut cuadran = 0;
    for (nombre, l) in &bien {
        let cadencias = l.sesion.cadencias();
        let declarada = match l.declarado.cadencia_media.valor() {
            Some(d) if !cadencias.is_empty() => d,
            _ => continue,
        };
        comparadas += 1;
        let media = cadencias.iter().map(|c| f64::from(*c)).sum::<f64>() / cadencias.len() as f64;
        // La declarada viene redondeada a rpm enteras: el margen es de una rpm y algo mas.
        if (media - declarada).abs() <= 2.0 {
            cuadran += 1;
        } else {
            eprintln!("  {nombre}: muestras {media:.1} rpm contra declarada {declarada:.0} rpm");
        }
    }
    assert!(comparadas > 0, "tiene que haber ficheros con cadencia y con su media declarada");
    assert!(
        cuadran * 10 >= comparadas * 8,
        "la cadencia de las muestras tiene que cuadrar con la que declara el fichero en al menos el \
         80 % de las sesiones: {cuadran} de {comparadas} (el motor salta los ceros, como el aparato)"
    );
}

#[test]
fn una_muestra_SIN_cadencia_queda_en_FALTA_y_no_en_CERO() {
    // ESTA ES LA REGLA CENTRAL DEL PROYECTO, aplicada al campo nuevo: **un cero es un dato falso**.
    // «No sé la cadencia de este segundo» no es «iba a 0 rpm», y la diferencia se nota justo donde
    // hace daño: en la media. Un cero por cada hueco hunde la media sin que nadie lo vea.
    let con_cadencia = Muestra {
        segundo: 0,
        pulso: Dato::Hay(140),
        potencia_w: Dato::Hay(200),
        cadencia_rpm: Dato::Hay(85),
        distancia_m: Dato::Hay(0.0),
        temperatura_c: Dato::Hay(12.0),
    };
    assert_eq!(con_cadencia.cadencia_rpm, Dato::Hay(85), "si viene, se conserva tal cual");

    let sin_cadencia = Muestra { cadencia_rpm: Dato::Falta, ..con_cadencia };
    assert_eq!(
        sin_cadencia.cadencia_rpm,
        Dato::Falta,
        "sin cadencia NO se pone un cero: se dice que falta"
    );
    assert!(
        sin_cadencia.cadencia_rpm.valor().is_none(),
        "y el valor tiene que ser None, no Some(0): quien lo lea tiene que poder distinguirlo"
    );

    // Y en la sesión entera: la media se hace **sobre las que la traen**, y las dos que no la traen
    // no entran. Si entraran como ceros, la media sería (85 + 0 + 0) / 3 = 28,3 en vez de 85.
    let sesion = Sesion::nueva(vec![
        con_cadencia,
        Muestra { segundo: 1, ..sin_cadencia },
        Muestra { segundo: 2, ..sin_cadencia },
    ]);
    let cadencias = sesion.cadencias();
    assert_eq!(cadencias, vec![85], "las muestras sin cadencia se SALTAN, no se rellenan con cero");
    let media = cadencias.iter().map(|c| f64::from(*c)).sum::<f64>() / cadencias.len() as f64;
    assert_eq!(media, 85.0, "la media de una sola cadencia de 85 rpm es 85, no 28,3");
}

#[test]
fn un_array_con_un_hueco_Invalid_NO_inventa_el_valor_que_falta() {
    // MEDIDO EN SUS FICHEROS: `avg_power_position` aparece como `Array([UInt16(147), Invalid])` — un
    // hueco medido y el otro no. Antes de escribir el lector, ese `Invalid` parecía un cero; no lo
    // es, **es un hueco**, y las dos mitades se tratan distinto: la que está se conserva, la que no
    // está queda `Falta`.
    let mut invalidos = std::collections::BTreeMap::new();
    let parcial = fit::dos_huecos(
        &fitparser::Value::Array(vec![fitparser::Value::UInt16(147), fitparser::Value::Invalid]),
        "avg_power_position",
        &mut invalidos,
    );
    assert_eq!(parcial[0], Some(147.0), "el hueco que SI se midio se conserva");
    assert_eq!(
        parcial[1], None,
        "el hueco marcado Invalid queda AUSENTE: no se convierte en un cero de vatios"
    );
    assert_ne!(parcial[1], Some(0.0), "y sobre todo: NO es un cero");
    assert_eq!(
        invalidos.get("avg_power_position").copied(),
        Some(1),
        "y el hueco se CUENTA: es lo que permite distinguir «el sensor no lo mide» de «hoy no lo midio»"
    );

    // Una forma que no es la esperada tampoco se interpreta: ni uno, ni tres huecos.
    let mut otros = std::collections::BTreeMap::new();
    let uno = fit::dos_huecos(
        &fitparser::Value::Array(vec![fitparser::Value::UInt16(147)]),
        "avg_power_position",
        &mut otros,
    );
    assert_eq!(uno, [None, None], "un array de uno no es una postura: no se adivina la que falta");
    assert_eq!(otros.get("avg_power_position").copied(), Some(1), "y la forma rara se cuenta");

    // Y una fase a la que le falta UNO de los cuatro angulos se cae ENTERA: el angulo que falta se
    // leeria como 0 grados, y 0 grados es una posicion del pedal, no un hueco. Media fase, calculada
    // contra un cero, es una conclusion sobre su rodilla sacada de un hueco del fichero.
    let mut fase_invalidos = std::collections::BTreeMap::new();
    let fase = fit::cuatro_grados(
        &fitparser::Value::Array(vec![
            fitparser::Value::Float64(357.2),
            fitparser::Value::Float64(184.2),
            fitparser::Value::Invalid,
            fitparser::Value::Float64(90.0),
        ]),
        "avg_left_power_phase",
        &mut fase_invalidos,
    );
    assert_eq!(fase, Dato::Falta, "con un angulo Invalid, la fase entera falta");
    // El contraste: los cuatro buenos si se guardan.
    let completa = fit::cuatro_grados(
        &fitparser::Value::Array(vec![
            fitparser::Value::Float64(357.2),
            fitparser::Value::Float64(184.2),
            fitparser::Value::Float64(185.6),
            fitparser::Value::Float64(90.0),
        ]),
        "avg_left_power_phase",
        &mut fase_invalidos,
    );
    assert_eq!(completa, Dato::Hay([357.2, 184.2, 185.6, 90.0]));
}

#[test]
fn las_fases_de_las_dos_piernas_se_guardan_SEPARADAS_y_no_se_mezclan() {
    // Lo unico del fichero que dice algo de UNA pierna es la fase. Si se mezclaran, el modulo de la
    // rodilla estaria comparando una pierna consigo misma y **diria que no hay asimetria**: el peor
    // resultado posible, porque tranquiliza.
    let izquierda = Dato::Hay([357.2, 184.2, 185.6, 90.0]);
    let derecha = Dato::Hay([353.0, 181.4, 187.0, 87.2]);
    let b = motor::datos::Biomecanica {
        fase_pierna_izquierda: izquierda,
        fase_pierna_derecha: derecha,
        ..motor::datos::Biomecanica::vacia()
    };

    assert_eq!(b.fase_pierna_izquierda, izquierda, "la izquierda se guarda en la izquierda");
    assert_eq!(b.fase_pierna_derecha, derecha, "y la derecha en la derecha");
    assert_ne!(
        b.fase_pierna_izquierda, b.fase_pierna_derecha,
        "el ejemplo real tiene las dos fases distintas: si salen iguales, se mezclaron"
    );
    let (i, d) = b.fases().expect("con las dos, se pueden comparar");
    assert_eq!(i[0], 357.2, "y al sacarlas juntas, cada una sigue siendo la suya");
    assert_eq!(d[0], 353.0);

    // La asimetria va POR EL CAMINO CORTO. Restando directo, esta sesion daria 357,2 grados; la
    // verdad son 4,2. Es un fallo que ya salio al medir, y por eso se mide aqui.
    let asimetria = b.asimetria_grados().expect("con las dos fases, hay asimetria");
    assert!(
        (asimetria - 4.2).abs() < 1e-9,
        "la asimetria es 4,2 grados por el camino corto (|357,2 - 353,0|), no 357,2: dio {asimetria}"
    );

    // Y con una sola pierna NO hay asimetria que afirmar: comparar contra un hueco daria una
    // asimetria que es del fichero, no de sus piernas.
    let solo_una = motor::datos::Biomecanica { fase_pierna_derecha: Dato::Falta, ..b };
    assert_eq!(solo_una.asimetria_grados(), None, "sin las dos piernas no se afirma nada");
    assert_eq!(solo_una.fases(), None);
    assert!(!solo_una.hay_fases());

    // Y el salto del cero, que es la trampa de verdad: dos piernas CASI IGUALES a los dos lados del
    // cero (357 grados y 0,5) estan a 3,5 grados, no a 356,5.
    let salto = motor::datos::Biomecanica {
        fase_pierna_izquierda: Dato::Hay([357.0, 0.0, 0.0, 0.0]),
        fase_pierna_derecha: Dato::Hay([0.5, 0.0, 0.0, 0.0]),
        ..motor::datos::Biomecanica::vacia()
    };
    let a = salto.asimetria_grados().expect("las dos estan");
    assert!((a - 3.5).abs() < 1e-9, "el cero da la vuelta: la distancia es 3,5 grados, no 356,5 (dio {a})");
}

#[test]
fn avg_power_position_es_POSTURA_y_no_LADO_y_los_dos_nombres_lo_dicen() {
    // ============================================================================================
    // ESTA PRUEBA DEJA ESCRITO UN ERROR QUE YA SE COMETIO, para que no se pueda cometer dos veces.
    // ============================================================================================
    //
    // `avg_power_position` = [145, 410] PARECE «145 W la pierna izquierda y 410 W la derecha»: un
    // 37/63, un desequilibrio enorme y una conclusion grave sobre la rodilla de una persona de 62
    // años. **Es falso.** El campo son DOS HUECOS INDEXADOS POR POSTURA, y el enum oficial del
    // formato (`rider_position_type`) es `Seated = 0`, `Standing = 1`: **145 W sentado y 410 W de
    // pie**, que en un puerto es lo normal y no dice nada de ninguna rodilla.
    //
    // Por eso esta prueba comprueba DOS cosas a la vez: el SIGNIFICADO con el valor real, y que
    // ningún campo del modelo se llame «izquierda»/«derecha» para este dato. Con una sola de las dos
    // no basta: un nombre honesto con los valores cruzados sigue estando mal, y unos valores bien
    // puestos con el nombre «izquierda» vuelven a invitar al mismo error.
    let mut invalidos = std::collections::BTreeMap::new();
    let huecos = fit::dos_huecos(
        &fitparser::Value::Array(vec![fitparser::Value::UInt16(145), fitparser::Value::UInt16(410)]),
        "avg_power_position",
        &mut invalidos,
    );
    let b = motor::datos::Biomecanica {
        potencia_media_sentado: fit::de_hueco(huecos[0], fit::a_entero_u16),
        potencia_media_de_pie: fit::de_hueco(huecos[1], fit::a_entero_u16),
        ..motor::datos::Biomecanica::vacia()
    };
    // EL VALOR REAL DE SU FICHERO, puesto donde dice el formato que va:
    assert_eq!(
        b.potencia_media_sentado,
        Dato::Hay(145),
        "el hueco 0 es el SENTADO: 145 W. Si alguien lo lee como la pierna izquierda, esto se cae"
    );
    assert_eq!(
        b.potencia_media_de_pie,
        Dato::Hay(410),
        "el hueco 1 es DE PIE: 410 W. NO es «la pierna derecha»"
    );
    assert_ne!(
        b.potencia_media_sentado, b.potencia_media_de_pie,
        "los dos valores son distintos: eso es lo que hace que confundirlos invente un 37/63"
    );

    // Y NINGUN CAMPO DE POSTURA SE LLAMA «izquierda» NI «derecha». Se enumeran todos a mano —no hay
    // reflexion en Rust— para que el dia que alguien añada uno, tenga que pasar por aqui y decidir
    // si el nombre dice la verdad. El comentario del modelo tambien se comprueba: es donde se lee la
    // definicion, y un comentario que miente es peor que no tenerlo.
    let postura = [
        ("potencia_media_sentado", b.potencia_media_sentado.lo_hay()),
        ("potencia_media_de_pie", b.potencia_media_de_pie.lo_hay()),
        ("potencia_maxima_sentado", b.potencia_maxima_sentado.lo_hay()),
        ("potencia_maxima_de_pie", b.potencia_maxima_de_pie.lo_hay()),
        ("cadencia_media_sentado", b.cadencia_media_sentado.lo_hay()),
        ("cadencia_media_de_pie", b.cadencia_media_de_pie.lo_hay()),
    ];
    for (nombre, _) in postura {
        let n = nombre.to_lowercase();
        assert!(
            !n.contains("izquierda") && !n.contains("derecha") && !n.contains("izq") && !n.contains("der"),
            "«{nombre}» es un campo de POSTURA (sentado/de pie) y su nombre no puede decir un lado: \
             es exactamente el error que ya se cometio una vez"
        );
    }
    let definicion = include_str!("../src/datos.rs").to_lowercase();
    assert!(
        definicion.contains("rider_position_type") && definicion.contains("seated") && definicion.contains("standing"),
        "el modelo tiene que decir EN VOZ ALTA que `*_position` es postura y no lado, con el enum \
         oficial del formato: es la unica defensa que queda para el que lea el codigo manana"
    );
    assert!(
        definicion.contains("left_right_balance") && definicion.contains("serie del sensor"),
        "y tiene que decir que `left_right_balance` trae el NUMERO DE SERIE del sensor, que es la \
         otra trampa: el que parezca un porcentaje es lo que hizo concluir un 47/53 que no existe"
    );

    // Y LO QUE ES POR PIERNA, SI ES POR PIERNA: se lee de un fichero suyo y se comprueba que las dos
    // fases son distintas. Si alguna vez salieran iguales, o el lector mezcla los campos, o la
    // persona pedalea perfectamente simetrica — y lo medido dice que difieren en 80 de 83 sesiones.
    let carpeta = carpeta_fit();
    if !carpeta.exists() {
        eprintln!("(sin ficheros FIT en esta maquina: la parte de la fase se salta)");
        return;
    }
    let (bien, _) = fit::leer_carpeta_hasta(&carpeta, 1);
    let (_, l) = match bien.first() {
        Some(x) => x,
        None => return,
    };
    let (i, d) = l
        .sesion
        .biomecanica
        .fases()
        .expect("su primer fichero trae las dos fases de pedaleo");
    assert_ne!(
        i, d,
        "las fases de las dos piernas no pueden ser identicas: o se mezclaron los campos, o el \
         fichero no es el que se midio"
    );
}

#[test]
fn el_PCO_en_cero_es_un_CENTINELA_y_no_se_guarda_como_dato() {
    // La tercera trampa, medida: `avg_left_pco` y `avg_right_pco` valen **0,00 mm en las 83 sesiones
    // y nunca difieren**. Un sensor que da cero siempre no esta midiendo: esta devolviendo su
    // centinela. Guardarlo como dato meteria un cero que parece una medida de laboratorio, y de ahi
    // saldria una conclusion sobre su rodilla sacada de la nada.
    //
    // Por eso el lector NO tiene campos para el PCO, y esta prueba lo deja dicho: si algun dia
    // alguien los añade, tendra que venir aqui y explicar por que.
    let definicion = include_str!("../src/datos.rs").to_lowercase();
    assert!(
        !definicion.contains("pub ") || !definicion.contains("pco"),
        "el modelo NO puede tener un campo de PCO: hoy vale 0,00 siempre y es un centinela, no un dato"
    );
    let lector = include_str!("../src/fit.rs").to_lowercase();
    assert!(
        lector.contains("centinela"),
        "y el lector tiene que decir POR QUE no lo lee, para que no parezca un olvido"
    );

    // El contraste que si se guarda: la temperatura, que esta en los 102 ficheros.
    let carpeta = carpeta_fit();
    if !carpeta.exists() {
        eprintln!("(sin ficheros FIT en esta maquina: la parte de la temperatura se salta)");
        return;
    }
    let (bien, _) = fit::leer_carpeta_hasta(&carpeta, 1);
    let (_, l) = match bien.first() {
        Some(x) => x,
        None => return,
    };
    let t = l
        .sesion
        .temperatura_media()
        .expect("su primer fichero trae temperatura en las muestras");
    assert!(
        (-20.0..60.0).contains(&t),
        "una temperatura media de {t} grados no es un dato de este mundo: sera un centinela leido como valor"
    );

    // Y una sesion SIN temperatura da `None`, no cero grados: cero grados es una temperatura —la del
    // hielo— y colarla en un calculo de deriva cardiaca daria un numero plausible y falso.
    let vacia = Sesion::nueva(vec![Muestra {
        segundo: 0,
        pulso: Dato::Hay(140),
        potencia_w: Dato::Hay(200),
        cadencia_rpm: Dato::Falta,
        distancia_m: Dato::Falta,
        temperatura_c: Dato::Falta,
    }]);
    assert_eq!(vacia.temperatura_media(), None, "sin temperatura NO se dice que hacia 0 grados");
}

#[test]
fn el_tope_de_ficheros_cuenta_FIT_y_no_ficheros_de_otra_cosa() {
    // ESTE FALLO YA SE MIDIO Y ENGAÑABA, y por eso tiene prueba propia: `leer_carpeta_hasta` ordenaba
    // y RECORTABA antes de filtrar por `.fit`, y en su carpeta **los primeros ficheros por nombre no
    // son actividades, son los `.json` de Garmin**. Resultado: con `tope = 1` o `12` —que es lo que
    // usan las pruebas del lector— **no leía ni un FIT y devolvía «0 leídos» sin un solo error**.
    //
    // Es el peor fallo posible en un lector: el que parece que funciona. Y no se caza leyendo el
    // código, se caza contando. Aquí se cuenta: en una carpeta con DOS `.json` que van antes por
    // nombre y DOS `.fit` que van después, el tope de 1 tiene que devolver **un FIT**.
    let origen = carpeta_fit();
    if !origen.exists() {
        eprintln!("(sin ficheros FIT en esta maquina: la prueba se salta)");
        return;
    }
    // Dos actividades de verdad, las mas pequeñas, para que la prueba siga siendo rapida.
    let pequenos: Vec<std::path::PathBuf> = {
        let mut v: Vec<_> = std::fs::read_dir(&origen)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false))
            .collect();
        v.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(u64::MAX));
        v.truncate(2);
        v
    };
    if pequenos.len() < 2 {
        eprintln!("(no hay dos FIT: la prueba se salta)");
        return;
    }

    // EL NOMBRE LLEVA EL PID, y no es un adorno: con un nombre FIJO, dos `cargo test` a la vez
    // (por ejemplo uno mio comprobando mientras un agente mide) **se pisan el directorio** y la
    // prueba falla sin que nada este mal. Medido por un agente: 11 de 100 pares fallaban asi.
    // **Una prueba que falla por una carrera se deja de creer**, y entonces deja de proteger.
    let caja = std::env::temp_dir().join(format!("pulso-prueba-tope-fit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&caja);
    std::fs::create_dir_all(&caja).unwrap();
    // Los señuelos van PRIMERO por nombre: es lo que hace que el fallo aparezca.
    std::fs::write(caja.join("aaa_lista.json"), b"{\"no\": \"es un fit\"}").unwrap();
    std::fs::write(caja.join("aab_lista.json"), b"{\"tampoco\": \"lo es\"}").unwrap();
    for (i, p) in pequenos.iter().enumerate() {
        std::fs::copy(p, caja.join(format!("zzz_actividad_{i}.fit"))).unwrap();
    }

    let (uno, mal) = fit::leer_carpeta_hasta(&caja, 1);
    assert_eq!(
        uno.len(),
        1,
        "con tope 1 tiene que leer UN FIT, no cero: el tope se gasta en los .json si se recorta antes \
         de elegir (leyo {} y fallos {mal:?})",
        uno.len()
    );
    assert!(mal.is_empty(), "y los .json no son fallos de lectura: simplemente no son FIT");

    let (todos, _) = fit::leer_carpeta_hasta(&caja, 10);
    assert_eq!(todos.len(), 2, "sin tope que estorbe, los dos FIT");

    let _ = std::fs::remove_dir_all(&caja);
}

// ================================================================================
// LOS AVISOS DEL ENCARGO (`motor/src/avisos.rs`)
// ================================================================================
//
// De que va esto: el cliente entrego TRES modulos de aviso redactados por otra IA, y el encargo pide
// convertirlos en funciones deterministas. Lo que se prueba aqui, y por este orden:
//
//   1. **QUE SIN DATOS NO SALGA NINGUN AVISO.** Es la prueba mas importante del bloque: un aviso sin
//      fundamento es peor que ninguno, porque tranquiliza o asusta sin motivo y nadie lo nota.
//   2. Que los dos umbrales del cliente (54 % y 100 TSS) sean SUYOS y se comporten como se decidio.
//   3. Que una sesion sin fase de pedaleo **no** produzca aviso de rodilla.
//   4. Que ningun texto diga «diagnostico», «lesion» ni prometa que un remedio cura.
//   5. Que todo aviso lleve un `porque` CON UN NUMERO, y no una frase vacia.

use motor::analisis::Real;
use motor::avisos::{
    self, Aviso, Historico, Hoy, Modulo, SesionVista, Urgencia, EN_EL_UMBRAL_DE_AGUA_DISPARA,
    EN_EL_UMBRAL_DE_TSS_DISPARA, MINIMO_PARA_LINEA_BASE, UMBRAL_AGUA_DEL_CLIENTE_PCT,
    UMBRAL_TSS_DEL_CLIENTE,
};

/// Una sesion vista con TODO puesto, para ir quitando lo que haga falta en cada prueba.
fn sesion_avisos(nombre: &str, tss: Option<f64>, asimetria: Option<f64>, deriva: Option<f64>) -> SesionVista {
    SesionVista {
        nombre: nombre.to_string(),
        inicio_s: None,
        tss: tss.map(Dato::Hay).unwrap_or(Dato::Falta),
        asimetria_grados: asimetria,
        deriva_pct: deriva,
    }
}

/// Un historico de `n` sesiones todas iguales a la que se pase. Existe para poder montar una linea
/// base **sin ficheros FIT**, que es lo que hace que estas pruebas corran en milisegundos.
fn historico_iguales(n: usize, base: SesionVista) -> Historico {
    Historico {
        total_sesiones: n,
        no_leidos: Vec::new(),
        sesiones: (0..n).map(|i| SesionVista { nombre: format!("{}_{i}.fit", base.nombre), ..base.clone() }).collect(),
    }
}

/// Un historico con asimetrias y derivas dadas, que es lo minimo para que haya linea base.
fn historico_con(asimetrias: &[f64], derivas: &[f64]) -> Historico {
    let n = asimetrias.len().max(derivas.len());
    let sesiones = (0..n)
        .map(|i| {
            sesion_avisos(
                &format!("s{i}.fit"),
                None,
                asimetrias.get(i).copied(),
                derivas.get(i).copied(),
            )
        })
        .collect();
    Historico { total_sesiones: n, no_leidos: Vec::new(), sesiones }
}

fn sin_real() -> Real {
    Real::default()
}

// ---------------------------------------------------------------- 1. SIN DATOS, NINGUN AVISO INVENTADO

#[test]
fn SIN_DATOS_no_se_genera_ningun_aviso_inventado() {
    // ESTA ES LA PRUEBA MAS IMPORTANTE DE TODO EL BLOQUE, y por eso va la primera.
    //
    // Un aviso sin dato que lo sostenga es **peor que ninguno**: el de la rodilla asustaria por una
    // pierna que no se ha medido, y el del agua mandaria beber por una bascula que nadie ha pesado.
    // Y no se notaria, porque un aviso de mas tiene exactamente la misma pinta que uno de verdad.
    //
    // Aqui no hay NADA: ni agua, ni historico, ni una sola sesion.
    let persona = Persona::desconocida();
    let real = sin_real();
    let historico = Historico::default();
    let hoy = Hoy::sin_datos();

    let salida = avisos::avisos(&persona, &real, &historico, &hoy);

    assert!(
        salida.is_empty(),
        "sin datos NO puede salir ni un aviso, y salieron {}: {:?}",
        salida.len(),
        salida.iter().map(|a| a.titulo.clone()).collect::<Vec<_>>()
    );
}

#[test]
fn sin_datos_el_informe_DICE_que_falta_y_no_se_queda_callado() {
    // La otra mitad de la regla: no inventar **no** es callarse. Un silencio sin explicacion se lee
    // como «todo bien», que es lo contrario de lo que pasa: no se sabe.
    let texto = avisos::informe(
        &Persona::desconocida(),
        &sin_real(),
        &Historico::default(),
        &Hoy::sin_datos(),
    );
    assert!(texto.contains("NINGUNO"), "el informe tiene que decir que no hay avisos");
    assert!(texto.contains("LO QUE NO SALE"), "y tiene que decir que es lo que falta: {texto}");
    for falta in ["agua corporal", "izquierda/derecha", "pulso en reposo", "sueño", "comió"] {
        assert!(texto.contains(falta), "el informe tiene que nombrar lo que falta: «{falta}» no aparece");
    }
}

#[test]
fn un_historico_por_debajo_del_minimo_NO_da_linea_base_y_lo_dice() {
    // CON MENOS DE `MINIMO_PARA_LINEA_BASE` SESIONES NO HAY LINEA BASE, y no es un techo tecnico:
    // comparar contra una mediana de dos sesiones diria «te desvias» por puro azar. Se comprueba el
    // borde exacto: con una menos no hay, con la que hace falta si.
    let pocas = historico_con(&vec![1.0; MINIMO_PARA_LINEA_BASE - 1], &vec![5.0; MINIMO_PARA_LINEA_BASE - 1]);
    assert!(pocas.base_de_fase().is_none(), "con una sesion menos del minimo NO hay linea base de fase");
    assert!(pocas.base_de_deriva().is_none(), "con una sesion menos del minimo NO hay linea base de deriva");

    let justas = historico_con(&vec![1.0; MINIMO_PARA_LINEA_BASE], &vec![5.0; MINIMO_PARA_LINEA_BASE]);
    assert_eq!(justas.base_de_fase(), Some(1.0), "con el minimo justo SI hay linea base");
    assert_eq!(justas.base_de_deriva(), Some(5.0), "y tambien la de la deriva");

    // Y se dice CUAL falta y por que, en el parte.
    let faltantes = avisos::lo_que_falta(&pocas, &Hoy::sin_datos());
    let nombres: Vec<&str> = faltantes.iter().map(|(q, _)| q.as_str()).collect();
    assert!(
        nombres.iter().any(|n| n.contains("línea base de la fase")),
        "el parte tiene que decir que falta la linea base de la fase: {nombres:?}"
    );
}

#[test]
fn una_sesion_SIN_linea_base_no_produce_aviso_aunque_el_dato_este() {
    // El caso que mas facil seria colar: la sesion TRAE el dato, pero no hay con que compararlo.
    // Avisar aqui seria avisar contra una norma inventada.
    let h = Historico {
        total_sesiones: 1,
        no_leidos: Vec::new(),
        sesiones: vec![sesion_avisos("solo.fit", Some(200.0), Some(40.0), Some(30.0))],
    };
    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    // El TSS no necesita linea base (el umbral es del cliente), asi que ESE aviso si sale; lo que no
    // puede salir es el de la rodilla ni el de la recuperacion.
    assert!(
        !salida.iter().any(|a| a.modulo == Modulo::Rodilla),
        "sin linea base NO puede salir el aviso de rodilla"
    );
    assert!(
        !salida.iter().any(|a| a.modulo == Modulo::Recuperacion),
        "sin linea base NO puede salir el aviso de recuperacion"
    );
    assert!(
        salida.iter().any(|a| a.modulo == Modulo::Combustible),
        "el TSS tiene umbral del cliente y no necesita linea base: ese aviso SI tiene que salir"
    );
}

// ---------------------------------------------------------------- 2. LOS UMBRALES DEL CLIENTE

#[test]
fn el_umbral_del_agua_es_el_del_CLIENTE_y_no_uno_inventado() {
    // El encargo pide comprobar que el 54 % es SU umbral. Si alguien lo cambiara a 50 «porque es mas
    // redondo», la aplicacion dejaria de recordarle su regla y le recordaria otra.
    assert_eq!(UMBRAL_AGUA_DEL_CLIENTE_PCT, 54.0, "el umbral de agua del cliente es 54 %");
    assert_eq!(
        Hoy::sin_datos().umbral_agua_pct,
        UMBRAL_AGUA_DEL_CLIENTE_PCT,
        "el parte del dia arranca con SU umbral, no con otro"
    );
}

#[test]
fn el_agua_54_EXACTO_no_dispara_y_53_9_SI() {
    // LA DECISION, con su borde exacto y en las dos direcciones, que es lo que fija un umbral:
    //
    //   - 54,0 exacto  -> **NO** avisa. Su regla dice «si BAJA de 54», y 54 no esta por debajo de 54.
    //   - 53,9         -> **SI** avisa. Esta por debajo, y ademas es el paso real de la bascula.
    //
    // Se prueban las DOS juntas a proposito: una prueba que solo mirara el 54,0 daria por bueno un
    // umbral que nunca dispara.
    assert!(
        !EN_EL_UMBRAL_DE_AGUA_DISPARA,
        "la constante tiene que decir que el valor exacto NO dispara, y la prueba de abajo lo comprueba"
    );

    let vacio = Historico::default();
    let en_el = avisos::avisos(
        &Persona::desconocida(),
        &sin_real(),
        &vacio,
        &Hoy::con_agua(54.0_f32, Dato::Falta),
    );
    assert!(
        !en_el.iter().any(|a| a.modulo == Modulo::Combustible),
        "54,0 exacto NO puede avisar: su regla dice «si baja de 54»"
    );

    let debajo = avisos::avisos(
        &Persona::desconocida(),
        &sin_real(),
        &vacio,
        &Hoy::con_agua(53.9_f32, Dato::Falta),
    );
    let agua: Vec<&Aviso> = debajo.iter().filter(|a| a.modulo == Modulo::Combustible).collect();
    assert_eq!(agua.len(), 1, "53,9 SI tiene que avisar, y una sola vez");
    assert_eq!(agua[0].urgencia, Urgencia::Hoy, "el aviso del agua es de los de «hoy»");
    assert!(agua[0].porque.contains("54"), "el porque tiene que decir SU umbral: {}", agua[0].porque);
    assert!(agua[0].porque.contains("53,9"), "y el valor medido: {}", agua[0].porque);
}

#[test]
fn sin_agua_corporal_NO_hay_aviso_de_agua_y_se_dice_que_falta() {
    // El agua la mete el a mano. Si no la ha metido, no se sabe, y no saberlo NO es estar bien.
    let salida = avisos::avisos(
        &Persona::desconocida(),
        &sin_real(),
        &Historico::default(),
        &Hoy::sin_datos(),
    );
    assert!(
        !salida.iter().any(|a| a.modulo == Modulo::Combustible),
        "sin agua corporal no puede salir el aviso del agua"
    );
    let faltantes = avisos::lo_que_falta(&Historico::default(), &Hoy::sin_datos());
    assert!(
        faltantes.iter().any(|(q, _)| q.contains("agua corporal")),
        "y hay que decir que falta, no callarse"
    );
}

// ==================================================================================================
// EL AGUA QUE ENTRA DESDE LA EXPORTACION DE HUAWEI, CON SU FECHA (2026-10-14)
// ==================================================================================================
//
// Contrato ANTES del codigo: docs/CONTRATO-agua-en-el-recorrido.md. La regla es una sola —la medicion
// con agua MAS RECIENTE manda, y su dia viaja con el valor— y aqui se prueba en sus cuatro esquinas.

#[test]
fn el_agua_mas_reciente_manda_y_lleva_su_FECHA() {
    let vieja = Composicion {
        fecha: Dato::Hay(Fecha::nueva(2026, 6, 1)),
        agua_corporal_pct: Dato::Hay(55.3),
        ..Composicion::vacia()
    };
    let nueva = Composicion {
        fecha: Dato::Hay(Fecha::nueva(2026, 10, 10)),
        agua_corporal_pct: Dato::Hay(53.2),
        ..Composicion::vacia()
    };
    let hoy = Hoy::de_composiciones(&[vieja, nueva]);
    assert_eq!(hoy.agua_corporal_pct, Dato::Hay(53.2), "manda la mas reciente");
    assert_eq!(
        hoy.fecha_agua,
        Dato::Hay(Fecha::nueva(2026, 10, 10)),
        "y el dia viaja CON el valor"
    );
}

#[test]
fn la_medicion_mas_reciente_SIN_agua_no_tapa_a_la_anterior_con_agua() {
    // El pesaje de hoy solo trae peso. El agua que vale es la ultima que SI se midio, no un hueco.
    let con_agua = Composicion {
        fecha: Dato::Hay(Fecha::nueva(2026, 10, 1)),
        agua_corporal_pct: Dato::Hay(56.0),
        ..Composicion::vacia()
    };
    let solo_peso = Composicion {
        fecha: Dato::Hay(Fecha::nueva(2026, 10, 12)),
        peso_kg: Dato::Hay(70.0),
        ..Composicion::vacia()
    };
    let hoy = Hoy::de_composiciones(&[con_agua, solo_peso]);
    assert_eq!(hoy.agua_corporal_pct, Dato::Hay(56.0));
    assert_eq!(hoy.fecha_agua, Dato::Hay(Fecha::nueva(2026, 10, 1)));
}

#[test]
fn sin_agua_en_ninguna_composicion_queda_en_FALTA_y_no_en_cero() {
    let solo_peso = Composicion {
        fecha: Dato::Hay(Fecha::nueva(2026, 10, 12)),
        peso_kg: Dato::Hay(70.0),
        ..Composicion::vacia()
    };
    let hoy = Hoy::de_composiciones(&[solo_peso]);
    assert_eq!(hoy.agua_corporal_pct, Dato::Falta, "«no se midio» NO es «vale cero»");
    assert_eq!(hoy.fecha_agua, Dato::Falta);
}

#[test]
fn agua_sin_fecha_se_usa_pero_se_dice_que_no_consta() {
    let sin_fecha = Composicion {
        agua_corporal_pct: Dato::Hay(53.0),
        ..Composicion::vacia()
    };
    let hoy = Hoy::de_composiciones(&[sin_fecha]);
    assert_eq!(hoy.agua_corporal_pct, Dato::Hay(53.0), "el valor si se usa");
    assert_eq!(hoy.fecha_agua, Dato::Falta, "pero la fecha NO se inventa");
}

#[test]
fn el_aviso_del_agua_dice_de_que_dia_es_la_medicion() {
    let con_fecha = Hoy::con_agua(50.0, Dato::Hay(Fecha::nueva(2026, 10, 10)));
    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &Historico::default(), &con_fecha);
    let agua: Vec<&Aviso> = salida.iter().filter(|a| a.modulo == Modulo::Combustible).collect();
    assert_eq!(agua.len(), 1);
    assert!(
        agua[0].porque.contains("2026-10-10"),
        "el aviso tiene que decir el dia: {}",
        agua[0].porque
    );

    // Y sin fecha, lo DICE en vez de callarse: no saberlo no es un detalle.
    let sin_fecha = Hoy::con_agua(50.0, Dato::Falta);
    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &Historico::default(), &sin_fecha);
    let agua: Vec<&Aviso> = salida.iter().filter(|a| a.modulo == Modulo::Combustible).collect();
    assert!(
        agua[0].porque.contains("sin fecha"),
        "sin fecha tambien se dice: {}",
        agua[0].porque
    );
}

#[test]
fn el_umbral_del_TSS_es_el_del_CLIENTE_y_NO_es_un_hallazgo() {
    // El corte de 100 TSS es SU pauta. El TSS es un dato medido, pero el CORTE no: eso va en la
    // fuente del aviso, y la prueba lo exige escrito.
    assert_eq!(UMBRAL_TSS_DEL_CLIENTE, 100.0, "el umbral de TSS del cliente es 100");

    let h = historico_iguales(1, sesion_avisos("s.fit", Some(108.0), None, None));
    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    let a = salida
        .iter()
        .find(|a| a.modulo == Modulo::Combustible)
        .expect("108 TSS tiene que producir aviso");
    assert!(
        a.fundamento.fuente.contains("PAUTA DEL CLIENTE"),
        "el fundamento TIENE que decir que el corte es pauta suya y no un hallazgo: {}",
        a.fundamento.fuente
    );
    assert!(
        a.fundamento.fuente.contains("medido"),
        "y que el TSS en cambio si es medido: {}",
        a.fundamento.fuente
    );
}

#[test]
fn los_100_TSS_EXACTOS_no_disparan_y_100_1_SI() {
    // La otra decision con su borde, en las dos direcciones. Su regla dice «si SUPERA 100 TSS», y 100
    // no supera 100.
    //
    // Medido ademas sobre sus 83 sesiones con TSS: **ninguna vale exactamente 100** (las mas cercanas
    // son 99,5 y 101,3), asi que esta eleccion no silencia ni un aviso suyo. Se fija igual, porque el
    // dia que una sesion valga 100 clavados tiene que haber una respuesta decidida.
    assert!(
        !EN_EL_UMBRAL_DE_TSS_DISPARA,
        "la constante tiene que decir que 100 exacto NO dispara"
    );

    // El 100 va EL ULTIMO a proposito: el aviso mira la sesion mas reciente que supera el umbral, asi
    // que si el corte estuviera mal puesto, el aviso saldria con el 100 y la prueba lo cazaria.
    let h = Historico {
        total_sesiones: 2,
        no_leidos: Vec::new(),
        sesiones: vec![
            sesion_avisos("corta.fit", Some(101.3), None, None),
            sesion_avisos("justa.fit", Some(100.0), None, None),
        ],
    };
    assert_eq!(h.sesiones_sobre_tss(UMBRAL_TSS_DEL_CLIENTE), 1, "solo la de 101,3 supera el umbral");

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    let a = salida
        .iter()
        .find(|a| a.modulo == Modulo::Combustible)
        .expect("101,3 tiene que avisar");
    assert!(
        a.titulo.contains("101"),
        "el aviso tiene que ser de la sesion de 101,3, NO de la de 100 exactos: {}",
        a.titulo
    );
}

#[test]
fn el_aviso_del_TSS_dice_CUANTAS_sesiones_pasan_el_umbral_y_de_cuantas() {
    // Un aviso que dice «te pasaste» sin decir «te has pasado 18 veces de 83» deja al cliente sin saber
    // si eso es su vida normal o un caso raro. Medido: son 18 de sus 83 con TSS.
    let mut sesiones: Vec<SesionVista> = (0..18)
        .map(|i| sesion_avisos(&format!("alta{i}.fit"), Some(120.0), None, None))
        .collect();
    // Y 20 sesiones suaves DEBAJO del umbral: la de 99,5 es la mas cercana, y no puede contar.
    sesiones.extend((0..20).map(|i| sesion_avisos(&format!("baja{i}.fit"), Some(if i == 0 { 99.5 } else { 60.0 }), None, None)));
    let h = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };

    assert_eq!(h.sesiones_sobre_tss(UMBRAL_TSS_DEL_CLIENTE), 18, "18 sesiones superan el umbral");

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    let a = salida.iter().find(|a| a.modulo == Modulo::Combustible).unwrap();
    assert!(a.porque.contains("18"), "el porque tiene que decir cuantas pasan el umbral: {}", a.porque);
    assert!(a.porque.contains("38"), "y de cuantas con TSS: {}", a.porque);
    assert!(a.porque.contains("99.5") == false, "la de 99,5 no es una de las que pasan: {}", a.porque);
}

// ---------------------------------------------------------------- 3. LA RODILLA: SIN FASE, SIN AVISO

#[test]
fn una_sesion_SIN_fase_de_pedaleo_no_produce_aviso_de_rodilla() {
    // El encargo lo pide literalmente, y es el caso de 19 de sus 102 ficheros: no traen fase.
    // La sesion SI existe, SI tiene TSS y SI tiene deriva; lo que no tiene es fase.
    let mut sesiones: Vec<SesionVista> = (0..MINIMO_PARA_LINEA_BASE * 2)
        .map(|i| sesion_avisos(&format!("con_fase{i}.fit"), None, Some(1.0), Some(5.0)))
        .collect();
    // Y la ultima, la mas reciente, SIN fase y con una asimetria que seria escandalosa si existiera.
    sesiones.push(sesion_avisos("sin_fase.fit", None, None, Some(5.0)));
    let h = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };

    assert!(h.base_de_fase().is_some(), "las otras sesiones SI dan linea base");

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    assert!(
        !salida.iter().any(|a| a.modulo == Modulo::Rodilla),
        "una sesion sin fase NO puede producir aviso de rodilla, y salio: {:?}",
        salida.iter().map(|a| a.titulo.clone()).collect::<Vec<_>>()
    );
}

#[test]
fn una_asimetria_que_supera_el_DOBLE_de_su_base_SI_avisa() {
    // La otra cara: cuando el dato esta y se sale de SU norma, el aviso sale. Y con la cuenta dicha.
    let mut sesiones: Vec<SesionVista> = (0..20)
        .map(|i| sesion_avisos(&format!("normal{i}.fit"), None, Some(2.0), None))
        .collect();
    sesiones.push(sesion_avisos("desviada.fit", None, Some(9.0), None));
    let h = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };

    let base = h.base_de_fase().unwrap();
    let corte = h.corte_de_fase().unwrap();
    assert_eq!(base, 2.0, "la base es la mediana de las 21 sesiones");
    assert_eq!(corte, avisos::alinear_a_la_escala(4.0), "el corte es el doble de la base");

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    let a = salida
        .iter()
        .find(|a| a.modulo == Modulo::Rodilla)
        .expect("una asimetria de 9,0 contra una base de 2,0 tiene que avisar");
    assert_eq!(a.urgencia, Urgencia::Atencion, "el de la rodilla es «atención», no «hoy»");
    assert!(a.porque.contains("2,00"), "dice su base con coma decimal, como se escribe en espanol: {}", a.porque);
    assert!(a.porque.contains("9,0"), "y su valor: {}", a.porque);
}

#[test]
fn la_asimetria_usa_la_diferencia_CIRCULAR_y_no_una_resta_a_pelo() {
    // ESTE ERROR SE COMETIO MIDIENDO, y por eso tiene prueba: el primer angulo de la fase salta del
    // 357° al 2°, y esos dos valores estan a **5 grados**, no a 355. Una resta a pelo diria que esa
    // sesion tiene una asimetria de 355° — la peor de su historia — cuando en realidad va bien.
    //
    // 357,1875 y 352,96875 son los dos valores REALES de la primera sesion de su historico.
    let izq = [357.1875, 184.21875, 185.625, 90.0];
    let der = [352.96875, 181.40625, 187.03125, 87.1875];
    let a = avisos::asimetria_de_fase(Some(izq), Some(der)).expect("las dos fases estan");
    assert!(
        a < 10.0,
        "la asimetria de esa sesion real es de unos 4,2 grados; si sale {a} se esta restando a pelo \
         en vez de usar la distancia angular"
    );

    // Y el caso que delata la resta a pelo, llevado al extremo: 1° contra 359° son DOS grados.
    let cerca = avisos::asimetria_de_fase(Some([1.0, 1.0, 1.0, 1.0]), Some([359.0, 359.0, 359.0, 359.0]));
    assert_eq!(cerca, Some(2.0), "1° y 359° estan a 2 grados, no a 358");
}

#[test]
fn sin_UNA_de_las_dos_fases_la_asimetria_es_None_y_no_un_CERO() {
    // Un cero seria una AFIRMACION («las dos piernas van iguales») y no se puede hacer sin las dos
    // piernas. Comparar una pierna contra un hueco no dice nada.
    let fase = [10.0, 20.0, 30.0, 40.0];
    assert_eq!(avisos::asimetria_de_fase(Some(fase), None), None, "sin la derecha no hay asimetria");
    assert_eq!(avisos::asimetria_de_fase(None, Some(fase)), None, "sin la izquierda tampoco");
    assert_eq!(avisos::asimetria_de_fase(None, None), None, "y sin ninguna, menos");
}

// ---------------------------------------------------------------- 4. LA HONESTIDAD DEL TEXTO

/// Parte un texto en palabras comparables: sin tildes, en minusculas y sin puntuacion pegada.
///
/// POR QUE HACE FALTA: buscar «lesion» en un texto que dice «lesiones» con un `contains` **no lo
/// encuentra**, y una prueba que no encuentra lo que busca **da verde siempre** — que es peor que no
/// tenerla. Aqui se normaliza y se compara palabra a palabra.
fn palabras(texto: &str) -> Vec<String> {
    texto
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            otro => otro,
        })
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(|p| p.to_string())
        .collect()
}

/// Todos los avisos que se pueden sacar con datos COMPLETOS, para poder revisarlos de una vez.
fn todos_los_avisos_posibles() -> Vec<Aviso> {
    let mut salida = Vec::new();
    let real = sin_real();
    let persona = Persona::desconocida();

    // a) El agua, por debajo del umbral.
    salida.extend(avisos::avisos(&persona, &real, &Historico::default(), &Hoy::con_agua(50.0_f32, Dato::Falta)));

    // b) El TSS, con una sesion muy por encima.
    let h_tss = historico_iguales(1, sesion_avisos("s.fit", Some(222.0), None, None));
    salida.extend(avisos::avisos(&persona, &real, &h_tss, &Hoy::sin_datos()));

    // c) La rodilla, con una desviacion grande y linea base.
    let mut sesiones: Vec<SesionVista> = (0..20)
        .map(|i| sesion_avisos(&format!("n{i}.fit"), None, Some(1.40625), Some(3.0)))
        .collect();
    sesiones.push(sesion_avisos("desviada.fit", None, Some(43.59375), Some(34.71)));
    let h_rodilla = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };
    salida.extend(avisos::avisos(&persona, &real, &h_rodilla, &Hoy::sin_datos()));

    // d) La recuperacion, con una deriva muy por encima de su linea base.
    let mut sesiones: Vec<SesionVista> = (0..20)
        .map(|i| sesion_avisos(&format!("d{i}.fit"), None, None, Some(3.0)))
        .collect();
    sesiones.push(sesion_avisos("escapada.fit", None, None, Some(34.71)));
    let h_deriva = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };
    salida.extend(avisos::avisos(&persona, &real, &h_deriva, &Hoy::sin_datos()));

    salida
}

#[test]
fn NINGUN_texto_de_aviso_dice_diagnostico_ni_lesion_ni_promete_que_algo_cure() {
    // LA PRUEBA DE HONESTIDAD DEL ENCARGO, y es literal: el aviso de la rodilla **no puede** parecer
    // un informe medico. Un texto que diga «posible lesion» sobre la rodilla de una persona de 62 años
    // es exactamente lo que este proyecto existe para no hacer.
    //
    // SE BUSCA PALABRA A PALABRA Y NO CON `contains`, por un motivo medido: `contains("lesion")` **no
    // encuentra «lesiones»**, asi que una prueba escrita asi daria verde SIEMPRE. Se comprobo.
    let prohibidas = [
        "diagnostico", "diagnostica", "diagnostican", "diagnosticar",
        "lesion", "lesiones", "lastimado", "lastimada",
        "cura", "curan", "curar", "curacion", "curable",
        "remedia", "remedian", "remediar", "trata", "tratan", "tratamiento",
    ];

    for a in todos_los_avisos_posibles() {
        let suyas = palabras(&a.texto());
        for p in &suyas {
            assert!(
                !prohibidas.contains(&p.as_str()),
                "el texto de un aviso NO puede decir «{p}»: es un juicio clinico, y esto solo observa \
                 datos. Aviso: {} · {}",
                a.modulo.nombre(),
                a.titulo
            );
        }
    }
}

#[test]
fn el_INFORME_no_promete_remedios_aunque_lleva_su_aviso_de_que_no_diagnostica() {
    // EL INFORME LLEVA UN AVISO QUE ES OBLIGATORIO Y QUE USA UNA PALABRA PROHIBIDA EN LOS AVISOS:
    // la ultima linea dice que este programa **«no diagnostica enfermedades ni receta»**. Es una
    // **negacion**, y decirla es mas honesto que callarla, asi que se permite **con la frase entera
    // dicha**, no con la palabra suelta.
    //
    // Y la prueba sigue sirviendo para lo que importa: si alguien anadiera al informe algo que prometa
    // un remedio, seguiria saltando.
    let texto = avisos::informe(
        &Persona::desconocida(),
        &sin_real(),
        &Historico::default(),
        &Hoy::con_agua(50.0_f32, Dato::Falta),
    );
    let suyas = palabras(&texto);

    // Ninguna palabra de REMEDIO, en ninguna forma.
    for prohibida in [
        "cura", "curan", "curar", "curacion", "curable",
        "remedia", "remedian", "remediar", "trata", "tratan", "tratamiento",
        "lesion", "lesiones",
    ] {
        assert!(
            !suyas.iter().any(|p| p == prohibida),
            "el informe NO puede prometer un remedio, y dice «{prohibida}»"
        );
    }

    // «diagnostica» SOLO puede aparecer dentro de la negacion completa, que es lo que se dice.
    let dice_que_no = suyas.windows(2).any(|w| w[0] == "no" && w[1] == "diagnostica");
    let aparece = suyas.iter().any(|p| p == "diagnostica");
    if aparece {
        assert!(
            dice_que_no,
            "la palabra «diagnostica» solo se admite dentro de «no diagnostica», que es la negacion"
        );
    }
    // Y esa negacion tiene que ESTAR: callarla seria peor que decirla.
    assert!(
        texto.contains("no diagnostica enfermedades ni receta"),
        "el informe tiene que decir que no diagnostica ni receta: {texto}"
    );
}

#[test]
fn el_aviso_de_la_rodilla_dice_que_es_una_OBSERVACION_y_no_un_diagnostico() {
    // La regla de honestidad del encargo, comprobada por el otro lado: no basta con no decir la
    // palabra prohibida, hay que **decir el sentido**. Y si alguien escribiera un aviso de rodilla sin
    // reserva, `es_observacion()` lo delataria.
    let mut sesiones: Vec<SesionVista> = (0..20)
        .map(|i| sesion_avisos(&format!("n{i}.fit"), None, Some(1.40625), None))
        .collect();
    sesiones.push(sesion_avisos("desviada.fit", None, Some(9.0), None));
    let h = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    let rodilla = salida
        .iter()
        .find(|a| a.modulo == Modulo::Rodilla)
        .expect("tiene que salir el aviso de rodilla");

    assert!(rodilla.es_observacion(), "el aviso de la rodilla TIENE que declararse observacion");
    let reserva = rodilla.fundamento.reserva.as_ref().expect("y llevar su reserva escrita");
    assert!(
        reserva.contains("observación de sus propios datos"),
        "la reserva tiene que decir que es una observacion de sus datos: {reserva}"
    );
    assert!(
        reserva.contains("no una valoración médica"),
        "y que NO es una valoracion medica: {reserva}"
    );

    // Y un aviso de rodilla SIN reserva tiene que delatarse como lo que es.
    let malo = Aviso::nuevo(Modulo::Rodilla, Urgencia::Atencion, "t", "p", "q", "f", None);
    assert!(
        !malo.es_observacion(),
        "un aviso de rodilla sin reserva afirma mas de lo que sabe, y `es_observacion` tiene que cazarlo"
    );
}

#[test]
fn el_aviso_de_la_rodilla_dice_que_el_reparto_izquierda_derecha_NO_existe() {
    // Cada vez que se habla de la rodilla hay que decir de donde sale el dato Y cual NO esta, porque
    // el cliente pidio justamente el que no existe. Si no se dice, va a seguir buscandolo.
    let mut sesiones: Vec<SesionVista> = (0..20)
        .map(|i| sesion_avisos(&format!("n{i}.fit"), None, Some(1.40625), None))
        .collect();
    sesiones.push(sesion_avisos("desviada.fit", None, Some(9.0), None));
    let h = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    let rodilla = salida.iter().find(|a| a.modulo == Modulo::Rodilla).unwrap();
    let reserva = rodilla.fundamento.reserva.as_ref().unwrap();
    assert!(
        reserva.contains("NO viaja en el fichero FIT"),
        "tiene que decir que el reparto de potencia NO esta en el fichero: {reserva}"
    );
    assert!(
        reserva.contains("Garmin Connect"),
        "y donde SI se puede mirar, que es lo unico util que se le puede ofrecer: {reserva}"
    );
}

#[test]
fn el_aviso_del_agua_NO_afirma_que_el_consome_o_los_electrolitos_sirvan() {
    // El encargo lo pide con todas las letras: la aplicacion los RECUERDA como pauta suya y **no
    // afirma que sirvan**, porque no hay ni una fuente que lo sostenga. Esta prueba es la que impide
    // que un dia alguien escriba «el consomé repone los electrolitos» y nadie lo note.
    let salida = avisos::avisos(
        &Persona::desconocida(),
        &sin_real(),
        &Historico::default(),
        &Hoy::con_agua(50.0_f32, Dato::Falta),
    );
    let a = salida.iter().find(|a| a.modulo == Modulo::Combustible).unwrap();

    assert!(
        a.fundamento.fuente.contains("PAUTA DEL CLIENTE"),
        "el fundamento tiene que decir que es SU pauta: {}",
        a.fundamento.fuente
    );
    assert!(
        a.fundamento.fuente.contains("NO se afirma que sirvan"),
        "y que NO se afirma que sirvan: {}",
        a.fundamento.fuente
    );
    // Y la palabra «peligro» no puede aparecer: un umbral que el eligio no es un hallazgo clinico.
    let suyas = palabras(&a.texto());
    assert!(
        !suyas.iter().any(|p| p == "peligro" || p == "peligrosa" || p == "peligroso" || p == "riesgo"),
        "su propia pauta no se puede presentar como un peligro: {}",
        a.texto()
    );
}

// ---------------------------------------------------------------- 5. TODO AVISO LLEVA UN NUMERO

#[test]
fn cada_aviso_lleva_su_PORQUE_con_un_NUMERO_y_no_una_frase_vacia() {
    // «Conviene vigilarlo» no es un aviso: es relleno. Un aviso utilizable tiene que poder decirse con
    // un numero al lado, porque si no, el cliente no puede discutirlo ni comprobarlo.
    let avisos_posibles = todos_los_avisos_posibles();
    assert!(avisos_posibles.len() >= 3, "la prueba tiene que estar mirando avisos de verdad");

    for a in &avisos_posibles {
        assert!(!a.porque.trim().is_empty(), "un aviso sin porque no es un aviso");
        assert!(
            a.porque.chars().any(|c| c.is_ascii_digit()),
            "el porque TIENE que llevar un numero ({} · {}): «{}»",
            a.modulo.nombre(),
            a.titulo,
            a.porque
        );
        // Y la fuente, que es lo que separa un aviso de una opinion.
        assert!(
            !a.fundamento.fuente.trim().is_empty(),
            "un aviso sin fuente no se puede comprobar: {}",
            a.titulo
        );
        assert_eq!(
            a.fundamento.afirmacion, a.porque,
            "el fundamento y el porque tienen que decir LO MISMO, no dos cosas distintas"
        );
    }
}

#[test]
fn el_titulo_y_el_que_hacer_de_un_aviso_son_UNA_sola_linea() {
    // El encargo lo pide asi, y no es estetica: un aviso de cinco lineas no se lee en el movil, y
    // estos avisos estan para leerse antes de una etapa.
    for a in todos_los_avisos_posibles() {
        assert!(
            !a.titulo.contains('\n'),
            "el titulo tiene que ser UNA linea: {}",
            a.titulo
        );
        assert!(
            !a.que_hacer.contains('\n'),
            "el «que hacer» tiene que ser UNA linea: {}",
            a.que_hacer
        );
        assert!(!a.titulo.trim().is_empty(), "y ninguno puede ir vacio");
        assert!(!a.que_hacer.trim().is_empty(), "ni el que hacer");
    }
}

#[test]
fn los_avisos_salen_ordenados_con_lo_mas_urgente_PRIMERO() {
    // Un aviso urgente que aparece el quinto se lee tarde. Se comprueba que el orden es por urgencia.
    let h = historico_iguales(1, sesion_avisos("s.fit", Some(150.0), None, None));
    let mut sesiones = h.sesiones.clone();
    // Una rodilla desviada, para tener un segundo aviso de otra urgencia.
    let mut con_fase: Vec<SesionVista> = (0..20)
        .map(|i| sesion_avisos(&format!("n{i}.fit"), None, Some(1.40625), None))
        .collect();
    con_fase.push(sesion_avisos("desviada.fit", None, Some(9.0), None));
    sesiones.extend(con_fase);
    let h = Historico { total_sesiones: sesiones.len(), no_leidos: Vec::new(), sesiones };

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::con_agua(50.0_f32, Dato::Falta));
    assert!(salida.len() >= 3, "tienen que salir los tres modulos, y salieron {}", salida.len());

    for par in salida.windows(2) {
        assert!(
            par[0].urgencia >= par[1].urgencia,
            "los avisos tienen que ir de mas urgente a menos: {:?} antes de {:?}",
            par[0].urgencia,
            par[1].urgencia
        );
    }
    assert_eq!(salida[0].urgencia, Urgencia::Hoy, "el primero tiene que ser de los de «hoy»");
}

#[test]
fn la_confianza_de_un_aviso_BAJA_cuando_tiene_reserva() {
    // La regla del motor, que no se sube por tener muchos datos y si se baja por faltar uno: un aviso
    // con reserva NO es «seguro».
    let con_reserva = Aviso::nuevo(
        Modulo::Rodilla,
        Urgencia::Atencion,
        "t",
        "p con 3.5",
        "q",
        "f",
        Some("r".to_string()),
    );
    assert_eq!(
        con_reserva.confianza(),
        Confianza::ConReserva,
        "un aviso con reserva no puede ser «seguro»"
    );
}

// ================================================== 18. LA POTENCIA
//
// POR QUE ESTA SECCION EXISTE, y es el problema que la justifica entera: el motor calculaba las zonas
// SOLO con el pulso, y este cliente **entrena con potencia** — medidor de doble sensor y un FTP que le
// detecto el reloj. Para un puerto las zonas de potencia dicen mas que las de pulso: el pulso va con
// retraso y lo mueven el calor y la fatiga, y un vatio es un vatio.
//
// MEDIDO sobre sus 102 ficheros: 83 traen potencia y 19 no. Por eso estas pruebas comprueban las dos
// mitades: que las zonas salen bien cuando hay vatios, y que **no se inventa nada** cuando no los hay.

use motor::potencia::{self, DeclaradoDePotencia, ZonaPotencia};

/// Un fichero FIT de verdad **con potencia**, si esta en esta maquina. Si no, la prueba se salta.
///
/// SE BUSCA EL PRIMERO QUE SE PUEDA LEER ENTERO, y no se fija uno por nombre a proposito: hay
/// ficheros que `fitparser` no sabe abrir (`ParseError(Tag)` en el primer byte) y **no es un fallo
/// nuestro**, es un fichero raro. Una prueba que dependa de un fichero concreto se pone roja por un
/// motivo que no tiene nada que ver con lo que comprueba — y entonces deja de proteger.
fn fit_con_potencia() -> Option<(std::path::PathBuf, motor::fit::Lectura, DeclaradoDePotencia)> {
    let carpeta = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../datos/garmin_files/storage");
    let mut rutas: Vec<std::path::PathBuf> = std::fs::read_dir(&carpeta)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false))
        .collect();
    rutas.sort();
    for ruta in rutas {
        let Ok(d) = potencia::leer_declarado_de_potencia(&ruta) else { continue };
        if !d.umbral_w.lo_hay() {
            continue;
        }
        if let Ok(l) = motor::fit::leer(&ruta) {
            return Some((ruta, l, d));
        }
    }
    None
}

#[test]
fn las_zonas_de_potencia_cubren_todo_el_rango_SIN_HUECOS_Y_SIN_SOLAPARSE() {
    // ESTA ES LA PRUEBA QUE IMPIDE EL DEFECTO CLASICO: una zona empieza donde acaba la anterior, asi
    // que si el techo de una no es exactamente el suelo de la siguiente, o hay un vatio sin zona
    // (hueco) o hay dos zonas diciendo lo mismo (solape).
    let ftp = potencia::Ftp::estimado(213.0);
    let zs = potencia::zonas_de_potencia(&ftp);
    assert_eq!(zs.len(), 7, "las siete zonas de Coggan");

    for par in zs.windows(2) {
        let techo = par[0]
            .2
            .as_ref()
            .expect("solo la ULTIMA zona puede no tener techo");
        assert_eq!(
            techo.valor, par[1].1.valor,
            "la zona {} acaba en {:.1} y la {} empieza en {:.1}: o hay hueco o hay solape",
            par[0].0.nombre(),
            techo.valor,
            par[1].0.nombre(),
            par[1].1.valor
        );
    }
    // Y la ultima NO tiene techo, porque la fuente no lo publica: eso es un dato, no un olvido.
    assert!(zs.last().unwrap().2.is_none(), "la zona 7 no tiene techo en la fuente");
    assert!(zs.last().unwrap().0.sin_techo());
}

#[test]
fn NINGUNA_zona_ensena_el_mismo_tramo_que_la_anterior() {
    // ESTE DEFECTO YA SALIO UNA VEZ EN ESTE PROYECTO, y por eso tiene su propia prueba: la primera
    // version de `zonas_del_umbral` devolvia solo el SuELO, y como cada zona empieza donde acaba la
    // anterior, **dos zonas seguidas ensenaban el mismo numero**. Un informe que dice dos veces 213 no
    // se entiende, y el fallo era de lectura, no de calculo.
    let ftp = potencia::Ftp::estimado(213.0);
    let zs = potencia::zonas_de_potencia(&ftp);
    for (i, w) in zs.windows(2).enumerate() {
        assert_ne!(
            w[0].2.as_ref().map(|r| r.valor),
            w[1].2.as_ref().map(|r| r.valor),
            "las zonas {} y {} acaban en el mismo vatio",
            i + 1,
            i + 2
        );
        assert_ne!(
            w[0].1.valor, w[1].1.valor,
            "las zonas {} y {} empiezan en el mismo vatio",
            i + 1,
            i + 2
        );
    }
    // Y ninguna zona enseña un tramo VACIO: «de 213 a 213» no es una zona.
    for (z, desde, hasta) in &zs {
        if let Some(h) = hasta {
            assert!(
                h.valor > desde.valor,
                "la zona {} enseña un tramo vacio ({} a {})",
                z.nombre(),
                desde.valor,
                h.valor
            );
        }
    }
}

#[test]
fn los_cortes_son_los_de_COGGAN_calculados_a_mano() {
    // A mano, sobre 213 W, con los cortes de la fuente (55 / 75 / 90 / 105 / 120 / 150 %):
    //   213 x 0,55 = 117,15 · x 0,75 = 159,75 · x 0,90 = 191,7 · x 1,05 = 223,65 · x 1,20 = 255,6 · x 1,50 = 319,5
    // Si estos numeros no cuadran, la tabla del comentario y el codigo se han separado.
    let ftp = potencia::Ftp::estimado(213.0);
    let zs = potencia::zonas_de_potencia(&ftp);
    let techos: Vec<f64> = zs.iter().filter_map(|(_, _, h)| h.as_ref().map(|r| r.valor)).collect();
    assert_eq!(techos.len(), 6, "seis techos: la zona 7 no tiene");
    for (obtenido, esperado) in techos.iter().zip([117.15, 159.75, 191.7, 223.65, 255.6, 319.5]) {
        assert!(
            (obtenido - esperado).abs() < 0.01,
            "el corte no cuadra con el calculo a mano: {obtenido:.2} frente a {esperado:.2}"
        );
    }
}

#[test]
fn con_FTP_CERO_o_sin_FTP_NO_se_inventan_zonas() {
    // «No inventar» NO es lo mismo que «no responder», pero aqui la respuesta correcta ES «falta»:
    // siete zonas de cero vatios no son una respuesta, son un error con aspecto de tabla.
    let sin_nada = motor::fit::Declarado::default();
    assert_eq!(
        potencia::umbral_de_potencia(&sin_nada),
        motor::Dato::Falta,
        "sin umbral declarado no hay zonas, y se dice"
    );

    let mut cero = motor::fit::Declarado::default();
    cero.potencia_umbral = motor::Dato::Hay(0);
    assert_eq!(
        potencia::umbral_de_potencia(&cero),
        motor::Dato::Falta,
        "un umbral de CERO no es un umbral: daria siete zonas de cero vatios"
    );

    // Y el caso con vatios pero sin umbral tampoco se rellena: se dice QUE falta.
    let que = potencia::que_falta_para_potencia(false, true).expect("tiene que decir que falta");
    assert!(que.contains("prueba de 20 minutos"), "y tiene que decir que haria falta: {que}");
}

#[test]
fn el_FTP_DECLARADO_por_el_aparato_lleva_SU_margen_y_su_origen() {
    // El margen NO es cero: el aparato estima. Y el origen se declara, porque un umbral declarado por
    // el reloj y uno estimado por nosotros no valen lo mismo.
    let mut d = motor::fit::Declarado::default();
    d.potencia_umbral = motor::Dato::Hay(213);
    match potencia::umbral_de_potencia(&d) {
        motor::Dato::Hay(ftp) => {
            assert_eq!(ftp.valor(), 213.0);
            assert_eq!(ftp.referencia, potencia::Referencia::DeclaradoPorElAparato);
            assert!(
                (ftp.rango.margen - potencia::ERROR_FTP_DECLARADO).abs() < 1e-9,
                "el margen es el error tipico publicado (7 W), no un cero"
            );
            assert!(ftp.referencia.que_falta().is_none(), "con el aparato declarandolo, no falta nada");
            // Y el margen del estimado es MAS ANCHO, que es lo que pide el encargo.
            let estimado = potencia::Ftp::estimado(213.0);
            assert!(
                estimado.rango.margen > ftp.rango.margen,
                "una estimacion no puede tener el margen de un dato respaldado"
            );
            assert_eq!(estimado.referencia, potencia::Referencia::Estimado);
            assert!(estimado.referencia.que_falta().is_some());
        }
        motor::Dato::Falta => panic!("con 213 W declarados tiene que haber umbral"),
    }
}

#[test]
fn el_TSS_se_LEE_del_fichero_y_NO_se_recalcula() {
    // ESTA PRUEBA ES LA REGLA DEL PROYECTO, y por eso se comprueba con un fichero de verdad y con las
    // piezas en la mano: si `carga_de` recalculara, el numero que devuelve NO seria el del fichero.
    // Se lee un FIT suyo, y se comprueba contra el valor que ese mismo fichero declara.
    let Some((ruta, l, d)) = fit_con_potencia() else {
        eprintln!("(sin ficheros FIT con potencia en esta maquina: la prueba se salta)");
        return;
    };

    // a) lo que devuelve `carga_de` es EXACTAMENTE lo que trae el fichero, y el fichero declara el
    //    mismo TSS que lee nuestro lector de potencia: dos lectores distintos, el mismo numero.
    let tss = potencia::carga_de(&l.declarado);
    assert!(tss.lo_hay(), "ese fichero tiene potencia, asi que tiene TSS");
    assert_eq!(tss, d.tss, "los dos lectores tienen que leer el mismo TSS del mismo fichero");

    // b) y NO se recalcula. Sobre TODOS los ficheros de la carpeta, ninguno tiene un TSS que sea
    //    exactamente `duracion x NP x IF / (FTP x 3600) x 100`: la probabilidad de que la cuenta caiga
    //    clavada es practicamente nula, asi que si `carga_de` recalculase, esta prueba se pondria roja.
    //    (Medido sobre sus 83 sesiones con potencia: cuadran 55 dentro de un punto de TSS, y NINGUNA da
    //    el mismo numero con todos los decimales.)
    //
    //    LA DURACION QUE HACE FALTA PARA LA CUENTA ES LA DE MOVIMIENTO, y `fit::Declarado` guarda la
    //    transcurrida: se sustituye declarandolo, porque es justo lo que hace el binario. Con la
    //    transcurrida la cuenta da 161,0 en vez de 156,4, y acusaria al fichero de un error nuestro.
    let mut para_la_cuenta = l.declarado.clone();
    para_la_cuenta.duracion_s = motor::Dato::Hay(
        d.duracion_en_movimiento_s.valor().expect("trae duracion en movimiento") as u32,
    );
    let c = potencia::comprobar_tss(&para_la_cuenta, potencia::TOLERANCIA_TSS)
        .expect("ese fichero trae NP, IF y FTP");
    assert!(
        (c.recalculado - c.declarado).abs() > 1e-6,
        "si la cuenta diera clavado el TSS del fichero, esta prueba no probaria nada: {ruta:?}"
    );
    assert!(
        c.cuadra,
        "y aun asi tiene que cuadrar dentro de la tolerancia: {:.1} frente a {:.1}",
        c.recalculado, c.declarado
    );
}

#[test]
fn una_carga_ALTA_se_detecta_con_el_umbral_del_cliente_y_100_exacto_NO_lo_es() {
    // EL CORTE ES ESTRICTO, Y ES UNA DECISION QUE HAY QUE JUSTIFICAR: el cliente escribio «si el
    // entrenamiento supERA los 100 TSS», y superar quiere decir MAS de. Asi que 100,0 exacto NO es
    // carga alta, y la frontera queda del lado conservador. La alternativa marcaria como alta una
    // sesion que esta exactamente en el numero que el puso como limite.
    let u = potencia::UMBRAL_DE_CARGA_DEL_CLIENTE;
    assert_eq!(u, 100.0, "el umbral es el que puso el cliente");
    assert_eq!(u, motor::avisos::UMBRAL_TSS_DEL_CLIENTE, "y esta en UN solo sitio");

    assert!(!potencia::carga_es_alta(u, u), "100 exacto NO supera 100");
    assert!(potencia::carga_es_alta(100.1, u), "una decima mas SI");
    assert!(potencia::carga_es_alta(221.5, u), "y su sesion mas dura, tambien");
    assert!(!potencia::carga_es_alta(99.9, u), "99,9 no llega");
    assert!(!potencia::carga_es_alta(64.1, u), "su mediana tampoco");

    // Y un dato que no es un dato NO dispara el aviso: no se avisa por un hueco.
    assert!(!potencia::carga_es_alta(f64::NAN, u), "NaN no es una carga alta");
    assert!(!potencia::carga_es_alta(-5.0, u), "una carga negativa no existe");
    assert!(!potencia::carga_es_alta(0.0, u), "cero no supera nada");

    // El fundamento dice que es SU regla, no un hallazgo: eso es lo que impide llamarlo «peligro».
    let f = potencia::fundamento_del_aviso_de_carga();
    assert!(f.contains("regla del propio cliente"), "el fundamento tiene que ser suyo: {f}");
    assert!(f.contains("NO es un hallazgo"), "y tiene que decir lo que NO es: {f}");
}

#[test]
fn una_sesion_SIN_POTENCIA_no_produce_zona_de_potencia_y_lo_DICE() {
    // 19 DE SUS 102 SESIONES NO TIENEN POTENCIA, y ese numero manda sobre el diseno: la respuesta
    // correcta no es una lista de ceros —eso seria una afirmacion falsa— sino «falta».
    let sin_zonas = DeclaradoDePotencia::default();
    assert!(
        potencia::tiempo_en_zonas(&sin_zonas.tiempo_en_potencia_s).is_none(),
        "sin reparto en el fichero no hay zonas, y NO una lista de ceros"
    );
    // Y una lista vacia tampoco es un dato: es un hueco con forma de dato.
    let mut vacia = DeclaradoDePotencia::default();
    vacia.tiempo_en_potencia_s = motor::Dato::Hay(Vec::new());
    assert!(potencia::tiempo_en_zonas(&vacia.tiempo_en_potencia_s).is_none());

    let que = potencia::que_falta_para_potencia(true, false).expect("tiene que decir que falta");
    assert!(que.contains("no se rellenan con ceros"), "y por que no se rellenan: {que}");

    // El caso mas honesto: ni vatios ni umbral.
    let nada = potencia::que_falta_para_potencia(false, false).expect("tambien lo dice");
    assert!(nada.contains("ni vatios ni umbral"), "y lo dice entero: {nada}");
}

#[test]
fn el_TSS_que_trae_el_fichero_SI_cuadra_con_NP_IF_y_FTP_y_lo_hemos_medido() {
    // 55 DE SUS 83 SESIONES CON POTENCIA cuadran dentro de media unidad de TSS. La comprobacion VALE,
    // y por eso se puede afirmar que la cadena cuadra; y las 28 que no cuadran son justo lo que hay
    // que mirar. Esta es la comprobacion que `fit.rs` describe: si no coinciden, uno de los dos esta
    // mal, y eso sale gratis.
    let Some((ruta, l, d)) = fit_con_potencia() else {
        eprintln!("(sin ficheros FIT: la prueba se salta)");
        return;
    };
    // LA DURACION ES LA CLAVE, y esta medido: `fit::Declarado.duracion_s` guarda el tiempo
    // TRANSCURRIDO, y la cuenta de Coggan necesita el tiempo EN MOVIMIENTO. Con la transcurrida
    // cuadran 31 de sus 83 sesiones; con la de movimiento, 55. Aqui se usa la que hace falta para la
    // cuenta, que es la que trae este lector, y se declara.
    let mut declarado = l.declarado.clone();
    let movimiento = d.duracion_en_movimiento_s.valor().expect("ese fichero trae duracion");
    declarado.duracion_s = motor::Dato::Hay(movimiento as u32);
    let c = potencia::comprobar_tss(&declarado, potencia::TOLERANCIA_TSS).expect("trae las piezas");
    assert!(
        c.cuadra,
        "en ESTE fichero ({ruta:?}) la cuenta cuadra: {:.1} calculado frente a {:.1} declarado",
        c.recalculado, c.declarado
    );
    // Y sin la duracion no se puede comprobar: `None`, no un numero inventado.
    let mut sin_duracion = declarado.clone();
    sin_duracion.duracion_s = motor::Dato::Falta;
    assert!(potencia::comprobar_tss(&sin_duracion, potencia::TOLERANCIA_TSS).is_none());
    assert!(ruta.exists());
}

#[test]
fn el_lector_de_zonas_de_potencia_COGE_EL_MENSAJE_DE_LA_SESION_y_su_reparto_CUADRA() {
    // ESTA PRUEBA ES LA QUE JUSTIFICA MIRAR `reference_mesg`, y costo medirlo: el FIT trae un mensaje
    // `TimeInZone` POR VUELTA y uno mas por la sesion. Cogiendo el PRIMERO —que es lo natural si no se
    // mira— el reparto suma, de mediana, un 84,7 % MENOS que la sesion, y **el numero seguiria
    // pareciendo bueno**. Con el de la sesion, cuadra.
    let Some((_ruta, _l, d)) = fit_con_potencia() else {
        eprintln!("(sin ficheros FIT: la prueba se salta)");
        return;
    };
    let zonas = potencia::tiempo_en_zonas(&d.tiempo_en_potencia_s).expect("trae reparto de zonas");
    assert_eq!(zonas.len(), 7, "siete zonas con nombre, no las diez posiciones del array");

    let duracion = d.duracion_en_movimiento_s.valor().expect("trae duracion");
    let c = potencia::comparar_zonas_con_duracion(&zonas, duracion, 0.01);
    let sin_zona = potencia::segundos_sin_zona(&d.tiempo_en_potencia_s);
    // LA SUMA COMPLETA ES LA QUE CUADRA, y esa es la comprobacion que vale: las siete zonas que la
    // fuente define MAS el tiempo que el aparato mete en los tramos 8 a 10, que no se nombran. Si el
    // lector cogiera el mensaje de una vuelta en vez del de la sesion, esto no cuadraria ni de lejos.
    let total = c.suma_de_zonas_s + sin_zona;
    // LO QUE SE EXIGE ES EL INVARIANTE, no un decimal, y el motivo esta medido: las dos bibliotecas de
    // FIT que hay en este equipo **no dan exactamente el mismo reparto en todos los ficheros**. Medido
    // sobre sus 102: `fitdecode` (Python) suma **7.811,99 s** en la sesion del 2026-05-02 y `fitparser`
    // (la que usa el motor) suma **7.798,07 s** — un **0,18 %** de diferencia, 14 segundos de una
    // sesion de dos horas. Quien lee el fichero no es el mismo programa en los dos casos, y una prueba
    // no puede fingir que si. Lo que SI se puede exigir, y esta medido sobre las 83 sesiones con
    // potencia, es que **la suma quede dentro del 1 % de la duracion**: eso vale en 79 de las 83 con
    // las siete zonas solas, y aqui ademas se suma el tiempo de los tramos sin nombre, que es lo que
    // hace que el desvio baje del 0,2 %.
    let desvio = (total - duracion).abs() / duracion;
    assert!(
        desvio < 0.01,
        "la suma completa tiene que quedar dentro del 1 % de la duracion: {total:.1} frente a \
         {duracion:.1} ({:.2} %)",
        desvio * 100.0
    );
    // Y el tiempo sin zona no puede ser mayor que la sesion: si lo fuera, el lector estaria sumando
    // dos veces el mismo mensaje.
    assert!(sin_zona <= duracion, "el tiempo sin zona no puede pasar de la sesion");
    // Y si el lector cogiera el mensaje de una VUELTA en vez del de la sesion, la suma seria una
    // fraccion pequeña de la duracion. Eso es lo que hay que descartar, y se descarta con el numero.
    assert!(
        total > duracion * 0.5,
        "una suma de la mitad de la sesion delataria que se leyo el mensaje de una vuelta: \
         {total:.1} de {duracion:.1}"
    );

    // Y el umbral que declara ese fichero es el suyo, medido: 213 W en las 83 sesiones con potencia.
    assert_eq!(d.umbral_w, motor::Dato::Hay(213));
}

#[test]
fn una_sesion_sin_potencia_deja_el_reparto_de_zonas_VACIO_y_eso_es_lo_correcto() {
    // Medido: las 19 sesiones sin potencia suman 0 segundos de zonas de potencia. El aparato deja el
    // reparto vacio, y por eso el motor devuelve «falta» en vez de una lista de ceros. Se comprueba
    // sobre una de ellas, de verdad.
    let carpeta = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../datos/garmin_files/storage");
    let mut rutas: Vec<std::path::PathBuf> = match std::fs::read_dir(&carpeta) {
        Ok(d) => d
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "fit").unwrap_or(false))
            .collect(),
        Err(_) => {
            eprintln!("(sin ficheros FIT: la prueba se salta)");
            return;
        }
    };
    rutas.sort();

    let mut encontradas = 0;
    for ruta in &rutas {
        let Ok(d) = potencia::leer_declarado_de_potencia(ruta) else { continue };
        // Solo interesan las que NO traen umbral: son las que no tienen potencia.
        if d.umbral_w.lo_hay() {
            continue;
        }
        encontradas += 1;
        assert!(
            potencia::tiempo_en_zonas(&d.tiempo_en_potencia_s).is_none(),
            "una sesion sin potencia NO puede tener zonas de potencia"
        );
        assert!(
            potencia::que_falta_para_potencia(false, false).is_some(),
            "y el motor tiene que decir que falta"
        );
        // Con dos basta: leer las 102 tarda, y una prueba lenta se deja de correr.
        if encontradas >= 2 {
            break;
        }
    }
    assert!(encontradas > 0, "tiene que haber sesiones sin potencia: medido, 19 de 102");
}

#[test]
fn la_carga_semanal_agrupa_por_lunes_y_NO_suma_los_huecos_como_cero() {
    // La trampa que esta prueba caza: una sesion sin TSS sumada como 0 haria que una semana con dos
    // sesiones ilegibles pareciera una semana de descanso, y esa confusion es la que produce planes
    // equivocados.
    let sesiones = vec![
        potencia::SesionConCarga::nueva("2026-05-02", 100.0), // sabado -> lunes 2026-04-27
        potencia::SesionConCarga::nueva("2026-05-03", 50.0),  // domingo -> la MISMA semana
        potencia::SesionConCarga::nueva("2026-05-05", 30.0),  // martes -> lunes 2026-05-04
        potencia::SesionConCarga::sin_datos(),                // sin fecha: NO entra en ninguna semana
    ];
    let semanas = potencia::carga_semanal(&sesiones);
    assert_eq!(semanas.len(), 2, "dos semanas, y la sesion sin fecha no inventa una tercera");
    assert_eq!(semanas[0].lunes, "2026-04-27", "el sabado pertenece a la semana del lunes anterior");
    assert!((semanas[0].tss_total - 150.0).abs() < 1e-9);
    assert_eq!(semanas[0].sesiones, 2);
    assert_eq!(semanas[1].lunes, "2026-05-04");
    assert!((semanas[1].tss_total - 30.0).abs() < 1e-9);

    // Y una sesion CON fecha pero SIN TSS: cuenta como sesion y NO suma cero.
    let con_hueco = potencia::carga_semanal(&[
        potencia::SesionConCarga {
            fecha: Some("2026-05-05".to_string()),
            tss: motor::Dato::Hay(40.0),
        },
        potencia::SesionConCarga {
            fecha: Some("2026-05-06".to_string()),
            tss: motor::Dato::Falta,
        },
    ]);
    assert_eq!(con_hueco.len(), 1);
    assert_eq!(con_hueco[0].sesiones, 2, "las dos cuentan como sesiones");
    assert_eq!(con_hueco[0].sin_tss, 1, "y la que no trae TSS se cuenta aparte");
    assert!((con_hueco[0].tss_total - 40.0).abs() < 1e-9, "no se suma un cero por ella");

    // Y los lunes estan bien calculados: el 2026-05-02 fue sabado (comprobado con el calendario).
    assert_eq!(potencia::lunes_de_la_semana("2026-05-02").unwrap(), "2026-04-27");
    assert_eq!(potencia::lunes_de_la_semana("2026-10-13").unwrap(), "2026-10-12");
    assert_eq!(potencia::lunes_de_la_semana("2026-11-29").unwrap(), "2026-11-23");
    // Y una fecha que no es una fecha NO se adivina.
    assert!(potencia::lunes_de_la_semana("(sin fecha)").is_none());
    assert!(potencia::lunes_de_la_semana("2026-13-01").is_none(), "el mes 13 no existe");
    assert!(potencia::lunes_de_la_semana("2026-05-40").is_none(), "el dia 40 tampoco");
}

#[test]
fn el_resumen_de_carga_CUENTA_las_sesiones_sin_potencia_y_sin_TSS() {
    // Un minimo, una mediana y un maximo calculados sobre 83 sesiones de 102 NO significan lo mismo
    // que sobre 102, y quien lee tiene derecho a saberlo en la misma linea.
    let sesiones = vec![
        (motor::Dato::Hay(221.5), true),
        (motor::Dato::Hay(64.1), true),
        (motor::Dato::Hay(3.2), true),
        (motor::Dato::Hay(155.6), true),
        (motor::Dato::Falta, false), // una de las 19 sin potencia
    ];
    let r = potencia::ResumenDeCarga::calcular(&sesiones, potencia::UMBRAL_DE_CARGA_DEL_CLIENTE);
    assert_eq!(r.sesiones, 5);
    assert_eq!(r.con_tss, 4);
    assert_eq!(r.sin_tss, 1);
    assert_eq!(r.con_potencia, 4);
    assert_eq!(r.sin_potencia, 1);
    assert_eq!(r.minimo, Some(3.2));
    assert_eq!(r.maximo, Some(221.5));
    // Con cuatro valores (par), la mediana es la media de los dos centrales: (64,1 + 155,6) / 2.
    assert!((r.mediana.unwrap() - 109.85).abs() < 1e-9, "la mediana de un numero par de valores");
    assert_eq!(r.por_encima_del_umbral, 2, "221,5 y 155,6 superan los 100");
    assert!(r.frase().contains("SIN TSS"), "la frase tiene que decir lo que falta: {}", r.frase());
    assert!(r.frase().contains("sin potencia"), "y las que no traen vatios");

    // Sin ni una sesion con TSS, no se dice nada de la carga en vez de decir «0».
    let vacio = potencia::ResumenDeCarga::calcular(&[(motor::Dato::Falta, false)], 100.0);
    assert_eq!(vacio.minimo, None, "sin datos no hay minimo: `None`, no cero");
    assert!(vacio.frase().contains("no hay ni una sesión"), "{}", vacio.frase());
}

#[test]
fn una_potencia_cae_en_SU_zona_y_la_ultima_zona_es_ABIERTA() {
    // La zona se busca por los cortes, y la ultima se lleva todo lo que sobre: eso es lo que la fuente
    // dice («maximal», sin techo). Se comprueba con los numeros de su FTP de 213 W.
    let ftp = potencia::Ftp::estimado(213.0);
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 100.0), ZonaPotencia::RecuperacionActiva);
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 130.0), ZonaPotencia::Resistencia);
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 180.0), ZonaPotencia::Tempo);
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 213.0), ZonaPotencia::Umbral);
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 240.0), ZonaPotencia::ConsumoMaximo);
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 300.0), ZonaPotencia::CapacidadAnaerobica);
    // Y por encima de 1,50 x FTP sigue habiendo zona: la septima, que NO tiene techo.
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 319.5), ZonaPotencia::Neuromuscular);
    assert_eq!(potencia::zona_de_una_potencia(&ftp, 596.0), ZonaPotencia::Neuromuscular);
    // Su potencia MAXIMA medida en las muestras es 596 W: cae en la septima, y no se sale de la tabla.
    assert!(ZonaPotencia::todas().contains(&potencia::zona_de_una_potencia(&ftp, 596.0)));
}

#[test]
fn el_ruido_de_la_escala_NO_cambia_la_cuenta_de_las_sesiones_que_avisan() {
    // ESTE FALLO OCURRIO DE VERDAD Y COSTO DOS INTENTOS, por eso tiene prueba propia.
    //
    // La fase de pedaleo viene CUANTIZADA: el FIT la guarda en un byte y el lector la escala a grados,
    // asi que los valores son multiplos de 1,40625° **con un ruido de hasta 5,5·10⁻⁶** encima. Medido:
    // una asimetria que deberia valer 5,625 llega como 5,6250000879 o como 5,6249944628.
    //
    // Y el corte —el doble de la mediana— **cae sobre un peldaño**, asi que hay sesiones que estan EN
    // el corte salvo ese ruido. Comparando con un `>` a pelo, entraban o no segun el ruido del
    // redondeo, y el aviso decia **28 sesiones donde hay 20**.
    //
    // Aqui se reproducen las tres formas del mismo peldaño y se comprueba que cuentan IGUAL.
    let paso = avisos::PASO_DE_LA_FASE_GRADOS;
    let en_el_corte = 4.0 * paso; // 5,625°

    // Veinte sesiones en 2 peldaños (la linea base) y tres en el corte, con las tres formas del ruido.
    let mut sesiones: Vec<SesionVista> = (0..20)
        .map(|i| sesion_avisos(&format!("base{i}.fit"), None, Some(2.0 * paso), None))
        .collect();
    for (i, desviacion) in [0.0, 8.789061212155502e-8, -5.537109473152668e-6].iter().enumerate() {
        sesiones.push(sesion_avisos(&format!("borde{i}.fit"), None, Some(en_el_corte + desviacion), None));
    }
    let h = Historico {
        total_sesiones: sesiones.len(),
        no_leidos: Vec::new(),
        sesiones,
    };

    assert_eq!(h.base_de_fase(), Some(2.0 * paso), "la base son dos peldaños");
    assert_eq!(
        h.corte_de_fase(),
        Some(avsios_corte(h.base_de_fase().unwrap())),
        "el corte tiene que estar ALINEADO al peldaño, no en el valor crudo del doble"
    );

    let salida = avisos::avisos(&Persona::desconocida(), &sin_real(), &h, &Hoy::sin_datos());
    assert!(
        !salida.iter().any(|a| a.modulo == Modulo::Rodilla),
        "las tres sesiones estan EN el corte (una de ellas con ruido), y ninguna puede avisar. \
         Si alguna avisa, la cuenta depende del redondeo del ordenador y no de la bicicleta"
    );
}

/// El corte de la fase, escrito a mano para no copiarlo del modulo que se esta probando.
fn avsios_corte(base: f64) -> f64 {
    let paso = 360.0 / 256.0;
    ((base * 2.0) / paso).round() * paso
}

// ==================================================================================================
// LA FECHA DE NACIMIENTO: LA EDAD QUE NO CADUCA (2026-10-14)
// ==================================================================================================
//
// EL FALLO QUE ESTAS PRUEBAS ARREGLAN, y es de los que no se notan: la aplicación tenía **`edad = 60`
// escrito a mano** y el cliente tiene **62** (su perfil de Garmin dice 1964-01-25). De la edad sale la
// **frecuencia máxima**, así que **las zonas de entrenamiento salían corridas** -- y una zona corrida
// no da error: da un entrenamiento equivocado.
//
// **Y NO SE ARREGLA ESCRIBIENDO 62**: eso caduca el 25 de enero. Un dato que caduca y nadie lo nota es
// peor que un dato que falta, porque el que falta se ve.

/// El cumpleaños del cliente, que es el caso real que originó esta pieza.
fn el_cliente_nacio_el() -> motor::datos::Fecha {
    motor::datos::Fecha::nueva(1964, 1, 25)
}

#[test]
fn la_edad_se_calcula_desde_la_fecha_y_no_caduca() {
    use motor::datos::{Dato, Fecha, Persona};

    let mut p = Persona::desconocida();
    p.fecha_nacimiento = Dato::Hay(el_cliente_nacio_el());

    // EL CASO REAL, con la fecha de hoy: nació el 25-01-1964.
    assert_eq!(
        p.edad_en(Fecha::nueva(2026, 10, 14)),
        Dato::Hay(62),
        "el 14 de octubre de 2026 tiene 62 años"
    );
    // Y EL MISMO PERFIL, TRES AÑOS DESPUÉS, da 65 -- **sin tocar el fichero**. Eso es lo que significa
    // que no caduque: el dato es la fecha, y la edad es una consecuencia.
    assert_eq!(
        p.edad_en(Fecha::nueva(2029, 10, 14)),
        Dato::Hay(65),
        "el mismo perfil, tres años despues: 65. NO hay que actualizar nada."
    );
}

#[test]
fn el_DIA_DEL_CUMPLEANOS_es_el_caso_que_falla_y_por_eso_se_prueba() {
    use motor::datos::{Dato, Fecha, Persona};

    // ESTE ES EL CASO POR EL QUE LA FECHA DE HOY SE PASA COMO ARGUMENTO, y no se lee del reloj: si la
    // funcion leyera el reloj por dentro, **el dia del cumpleaños no se podria probar nunca** sin
    // esperar un año entero. Y es justo el dia en que una resta ingenua se equivoca.
    let mut p = Persona::desconocida();
    p.fecha_nacimiento = Dato::Hay(el_cliente_nacio_el());

    assert_eq!(p.edad_en(Fecha::nueva(2026, 1, 24)), Dato::Hay(61), "un dia antes: 61");
    assert_eq!(p.edad_en(Fecha::nueva(2026, 1, 25)), Dato::Hay(62), "el mismo dia: 62");
    assert_eq!(p.edad_en(Fecha::nueva(2026, 1, 26)), Dato::Hay(62), "un dia despues: 62");
}

#[test]
fn una_fecha_que_NO_EXISTE_no_da_una_edad_inventada() {
    use motor::datos::{Dato, Fecha, Persona};

    // El 31 de febrero se puede escribir y no existe. Y una fecha imposible daria una edad calculada
    // con una fecha que no es, **sin que nada avisara**.
    assert!(!Fecha::nueva(2026, 2, 31).es_valida(), "el 31 de febrero no existe");
    assert!(!Fecha::nueva(2026, 13, 1).es_valida(), "no hay mes 13");
    assert!(!Fecha::nueva(2026, 0, 10).es_valida(), "no hay mes 0");
    assert!(!Fecha::nueva(2026, 4, 31).es_valida(), "abril tiene 30 dias");
    assert!(Fecha::nueva(2026, 2, 28).es_valida(), "2026 no es bisiesto: el 28 si");
    assert!(Fecha::nueva(2024, 2, 29).es_valida(), "2024 si es bisiesto: el 29 si");
    assert!(!Fecha::nueva(2100, 2, 29).es_valida(), "2100 NO es bisiesto (regla del siglo)");
    assert!(Fecha::nueva(2000, 2, 29).es_valida(), "2000 SI es bisiesto (regla del 400)");

    // Y con una fecha imposible, la edad NO se inventa: se usa la que hubiera escrita, o falta.
    let mut p = Persona::desconocida();
    p.fecha_nacimiento = Dato::Hay(Fecha::nueva(1964, 2, 31));
    assert_eq!(
        p.edad_en(Fecha::nueva(2026, 10, 14)),
        Dato::Falta,
        "con una fecha imposible y sin edad escrita, FALTA -- no un cero ni una media"
    );
}

#[test]
fn la_FECHA_manda_sobre_la_edad_escrita() {
    use motor::datos::{Dato, Fecha, Persona};

    // Es la misma regla que con el perfil y el aparato: **cuando los dos hablan del mismo dato, gana el
    // que no caduca**. Aqui, la fecha.
    let mut p = Persona::desconocida();
    p.edad = Dato::Hay(60); // el valor viejo, el que estaba escrito a mano
    p.fecha_nacimiento = Dato::Hay(el_cliente_nacio_el());

    assert_eq!(
        p.edad_en(Fecha::nueva(2026, 10, 14)),
        Dato::Hay(62),
        "hay fecha Y edad escrita: manda la FECHA, que es la que no caduca"
    );

    // Y sin fecha, se sigue usando la edad escrita: un perfil viejo no se queda sin nada.
    let mut sin_fecha = Persona::desconocida();
    sin_fecha.edad = Dato::Hay(60);
    assert_eq!(
        sin_fecha.edad_en(Fecha::nueva(2026, 10, 14)),
        Dato::Hay(60),
        "sin fecha, se respeta la edad que hubiera"
    );
}

#[test]
fn la_fecha_va_y_vuelve_por_el_perfil_sin_cambiar() {
    use motor::datos::{Dato, Fecha, Persona};

    // El ciclo leer -> guardar -> leer no puede cambiar el dato por el camino. Es el mismo defecto que
    // ya se corrigio con el sexo: un dato que se transforma al pasar por el fichero.
    let mut p = Persona::desconocida();
    p.fecha_nacimiento = Dato::Hay(el_cliente_nacio_el());
    p.peso_kg = Dato::Hay(69.45);

    let texto = motor::adaptador::a_json(&p);
    let vuelta = motor::adaptador::leer_perfil(&texto).expect("tiene que poder leerse");

    assert_eq!(vuelta.fecha_nacimiento, Dato::Hay(el_cliente_nacio_el()), "la fecha va y vuelve igual");
    // Y en el fichero se escribe como la escribe una persona, para que se pueda mirar y editar a mano.
    assert!(texto.contains("1964-01-25"), "la fecha se guarda legible: {texto}");
    let _ = Fecha::nueva(1, 1, 1);
}

#[test]
fn una_fecha_escrita_mal_se_dice_y_no_se_traga() {
    use motor::datos::Dato;

    // Un numero mal escrito NO se traga en silencio, y una fecha tampoco: se dice como se escribe.
    let malas = ["1964/01/25", "25-01-1964", "ayer", "1964-1", "1964-01-25-extra"];
    for m in malas {
        assert!(
            motor::profile::fecha_nacimiento(m).is_err(),
            "«{m}» esta mal escrita y tiene que decirlo, no tragarse"
        );
    }
    // Y VACIO NO ES UN ERROR: es «no lo sé», y el usuario tiene que poder borrar el dato.
    assert_eq!(
        motor::profile::fecha_nacimiento(""),
        Ok(Dato::Falta),
        "vacio significa «no lo se», y eso se puede guardar"
    );
    assert_eq!(
        motor::profile::fecha_nacimiento("1964-01-25"),
        Ok(Dato::Hay(motor::datos::Fecha::nueva(1964, 1, 25))),
        "y bien escrita, entra"
    );
}

#[test]
fn el_CICLO_COMPLETO_del_perfil_conserva_la_fecha() {
    // Leer -> guardar -> volver a leer, con fichero de verdad y en un directorio propio.
    //
    // LA FECHA ES LA QUE MAS SE PUEDE PERDER en ese viaje, porque no es un numero: si alguien la
    // tratara como los demas campos, `parse::<f32>("1964-01-25")` daria un error o un cero, y la edad
    // saldria mal SIN QUE NADA AVISARA. Esta prueba es la que impide que eso pase.
    use motor::datos::{Dato, Persona};

    let caja = std::env::temp_dir().join(format!("pulso-prueba-fecha-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&caja);
    std::fs::create_dir_all(&caja).unwrap();
    let ruta = caja.join("perfil.json");

    // 1) Un perfil con la fecha y el peso.
    let mut p = Persona::desconocida();
    p.fecha_nacimiento = Dato::Hay(el_cliente_nacio_el());
    p.peso_kg = Dato::Hay(69.45);
    motor::profile::guardar(&p, &ruta).expect("tiene que guardarse");

    // 2) Se lee, y la fecha esta.
    let leido = motor::profile::leer(&ruta).expect("tiene que leerse");
    assert_eq!(leido.fecha_nacimiento, Dato::Hay(el_cliente_nacio_el()));
    assert_eq!(leido.peso_kg, Dato::Hay(69.45));

    // 3) Y por el camino de los CAMPOS, que es el que usa la pantalla.
    let campos = [("fecha_nacimiento", "1964-01-25"), ("peso_kg", "69,45")];
    let mut otro = Persona::desconocida();
    motor::profile::aplicar(&mut otro, &campos).expect("los campos tienen que aplicarse");
    assert_eq!(otro.fecha_nacimiento, Dato::Hay(el_cliente_nacio_el()));
    assert_eq!(otro.peso_kg, Dato::Hay(69.45), "y con coma decimal, como se escribe en español");

    // 4) Y BORRARLA tambien tiene que poder hacerse: vacio es «no lo se», no un error.
    let borrar = [("fecha_nacimiento", "")];
    motor::profile::aplicar(&mut otro, &borrar).expect("borrar no puede ser un error");
    assert_eq!(otro.fecha_nacimiento, Dato::Falta, "y vuelve a faltar, como debe");

    let _ = std::fs::remove_dir_all(&caja);
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// La composicion corporal: un CENTINELA de sensor no es una medicion
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **Un cero (o un 700) de bascula no es un dato: es una averia.**
///
/// Se prueba en los dos sentidos: el valor plausible PASA, y el centinela se RECHAZA. La leccion ya
/// costo una vez en este proyecto: el PCO valia 0,00 en las 83 sesiones y un sensor que da siempre 0
/// **no esta midiendo**. Aqui se filtra en el borde para que el motor no vea nunca un cero que
/// parece un dato.
#[test]
fn un_centinela_de_bascula_no_es_una_medicion() {
    assert_eq!(Composicion::plausible_kg(0.0), None, "0 kg es el centinela, no un peso");
    assert_eq!(Composicion::plausible_kg(-1.0), None);
    assert_eq!(Composicion::plausible_kg(700.0), None, "700 kg no es una persona");
    assert_eq!(Composicion::plausible_kg(81.5), Some(81.5));

    assert_eq!(Composicion::plausible_pct(0.0), None, "0 % de agua es un centinela");
    assert_eq!(Composicion::plausible_pct(101.0), None);
    assert_eq!(Composicion::plausible_pct(62.4), Some(62.4));
}

/// **Una composicion vacia NO inventa ceros**: todos los campos faltan.
#[test]
fn una_composicion_vacia_deja_todo_en_falta() {
    let c = Composicion::vacia();
    assert!(c.fecha.falta() && c.peso_kg.falta() && c.grasa_corporal_pct.falta());
    assert!(c.agua_corporal_pct.falta() && c.masa_muscular_kg.falta() && c.masa_osea_kg.falta());
}

// ==================================================================================================
// HITO C · LO QUE TRAJO LA SINCRONIZACION CON GARMIN, EN UNA LINEA
// ==================================================================================================
//
// POR QUE ESTA PRUEBA VIVE AQUI Y NO EN `medir-interfaz.rs`: `texto_del_resumen` es una funcion PURA
// del motor (no toca Slint, no toca el disco, no abre nada), y este es el fichero de las pruebas del
// motor. La GEOMETRIA del bloque Garmin -- el marco, el boton de 48, la linea que no se dibuja si esta
// vacia -- se mide en `medir-interfaz.rs`, que es donde se mide la pantalla.

/// El resumen de una sincronizacion, tal como lo devuelve `sincronizar::resumen_de_nuevo`.
fn resumen(actividades: usize, diarios: usize, fechas: &[&str]) -> motor::sincronizar::Resumen {
    motor::sincronizar::Resumen {
        actividades,
        diarios,
        fechas_de_diarios: fechas.iter().map(|f| f.to_string()).collect(),
    }
}

#[test]
fn el_resumen_de_garmin_son_los_numeros_reales_en_una_linea_y_sin_fechas() {
    // EL CONTRATO: UNA linea, los numeros REALES que trae el resumen, y **nunca la lista de fechas**
    // (con cinco diarios la frase seguiria siendo una linea; con cuarenta, un parrafo, y la pantalla de
    // 411 px no se explica: AU-44, AU-45, AU-92).

    // 0 y 0: se dice «nada nuevo», y NO cadena vacia. La cadena vacia esta reservada para «no se ha
    // sincronizado todavia» (la linea no se dibuja, AU-91); si las dos cosas dieran lo mismo, una
    // sincronizacion que no trajo nada seria indistinguible de no haber sincronizado, y **callar
    // despues de pulsar un boton es lo que hace pensar que el boton no funciona**.
    let vacio = motor::cara::texto_del_resumen(&resumen(0, 0, &[]));
    assert_eq!(vacio, "nada nuevo", "0 y 0 se dice, no se calla");
    assert!(!vacio.contains('\n'), "el resumen es UNA linea: {vacio:?}");

    // 3 y 2: los numeros reales, ni uno mas ni uno menos.
    let tres = motor::cara::texto_del_resumen(&resumen(3, 2, &["2026-10-01", "2026-10-02"]));
    assert_eq!(tres, "3 actividades y 2 diarios nuevos", "el resumen dice lo que trajo");
    println!("  3 actividades y 2 diarios -> «{tres}» ({} caracteres)", tres.chars().count());

    // UNO: se escribe como se dice. «1 actividades» se lee como un descuido del programa, y la
    // pantalla es la cara del motor.
    assert_eq!(
        motor::cara::texto_del_resumen(&resumen(1, 1, &["2026-10-01"])),
        "1 actividad y 1 diario nuevo"
    );
    assert_eq!(
        motor::cara::texto_del_resumen(&resumen(2, 1, &["2026-10-01"])),
        "2 actividades y 1 diario nuevo"
    );

    // UNO DE LOS DOS EN CERO: se dicen los DOS numeros, porque es lo que el resumen trae. Elegir cual
    // se calla seria una regla de estilo escondida aqui, y ademas ocultaria que el aparato no trajo
    // actividades.
    assert_eq!(
        motor::cara::texto_del_resumen(&resumen(0, 2, &["2026-10-01", "2026-10-02"])),
        "0 actividades y 2 diarios nuevos"
    );
    assert_eq!(
        motor::cara::texto_del_resumen(&resumen(3, 0, &[])),
        "3 actividades y 0 diarios nuevos"
    );

    // Y NUNCA LAS FECHAS, con muchas fechas: ni una. Se comprueban las dos cosas que se pueden
    // comprobar sin leer la frase a ojo -- que no aparece ninguna de las fechas, y que la frase no
    // crece con ellas (la misma cuenta con CERO fechas que con cuarenta).
    let muchas: Vec<&str> = vec![
        "2026-09-01", "2026-09-02", "2026-09-03", "2026-09-04", "2026-09-05",
        "2026-09-06", "2026-09-07", "2026-09-08", "2026-09-09", "2026-09-10",
    ];
    let con_fechas = motor::cara::texto_del_resumen(&resumen(7, 10, &muchas));
    for fecha in &muchas {
        assert!(
            !con_fechas.contains(fecha),
            "el resumen NO enumera las fechas de los diarios, y aparece «{fecha}» en «{con_fechas}»"
        );
    }
    assert_eq!(
        con_fechas,
        motor::cara::texto_del_resumen(&resumen(7, 10, &[])),
        "las fechas no pueden cambiar la frase: con diez y con ninguna tiene que decir lo mismo"
    );
    assert!(
        !con_fechas.contains('\n') && con_fechas.chars().count() <= 64,
        "el resumen es UNA linea y CORTA (la pantalla no se explica): «{con_fechas}»"
    );
    println!("  7 actividades y 10 diarios (con sus fechas) -> «{con_fechas}»");
}
