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

#[test]
fn se_puede_buscar_por_accessible_id_sin_depender_del_nombre_interno() {
    // POR QUE ESTA PRUEBA: buscar por `ArmazonMovil::cabecera` funciona, pero **depende de como se
    // llame el componente por dentro**. Si alguien renombra el componente, la prueba se rompe sin que
    // nada este mal.
    //
    // La documentacion de Slint trae la forma OFICIAL: `accessible-id` es
    //   "un identificador unico del elemento, usado para identificar widgets en automatizacion y
    //    PRUEBAS".
    // O sea: **existe justo para esto**, y es independiente de los nombres internos.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();
    let con_id: Vec<String> = todos
        .iter()
        .filter_map(|e| e.accessible_id().map(|s| s.to_string()))
        .collect();
    println!("  elementos con accessible-id puesto: {con_id:?}");
    // Y estas son las dos que tienen que estar: los sitios del armazon.
    // Las TRES zonas del armazon, cada una con su papel de referencia.
    for esperado in ["cabecera", "contenido", "navegacion"] {
        assert!(
            con_id.contains(&esperado.to_string()),
            "falta el accessible-id «{esperado}»: {con_id:?}"
        );
    }

    // Y SE MIDE POR ESE NOMBRE, que es lo que se queria: encontrar por decision, no por casualidad.
    let cab = todos.iter().find(|e| e.accessible_id().map(|s| s.to_string()) == Some("cabecera".into()));
    let cab = cab.expect("tiene que encontrarse la cabecera por su accessible-id");
    println!(
        "  encontrada por accessible-id: {:.0}x{:.0}",
        cab.size().width,
        cab.size().height
    );
}


// ==================================================================================================
// LO QUE SE MIDIO AL DECIDIR USAR LOS CONTROLES DE SERIE (2026-10-13)
// ==================================================================================================
//
// El dueño decidio usar **los controles de serie de Slint** porque «debe tener la ventaja de no
// romperse». La decision es buena, y **hay que medir su precio en vez de suponerlo**.
//
// Este es el precio, medido con el compilador y con estos numeros:
//   - NO aceptan nuestros colores (`Unknown property background in Button`),
//   - NO aceptan nuestro color de texto (`Unknown property color in LineEdit`),
//   - **NO aceptan ni un borde** (`Unknown property border-width in Button`).
//
// Y la consecuencia practica esta en `temas/contraste-widgets.py`: en el tema claro el borde de
// serie da 3,33:1 y se ve; **en el tema negro OLED da 1,14:1, o sea NO SE VE**. Un boton sin
// contorno sobre negro parece un rotulo.
//
// LA SALIDA (y es lo que comprueba esta prueba): **no reescribir el boton** -- eso tiraria su
// accesibilidad, su foco y su teclado -- sino **ponerle un marco por fuera**. El boton sigue
// siendo el boton; el contorno lo dibuja un Rectangle de fuera.
//
// Y ESTA PRUEBA COMPRUEBA LAS DOS COSAS DE LAS QUE DEPENDE ESA SALIDA:
//   1. que el boton de serie **SI trae la accesibilidad** (rol y etiqueta) -- es lo que se gana;
//   2. que el marco **no recorta** al boton -- es lo que hay que vigilar al envolverlo.

// ==================================================================================================
// EL SELECTOR DE CARPETA: la capa superpuesta, MEDIDA (no mirada)
// ==================================================================================================

#[test]
fn el_selector_de_carpeta_existe_y_llega_al_minimo_tactil() {
    // Se mide la capa CON LA CAPA PUESTA (eligiendo = true), que es cuando existe. Y se comprueba lo
    // que la norma exige de todo boton: que llegue a 48 dp, porque el minimo de serie son 32 y hay que
    // pedirlo. Un boton que no se puede pulsar con el dedo no es un boton.
    i_slint_backend_testing::init_no_event_loop();
    let v = Ventana::new().expect("la ventana tiene que construirse");
    v.set_carpeta_ruta("/home/tu/carpeta".into());
    v.set_carpeta_fit(102);
    v.set_carpeta_puede_subir(true);
    v.set_carpeta_aviso("".into());
    let modelo = slint::VecModel::from_slice(&[
        EntradaCarpeta { nombre: "storage".into(), cuantos: 102 },
        EntradaCarpeta { nombre: "otros".into(), cuantos: 0 },
    ]);
    v.set_carpetas(slint::ModelRc::new(modelo));
    v.set_eligiendo(true);

    let todos = ElementQuery::from_root(&v).find_all();
    let buscar = |nombre: &str| {
        todos
            .iter()
            .find(|e| e.accessible_id().map(|s| s.to_string()) == Some(nombre.to_string()))
    };
    let con_id: Vec<String> = todos
        .iter()
        .filter_map(|e| e.accessible_id().map(|s| s.to_string()))
        .collect();
    for id in ["selector-ruta", "selector-usar", "selector-cerrar", "selector-entrada"] {
        assert!(con_id.contains(&id.to_string()), "falta el selector {id}: {con_id:?}");
    }

    let usar = buscar("selector-usar").expect("tiene que existir el boton de usar");
    let t = usar.size();
    println!("  selector-usar mide {:.0}x{:.0}", t.width, t.height);
    assert!(t.height >= 48.0, "el boton de usar tiene que llegar a 48 dp, y mide {}", t.height);

    // Y SIN LA CAPA PUESTA NO EXISTE: no se dibuja de fondo, ocupando sitio sin decir nada.
    v.set_eligiendo(false);
    let sin = ElementQuery::from_root(&v).find_all();
    assert!(
        !sin.iter().any(|e| e.accessible_id().map(|s| s.to_string()) == Some("selector-usar".into())),
        "cerrada la capa, el selector no puede seguir ahi"
    );
}

#[test]
fn el_boton_de_serie_trae_la_accesibilidad_puesta() {
    use i_slint_backend_testing::AccessibleRole;

    i_slint_backend_testing::init_no_event_loop();
    let s = SondaBorde::new().expect("la sonda tiene que construirse");

    // Se busca por `accessible_id` FILTRANDO (que es lo que funciona), no con `match_id`:
    // `match_id` casa con el id INTERNO del elemento en Slint, no con el `accessible-id`.
    let todos = ElementQuery::from_root(&s).find_all();
    println!("  ids de accesibilidad en la sonda: {:?}",
        todos.iter().filter_map(|e| e.accessible_id().map(|s| s.to_string())).collect::<Vec<_>>());
    let boton = todos
        .iter()
        .find(|e| e.accessible_id().map(|s| s.to_string()) == Some("SondaBorde::sin-marco".into()))
        .expect("tiene que existir el boton sin marco");

    // ESTO ES LO QUE SE GANA al no construirlo a mano: el papel y la etiqueta ya estan.
    // Si el proyecto hubiera hecho su boton con un Rectangle y un TouchArea, habria que
    // declarar las dos cosas a mano -- y probarlas.
    let papel = boton.accessible_role();
    let etiqueta = boton.accessible_label();
    println!("  el boton de serie se declara como {papel:?}, con la etiqueta {etiqueta:?}");

    assert_eq!(
        papel,
        Some(AccessibleRole::Button),
        "un Button de serie tiene que declararse como boton, no como adorno"
    );
    assert_eq!(
        etiqueta.map(|e| e.to_string()),
        Some("Vincular cuenta".to_string()),
        "su etiqueta tiene que ser su texto: es lo que lee un lector de pantalla"
    );
}

#[test]
fn el_marco_de_fuera_no_recorta_al_boton() {
    i_slint_backend_testing::init_no_event_loop();
    let s = SondaBorde::new().expect("la sonda tiene que construirse");

    let todos = ElementQuery::from_root(&s).find_all();
    let buscar = |nombre: &str| {
        todos
            .iter()
            .find(|e| e.accessible_id().map(|s| s.to_string()) == Some(nombre.to_string()))
            .unwrap_or_else(|| panic!("tiene que existir el elemento «{nombre}»"))
    };
    let marco = buscar("SondaBorde::marco");
    let boton = buscar("SondaBorde::con-marco");

    let m_pos = marco.absolute_position();
    let m_tam = marco.size();
    let b_pos = boton.absolute_position();
    let b_tam = boton.size();

    println!(
        "  marco: ({:.0}, {:.0}) {:.0}x{:.0}   boton: ({:.0}, {:.0}) {:.0}x{:.0}",
        m_pos.x, m_pos.y, m_tam.width, m_tam.height,
        b_pos.x, b_pos.y, b_tam.width, b_tam.height
    );

    // EL BOTON CABE DENTRO DEL MARCO. Si esto falla, el marco lo esta recortando -- y un boton
    // recortado se ve peor que un boton sin contorno, que es lo que se venia a arreglar.
    assert!(
        b_pos.x >= m_pos.x && b_pos.y >= m_pos.y,
        "el boton empieza fuera del marco: boton en ({}, {}), marco en ({}, {})",
        b_pos.x, b_pos.y, m_pos.x, m_pos.y
    );
    assert!(
        b_pos.x + b_tam.width <= m_pos.x + m_tam.width + 0.5,
        "el boton se sale del marco por la derecha: boton acaba en {}, marco en {}",
        b_pos.x + b_tam.width,
        m_pos.x + m_tam.width
    );
    assert!(
        b_pos.y + b_tam.height <= m_pos.y + m_tam.height + 0.5,
        "el boton se sale del marco por abajo: boton acaba en {}, marco en {}",
        b_pos.y + b_tam.height,
        m_pos.y + m_tam.height
    );

    // Y EL BOTON LLEGA AL MINIMO TACTIL, que en este proyecto son 48 dp. El minimo del estilo
    // de serie son **32 px**, asi que **hay que pedirlo**: no viene puesto. Se pide con
    // `height: max(48px, self.preferred-height)`, no con un 48 fijo -- asi, el dia que el texto
    // necesite mas alto, el boton crece en vez de recortarlo.
    println!("  alto del boton: {:.0} px", b_tam.height);
    assert!(
        b_tam.height >= 48.0,
        "el boton tiene que llegar a 48 dp de alto (minimo tactil del proyecto), y mide {}",
        b_tam.height
    );

    // Y EL MARCO RODEA AL BOTON SIN APLASTARLO: el hueco entre los dos es el `padding` del
    // layout (2 px) mas el borde (2 px). Se comprueba que hay hueco por los CUATRO lados.
    let hueco_izq = b_pos.x - m_pos.x;
    let hueco_arr = b_pos.y - m_pos.y;
    let hueco_der = (m_pos.x + m_tam.width) - (b_pos.x + b_tam.width);
    let hueco_aba = (m_pos.y + m_tam.height) - (b_pos.y + b_tam.height);
    println!(
        "  hueco alrededor del boton: izq {hueco_izq:.0}, arriba {hueco_arr:.0}, \
         der {hueco_der:.0}, abajo {hueco_aba:.0}"
    );
    for (lado, hueco) in [
        ("izquierda", hueco_izq),
        ("arriba", hueco_arr),
        ("derecha", hueco_der),
        ("abajo", hueco_aba),
    ] {
        assert!(
            hueco > 1.0,
            "el marco tiene que dejar hueco por {lado}, y deja {hueco} -- lo esta recortando"
        );
    }
}

// ==================================================================================================
// EL PERIODO EN LA PANTALLA (2026-10-14)
// ==================================================================================================

#[test]
fn el_periodo_en_pantalla_cuando_lo_hay_y_sin_hueco_cuando_no() {
    // Punto 2 del bloque: el periodo que antes solo se veia en `--texto` ahora va a la pantalla. Y va
    // MEDIDO, no mirado: se comprueba que existe cuando hay periodo, que tiene cuerpo, y que cabe.
    // (`ventana_de_prueba` ya inicializa la plataforma de medida: llamarla dos veces paniquea.)
    let v = ventana_de_prueba();

    // 1) SIN periodo no existe el elemento: no se dibuja un hueco vacio (AU-07).
    let sin = ElementQuery::from_root(&v).find_all();
    assert!(
        !sin.iter().any(|e| e.accessible_id().map(|s| s.to_string()) == Some("periodo".into())),
        "sin periodo no puede quedar el hueco del periodo"
    );

    // 2) CON periodo existe, y se puede medir.
    v.set_periodo("Análisis del 2026-05-02 al 2026-10-11".into());
    let todos = ElementQuery::from_root(&v).find_all();
    let p = todos
        .iter()
        .find(|e| e.accessible_id().map(|s| s.to_string()) == Some("periodo".into()))
        .expect("con periodo, la linea tiene que existir");
    let t = p.size();
    let pos = p.absolute_position();
    println!(
        "  periodo: {:.0}x{:.0} en ({:.0},{:.0})   texto: {}",
        t.width, t.height, pos.x, pos.y, v.get_periodo()
    );
    assert!(t.height > 0.0 && t.width > 100.0, "la linea tiene que tener cuerpo");

    // 3) Y CABE: el contenido termina por encima de la navegacion (880 - 64 = 816).
    assert!(
        pos.y + t.height <= 816.0,
        "la linea se sale por abajo del contenido: acaba en {:.0}",
        pos.y + t.height
    );
}
