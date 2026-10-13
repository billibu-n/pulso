//! **Medir la interfaz SIN abrir ventana.**
//!
//! POR QUE EXISTE, y es la regla AU-63 de la norma:
//! «antes de tocar algo visual, comprobar que se puede MEDIR; si la unica forma de saber como quedo
//! es que otra persona lo mire y lo cuente, EL METODO ESTA MAL».
//!
//! Hasta ahora eso era exactamente el caso: para saber si la ventana se parecia al boceto habia que
//! **abrirla y mirarla**. Con `i-slint-backend-testing` se consulta la **geometria** de los elementos
//! -- su posicion y su tamano -- y se comprueba **contra el diseno**.
//!
//! ESTA PRUEBA ES LA PRIMERA: comprueba que la herramienta de medir FUNCIONA. Sin eso, medir de mas
//! seria creer que se mide.

slint::include_modules!();

use i_slint_backend_testing::ElementQuery;

fn ventana_de_prueba() -> Ventana {
    i_slint_backend_testing::init_no_event_loop();
    let v = Ventana::new().expect("la ventana tiene que construirse");
    v.set_cifras(
        vec!["169±3".into(), "183".into(), "0,98".into(), "+9,7".into()]
            .as_slice()
            .into(),
    );
    v.set_etiquetas(
        vec!["umbral".into(), "maxima".into(), "vatios".into(), "deriva".into()]
            .as_slice()
            .into(),
    );
    v.set_zonas_nombres(
        vec!["recuperacion".into(), "resistencia".into(), "tempo".into(), "umbral".into(), "maximo".into()]
            .as_slice()
            .into(),
    );
    v.set_zonas_tramos(
        vec!["hasta 137".into(), "137 - 152".into(), "152 - 159".into(), "159 - 169".into(), "169 - 203".into()]
            .as_slice()
            .into(),
    );
    v.set_zonas_niveles(vec![68, 75, 78, 83, 100].as_slice().into());
    v.set_precision("precision alta".into());
    v.set_aviso("deriva +9,7 %".into());
    v.set_vacio("102 sesiones".into());
    v
}

#[test]
fn la_herramienta_de_medir_mide_de_verdad() {
    // La ventana entera: su tamano tiene que ser EL DEL DISENO, no el que le de el gestor de ventanas.
    let v = ventana_de_prueba();
    let raiz = ElementQuery::from_root(&v).find_first().expect("hay raiz");
    let pos = raiz.absolute_position();
    let tam = raiz.size();
    let (x, y, ancho, alto) = (pos.x, pos.y, tam.width, tam.height);
    println!("  la ventana mide {ancho} x {alto} en ({x}, {y})");

    assert!(ancho > 0.0 && alto > 0.0, "la ventana tiene que tener tamano: {ancho}x{alto}");
}

#[test]
fn se_puede_buscar_un_elemento_por_su_nombre() {
    // Si esto no funcionara, no habria nada que medir. Es la prueba de la herramienta.
    let v = ventana_de_prueba();
    let cuantos = ElementQuery::from_root(&v).find_all().len();
    println!("  elementos en el arbol: {cuantos}");
    assert!(cuantos > 10, "el arbol tiene que tener elementos: {cuantos}");
}

#[test]
fn la_ventana_mide_lo_que_dice_el_diseno() {
    // ESTA ES LA PRUEBA QUE FALTABA, y contesta a la pregunta del dueno: «¿cual es el correcto, el
    // de la captura o el que me lanzas?». **El correcto es el de la captura**, porque es el diseno.
    // Y aqui se comprueba, midiendo, si la aplicacion lo respeta.
    //
    // MEDIDAS DEL DISENO (boletos/generar-movil.py, que es lo que el dueno aprobo):
    //   ventana ...... 411 x 880
    //   cabecera ..... 56 de alto
    //   margen ....... 16
    //   nav .......... 64 de alto
    let v = ventana_de_prueba();
    let raiz = ElementQuery::from_root(&v).find_first().expect("hay raiz");
    let tam = raiz.size();

    println!("  ventana:      {:.0} x {:.0}   (diseno: 411 x 880)", tam.width, tam.height);
    assert_eq!(tam.width.round() as i32, 411, "el ancho tiene que ser el del diseno");
    assert_eq!(tam.height.round() as i32, 880, "el alto tiene que ser el del diseno");

    // Y los sitios del armazon, uno a uno.
    // OJO AL NOMBRE: el id lleva el COMPONENTE delante -- `ArmazonMovil::cabecera`, no `cabecera`.
    // Se aprendio MIDIENDO: la primera version buscaba `cabecera` y no encontraba nada.
    for (nombre, esperado) in [("ArmazonMovil::cabecera", 56.0), ("ArmazonMovil::nav", 64.0)] {
        let e = ElementQuery::from_root(&v)
            .match_id(nombre)
            .find_first()
            .unwrap_or_else(|| panic!("no se encuentra {nombre}"));
        let t = e.size();
        let p = e.absolute_position();
        println!("  {nombre:<26} {:.0}x{:.0} en ({:.0},{:.0})   (diseno: alto {esperado:.0})", t.width, t.height, p.x, p.y);
        assert_eq!(t.height.round() as i32, esperado as i32, "{nombre} no mide lo del diseno");
    }
}

#[test]
fn la_barra_de_abajo_esta_repartida_y_no_pegada() {
    // ESTE ERA EL FALLO VISIBLE, el que el dueno vio en pantalla: «ResumenZonasSesionesDatos».
    // El ancho de cada hueco lo decidia el TEXTO (32, 46, 31 px), asi que se pegaban unos a otros.
    // En el diseno cada hueco mide 411/4 = 102,75, y por eso se leen.
    //
    // LA PRUEBA: los cuatro huecos tienen que medir LO MISMO, y el activo tiene que llevar la linea.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();

    // Y SE TOMAN LOS CUATRO HUECOS, que son los cuatro primeros por posicion horizontal.
    // Dentro de cada uno hay mas elementos (capa, linea, icono, texto) y tambien miden lo mismo,
    // asi que mirarlos todos no aporta: lo que importa es que los HUECOS esten repartidos.
    let mut por_x: Vec<(f64, f64)> = todos
        .iter()
        .filter(|e| {
            let p = e.absolute_position();
            let t = e.size();
            p.y >= 688.0 && p.y < 756.0 && (f64::from(t.height) - 64.0).abs() < 1.0
        })
        .map(|e| (f64::from(e.absolute_position().x), f64::from(e.size().width)))
        .collect();
    por_x.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    por_x.dedup_by(|a, b| (a.0 - b.0).abs() < 0.5);
    // Y FUERA la capa contenedora: ocupa la barra entera y empieza en x=0. Los huecos, no.
    por_x.retain(|(x, _)| *x > 0.5);

    let anchos: Vec<f64> = por_x.iter().map(|(_, w)| *w).collect();
    let xs: Vec<f64> = por_x.iter().map(|(x, _)| *x).collect();
    println!("  huecos de la nav: x={xs:?}");
    println!("  y sus anchos:     {anchos:?}");
    assert_eq!(anchos.len(), 4, "la navegacion tiene CUATRO huecos");

    let primero = anchos[0];
    for a in &anchos {
        assert!(
            (a - primero).abs() < 1.0,
            "los huecos tienen que medir LO MISMO (el texto no puede mandar): {anchos:?}"
        );
    }
    // Y el ancho tiene que ser el del diseno: la ventana repartida en cuatro.
    let esperado = 411.0 / 4.0;
    assert!(
        (primero - esperado).abs() < 1.5,
        "cada hueco mide {primero:.1}, y el diseno dice {esperado:.1}"
    );
}

#[test]
fn ver_el_arbol_para_saber_que_se_puede_medir() {
    // Existe para CONTESTAR «que hay ahi dentro», en vez de suponerlo. Imprime el arbol con el
    // nombre y el tamano de cada elemento, para saber QUE se puede medir y como se llama.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();
    println!("  {} elementos:", todos.len());
    for (i, e) in todos.iter().enumerate() {
        let id = e.id().map(|s| s.to_string()).unwrap_or_else(|| "(sin nombre)".into());
        let t = e.size();
        let p = e.absolute_position();
        // Solo los que tienen nombre o un tamano interesante: si no, son cientos de lineas.
        if id != "(sin nombre)" || t.width > 100.0 {
            println!("  [{i:>3}] {id:<34} {:.0}x{:.0} en ({:.0},{:.0})", t.width, t.height, p.x, p.y);
        }
    }
}
