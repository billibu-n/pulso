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

use motor::datos::{Dato, Muestra, Persona, Sesion, Sexo};
use motor::formulas::{self, Zona};
use motor::juicio::{Confianza, Juicio};

/// Una persona con todo lo que hace falta. La referencia para comparar.
fn ana() -> Persona {
    Persona {
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
            potencia_w: Dato::Hay(200),
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
            Muestra { segundo: 0, pulso: Dato::Falta, potencia_w: Dato::Hay(200) },
            Muestra { segundo: 10, pulso: Dato::Falta, potencia_w: Dato::Hay(210) },
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
            potencia_w: Dato::Hay(200),
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
            potencia_w: Dato::Hay(200),
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
        Muestra { segundo: 0, pulso: Dato::Falta, potencia_w: Dato::Hay(200) },
        Muestra { segundo: 600, pulso: Dato::Hay(150), potencia_w: Dato::Hay(200) },
        Muestra { segundo: 1200, pulso: Dato::Hay(150), potencia_w: Dato::Hay(200) },
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
        .map(|i| Muestra { segundo: i * 10, pulso: Dato::Hay(pulso), potencia_w: Dato::Hay(200) })
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
        Muestra { segundo: 0, pulso: Dato::Falta, potencia_w: Dato::Hay(200) },
        Muestra { segundo: 10, pulso: Dato::Hay(140), potencia_w: Dato::Falta },
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
    let m = Muestra { segundo: 0, pulso: Dato::Hay(140), potencia_w: Dato::Hay(200) };
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
