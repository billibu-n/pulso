//! **EL RECORRIDO DEL USUARIO, paso a paso, y comprobado.**
//!
//! POR QUE EXISTE ESTE FICHERO, y es la critica mas dura que se le hizo al proyecto:
//!
//! > *«Las 86 pruebas comprueban **el motor**, y las 8 de medicion comprueban **la geometria**.
//! > **Cero pruebas comprueban que un usuario pueda hacer algo.**»*
//!
//! Y era verdad. El motor tenia 625 resultados con su margen y su fuente, y **nadie habia escrito ni
//! comprobado el camino que los une**. Eso explica por que costaba tanto «cumplir la tarea»: **no
//! habia tarea escrita, solo piezas sueltas**.
//!
//! ESTAS PRUEBAS SON ESE CAMINO. Cada una es **un paso del recorrido** de
//! `docs/RECORRIDO-del-usuario.md`, y todas juntas comprueban **una sola cosa**: que la aplicacion
//! **le sirve a una persona de verdad**, desde que la abre hasta que ve sus numeros.
//!
//! LA COSTURA QUE HACEN POSIBLE, y es la decision de diseño que trae este fichero: **lo que la
//! pantalla enseña se obtiene con una funcion, `cara::lo_que_ensena`, que NO abre ninguna ventana.**
//! La ventana solo pinta lo que esa funcion devuelve. Sin esa separacion, comprobar el recorrido
//! exigiria abrir la aplicacion y mirarla -- que es exactamente lo que la norma prohibe (AU-65):
//! **si solo se puede validar mirandolo, no esta terminado.**
//!
//! Y ESTAS PRUEBAS NO ABREN VENTANA NUNCA. Es una regla del proyecto y tiene un motivo que ya costo
//! caro: una vez se abrio, sin querer, en la pantalla del dueño.

// Los nombres de estas pruebas llevan Mayusculas a proposito, para que se lea QUE comprueban cuando
// fallan. `pruebas.rs` ya tiene esta misma excepcion, y sin ella el compilador da 8 avisos.
#![allow(non_snake_case)]

use motor::analisis::Real;
use motor::cara::{self, Ensenado};
use motor::datos::{Dato, Persona, Sexo};
use motor::umbral::ZonaCiclismo;

/// Una persona **de la que no sabemos nada**. Es como empieza todo, y es honesto.
fn nadie() -> Persona {
    Persona::desconocida()
}

/// El cliente, con lo que declaro: 60 años, y su umbral medido en una prueba de lactato.
///
/// **Estos numeros NO van en el codigo de la aplicacion**: esto es una prueba, y aqui si tienen que
/// estar, porque una prueba se escribe **con un caso concreto delante**. Lo que no puede haber es un
/// caso concreto **dentro del programa**.
fn el_cliente_declara_lo_suyo() -> Persona {
    let mut p = Persona::desconocida();
    p.edad = Dato::Hay(60);
    p.peso_kg = Dato::Hay(70.2);
    p.altura_cm = Dato::Hay(178.0);
    p.pulso_reposo = Dato::Hay(48);
    p.sexo = Dato::Hay(Sexo::Masculino);
    p.umbral_lactato = Dato::Hay(169);
    p
}

/// Unas sesiones que SI se leyeron. Se construye a mano **a proposito**: asi la prueba del recorrido
/// no depende de tener 1.003 MB de datos al lado, y **se ejecuta siempre**. Una prueba que necesita
/// datos que no estan es una prueba que se deja de correr.
fn sus_sesiones() -> Real {
    Real {
        sesiones: 102,
        muestras: 549_842,
        rr: 1_259_082,
        deriva_media: Some(9.682),
        eficiencia_media: Some(0.98),
        escapados: 43,
        hrv_medio: Some(9.2),
        maxima_vista: Some(183),
        no_leidos: Vec::new(),
        desde: Some(1_700_000_000),
        hasta: Some(1_720_000_000),
    }
}

// ==================================================================================================
// PASO 1 · EL PRIMER ARRANQUE, SIN NADA
// ==================================================================================================

#[test]
fn paso_1_sin_nada_la_aplicacion_dice_que_no_puede_diagnosticar_y_NO_enseña_numeros() {
    // ESTE ERA EL FALLO MAS GRAVE Y EL MAS DIFICIL DE VER, porque **no se notaba**: la pantalla
    // enseñaba `169±3` y «Precision alta» con un umbral **escrito a mano en el codigo**, y a la vez
    // decia «Sin datos todavia». **Se contradecia**, y los numeros parecian buenos.
    //
    // Un motor que se inventa un numero es peor que uno que no arranca: el que no arranca se nota.
    let ensenado = cara::lo_que_ensena(&nadie(), None);

    assert!(
        !ensenado.puede_diagnosticar,
        "sin datos de la persona NO se puede diagnosticar, y tiene que decirlo"
    );
    assert!(
        ensenado.cifras.is_empty(),
        "y entonces NO puede enseñar cifras calculadas; enseño: {:?}",
        ensenado.cifras
    );
    assert!(
        ensenado.zonas.is_empty(),
        "ni zonas: sin umbral no hay zonas que valgan"
    );
    assert!(
        !ensenado.que_falta.is_empty(),
        "y tiene que decir QUE FALTA, no solo que falta algo"
    );
}

#[test]
fn paso_1_lo_que_falta_se_dice_en_UNA_linea_y_dice_donde_se_rellena() {
    // La norma del proyecto (AU-44 y AU-45): **la pantalla no se explica**. Un vacio se dice en UNA
    // linea, y **diciendo que hacer**, no dando pena. Y el texto va en la pantalla, asi que tiene que
    // caber: una parrafada en un movil de 411 px no se lee, se salta.
    let ensenado = cara::lo_que_ensena(&nadie(), None);

    let lineas = ensenado.que_falta.lines().count();
    assert!(lineas <= 2, "lo que falta se dice en una o dos lineas, no en {lineas}");

    let texto = ensenado.que_falta.to_lowercase();
    assert!(
        texto.contains("dato") || texto.contains("perfil") || texto.contains("rellena"),
        "tiene que decir DONDE se rellena, no solo que falta: «{}»",
        ensenado.que_falta
    );
}

// ==================================================================================================
// PASO 5 · LOS DATOS DE LA PERSONA
// ==================================================================================================

#[test]
fn paso_5_con_sus_datos_ya_se_puede_diagnosticar_y_los_numeros_son_SUYOS() {
    // ESTE ES EL PASO QUE HACE QUE LOS RESULTADOS SEAN SUYOS. Mientras los datos esten escritos en el
    // codigo, la aplicacion **no es usable por nadie mas que por quien la compilo**.
    let p = el_cliente_declara_lo_suyo();
    let ensenado = cara::lo_que_ensena(&p, None);

    assert!(
        ensenado.puede_diagnosticar,
        "con su umbral declarado ya se puede diagnosticar"
    );
    assert_eq!(ensenado.cifras.len(), 4, "y salen las cuatro cifras");

    // Y EL NUMERO TIENE QUE SER EL SUYO, no uno plausible cualquiera. Su umbral es 169.
    let umbral = &ensenado.cifras[0];
    assert!(
        umbral.contains("169"),
        "la primera cifra es su umbral, y el suyo es 169; salio «{umbral}»"
    );
    // Y LLEVA SU MARGEN, que es lo que este motor tiene de distinto: un numero solo es una promesa.
    assert!(
        umbral.contains('±'),
        "el margen va CON el numero, no en una nota al pie; salio «{umbral}»"
    );
}

#[test]
fn paso_5_una_persona_sin_umbral_pero_con_edad_da_un_margen_MAS_ANCHO_y_lo_dice() {
    // La honestidad del motor, comprobada en el recorrido: **cuanto menos se sabe, mas ancho es el
    // margen**. Si los dos margenes salieran iguales, el margen no estaria diciendo nada.
    let mut solo_la_edad = Persona::desconocida();
    solo_la_edad.edad = Dato::Hay(60);

    let con_umbral = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);
    let sin_umbral = cara::lo_que_ensena(&solo_la_edad, None);

    assert!(sin_umbral.puede_diagnosticar, "con la edad ya se puede estimar");
    assert_ne!(
        con_umbral.cifras[0], sin_umbral.cifras[0],
        "estimar y medir NO pueden dar el mismo numero ni el mismo margen"
    );
    // Y la referencia tiene que decirlo: «medido» frente a «estimado».
    assert_ne!(
        con_umbral.precision.to_lowercase(),
        sin_umbral.precision.to_lowercase(),
        "la pantalla tiene que decir DE DONDE sale el dato: medido no es lo mismo que estimado"
    );
}

// ==================================================================================================
// PASO 4 y 6 · LOS DATOS DEL APARATO, Y EL ANALISIS
// ==================================================================================================

#[test]
fn paso_6_con_sus_sesiones_aparecen_la_deriva_y_la_eficiencia() {
    // Sin datos del aparato, la deriva y la eficiencia **no existen** (no son cero: no existen). Con
    // ellos, aparecen. Esta prueba comprueba el cambio, que es lo que el usuario ve.
    let p = el_cliente_declara_lo_suyo();

    let sin_sesiones = cara::lo_que_ensena(&p, None);
    let con_sesiones = cara::lo_que_ensena(&p, Some(&sus_sesiones()));

    assert!(
        !sin_sesiones.aviso.contains("deriva"),
        "sin sesiones no se puede hablar de deriva; dijo «{}»",
        sin_sesiones.aviso
    );
    assert!(
        con_sesiones.aviso.contains("9,7"),
        "con sus 102 sesiones, la deriva media es 9,7 %; dijo «{}»",
        con_sesiones.aviso
    );
    assert!(
        con_sesiones.aviso.contains("102"),
        "y de cuantas sesiones sale; dijo «{}»",
        con_sesiones.aviso
    );
}

#[test]
fn paso_6_las_zonas_son_TRAMOS_y_no_limites_repetidos() {
    // ESTE DEFECTO YA SALIO UNA VEZ y por eso esta comprobado aqui: la primera version devolvia solo
    // el suelo de cada zona, y entonces «recuperacion» y «resistencia» salian con **el mismo numero**,
    // porque las dos empiezan donde acaba la anterior. **Un informe que dice dos veces 137 no se
    // entiende**, y el fallo era de lectura, no de calculo.
    let ensenado = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);

    assert_eq!(ensenado.zonas.len(), 5, "son cinco zonas");

    let nombres: Vec<&String> = ensenado.zonas.iter().map(|(n, _, _)| n).collect();
    assert_eq!(
        nombres,
        vec!["recuperación", "resistencia", "tempo", "umbral", "consumo máximo"],
        "los nombres de las zonas, en orden"
    );

    // Y los tramos: ninguno puede ser igual al anterior, que era el defecto.
    let tramos: Vec<&String> = ensenado.zonas.iter().map(|(_, t, _)| t).collect();
    for par in tramos.windows(2) {
        assert_ne!(
            par[0], par[1],
            "dos zonas seguidas no pueden enseñar el mismo tramo: {:?}",
            tramos
        );
    }
    assert!(
        tramos[0].starts_with("hasta"),
        "la primera zona empieza en el suelo, asi que se dice «hasta»: «{}»",
        tramos[0]
    );
}

// ==================================================================================================
// LO QUE EL RECORRIDO NO PIDE Y DECIDE SI ESTO SIRVE
// ==================================================================================================

#[test]
fn el_recorrido_de_una_persona_entera_se_puede_recorrer_SIN_ABRIR_VENTANA() {
    // ESTA PRUEBA ES LA QUE FALTABA, dicha de una vez: recorre los pasos **en orden** y comprueba que
    // cada uno **deja el estado en el que el siguiente se apoya**.
    //
    // Y lo hace **sin abrir una ventana**. Eso no es un detalle de comodidad: es lo que permite
    // comprobarlo **siempre**, incluso mientras el dueño trabaja en este equipo.
    let mut paso = 0;

    // --- Arranca sin nada, y lo dice ---
    let vacio = cara::lo_que_ensena(&nadie(), None);
    assert!(!vacio.puede_diagnosticar, "paso {paso}: sin nada no se diagnostica");
    paso += 1;

    // --- Mete sus datos: ya puede ---
    let p = el_cliente_declara_lo_suyo();
    let con_datos = cara::lo_que_ensena(&p, None);
    assert!(con_datos.puede_diagnosticar, "paso {paso}: con sus datos ya puede");
    assert_eq!(con_datos.cifras.len(), 4, "paso {paso}");
    paso += 1;

    // --- Trae sus sesiones: aparecen la deriva y la eficiencia ---
    let con_sesiones = cara::lo_que_ensena(&p, Some(&sus_sesiones()));
    assert!(con_sesiones.aviso.contains("9,7"), "paso {paso}");
    assert_eq!(con_sesiones.zonas.len(), 5, "paso {paso}");
    paso += 1;

    // --- Y VUELVE OTRO DIA: la memoria le devuelve LO MISMO, sin releer ---
    //
    // Aqui se comprueba lo que de verdad decide si esto se usa dos veces: que volver **no cambie el
    // resultado**. Si al volver los numeros bailaran, no se podria comparar un dia con otro -- y
    // comparar es exactamente para lo que sirve esto.
    let el_mismo_dia_siguiente = cara::lo_que_ensena(&p, Some(&sus_sesiones()));
    assert_eq!(
        con_sesiones, el_mismo_dia_siguiente,
        "paso {paso}: volver otro dia tiene que dar EL MISMO resultado"
    );
    paso += 1;

    // --- Y cambia sus datos: los numeros cambian CON el ---
    let mut mas_viejo = p.clone();
    mas_viejo.umbral_lactato = Dato::Hay(160);
    let cambiado = cara::lo_que_ensena(&mas_viejo, Some(&sus_sesiones()));
    assert_ne!(
        cambiado.cifras[0], con_sesiones.cifras[0],
        "paso {paso}: si cambia su umbral, el numero tiene que cambiar"
    );

    assert_eq!(paso, 4, "se recorrieron los cuatro estados del recorrido");
}

#[test]
fn las_cifras_y_las_zonas_van_siempre_con_su_margen_y_su_fuente() {
    // La promesa de este motor, comprobada en la capa que la enseña: **un numero sin su margen y sin
    // su fuente no se enseña**. Es la diferencia entre un dato y una promesa.
    let ensenado = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), Some(&sus_sesiones()));

    assert!(
        !ensenado.precision.is_empty(),
        "tiene que decir DE DONDE sale: medido, estimado, o de poblacion"
    );
    assert!(
        ensenado.precision.to_lowercase().contains("medido")
            || ensenado.precision.to_lowercase().contains("estimad"),
        "y decirlo con palabras que se entiendan: «{}»",
        ensenado.precision
    );

    // Y el margen en las cifras que son un rango. Las que son un recuento (sesiones, lecturas) no
    // llevan margen, y es correcto: contar no se estima.
    assert!(
        ensenado.cifras[0].contains('±'),
        "el umbral es un rango, asi que lleva margen"
    );
}

#[test]
fn lo_que_se_enseña_no_lleva_NINGUN_numero_escrito_a_mano() {
    // LA COMPROBACION QUE CAZA EL FALLO DE ORIGEN. Si alguien vuelve a escribir `169` dentro del
    // codigo de la aplicacion, esta prueba lo delata: con una persona de la que no sabemos nada, **no
    // puede aparecer ningun numero de nadie**.
    let ensenado = cara::lo_que_ensena(&nadie(), None);

    let todo = format!(
        "{:?} {} {} {}",
        ensenado.cifras, ensenado.aviso, ensenado.precision, ensenado.vacio
    );
    for sospechoso in ["169", "183", "60", "178", "70,2", "48"] {
        assert!(
            !todo.contains(sospechoso),
            "sin saber nada de la persona no puede salir «{sospechoso}» en pantalla. \
             Eso seria un dato de nadie con aspecto de dato. Texto: {todo}"
        );
    }
}

#[test]
fn el_orden_de_las_zonas_es_el_del_esfuerzo_y_no_el_alfabetico() {
    // Un detalle que parece tonto y no lo es: si las zonas salieran ordenadas por nombre, la pantalla
    // enseñaria el orden del diccionario en vez del orden del esfuerzo -- y el ciclista leeria su
    // entrenamiento al reves.
    let esperadas = [
        ZonaCiclismo::Recuperacion,
        ZonaCiclismo::Resistencia,
        ZonaCiclismo::Tempo,
        ZonaCiclismo::Umbral,
        ZonaCiclismo::ConsumoMaximo,
    ];
    let por_esfuerzo: Vec<String> = esperadas.iter().map(|z| z.nombre().to_string()).collect();
    let por_nombre: Vec<String> = {
        let mut v = por_esfuerzo.clone();
        v.sort();
        v
    };
    assert_ne!(
        por_esfuerzo, por_nombre,
        "si coincidieran, esta prueba no comprobaria nada y habria que borrarla"
    );

    let ensenado = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);
    let en_pantalla: Vec<String> = ensenado.zonas.iter().map(|(n, _, _)| n.clone()).collect();
    assert_eq!(en_pantalla, por_esfuerzo, "las zonas van del esfuerzo mas bajo al mas alto");
}

// ==================================================================================================
// Y UNA COSA QUE NO ES DEL RECORRIDO PERO DECIDE SI SIRVE
// ==================================================================================================

#[test]
fn Ensenado_se_puede_comparar_para_saber_si_algo_CAMBIO() {
    // Por que existe: la prueba del «vuelve otro dia» necesita comparar dos estados. Si `Ensenado` no
    // se pudiera comparar, habria que comparar campo a campo **y el dia que se añada un campo nuevo
    // la comparacion se olvidaria** -- y la prueba pasaria sin comprobar nada.
    //
    // Es el mismo fallo que la norma describe (AU-73): **lo que existe dos veces acaba divergiendo**.
    let a: Ensenado = cara::lo_que_ensena(&nadie(), None);
    let b: Ensenado = cara::lo_que_ensena(&nadie(), None);
    assert_eq!(a, b, "dos veces lo mismo tiene que dar lo mismo");

    let c = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);
    assert_ne!(a, c, "y dos cosas distintas no pueden salir iguales");
}

// ==================================================================================================
// EL PERIODO DEL ANALISIS, EN LA PANTALLA (2026-10-14)
// ==================================================================================================
//
// Punto 2 del bloque: antes el periodo SOLO se veia en `--texto`. Ahora lo recibe la ventana, ya escrito
// en una sola cadena. La ventana no calcula: recibe.

#[test]
fn el_periodo_del_analisis_va_a_la_pantalla_en_UNA_linea() {
    let r = sus_sesiones();
    let e = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), Some(&r));
    assert!(!e.periodo.is_empty(), "con fechas, el periodo tiene que estar");
    assert_eq!(e.periodo.lines().count(), 1, "y en UNA linea: {}", e.periodo);

    // Las fechas salen del MISMO formateador que el resto del proyecto, no de una copia.
    let desde = motor::avisos::fecha_de_epoch(r.desde.unwrap());
    let hasta = motor::avisos::fecha_de_epoch(r.hasta.unwrap());
    assert!(
        e.periodo.contains(&desde) && e.periodo.contains(&hasta),
        "tiene que nombrar las dos: {} (esperaba {desde} y {hasta})",
        e.periodo
    );
}

#[test]
fn sin_fechas_el_periodo_dice_que_no_consta_y_NO_se_inventa() {
    // Una sesion sin fecha no puede meter una fecha falsa. Es la regla de todo el proyecto.
    let mut r = sus_sesiones();
    r.desde = None;
    r.hasta = None;
    let e = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), Some(&r));
    assert!(
        e.periodo.contains("sin fechas"),
        "sin fechas se dice, no se rellena: «{}»",
        e.periodo
    );
}

#[test]
fn sin_frecuencia_maxima_medida_la_cifra_NO_EXISTE_y_no_es_un_cero_ni_un_guion() {
    // LO CAZO LA CAPTURA DE PANTALLA (2026-10-14): con el umbral de lactato pero SIN maxima medida,
    // la tarjeta de «frecuencia maxima» ensenaba «0». No se midio 0: no se midio NADA. Es la regla
    // que sostiene todo el proyecto («Falta» no es «0»), y las otras dos cifras ya la cumplian.
    //
    // Y ESTA PRUEBA CAMBIO EL 2026-10-14, con el contrato `docs/CONTRATO-cara-datos.md` (hueco 2 de
    // §E13): antes exigia la cadena «—»; ahora exige `None`, que es lo que dice «esta cifra no
    // existe» sin que nadie tenga que interpretar un simbolo. El guion esta PROHIBIDO (AU-91).
    let p = el_cliente_declara_lo_suyo();
    assert!(p.pulso_maximo_medido.falta(), "el caso de la prueba no trae maxima medida");
    let e = cara::lo_que_ensena(&p, None);
    assert!(e.puede_diagnosticar, "con umbral ya se puede diagnosticar");

    let la_maxima = &e.cifras_atadas[1];
    assert_eq!(
        la_maxima.dato, None,
        "sin maxima medida la cifra NO EXISTE; salio {:?}",
        la_maxima.dato
    );
    // Y el guion NO puede volver por ninguna puerta: ni en la cifra nueva ni en el campo viejo.
    assert_ne!(
        la_maxima.dato.as_deref(),
        Some("—"),
        "el guion de 26 px esta prohibido y el motor deja de mandarlo (anexo A, cambio 7)"
    );
    assert_eq!(e.cifras[1], "—", "el campo viejo aun lo lleva: la UI de hoy lo consume (Fase 2 lo borra)");
}

// ==================================================================================================
// LOS SEIS CONTRATOS DE `docs/CONTRATO-cara-datos.md` (2026-10-14)
// ==================================================================================================
//
// Escritas ANTES de implementar, como manda `arq:150-163`, y ejecutadas una vez para verlas FALLAR
// (el resultado esta en `datos/trabajo-ui/contrato-cara-informe.md`). Una prueba que nunca se vio
// roja no se sabe si comprueba algo (AU-91 y la regla de B4).

/// **Contrato 1.** Los tramos van en ppm, con su margen, y el que manda es el umbral de la persona.
#[test]
fn las_zonas_van_en_ppm_CON_su_margen_y_el_umbral_manda() {
    let e = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);

    assert_eq!(e.zonas_ppm.len(), 5, "son cinco zonas, y en numeros");

    // El umbral es 169 y su margen 3, asi que los cortes son los del contrato, calculados a mano:
    // 0,81 · 169 = 136,89 (recuperacion y suelo de resistencia), 0,90 · 169 = 152,10,
    // 0,94 · 169 = 158,86, 1,00 · 169 = 169, y 1,20 · 169 = 202,80.
    let esperados = [
        ("recuperación", 136.89, 136.89),
        ("resistencia", 136.89, 152.10),
        ("tempo", 152.10, 158.86),
        ("umbral", 158.86, 169.00),
        ("consumo máximo", 169.00, 202.80),
    ];
    for (z, (nombre, inicio, fin)) in e.zonas_ppm.iter().zip(esperados) {
        assert_eq!(z.nombre, nombre, "el nombre de la zona");
        assert!(
            (z.inicio_ppm - inicio).abs() < 0.01,
            "{nombre}: inicio en ppm; salio {} y se esperaba {inicio}",
            z.inicio_ppm
        );
        assert!(
            (z.fin_ppm - fin).abs() < 0.01,
            "{nombre}: fin en ppm; salio {} y se esperaba {fin}",
            z.fin_ppm
        );
        // Y EL MARGEN VA CON CADA NUMERO, que es la promesa del motor: 3 · 0,81 en la primera.
        assert!(
            (z.margen_inicio_ppm - 3.0 * (inicio / 169.0)).abs() < 0.01,
            "{nombre}: el margen del inicio viaja con el numero; salio {}",
            z.margen_inicio_ppm
        );
        assert!(
            (z.margen_fin_ppm - 3.0 * (fin / 169.0)).abs() < 0.01,
            "{nombre}: el margen del fin viaja con el numero; salio {}",
            z.margen_fin_ppm
        );
        assert!(z.margen_inicio_ppm >= 0.0, "{nombre}: un margen no puede ser negativo");
    }

    // NINGUN TRAMO TERMINA ANTES DE EMPEZAR, y estan en orden de esfuerzo. Y las zonas son
    // CONTIGUAS: cada una empieza donde acaba la anterior, que es lo que las hace un mapa del
    // esfuerzo y no cinco numeros sueltos.
    //
    // OJO, Y ESTO SE APRENDIO ESCRIBIENDO ESTA PRUEBA (2026-10-14): los INICIOS **no** son distintos.
    // «recuperación» y «resistencia» comparten suelo (0,81 · umbral), y por eso el informe viejo
    // necesitaba la palabra «hasta». Lo que no puede repetirse es el PAR (inicio, fin): un tramo
    // distinto para cada zona.
    for par in e.zonas_ppm.windows(2) {
        assert!(
            (par[0].fin_ppm - par[1].inicio_ppm).abs() < 0.01,
            "{} acaba en {} y {} empieza en {}: las zonas son contiguas",
            par[0].nombre,
            par[0].fin_ppm,
            par[1].nombre,
            par[1].inicio_ppm
        );
        assert!(
            (par[0].fin_ppm - par[1].fin_ppm).abs() > 0.01,
            "{} y {} no pueden acabar en el mismo sitio: {:?}",
            par[0].nombre,
            par[1].nombre,
            e.zonas_ppm
        );
    }
    // Ninguna zona puede tener los dos numeros iguales... salvo la recuperacion, que es un TECHO.
    for (i, z) in e.zonas_ppm.iter().enumerate() {
        if i == 0 {
            assert!(
                (z.inicio_ppm - z.fin_ppm).abs() < 0.01,
                "la recuperacion empieza y acaba en el suelo: es un techo, no un tramo"
            );
        } else {
            assert!(
                z.fin_ppm > z.inicio_ppm,
                "{}: un tramo tiene que tener ancho ({} a {})",
                z.nombre,
                z.inicio_ppm,
                z.fin_ppm
            );
        }
    }

    // Y SI EL UMBRAL CAMBIA, LOS NUMEROS CAMBIAN CON EL. Sin esto, la prueba no probaria que los
    // numeros salen de SUS datos.
    let mut mas_bajo = el_cliente_declara_lo_suyo();
    mas_bajo.umbral_lactato = Dato::Hay(160);
    let con_160 = cara::lo_que_ensena(&mas_bajo, None);
    assert!(
        (con_160.zonas_ppm[4].fin_ppm - 192.0).abs() < 0.01,
        "con umbral 160 el techo de la ultima zona es 192; salio {}",
        con_160.zonas_ppm[4].fin_ppm
    );
}

/// **Contrato 1**, la esquina que ya fallo una vez: dos zonas seguidas con el MISMO numero.
#[test]
fn dos_zonas_seguidas_no_pueden_ser_el_mismo_tramo() {
    // El defecto de la primera version: se devolvia solo el SUELO de cada zona, asi que «recuperacion»
    // y «resistencia» salian con el mismo 137, y un informe que dice dos veces 137 no se entiende.
    //
    // Y ESTA PRUEBA CORRIGIO AL CONTRATO, no al reves: en ppm los dos suelos SIGUEN siendo el mismo
    // numero (0,81 · 169), porque es la verdad -- una zona empieza donde acaba la otra. Lo que no
    // puede repetirse es el TRAMO: cada zona tiene que ser un par (inicio, fin) distinto.
    let e = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);
    let tramos: Vec<(f64, f64)> = e.zonas_ppm.iter().map(|z| (z.inicio_ppm, z.fin_ppm)).collect();
    for (i, a) in tramos.iter().enumerate() {
        for b in tramos.iter().skip(i + 1) {
            assert!(
                (a.0 - b.0).abs() > 0.01 || (a.1 - b.1).abs() > 0.01,
                "dos zonas no pueden enseñar el mismo tramo: {tramos:?}"
            );
        }
    }
    // Y las dos que comparten suelo NO comparten techo, que es lo que las distingue en pantalla.
    assert!(
        (e.zonas_ppm[0].inicio_ppm - e.zonas_ppm[1].inicio_ppm).abs() < 0.01,
        "recuperacion y resistencia comparten suelo: es la verdad, y por eso el texto dice «hasta»"
    );
    assert!(
        (e.zonas_ppm[0].fin_ppm - e.zonas_ppm[1].fin_ppm).abs() > 1.0,
        "pero no comparten techo: {} frente a {}",
        e.zonas_ppm[0].fin_ppm,
        e.zonas_ppm[1].fin_ppm
    );
    // La recuperacion es la unica que empieza y acaba en el mismo sitio: es su significado.
    assert!(
        (e.zonas_ppm[0].inicio_ppm - e.zonas_ppm[0].fin_ppm).abs() < 0.01,
        "la recuperacion es un techo, no un tramo"
    );
}

/// **Contrato 2.** «Esta cifra no existe» es un valor explicito, y NO se confunde con «vacio».
#[test]
fn una_cifra_que_no_existe_es_None_y_NO_una_cadena_vacia() {
    // AU-88, y con el caso real: el cliente NO trae frecuencia maxima medida. La tarjeta 2 no se
    // dibuja, y lo que lo decide es `None`, no una cadena vacia (que la UI dibujaria en blanco) ni un
    // cero (que es un dato FALSO) ni el guion (prohibido, AU-91).
    let p = el_cliente_declara_lo_suyo();
    assert!(p.pulso_maximo_medido.falta(), "el caso no trae maxima medida");
    let e = cara::lo_que_ensena(&p, None);

    assert_eq!(e.cifras_atadas.len(), 4, "son cuatro cifras, con etiqueta o sin dato");
    let sin_dato: Vec<&str> = e
        .cifras_atadas
        .iter()
        .filter(|c| c.dato.is_none())
        .map(|c| c.etiqueta.as_str())
        .collect();
    // SIN DATOS DEL APARATO NO EXISTEN TRES: la maxima medida (no se la ha medido), la eficiencia y
    // la deriva (no hay sesiones de las que salgan). Y la que SI existe -- su umbral -- existe con su
    // margen, que es la promesa del motor.
    assert_eq!(
        sin_dato,
        vec!["frecuencia máxima", "vatios por pulso", "deriva %"],
        "las tres que no se han medido NO EXISTEN; y el umbral, que si existe, existe"
    );
    assert!(
        e.cifras_atadas[0].dato.is_some(),
        "su umbral declarado SI existe, aunque no haya aparato: {:?}",
        e.cifras_atadas[0]
    );

    // Y SIN SESIONES, la eficiencia y la deriva tampoco existen: son tres las que no existen.
    assert_eq!(
        e.cifras_atadas.iter().filter(|c| c.dato.is_none()).count(),
        3,
        "sin aparato no hay maxima medida, ni eficiencia, ni deriva: tres cifras que NO EXISTEN \
         (con el aparato, solo una). Cifras: {:?}",
        e.cifras_atadas
    );
    let con_aparato = cara::lo_que_ensena(&p, Some(&sus_sesiones()));
    assert_eq!(
        con_aparato.cifras_atadas.iter().filter(|c| c.dato.is_none()).count(),
        1,
        "con sus sesiones solo falta la maxima medida: {:?}",
        con_aparato.cifras_atadas
    );
    assert_eq!(
        con_aparato.cifras_atadas[1].dato, None,
        "y la que falta sigue siendo la maxima medida"
    );
}

/// **Contrato 2**, dicho desde el otro lado: «vacio» y «no medido» son DOS estados, no uno.
#[test]
fn vacio_y_no_medido_NO_se_confunden() {
    // La diferencia es la que hace que la pantalla no mienta:
    //   `None`             -> esta cifra no existe  -> la tarjeta se saca del arbol
    //   `Some("")`         -> la cifra existe y su texto es vacio -> la tarjeta existe
    let no_existe = cara::Cifra { etiqueta: "deriva".to_string(), dato: None };
    let vacia = cara::Cifra { etiqueta: "deriva".to_string(), dato: Some(String::new()) };

    assert_ne!(
        no_existe, vacia,
        "si estos dos fueran iguales, la UI no podria distinguir «no lo se» de «es vacio» (AU-88)"
    );
    assert!(!no_existe.existe(), "sin dato, la cifra no existe");
    assert!(vacia.existe(), "con dato vacio, la cifra existe igual");

    // Y en el caso real: con umbral, las cuatro etiquetas estan SIEMPRE, aunque falte el dato.
    let e = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);
    let etiquetas: Vec<&str> = e.cifras_atadas.iter().map(|c| c.etiqueta.as_str()).collect();
    assert_eq!(etiquetas.len(), 4, "una cifra que no existe NO se cae de la lista: se queda sin dato");
}

/// **Contrato 3.** El eje: del suelo de la primera zona al techo de la ultima.
#[test]
fn el_eje_va_del_suelo_de_la_primera_zona_al_techo_de_la_ultima() {
    let e = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);

    assert!(e.eje.es_valido(), "con zonas, el eje tiene que valer");
    assert!(
        (e.eje.inicio_ppm - 136.89).abs() < 0.01,
        "el eje empieza donde empieza la primera zona; salio {}",
        e.eje.inicio_ppm
    );
    assert!(
        (e.eje.fin_ppm - 202.80).abs() < 0.01,
        "y acaba donde acaba la ultima; salio {}",
        e.eje.fin_ppm
    );

    // EL EJE ES EL DE VERDAD: no puede quedarse corto ni pasarse, o las bandas se saldrian.
    let minima = e.zonas_ppm.iter().map(|z| z.inicio_ppm).fold(f64::MAX, f64::min);
    let maxima = e.zonas_ppm.iter().map(|z| z.fin_ppm).fold(f64::MIN, f64::max);
    assert!((e.eje.inicio_ppm - minima).abs() < 0.01, "la escala es una afirmacion: sale de los datos");
    assert!((e.eje.fin_ppm - maxima).abs() < 0.01, "y su techo tambien");
    assert!(e.eje.fin_ppm > e.eje.inicio_ppm, "un eje de ancho cero no es un eje");
}

/// **Contrato 3**, la esquina: sin zonas no hay eje que valga, y no se inventa uno.
#[test]
fn sin_datos_no_hay_eje_que_valga() {
    let e = cara::lo_que_ensena(&nadie(), None);
    assert!(!e.eje.es_valido(), "sin zonas no se puede pintar un eje: seria inventarse la escala");
    assert_eq!(e.zonas_ppm.len(), 0, "ni zonas en numeros");
}

/// **Contrato 4.** La precision: UNA frase en pantalla, y el parrafo a la ayuda.
#[test]
fn la_precision_en_pantalla_es_UNA_frase_y_el_parrafo_va_a_la_ayuda() {
    let el_cliente = el_cliente_declara_lo_suyo();
    let con_umbral = cara::lo_que_ensena(&el_cliente, None);

    assert!(
        !con_umbral.precision_corta.is_empty(),
        "la pantalla tiene que decir de donde sale el dato"
    );
    assert_eq!(
        con_umbral.precision_corta.lines().count(),
        1,
        "UNA linea, no un parrafo: «{}»",
        con_umbral.precision_corta
    );
    // Y CORTA de verdad: si no, no cabe en 411 px (AU-44/AU-45: la pantalla no se explica).
    assert!(
        con_umbral.precision_corta.chars().count() <= 60,
        "la frase de pantalla mide {} caracteres; el tope del contrato son 60: «{}»",
        con_umbral.precision_corta.chars().count(),
        con_umbral.precision_corta
    );
    // Un punto, y uno solo: una frase.
    assert_eq!(
        con_umbral.precision_corta.matches(". ").count(),
        0,
        "una frase, no dos: «{}»",
        con_umbral.precision_corta
    );

    // EL PARRAFO NO SE PIERDE: va a la ayuda, que es donde AU-44/AU-45 lo mandan. Y es MAS LARGO.
    assert!(
        con_umbral.precision.len() > con_umbral.precision_corta.len(),
        "el parrafo de la ayuda tiene que seguir estando y decir mas"
    );
    assert!(
        con_umbral.precision.to_lowercase().contains("medido")
            || con_umbral.precision.to_lowercase().contains("estimad"),
        "y seguir diciendo de donde sale: «{}»",
        con_umbral.precision
    );

    // Y LAS DOS DICEN LO MISMO, no una cosa cada una: la frase es el resumen del parrafo.
    let mut solo_edad = Persona::desconocida();
    solo_edad.edad = Dato::Hay(60);
    let sin_umbral = cara::lo_que_ensena(&solo_edad, None);
    assert_ne!(
        con_umbral.precision_corta, sin_umbral.precision_corta,
        "medido no es lo mismo que estimado, y la frase de pantalla tiene que distinguirlo"
    );
    assert_ne!(
        con_umbral.precision, sin_umbral.precision,
        "ni el parrafo de la ayuda"
    );
}

/// **Contrato 5.** Lo que falta: UNA frase en pantalla, y sigue diciendo donde se rellena.
#[test]
fn lo_que_falta_en_pantalla_es_UNA_frase_y_sigue_diciendo_donde() {
    let e = cara::lo_que_ensena(&nadie(), None);

    assert!(
        !e.que_falta_corto.is_empty(),
        "si no se puede diagnosticar, hay que decir que falta"
    );
    assert_eq!(
        e.que_falta_corto.lines().count(),
        1,
        "UNA linea: era el defecto 2 de la auditoria, la linea mas larga de la pantalla: «{}»",
        e.que_falta_corto
    );
    assert!(
        e.que_falta_corto.chars().count() <= 70,
        "la frase mide {} caracteres; el tope del contrato son 70: «{}»",
        e.que_falta_corto.chars().count(),
        e.que_falta_corto
    );
    // Y DICE DONDE SE RELLENA, que es lo unico que la hace util (AU-44/AU-45).
    let texto = e.que_falta_corto.to_lowercase();
    assert!(
        texto.contains("rellena"),
        "tiene que decir DONDE se rellena, no solo que falta: «{}»",
        e.que_falta_corto
    );
    // Y EL CAMPO VIEJO SIGUE: la UI de hoy lo consume.
    assert!(!e.que_falta.is_empty(), "el campo viejo se conserva en este hito");

    // CON SUS DATOS NO FALTA NADA, y no puede quedar una frase suelta en la pantalla.
    let con_datos = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None);
    assert!(
        con_datos.que_falta_corto.is_empty() && con_datos.que_falta.is_empty(),
        "si se puede diagnosticar, no falta nada: corto «{}», largo «{}»",
        con_datos.que_falta_corto,
        con_datos.que_falta
    );
}

/// **Contrato 6.** La etiqueta va DENTRO de su cifra, y no en una lista paralela.
#[test]
fn cada_cifra_lleva_su_etiqueta_dentro_y_no_en_una_lista_paralela() {
    // AU-73: con dos listas, quitar una cifra desalinea las etiquetas, y eso no se ve en una captura.
    let e = cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), Some(&sus_sesiones()));

    assert_eq!(e.cifras_atadas.len(), 4, "cuatro pares");
    let etiquetas: Vec<&str> = e.cifras_atadas.iter().map(|c| c.etiqueta.as_str()).collect();
    assert_eq!(
        etiquetas,
        vec!["umbral (pulsaciones)", "frecuencia máxima", "vatios por pulso", "deriva %"],
        "y en su orden, que lo fija el motor"
    );

    // Y EL PAR VA JUNTO: la etiqueta 1 describe el dato 1. Con dos listas esto se comprobaba a mano.
    assert!(
        e.cifras_atadas[0].dato.as_deref().is_some_and(|d| d.contains("169")),
        "el primer par es el umbral, y el suyo es 169: {:?}",
        e.cifras_atadas[0]
    );
    assert!(
        e.cifras_atadas[3].dato.as_deref().is_some_and(|d| d.starts_with('+')),
        "el cuarto par es la deriva, y la suya es positiva: {:?}",
        e.cifras_atadas[3]
    );

    // Y NO HAY DOS LISTAS: el campo viejo existe solo para la UI de hoy, y estas dos tienen que
    // contar lo mismo o la Fase 2 migraria con datos distintos.
    assert_eq!(
        e.cifras.len(),
        e.cifras_atadas.len(),
        "la lista vieja y la nueva no pueden discrepar en tamaño"
    );
    assert_eq!(
        e.etiquetas.len(),
        e.cifras_atadas.len(),
        "ni las etiquetas viejas"
    );
    for (vieja, nueva) in e.etiquetas.iter().zip(e.cifras_atadas.iter()) {
        assert_eq!(vieja, &nueva.etiqueta, "la etiqueta vieja y la nueva tienen que ser la MISMA");
    }
}

#[test]
fn sin_sesiones_no_hay_linea_de_periodo_ni_hueco() {
    // Sin datos no se anuncia nada: el recuento ya habla por si solo (AU-07).
    for e in [
        cara::lo_que_ensena(&el_cliente_declara_lo_suyo(), None),
        cara::lo_que_ensena(&nadie(), None),
    ] {
        assert!(e.periodo.is_empty(), "sin sesiones no hay periodo: «{}»", e.periodo);
    }
}
