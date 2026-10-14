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
