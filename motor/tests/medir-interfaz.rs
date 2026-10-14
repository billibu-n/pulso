//! **Medir la interfaz SIN abrir ventana.**
//!
//! POR QUE EXISTE, y es la regla AU-63 de la norma:
//! «antes de tocar algo visual, comprobar que se puede MEDIR; si la unica forma de saber como quedo
//! es que otra persona lo mire y lo cuente, EL METODO ESTA MAL».
//!
//! Hasta ahora eso era exactamente el caso: para saber si la ventana se parecia al diseno habia que
//! **abrirla y mirarla**. Con `i-slint-backend-testing` se consulta la **geometria** de los elementos
//! -- su posicion y su tamano -- y se comprueba **contra el diseno**.
//!
//! ESTA PRUEBA ES LA PRIMERA: comprueba que la herramienta de medir FUNCIONA. Sin eso, medir de mas
//! seria creer que se mide.
//!
//! DESDE LA FASE 1, EL DISENO ES LA ESPECIFICACION CONGELADA:
//! `docs/ESPECIFICACION-interfaz-resumen.md` (fase 0, 2026-10-14). Cada numero de aqui abajo sale de
//! ahi, y el comentario dice de que linea (`doc:N`) y de que prueba. **El contrato manda sobre el
//! codigo**: si una prueba falla, lo que esta mal es el armazon, no la prueba.

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
    // La ventana de prueba representa el estado CON datos. Su contrario se prueba aparte.
    v.set_puede(true);
    v
}

// ==================================================================================================
// HERRAMIENTAS DE MEDICION COMPARTIDAS
// ==================================================================================================
//
// POR QUE ESTAN AQUI Y NO REPETIDAS EN CADA PRUEBA (AU-73: un valor vive en UN sitio; AU-10): las
// constantes del contrato se escriben una vez. Si el contrato cambia, cambia esta lista y **las
// pruebas dicen cual de los tres sitios se quedo atras**.

/// Las medidas de la especificacion (`doc:43-54`), con su nombre para el mensaje de fallo.
mod doc {
    /// doc:43 · lienzo. Y el anclaje NO se confia al gestor de ventanas: 880 = 64 + 736 + 80.
    pub const ANCHO: f64 = 411.0;
    pub const ALTO: f64 = 880.0;
    /// doc:44 · cabecera (top app bar small, `estudio:24`).
    pub const CABECERA: f64 = 64.0;
    /// doc:45 · titulo de la cabecera (title-large): **22 px de TIPOGRAFIA**. OJO con confundirlo
    /// con lo que se mide: la prueba mide la CAJA del texto, y una linea de 22 px de tipografia mide
    /// **30** de alto. Comprobar «22» contra «22» no probaria nada: seria el contrato citandose a si
    /// mismo. Lo que se comprueba es que la caja sea la de la tipografia de 22 (30 medido), y el 22
    /// en si lo mira quien ve la captura.
    pub const CAJA_TITULO: f64 = 30.0;
    /// doc:46 · barra inferior (navigation bar, `estudio:25`).
    pub const NAV: f64 = 80.0;
    /// doc:47 · icono de la barra inferior.
    pub const ICONO: f64 = 24.0;
    /// doc:48 · etiqueta de la barra inferior (label-medium) a **12 px de tipografia**; su CAJA de
    /// una linea mide 17 de alto (medido).
    pub const CAJA_ETIQUETA: f64 = 17.0;
    /// doc:49 · pildora del activo: 64 x 32, radio 16.
    pub const PILDORA_ANCHO: f64 = 64.0;
    pub const PILDORA_ALTO: f64 = 32.0;
    /// doc:50 · area tactil minima (`minimumInteractiveComponentSize`).
    pub const TACTIL: f64 = 48.0;
    /// doc:52 · margen de la pantalla, por los CUATRO lados.
    pub const MARGEN: f64 = 16.0;
    /// doc:53 · hueco entre bloques del contenido.
    pub const HUECO: f64 = 14.0;
    /// doc:182 · alto del contenido: 880 - 64 - 80 = 736, declarado como MINIMO (AU-93).
    pub const ALTO_CONTENIDO: f64 = 736.0;
    /// doc:161 · el reparto del contrato: 411 / 4 = **102,75** por hueco. Es el numero que manda el
    /// contrato; con el aire que el propio contrato pide (doc:169), cada hueco queda en 97,75 (ver
    /// `HUECO_NAV_REAL`), y las dos cifras se comprueban.
    pub const HUECO_NAV: f64 = 411.0 / 4.0;
    /// doc:169 · aire entre huecos de la barra.
    pub const AIRE_NAV: f64 = 4.0;
}

/// `doc:161-169`: los 411 de la barra se reparten asi: 4 de margen a cada lado, 4 entre cada dos
/// huecos (x3) y 4 huecos iguales. Un hueco mide, por tanto, (411 - 4 - 4 - 12) / 4 = 97,75.
///
/// **Y ESTO NO ES UN REDONDEO DEL CONTRATO, ES ARITMETICA**: 411 entre 4 son 102,75, pero con el aire
/// que el propio contrato pide (doc:169) los 411 ya no estan enteros para los cuatro huecos. Lo que el
/// contrato pide de verdad -- y lo que se ve -- es que **los cuatro midan LO MISMO y lleguen juntos a
/// los 411**. Eso es lo que se comprueba.
const HUECO_NAV_REAL: f64 = (doc::ANCHO - 4.0 - 4.0 - 3.0 * doc::AIRE_NAV) / 4.0;

/// Busca un elemento por su `accessible-id`, que es la forma OFICIAL de nombrar algo para medirlo.
fn buscar<'a>(todos: &'a [i_slint_backend_testing::ElementHandle], id: &str) -> Option<&'a i_slint_backend_testing::ElementHandle> {
    todos
        .iter()
        .find(|e| e.accessible_id().map(|s| s.to_string()) == Some(id.to_string()))
}

#[test]
fn la_herramienta_de_medir_mide_de_verdad() {
    // La ventana entera: su tamano tiene que ser EL DEL DISENO, no el que le de el gestor de ventanas.
    let v = ventana_de_prueba();
    let raiz = ElementQuery::from_root(&v).find_first().expect("hay raiz");
    let pos = raiz.absolute_position();
    let tam = raiz.size();
    let (x, y, ancho, alto) = (f64::from(pos.x), f64::from(pos.y), f64::from(tam.width), f64::from(tam.height));
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
    // MEDIDAS DEL CONTRATO (doc:43-46), no del boceto:
    //   ventana ...... 411 x 880
    //   cabecera ..... 64 de alto (Material 3, `estudio:24`)
    //   nav .......... 80 de alto (Material 3, `estudio:25`)
    let v = ventana_de_prueba();
    let raiz = ElementQuery::from_root(&v).find_first().expect("hay raiz");
    let tam = raiz.size();

    println!("  ventana:      {:.0} x {:.0}   (contrato: 411 x 880)", f64::from(tam.width), f64::from(tam.height));
    assert_eq!(f64::from(tam.width).round() as i32, doc::ANCHO as i32, "el ancho tiene que ser el del contrato");
    assert_eq!(f64::from(tam.height).round() as i32, doc::ALTO as i32, "el alto tiene que ser el del contrato");

    // Y los sitios del armazon, uno a uno.
    // OJO AL NOMBRE: el id lleva el COMPONENTE delante -- `ArmazonMovil::cabecera`, no `cabecera`.
    // Se aprendio MIDIENDO: la primera version buscaba `cabecera` y no encontraba nada.
    // (`buscar` va por `accessible-id`, que no depende del nombre interno; `match_id` se queda para
    // comprobar que el id del componente sigue siendo el contractual, que es otra cosa que se vigila.)
    for (nombre, esperado) in [("ArmazonMovil::cabecera", doc::CABECERA), ("ArmazonMovil::nav", doc::NAV)] {
        let e = ElementQuery::from_root(&v)
            .match_id(nombre)
            .find_first()
            .unwrap_or_else(|| panic!("no se encuentra {nombre}"));
        let t = e.size();
        let p = e.absolute_position();
        println!("  {nombre:<26} {:.0}x{:.0} en ({:.0},{:.0})   (contrato: alto {esperado:.0})", f64::from(t.width), f64::from(t.height), f64::from(p.x), f64::from(p.y));
        assert_eq!(f64::from(t.height).round() as i32, esperado as i32, "{nombre} no mide lo del contrato");
    }
}

// ==================================================================================================
// FASE 1 · LA CABECERA (doc:139-154)
// ==================================================================================================

#[test]
fn la_cabecera_lleva_el_titulo_de_la_seccion_a_22_a_la_izquierda_y_centrado() {
    // CONTRATO (doc:143-148): 64 de alto; el titulo es LA SECCION (no el nombre de la app), a 22 px
    // (title-large), a la IZQUIERDA con un margen de 16, y centrado en los 64.
    //
    // POR QUE ESTA PRUEBA: el titulo decia «Pulso» -- el nombre de la app -- y el top app bar de
    // Android dice DONDE ESTAS (auditoria:39). Y estaba colocado con `x:` dentro de un layout, que
    // es la trampa (`slint:19-35`): se movia solo cuando cambiaba el layout.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();

    let cab = buscar(&todos, "cabecera").expect("la cabecera tiene que existir");
    let c_pos = cab.absolute_position();
    let c_tam = cab.size();
    println!("  cabecera: {:.0}x{:.0} en ({:.0},{:.0})", f64::from(c_tam.width), f64::from(c_tam.height), f64::from(c_pos.x), f64::from(c_pos.y));
    assert_eq!(f64::from(c_tam.height).round() as i32, doc::CABECERA as i32, "la cabecera no mide 64");
    assert_eq!(f64::from(c_pos.y).round() as i32, 0, "la cabecera va arriba");
    assert_eq!(f64::from(c_tam.width).round() as i32, doc::ANCHO as i32, "la cabecera ocupa el ancho entero");

    // EL TITULO: se localiza por lo que lo define -- un texto a 22 px dentro de la cabecera--, no por
    // su texto, porque el texto cambia con la seccion. Asi la prueba sirve para las cuatro.
    let titulo = todos
        .iter()
        .find(|e| {
            let p = e.absolute_position();
            let t = e.size();
            f64::from(t.height) > 0.0
                && f64::from(p.y) >= f64::from(c_pos.y) - 0.5
                && f64::from(p.y) + f64::from(t.height) <= f64::from(c_pos.y) + f64::from(c_tam.height) + 0.5
                && (f64::from(t.height) - doc::CAJA_TITULO).abs() < 1.5
        })
        .expect("la cabecera tiene que tener su titulo");
    let t_pos = titulo.absolute_position();
    let t_tam = titulo.size();
    println!(
        "  titulo:   {:.0}x{:.0} en ({:.0},{:.0})   (contrato: 22 px, margen 16, centrado en 64)",
        f64::from(t_tam.width), f64::from(t_tam.height), f64::from(t_pos.x), f64::from(t_pos.y)
    );

    assert_eq!(f64::from(c_tam.height).round() as i32, doc::CABECERA as i32, "doc:143");
    assert!(
        (f64::from(t_tam.height) - doc::CAJA_TITULO).abs() < 1.5,
        "el titulo tiene que ir a 22 px (title-large, doc:146): su caja mide {:.0}",
        f64::from(t_tam.height)
    );
    assert!(
        (f64::from(t_pos.x) - doc::MARGEN).abs() < 1.0,
        "el titulo va a la izquierda con margen 16 (doc:147): empieza en {:.1}",
        f64::from(t_pos.x)
    );
    // CENTRADO EN LOS 64: se mide por el centro de la caja, con la tolerancia del interlineado
    // (la caja de una linea es algo mas alta que la tinta).
    let centro_titulo = f64::from(t_pos.y) + f64::from(t_tam.height) / 2.0;
    let centro_cabecera = f64::from(c_pos.y) + doc::CABECERA / 2.0;
    assert!(
        (centro_titulo - centro_cabecera).abs() < 4.0,
        "el titulo va centrado verticalmente en los 64 (doc:148): su centro esta en {centro_titulo:.1} y el de la cabecera en {centro_cabecera:.1}"
    );
    assert!(
        f64::from(t_pos.y) >= f64::from(c_pos.y) - 0.5 && f64::from(t_pos.y) + f64::from(t_tam.height) <= f64::from(c_pos.y) + f64::from(c_tam.height) + 0.5,
        "el titulo no puede salirse de la cabecera: {:.1}..{:.1}",
        f64::from(t_pos.y),
        f64::from(t_pos.y) + f64::from(t_tam.height)
    );
}

#[test]
fn el_titulo_de_la_cabecera_es_la_seccion_no_el_nombre_de_la_app() {
    // doc:145 y auditoria:39, 44-45: **la app se llama «Pulso» y su nombre NO va en la barra**. Si
    // vuelve a poner «Pulso», esta prueba lo dice: no es un gusto, es el contrato.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();
    let cab = buscar(&todos, "cabecera").expect("la cabecera tiene que existir");
    let c_pos = cab.absolute_position();
    let c_tam = cab.size();
    let titulo = todos
        .iter()
        .find(|e| {
            let p = e.absolute_position();
            let t = e.size();
            f64::from(t.height) > 0.0
                && f64::from(p.y) >= f64::from(c_pos.y) - 0.5
                && f64::from(p.y) + f64::from(t.height) <= f64::from(c_pos.y) + f64::from(c_tam.height) + 0.5
                && (f64::from(t.height) - doc::CAJA_TITULO).abs() < 1.5
        })
        .expect("la cabecera tiene que tener su titulo");
    let cual = titulo
        .accessible_label()
        .map(|s| s.to_string())
        .unwrap_or_default();
    println!("  el titulo de la cabecera dice «{cual}»");
    assert_ne!(cual, "Pulso", "el nombre de la app NO va en la cabecera (doc:151-153)");
    assert!(
        !cual.is_empty() && cual != "-",
        "el titulo tiene que decir la seccion, y dice «{cual}»"
    );
}

// ==================================================================================================
// FASE 1 · LA BARRA DE ABAJO (doc:156-174)
// ==================================================================================================

#[test]
fn la_barra_de_abajo_esta_repartida_y_no_pegada() {
    // ESTE ERA EL FALLO VISIBLE, el que el dueno vio en pantalla: «ResumenZonasSesionesDatos».
    // El ancho de cada hueco lo decidia el TEXTO (32, 46, 31 px), asi que se pegaban unos a otros.
    //
    // CONTRATO (doc:161-169): **CUATRO huecos, repartidos por igual** (411/4), con `spacing: 4` y
    // `padding` lateral 4 entre ellos. Aqui se comprueba que los cuatro miden LO MISMO y que entre
    // los cuatro (mas su aire) ocupan el ancho entero de la ventana.
    //
    // SE BUSCAN POR SU NOMBRE (`nav-hueco-N`), no por su tamano: antes esta prueba adivinaba cuales
    // eran los huecos filtrando por posicion y altura, y eso se rompe solo con anadir un elemento.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();

    let mut huecos: Vec<(f64, f64)> = (0..4)
        .map(|i| {
            let e = buscar(&todos, &format!("nav-hueco-{i}"))
                .unwrap_or_else(|| panic!("falta el hueco {i} de la barra (doc:161)"));
            let t = e.size();
            let p = e.absolute_position();
            (f64::from(p.x), f64::from(t.width))
        })
        .collect();
    huecos.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let xs: Vec<f64> = huecos.iter().map(|(x, _)| *x).collect();
    let anchos: Vec<f64> = huecos.iter().map(|(_, w)| *w).collect();
    println!("  huecos de la nav: x={xs:?}");
    println!("  y sus anchos:     {anchos:?}   (contrato: {HUECO_NAV_REAL:.2} cada uno)");

    // 1) LOS CUATRO MIDEN LO MISMO: el reparto no lo decide el texto (AU-68).
    let primero = anchos[0];
    for a in &anchos {
        assert!(
            (a - primero).abs() < 1.0,
            "los huecos tienen que medir LO MISMO (el texto no puede mandar): {anchos:?}"
        );
    }
    // 2) Y EL ANCHO ES EL DEL REPARTO, que son DOS numeros del contrato y los dos se dicen:
    //    - el reparto puro: 411 / 4 = 102,75 (doc:161) -- es el numero que manda;
    //    - lo que sale al quitarle el aire que el propio contrato pide (doc:169): 97,75.
    //    El medido tiene que caer entre los dos, y mas cerca del segundo.
    assert!(
        primero <= doc::HUECO_NAV + 0.5 && primero >= HUECO_NAV_REAL - 0.5,
        "cada hueco mide {primero:.2}: el contrato dice 411/4 = {:.2} y, con su aire, {HUECO_NAV_REAL:.2}",
        doc::HUECO_NAV
    );
    // 3) Y EL AIRE ENTRE HUECOS ES EL DEL CONTRATO: 4 (doc:169). Sin el, los textos se pegan.
    for (a, b) in [(0, 1), (1, 2), (2, 3)] {
        let aire = xs[b] - (xs[a] + anchos[a]);
        assert!(
            (aire - doc::AIRE_NAV).abs() < 0.5,
            "entre el hueco {a} y el {b} tienen que quedar {} de aire (doc:169), y quedan {aire:.2}",
            doc::AIRE_NAV
        );
    }
    // 4) ENTRE LOS CUATRO OCUPAN LA VENTANA ENTERA: no sobra un trozo a la derecha.
    let ancho_ocupado = anchos.iter().sum::<f64>() + doc::AIRE_NAV * 3.0 + 4.0 + 4.0;
    assert!(
        (ancho_ocupado - doc::ANCHO).abs() < 1.0,
        "los cuatro huecos mas su aire tienen que ocupar los 411: ocupan {ancho_ocupado:.2}"
    );
    assert!(
        (xs[0] - doc::AIRE_NAV).abs() < 1.0,
        "el primer hueco empieza en el margen de 4 (doc:169), y empieza en {:.1}",
        xs[0]
    );

    // 5) Y EL REPARTO NO TIENE NUMEROS SUELTOS EN EL ARMAZON (AU-67, AU-73). Esta comprobacion existe
    // porque el ancho de los huecos se DERIVA: si alguien escribe `(parent.width - 8px - 12px) / 4`
    // a mano, el dia que cambie el aire los dos numeros divergen y la pantalla se aprieta sola. Se
    // lee la FUENTE del armazon y se exige que los tres sumandos salgan de `Medidas`.
    let fuente = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/armazon.slint"))
        .expect("el armazon tiene que leerse");
    let derivado = fuente.contains(
        "width: (parent.width - 2 * Medidas.aire_nav - 3 * Medidas.aire_nav) / 4;",
    );
    assert!(
        derivado,
        "el ancho de cada hueco tiene que DERIVARSE de `Medidas.aire_nav` (AU-67), y en el armazon \
         no esta esa linea"
    );
    let aire_declarado = fuente.contains(&format!("aire_nav: {}px;", doc::AIRE_NAV as i32));
    assert!(
        aire_declarado,
        "`Medidas.aire_nav` tiene que valer {} (doc:169)",
        doc::AIRE_NAV
    );
    println!("  el ancho de los huecos se deriva de Medidas.aire_nav = {} (doc:169)", doc::AIRE_NAV);
}

#[test]
fn cada_hueco_de_la_barra_es_area_tactil_y_lleva_icono_de_24_y_etiqueta_de_12() {
    // CONTRATO (doc:162-167): icono REAL del catalogo a **24**, etiqueta a **12** centrada en su
    // hueco, y **area tactil ≥ 48 por hueco entero** (`estudio:27`). Orden vertical: pildora+icono
    // arriba, etiqueta debajo, con 2 de hueco, todo centrado.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();

    for i in 0..4 {
        let hueco = buscar(&todos, &format!("nav-hueco-{i}")).expect("hueco de la barra");
        let h_pos = hueco.absolute_position();
        let h_tam = hueco.size();
        let (hx, hy) = (f64::from(h_pos.x), f64::from(h_pos.y));
        let (hw, hh) = (f64::from(h_tam.width), f64::from(h_tam.height));

        // EL AREA TACTIL ES EL HUECO ENTERO (doc:166): el boton ocupa los 80 de la barra.
        assert!(
            hw >= doc::TACTIL && hh >= doc::TACTIL,
            "el hueco {i} tiene que llegar a 48x48 de area tactil (doc:166): mide {hw:.1}x{hh:.1}"
        );

        // Y DENTRO, tres cosas: la pildora, su icono y la etiqueta.
        let dentro = |e: &&i_slint_backend_testing::ElementHandle| {
            let p = e.absolute_position();
            f64::from(p.x) >= f64::from(h_pos.x) - 0.5
                && f64::from(p.x) + f64::from(e.size().width) <= f64::from(h_pos.x) + f64::from(h_tam.width) + 0.5
                && f64::from(p.y) >= f64::from(h_pos.y) - 0.5
                && f64::from(p.y) + f64::from(e.size().height) <= f64::from(h_pos.y) + f64::from(h_tam.height) + 0.5
        };
        // LA CAJA DEL ICONO: 24 x 24, con 1 px de tolerancia por lado. ES UN FILTRO, no la
        // comprobacion: la comprobacion es la de abajo, contra `doc::ICONO`. Se ensancha a proposito
        // porque lo que se busca es «cuales de estos elementos son el icono», y afinarlo aqui no
        // anadiria nada: lo que importa es que el elegido mida 24.
        let caja_icono = (doc::ICONO - 2.0, doc::ICONO + 2.0);
        let iconos: Vec<&i_slint_backend_testing::ElementHandle> = todos
            .iter()
            .filter(|e| {
                let t = e.size();
                let w = f64::from(t.width);
                let h = f64::from(t.height);
                dentro(e) && (w - caja_icono.0).abs() <= 2.0 && (h - caja_icono.0).abs() <= 2.0
            })
            .collect();
        let icono = iconos
            .iter()
            .copied()
            .max_by(|a, b| {
                f64::from(a.size().width)
                    .partial_cmp(&f64::from(b.size().width))
                    .unwrap()
            })
            .unwrap_or_else(|| panic!("el hueco {i} no tiene icono de 24 (doc:162)"));
        let i_pos = icono.absolute_position();
        let i_tam = icono.size();
        println!(
            "  hueco {i}: {hw:.1}x{hh:.1} en ({hx:.1},{hy:.1}) · icono {:.0}x{:.0} en ({:.1},{:.1})",
            f64::from(i_tam.width), f64::from(i_tam.height), f64::from(i_pos.x), f64::from(i_pos.y)
        );
        assert!(
            (f64::from(i_tam.width) - doc::ICONO).abs() < 1.0
                && (f64::from(i_tam.height) - doc::ICONO).abs() < 1.0,
            "el icono del hueco {i} tiene que medir 24 (doc:162): mide {:.1}x{:.1}",
            f64::from(i_tam.width),
            f64::from(i_tam.height)
        );

        // LA PILDORA: 64 x 32 (doc:165). Y va DETRAS del icono: mismo centro horizontal y ARRIBA
        // de el en el eje vertical.
        //
        // SE BUSCA POR SU NOMBRE, no por su tamano: el contenedor, la fila y la capa de dentro miden
        // 64 x 32 igual que ella, y buscarla por tamano devolvia cuatro «pildoras» por hueco.
        // Y LA PILDORA EXISTE EN CADA HUECO, este activo o no: **una accion, una sola forma**
        // (AU-41). Su estado va en el nombre (`-activa` / `-apagada`), porque el fondo de una capa
        // transparente no se puede consultar desde una prueba.
        let pildora = buscar(&todos, &format!("nav-pildora-{i}-activa"))
            .or_else(|| buscar(&todos, &format!("nav-pildora-{i}-apagada")))
            .unwrap_or_else(|| panic!("el hueco {i} no tiene la pildora (doc:165)"));
        let p_tam = pildora.size();
        assert!(
            (f64::from(p_tam.width) - doc::PILDORA_ANCHO).abs() < 1.0
                && (f64::from(p_tam.height) - doc::PILDORA_ALTO).abs() < 1.0,
            "la pildora mide 64 x 32 (doc:165), y mide {:.1}x{:.1}",
            f64::from(p_tam.width),
            f64::from(p_tam.height)
        );
        let p_pos = pildora.absolute_position();
        let centro_pildora = f64::from(p_pos.x) + doc::PILDORA_ANCHO / 2.0;
        let centro_icono = f64::from(i_pos.x) + f64::from(i_tam.width) / 2.0;
        assert!(
            (centro_pildora - centro_icono).abs() < 1.0,
            "la pildora va DETRAS del icono (doc:165): su centro esta en {centro_pildora:.1} y el del icono en {centro_icono:.1}"
        );
        assert!(
            f64::from(p_pos.y) < f64::from(i_pos.y),
            "la pildora va ARRIBA del icono (el icono se pinta encima y dentro): pildora en {:.1}, icono en {:.1}",
            f64::from(p_pos.y),
            f64::from(i_pos.y)
        );

        // LA ETIQUETA: 12 px (doc:164) y centrada en su hueco. Se localiza por su tamano, y se
        // comprueba que TAPA EL MISMO EJE que la pildora: eso es «centrada en su hueco».
        let etiqueta = todos
            .iter()
            .filter(dentro)
            .find(|e| (f64::from(e.size().height) - doc::CAJA_ETIQUETA).abs() < 1.0)
            .unwrap_or_else(|| panic!("el hueco {i} no tiene etiqueta (doc:164)"));
        let et_pos = etiqueta.absolute_position();
        let et_tam = etiqueta.size();
        let centro_etiqueta = f64::from(et_pos.x) + f64::from(et_tam.width) / 2.0;
        println!(
            "           etiqueta «{}» de {:.0} de alto, centrada en {:.1}",
            etiqueta.accessible_label().map(|s| s.to_string()).unwrap_or_default(),
            f64::from(et_tam.height),
            centro_etiqueta
        );
        assert!(
            (f64::from(et_tam.height) - doc::CAJA_ETIQUETA).abs() < 1.0,
            "la etiqueta va a 12 px (doc:164): su caja mide {:.0}",
            f64::from(et_tam.height)
        );
        assert!(
            (centro_etiqueta - centro_icono).abs() < 1.0,
            "la etiqueta va centrada en su hueco, en el eje del icono: etiqueta en {centro_etiqueta:.1}, icono en {centro_icono:.1}"
        );
        // Y DEBAJO DEL ICONO, con su hueco de 2 (doc:167): una etiqueta encima del icono no se lee.
        assert!(
            f64::from(et_pos.y) >= f64::from(i_pos.y) + f64::from(i_tam.height) - 0.5,
            "la etiqueta va DEBAJO del icono (doc:167)"
        );
    }
}

#[test]
fn la_pildora_solo_marca_el_hueco_activo() {
    // doc:165-173: la pildora marca EL ACTIVO, y **el activo es uno**. Cuatro pildoras encendidas
    // serian cuatro activos: no diria nada.
    //
    // COMO SE MIDE, y esto hay que decirlo porque es el limite de la herramienta: **el fondo de una
    // capa transparente NO se puede consultar desde una prueba de geometria**. Y el tamano tampoco
    // lo delata: la pildora apagada mide 64 x 32 igual que la encendida (medido). Asi que el estado
    // va en el NOMBRE del elemento (`nav-pildora-N-activa` / `-apagada`), que es lo unico que una
    // prueba puede leer. **El color en si -- el acento al 18 % -- lo mira quien ve la captura.**
    let v = ventana_de_prueba();
    let pildoras = |v: &Ventana| -> Vec<(String, f64)> {
        ElementQuery::from_root(v)
            .find_all()
            .iter()
            .filter_map(|e| {
                let nombre = e.accessible_id().map(|s| s.to_string()).unwrap_or_default();
                if !nombre.starts_with("nav-pildora-") {
                    return None;
                }
                Some((nombre, f64::from(e.absolute_position().x)))
            })
            .collect()
    };

    let todas = pildoras(&v);
    let activas: Vec<&(String, f64)> = todas.iter().filter(|(n, _)| n.ends_with("-activa")).collect();
    println!(
        "  pildoras: {} declaradas, {} encendidas {:?}",
        todas.len(),
        activas.len(),
        activas.iter().map(|(n, x)| format!("{n} en x={x:.1}")).collect::<Vec<_>>()
    );
    // UNA PILDORA POR HUECO: la misma forma en las cuatro, este activa o no (AU-41).
    assert_eq!(todas.len(), 4, "cada hueco lleva su pildora, aunque no este activa (AU-41)");
    // Y UNA SOLA ENCENDIDA (doc:165).
    assert_eq!(activas.len(), 1, "tiene que haber UNA pildora encendida: la del hueco activo (doc:165)");
    assert_eq!(activas[0].0, "nav-pildora-0-activa", "el activo por defecto es la seccion 0 (Resumen)");

    // Y AL CAMBIAR DE SECCION LA PILDORA SE MUEVE, no se duplica.
    v.set_seccion_activa(2);
    let todas2 = pildoras(&v);
    let activas2: Vec<&(String, f64)> = todas2.iter().filter(|(n, _)| n.ends_with("-activa")).collect();
    println!(
        "  con la seccion 2 activa: {} encendidas {:?}",
        activas2.len(),
        activas2.iter().map(|(n, x)| format!("{n} en x={x:.1}")).collect::<Vec<_>>()
    );
    assert_eq!(activas2.len(), 1, "la pildora sigue siendo UNA al cambiar de seccion");
    assert_eq!(activas2[0].0, "nav-pildora-2-activa", "la pildora se mueve al hueco de la seccion activa (doc:165)");

    // Y CAE DENTRO DE SU HUECO, que es lo que la hace legible: no vale estar «cerca».
    let todos = ElementQuery::from_root(&v).find_all();
    let hueco = buscar(&todos, "nav-hueco-2").expect("hueco 2");
    let h_x = f64::from(hueco.absolute_position().x);
    let h_w = f64::from(hueco.size().width);
    let p_x = activas2[0].1;
    assert!(
        p_x >= h_x - 0.5 && p_x + doc::PILDORA_ANCHO <= h_x + h_w + 0.5,
        "la pildora tiene que estar dentro del hueco de su seccion: {p_x:.1} no cae en {h_x:.1}..{:.1}",
        h_x + h_w
    );
}

// ==================================================================================================
// FASE 1 · EL CONTENIDO Y EL ANCLAJE A LA VENTANA (doc:176-192, AU-93)
// ==================================================================================================

#[test]
fn el_contenido_declara_su_altura_minima_y_la_barra_no_se_mueve() {
    // EL FALLO QUE ESTA PRUEBA VIGILA, y costo un rato: al dejar de dibujar los vacios (AU-91) el
    // armazon ENCOGIO y **la barra de abajo subio al centro de la pantalla**. Se cazo midiendo, no
    // mirando (`estado:486-487`). De ahi la regla AU-93 y el numero 736 = 880 - 64 - 80.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();

    let cont = buscar(&todos, "contenido").expect("el contenido tiene que existir");
    let c_pos = cont.absolute_position();
    let c_tam = cont.size();
    let nav = buscar(&todos, "navegacion").expect("la navegacion tiene que existir");
    let n_pos = nav.absolute_position();
    let n_tam = nav.size();
    println!(
        "  contenido: {:.0}x{:.0} en ({:.0},{:.0}) · nav: {:.0}x{:.0} en ({:.0},{:.0})",
        f64::from(c_tam.width), f64::from(c_tam.height), f64::from(c_pos.x), f64::from(c_pos.y), f64::from(n_tam.width), f64::from(n_tam.height), f64::from(n_pos.x), f64::from(n_pos.y)
    );

    // EL CONTENIDO DECLARA SU ALTURA MINIMA (doc:182): 880 - 64 - 80 = 736.
    assert!(
        f64::from(c_tam.height) >= doc::ALTO_CONTENIDO - 0.5,
        "el contenido tiene que medir al menos 736 de alto (AU-93, doc:182): mide {:.1}",
        f64::from(c_tam.height)
    );
    // Y EMPIEZA DONDE ACABA LA CABECERA.
    assert!(
        (f64::from(c_pos.y) - doc::CABECERA).abs() < 0.5,
        "el contenido empieza a los 64 de la cabecera: empieza en {:.1}",
        f64::from(c_pos.y)
    );
    // LA BARRA DE ABAJO NO SE MUEVE (AU-93): pegada al borde inferior, con 80 de alto.
    assert_eq!(f64::from(n_tam.height).round() as i32, doc::NAV as i32, "la barra mide 80 (doc:160)");
    assert!(
        (f64::from(n_pos.y) - (doc::ALTO - doc::NAV)).abs() < 0.5,
        "la barra va pegada abajo (y = 800), y esta en {:.1}",
        f64::from(n_pos.y)
    );
    assert!(
        (f64::from(n_pos.y) + f64::from(n_tam.height) - doc::ALTO).abs() < 0.5,
        "la barra no puede dejar hueco por debajo"
    );
    // Y SIN HUECO ENTRE LOS DOS: es el defecto 7 de la auditoria (un hueco grande antes de la barra).
    // (El alto minimo declarado de 736 es lo que lo garantiza: el hueco de bloques de dentro es
    // `doc::HUECO`, 14, y va comprobado en `el_contenido_lleva_margen_de_16_por_los_cuatro_lados`.)
    let hueco_antes_de_la_barra = f64::from(n_pos.y) - (f64::from(c_pos.y) + f64::from(c_tam.height));
    println!("  hueco entre el contenido y la barra: {hueco_antes_de_la_barra:.1}");
    assert!(
        hueco_antes_de_la_barra.abs() < 0.5,
        "el contenido llega hasta la barra, sin hueco: quedan {hueco_antes_de_la_barra:.1}"
    );

    // EL ANCLAJE NO DEPENDE DEL CONTENIDO: con la persona SIN datos -- el caso que encogia el
    // armazon-- la barra sigue en el mismo sitio.
    v.set_puede(false);
    let sin_datos = ElementQuery::from_root(&v).find_all();
    let nav2 = buscar(&sin_datos, "navegacion").expect("la navegacion sigue existiendo");
    let n2_pos = nav2.absolute_position();
    println!("  sin datos, la barra sigue en y={:.1}", n2_pos.y);
    assert!(
        (f64::from(n2_pos.y) - f64::from(n_pos.y)).abs() < 0.5,
        "quitar bloques NO puede mover la barra (AU-93): estaba en {:.1} y quedo en {:.1}",
        f64::from(n_pos.y),
        n2_pos.y
    );
    let cont2 = buscar(&sin_datos, "contenido").expect("el contenido sigue existiendo");
    assert!(
        f64::from(cont2.size().height) >= doc::ALTO_CONTENIDO - 0.5,
        "sin datos el contenido SIGUE midiendo 736 (doc:182): mide {:.1}",
        cont2.size().height
    );
}

#[test]
fn el_primer_bloque_del_contenido_empieza_con_el_margen_de_16() {
    // ESTA PRUEBA YA NO ESTA IGNORADA (2026-10-14): el margen lo entrega CADA BLOQUE, y se mide.
    //
    // POR QUE ESTUVO IGNORADA, y no era un capricho: en la fase 1 el armazon dejo de poner el margen
    // (con `padding` dentro del `ScrollView` el `ScrollView` pinta su barra horizontal: medido, una
    // linea gris de 370 px en y = 794), y los bloques de dentro eran los de la fase 2, que se
    // reescriben. Se dejo ignorada porque MEDIA ALGO QUE FALTABA, y se des-ignora cuando ese algo
    // existe. Existe: `Bloque` (`ventana.slint`) le pone a cada bloque sus dos huecos de
    // `Medidas.margen`, y el armazon le pone el aire de arriba y de abajo del contenido.
    //
    // doc:180: el contenido tiene **margen 16** por los cuatro lados. **QUE ELEMENTO SE MIDE, y aqui
    // hubo que corregir la prueba (2026-10-14)**: se mide LA TARJETA (la caja visible del bloque),
    // que es la que respeta el margen, y NO el envoltorio `Bloque`. El envoltorio mide **411**, el
    // ancho ENTERO del contenido -- los dos huecos de 16 son HIJOS suyos, para que el `ScrollView` no
    // vea un contenido de 443 donde caben 411 (que es lo que le hace pintar la barra) --, asi que
    // exigirle 16 al envoltorio seria exigir que el contenido midiera 443: la cuenta que no cabe.
    // El elemento que se mide mide, por tanto, **411 − 2·16 = 379**, y esa es la cuenta que se
    // escribe aqui. (Antes esta prueba buscaba un elemento de 411 de ancho: eso era la geometria
    // VIEJA, la del `padding` en el layout del `ScrollView`, y con ella el margen volvia a 0.)
    //
    // Y no se cuentan «elementos sueltos»: se exige que haya **uno solo** que cumpla la geometria del
    // contrato. Contar candidatos sueltos fue lo que hizo que esta prueba pasara **midiendo 0** cuando
    // el armazon dejo de poner el margen (medido, y se vio porque la prueba imprime el numero).
    // UNA VENTANA CON UN SOLO BLOQUE: asi «el primer bloque» es, sin dudas, la tarjeta del periodo,
    // y no lo que resulte estar primero. (Con el estado con datos, el primero es la rejilla de
    // cifras, que es de la fase 2 y trae sus propios margenes.)
    i_slint_backend_testing::init_no_event_loop();
    let v = Ventana::new().expect("la ventana tiene que construirse");
    v.set_periodo("Análisis del 2026-05-02 al 2026-10-11".into());
    let todos = ElementQuery::from_root(&v).find_all();

    let cont = buscar(&todos, "contenido").expect("el contenido tiene que existir");
    let c_pos = cont.absolute_position();
    let c_tam = cont.size();
    // La tarjeta del periodo: la caja de 52 de alto, y de 411 − 2·16 = 379 de ancho (el ancho del
    // contenido MENOS sus dos margenes), que es la que tiene que estar a 16 de cada lado.
    let ancho_de_la_tarjeta = doc::ANCHO - 2.0 * doc::MARGEN;
    let candidatas: Vec<&i_slint_backend_testing::ElementHandle> = todos
        .iter()
        .filter(|e| {
            let t = e.size();
            (f64::from(t.height) - 52.0).abs() < 0.5
                && (f64::from(t.width) - ancho_de_la_tarjeta).abs() < 1.0
                && f64::from(e.absolute_position().y) >= f64::from(c_pos.y) - 0.5
        })
        .collect();
    assert!(
        !candidatas.is_empty(),
        "tiene que haber una caja de {ancho_de_la_tarjeta} x 52 en el contenido: es la tarjeta del \
         periodo con sus dos margenes"
    );
    // Y TODAS LAS QUE HAYA TIENEN QUE ESTAR EN EL MISMO SITIO, que es lo que hace que la medida no
    // dependa de cual se elija: la tarjeta y sus dos envoltorios de layout son cajas CONCENTRICAS (la
    // misma caja, tres elementos). Si alguna se despegara, aqui se ve, en vez de elegir «la primera».
    let primera = candidatas[0].absolute_position();
    for (cual, e) in candidatas.iter().enumerate() {
        let p = e.absolute_position();
        assert!(
            (f64::from(p.x) - f64::from(primera.x)).abs() < 0.5
                && (f64::from(p.y) - f64::from(primera.y)).abs() < 0.5,
            "las {} cajas de la tarjeta del periodo tienen que estar concentricas, y la {cual} esta \
             en ({:.1},{:.1}) y la primera en ({:.1},{:.1})",
            candidatas.len(),
            f64::from(p.x),
            f64::from(p.y),
            f64::from(primera.x),
            f64::from(primera.y)
        );
    }
    let tarjeta = candidatas[0];
    let t_pos = tarjeta.absolute_position();
    let t_tam = tarjeta.size();
    let izq = f64::from(t_pos.x) - f64::from(c_pos.x);
    let arriba = f64::from(t_pos.y) - f64::from(c_pos.y);
    let der = (f64::from(c_pos.x) + f64::from(c_tam.width))
        - (f64::from(t_pos.x) + f64::from(t_tam.width));
    println!("  primer bloque: izq {izq:.1} · arriba {arriba:.1} · der {der:.1} (contrato: 16)");
    for (lado, valor) in [("izquierda", izq), ("arriba", arriba), ("derecha", der)] {
        assert!(
            (valor - doc::MARGEN).abs() < 1.0,
            "el margen por {lado} tiene que ser 16 (doc:180), y mide {valor:.1}"
        );
    }
}

#[test]
fn el_armazon_no_pone_el_margen_dentro_de_lo_que_se_desplaza() {
    // ESTA ES UNA DECISION DE LA FASE 1, Y AQUI SE VIGILA PARA QUE NO SE DESHAGA SIN QUERER.
    //
    // El margen de 16 lo lleva **cada bloque**, no el layout que esta dentro del `ScrollView`.
    // Medido, y por eso: con cualquier `padding` ahi dentro, el contenido se mide a 411 + 2·16 = 443,
    // el `ScrollView` cree que hay algo que desplazar a lo ancho y **pinta su barra**: una linea gris
    // de 370 px pegada al borde de abajo del contenido (y = 794). Probado tambien con 20 de relleno:
    // sale igual. Y pedirle al `ScrollView` su ancho y su alto no lo arregla.
    //
    // Si alguien devuelve el `padding` al layout de dentro (para «cumplir el doc:180 al pie de la
    // letra»), esta prueba lo dice, y la linea gris vuelve: entonces hay que decidir que se prefiere,
    // y esa decision es del orquestador, no de la prueba.
    let fuente = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/armazon.slint"))
        .expect("el armazon tiene que leerse");
    let dentro_del_scroll = fuente
        .split("ScrollView {")
        .nth(1)
        .and_then(|t| t.split("@children").next())
        .expect("el armazon tiene que tener el ScrollView con sus hijos");
    // SOLO EL CODIGO: los comentarios de ese tramo EXPLICAN por que no hay `padding`, y nombrarlo
    // para explicarlo no es ponerlo.
    let codigo: String = dentro_del_scroll
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !codigo.contains("padding"),
        "el layout que esta DENTRO del ScrollView no puede llevar `padding`: hace que el ScrollView \
         pinte su barra horizontal (medido: una linea gris de 370 px en y = 794). El margen de 16 lo \
         lleva cada bloque."
    );
    println!("  el margen de 16 lo lleva cada bloque: el layout del ScrollView no declara padding");
}

#[test]
fn el_armazon_deja_14_de_hueco_entre_los_bloques_del_contenido() {
    // doc:181: **hueco entre bloques del contenido = 14**. Es una promesa del ARMAZON: lo pone su
    // layout, no cada bloque. Y se comprueba **por la fuente, no por la geometria**, y aqui hay que
    // decir por que, porque no es lo mismo:
    //
    //   - los bloques que hay hoy dentro son los de la fase 2 (`Contenido`), y ese componente trae su
    //     PROPIO `spacing: 12`: medido, los huecos de la pantalla de hoy salen a 12, y la fase 2 los
    //     reescribe. Medir la pantalla de hoy seria medir el bloque de la fase 2, no el armazon;
    //   - y los bloques de prueba que se probaron a meter (dos `Text`) **no llegan al arbol medible**
    //     cuando el contenido no cabe en 736: el desplazamiento interno del `ScrollView` los deja
    //     fuera de lo que el backend de medida devuelve (medido).
    //
    // Asi que se comprueba lo que si es comprobable: que el armazon DECLARA ese hueco, que es el
    // numero del contrato, y que lo saca de `Medidas` (AU-67, una medida un sitio) en vez de escribirlo
    // a mano. Si alguien cambia el 14 por un 12 «para que quede mejor», esto salta.
    let fuente = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/armazon.slint"))
        .expect("el armazon tiene que leerse");
    let declarado = fuente.contains("spacing: Medidas.hueco_bloques;");
    assert!(
        declarado,
        "el armazon tiene que declarar `spacing: Medidas.hueco_bloques;` en el contenido (doc:181)"
    );
    let en_medidas = fuente.contains(&format!("hueco_bloques: {}px;", doc::HUECO as i32));
    assert!(
        en_medidas,
        "`Medidas.hueco_bloques` tiene que valer {} (doc:181, AU-67)",
        doc::HUECO
    );
    println!(
        "  hueco entre bloques: declarado en el armazon como Medidas.hueco_bloques = {} (doc:181)",
        doc::HUECO
    );
}

// ==================================================================================================
// FASE 1 · EL REPARTO DE LA BARRA: 411/4, y el anclaje a 411 x 880
// ==================================================================================================

#[test]
fn el_armazon_esta_anclado_a_411x880_y_nada_se_sale() {
    // EL ANCLAJE (doc:43, doc:182, AU-93): la pantalla es **un telefono de 411 x 880** y no se
    // redimensiona. La barra se apoya en el borde de abajo (y = 880 - 80 = 800) y el contenido mide
    // 736. Se comprueban las CUATRO fronteras, y que ningun elemento del arbol se salga.
    let v = ventana_de_prueba();
    let todos = ElementQuery::from_root(&v).find_all();

    let cab = buscar(&todos, "cabecera").expect("cabecera");
    let cont = buscar(&todos, "contenido").expect("contenido");
    let nav = buscar(&todos, "navegacion").expect("navegacion");
    let (c_y, c_h) = (f64::from(cab.absolute_position().y), f64::from(cab.size().height));
    let (t_y, t_h) = (f64::from(cont.absolute_position().y), f64::from(cont.size().height));
    let (n_y, n_h) = (f64::from(nav.absolute_position().y), f64::from(nav.size().height));
    println!("  reparto vertical: cabecera {c_y:.1}..{:.1} · contenido {t_y:.1}..{:.1} · nav {n_y:.1}..{:.1}", c_y + c_h, t_y + t_h, n_y + n_h);

    // LOS TRES SUMAN EL LIENZO, sin solapes ni huecos: 64 + 736 + 80 = 880.
    assert!((c_y - 0.0).abs() < 0.5, "la cabecera empieza en 0");
    assert!((t_y - (c_y + c_h)).abs() < 0.5, "el contenido empieza donde acaba la cabecera");
    assert!((n_y - (t_y + t_h)).abs() < 0.5, "la barra empieza donde acaba el contenido");
    assert!(
        (n_y + n_h - doc::ALTO).abs() < 0.5,
        "la barra tiene que acabar en el borde de abajo (880), y acaba en {:.1}",
        n_y + n_h
    );
    assert!(
        (c_h + t_h + n_h - doc::ALTO).abs() < 0.5,
        "64 + 736 + 80 tiene que dar 880, y da {:.1}",
        c_h + t_h + n_h
    );

    // Y NINGUN ELEMENTO SE SALE DEL LIENZO. El ancho es un dato del contrato, no una consecuencia:
    // si algo se sale, es que se le dio un ancho de mas.
    let mut peores: Vec<(f64, String, f64, f64, f64, f64)> = Vec::new();
    for e in todos.iter() {
        let t = e.size();
        let p = e.absolute_position();
        let (w, h) = (f64::from(t.width), f64::from(t.height));
        if w <= 0.0 && h <= 0.0 {
            continue;
        }
        let (x, y) = (f64::from(p.x), f64::from(p.y));
        let der = x + w;
        let aba = y + h;
        let sobra = (der - doc::ANCHO).max(aba - doc::ALTO).max(-x).max(-y);
        if sobra > 0.5 {
            peores.push((sobra, e.accessible_id().map(|s| s.to_string()).unwrap_or_else(|| "(sin nombre)".into()), x, y, w, h));
        }
    }
    if !peores.is_empty() {
        peores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (sobra, id, x, y, w, h) in peores.iter().take(5) {
            println!("  SE SALE {sobra:.1} px: {id} {w:.0}x{h:.0} en ({x:.0},{y:.0})");
        }
        panic!("hay {} elementos fuera del lienzo 411x880", peores.len());
    }
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
    println!("  selector-usar mide {:.0}x{:.0}", f64::from(t.width), f64::from(t.height));
    assert!(f64::from(t.height) >= 48.0, "el boton de usar tiene que llegar a 48 dp, y mide {}", f64::from(t.height));

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
        f64::from(m_pos.x), f64::from(m_pos.y), f64::from(m_tam.width), f64::from(m_tam.height),
        f64::from(b_pos.x), f64::from(b_pos.y), f64::from(b_tam.width), f64::from(b_tam.height)
    );

    // EL BOTON CABE DENTRO DEL MARCO. Si esto falla, el marco lo esta recortando -- y un boton
    // recortado se ve peor que un boton sin contorno, que es lo que se venia a arreglar.
    assert!(
        f64::from(b_pos.x) >= f64::from(m_pos.x) && f64::from(b_pos.y) >= f64::from(m_pos.y),
        "el boton empieza fuera del marco: boton en ({}, {}), marco en ({}, {})",
        f64::from(b_pos.x), f64::from(b_pos.y), f64::from(m_pos.x), f64::from(m_pos.y)
    );
    assert!(
        f64::from(b_pos.x) + f64::from(b_tam.width) <= f64::from(m_pos.x) + f64::from(m_tam.width) + 0.5,
        "el boton se sale del marco por la derecha: boton acaba en {}, marco en {}",
        f64::from(b_pos.x) + f64::from(b_tam.width),
        f64::from(m_pos.x) + f64::from(m_tam.width)
    );
    assert!(
        f64::from(b_pos.y) + f64::from(b_tam.height) <= f64::from(m_pos.y) + f64::from(m_tam.height) + 0.5,
        "el boton se sale del marco por abajo: boton acaba en {}, marco en {}",
        f64::from(b_pos.y) + f64::from(b_tam.height),
        f64::from(m_pos.y) + f64::from(m_tam.height)
    );

    // Y EL BOTON LLEGA AL MINIMO TACTIL, que en este proyecto son 48 dp. El minimo del estilo
    // de serie son **32 px**, asi que **hay que pedirlo**: no viene puesto. Se pide con
    // `height: max(48px, self.preferred-height)`, no con un 48 fijo -- asi, el dia que el texto
    // necesite mas alto, el boton crece en vez de recortarlo.
    println!("  alto del boton: {:.0} px", f64::from(b_tam.height));
    assert!(
        f64::from(b_tam.height) >= 48.0,
        "el boton tiene que llegar a 48 dp de alto (minimo tactil del proyecto), y mide {}",
        f64::from(b_tam.height)
    );

    // Y EL MARCO RODEA AL BOTON SIN APLASTARLO: el hueco entre los dos es el `padding` del
    // layout (2 px) mas el borde (2 px). Se comprueba que hay hueco por los CUATRO lados.
    let hueco_izq = f64::from(b_pos.x) - f64::from(m_pos.x);
    let hueco_arr = f64::from(b_pos.y) - f64::from(m_pos.y);
    let hueco_der = (f64::from(m_pos.x) + f64::from(m_tam.width)) - (f64::from(b_pos.x) + f64::from(b_tam.width));
    let hueco_aba = (f64::from(m_pos.y) + f64::from(m_tam.height)) - (f64::from(b_pos.y) + f64::from(b_tam.height));
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
        f64::from(t.width), f64::from(t.height), f64::from(pos.x), f64::from(pos.y), v.get_periodo()
    );
    assert!(f64::from(t.height) > 0.0 && f64::from(t.width) > 100.0, "la linea tiene que tener cuerpo");

    // 3) Y CABE: el contenido termina por encima de la navegacion (880 - 64 = 816).
    assert!(
        f64::from(pos.y) + f64::from(t.height) <= 800.0,
        "la linea se sale por abajo del contenido: acaba en {:.0}",
        f64::from(pos.y) + f64::from(t.height)
    );
}

// ==================================================================================================
// EL PORTERO DE ICONOS DE LA BARRA (2026-10-14)
// ==================================================================================================

#[test]
fn sin_poder_diagnosticar_la_pantalla_no_dibuja_tarjetas_vacias() {
    // EL FALLO QUE VIO EL DUEÑO: con la persona sin datos, la pantalla enseñaba CUATRO tarjetas de
    // cifras vacias y CINCO barras de zona sin nombre. No es que faltara el dato: es que se dibujaba
    // el marco. Aqui se comprueba que, sin poder diagnosticar, NO hay tarjetas ni filas de zona.
    let v = ventana_de_prueba();
    let con_datos = ElementQuery::from_root(&v).find_all();
    let tarjetas_con = con_datos
        .iter()
        .filter(|e| (f64::from(e.size().height) - 66.0).abs() < 1.0)
        .count();
    assert!(tarjetas_con >= 4, "con datos hay tarjetas de cifras: {tarjetas_con}");

    v.set_puede(false);
    let sin_datos = ElementQuery::from_root(&v).find_all();
    for (alto, que) in [(66.0, "tarjetas de cifras"), (40.0, "filas de zona")] {
        let cuantas = sin_datos
            .iter()
            .filter(|e| (f64::from(e.size().height) - alto).abs() < 1.0)
            .count();
        assert_eq!(cuantas, 0, "sin poder diagnosticar NO puede haber {que} vacias: {cuantas}");
    }
}

/// Los numeros de un trazado SVG, EN ORDEN, sin perder ninguno.
///
/// OJO CON EL SEPARADOR, y costo un fallo: en SVG el signo menos SEPARA numeros sin coma
/// (`... 0 0 0-1-1h-4` son tres numeros: -1, -1). Por eso no vale partir por espacios.
fn numeros_del_trazado(d: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let b: Vec<char> = d.chars().collect();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == '-' || c == '+' || c == '.' || c.is_ascii_digit() {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_digit() || b[j] == '.') {
                j += 1;
            }
            if let Ok(v) = b[i..j].iter().collect::<String>().parse::<f64>() {
                out.push(v);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn el_trazado_de_los_iconos_de_la_barra_cabe_en_el_lienzo_y_esta_entero() {
    // POR QUE EXISTE, y es el defecto que VIO EL DUEÑO: «los iconos estan incompletos o cortados, y
    // el de la bicicleta no se ve». La causa NO fue el recorte: fue que el catalogo se habia generado
    // leyendo solo los `<path>` del sprite, y **la bicicleta de Lucide son DOS RUEDAS (`<circle>`),
    // un manillar y el cuadro**. Sin las ruedas, el icono era un cuadro suelto: medido en la captura,
    // ocupaba 8 x 10 en un lienzo de 24. Arreglado el conversor
    // (`herramientas/iconos-a-slint.py`), la bicicleta mide 21 x 18.
    //
    // SE COMPRUEBAN DOS COSAS, y las dos son CONTABLES sobre el trazado (sin abrir ventana):
    //
    //   1. QUE NINGUN NUMERO PASE DEL LIENZO. El `Icono` es un `Path` con `clip: true`, viewbox 24 y
    //      `stroke-width: 2`, asi que el trazo ocupa 1 unidad a cada lado de la geometria: si un
    //      numero llega a 24, **la mitad del trazo se recorta** contra el limite del elemento. Lucide
    //      deja su contenido dentro de 2..22 justo por eso.
    //
    //      **Y SE MIRAN TODOS LOS NUMEROS, no solo los que son coordenadas**, porque separar unos de
    //      otros exige recorrer las ordenes y las de arco son un laberinto (se intento: el arco se
    //      define por radio y extremos, y hay que calcularle el centro). Con numeros ABSOLUTOS de
    //      trazado, cualquier numero por encima del lienzo es un error: una coordenada fuera, o una
    //      orden mal convertida (fue el caso de la bicicleta). En cambio un numero NEGATIVO es un
    //      DESPLAZAMIENTO (`v-8` baja 8) y no se le puede pedir que quepa: por eso solo se vigila el
    //      limite de arriba, que es donde se recorta.
    //   2. QUE EL TRAZADO ESTE ENTERO. Un icono al que le falta una pieza se delata por su TAMAÑO: se
    //      exige que recorra al menos 14 de las 24 unidades. Los cuatro de la barra recorren entre 18
    //      y 29 (casa 29 · corazon 27,5 · bicicleta 24,5 · ajustes 18), y
    //      «la bicicleta sin ruedas» recorria 10: no puede volver sin que esta prueba lo diga.
    let fuente = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/iconos.slint"))
        .expect("el catalogo de iconos tiene que leerse");
    let con_clip = fuente.contains("clip: true");
    let grosor: f64 = fuente
        .split("stroke-width:")
        .nth(1)
        .and_then(|t| t.split("px").next())
        .and_then(|t| t.trim().parse().ok())
        .expect("el catalogo declara el grosor del trazo");
    println!("  el catalogo dibuja con clip: {con_clip} y trazo de {grosor}");
    assert!(con_clip, "esta prueba mide el recorte contra el borde: se espera `clip: true`");
    let limite = 24.0 - grosor / 2.0;

    let mut comprobados = 0;
    for nombre in ["casa", "corazon", "bicicleta", "ajustes"] {
        let marca = format!("nombre == \"{nombre}\" ? \"");
        let trazado = fuente
            .split(&marca)
            .nth(1)
            .and_then(|t| t.split('"').next())
            .unwrap_or_else(|| panic!("el icono «{nombre}» no esta en el catalogo"));
        let nums = numeros_del_trazado(trazado);
        assert!(!nums.is_empty(), "el trazado de «{nombre}» no tiene numeros: {trazado}");
        let mayor = nums.iter().cloned().fold(f64::MIN, f64::max);
        let menor = nums.iter().cloned().fold(f64::MAX, f64::min);
        println!(
            "  {nombre:<14} {} numeros · mayor {mayor:.2} · menor {menor:.2}  (lienzo 24, limite {limite:.1})",
            nums.len()
        );
        assert!(
            mayor <= limite + 0.01,
            "el trazado de «{nombre}» tiene un numero en {mayor:.2}: con el trazo de {grosor} y \
             `clip: true`, el trazo se recortaria contra el borde del lienzo (limite {limite:.1})"
        );
        // El TAMAÑO del icono: la distancia entre su numero mas bajo y el mas alto. Un fragmento
        // ocupa muy poco; un icono entero, casi todo el lienzo.
        assert!(
            mayor - menor >= 14.0,
            "el trazado de «{nombre}» solo recorre {:.1} de los 24 del lienzo: le falta una pieza \
             (fue el caso de la bicicleta, que perdio las dos ruedas al convertir el sprite)",
            mayor - menor
        );
        comprobados += 1;
    }
    assert_eq!(comprobados, 4, "los CUATRO iconos de la barra, uno a uno");
}

#[test]
fn el_catalogo_dibuja_todos_los_subcaminos_del_sprite() {
    // AU-94, y esta es la COMPROBACION DE TINTA que el defecto pedia: **no basta con que el nombre
    // del icono exista**, ni con que el trazado quepa. Hay que comprobar que se dibuja TODO.
    //
    // EL DEFECTO QUE LA TRAJO, contado entero: el sprite de Lucide declara la bicicleta con
    // `<circle>` para las DOS RUEDAS, `<circle>` para el manillar y `<path>` para el cuadro. El
    // conversor leia solo los `<path>`, asi que el catalogo se quedo con **el cuadro y sin ruedas**:
    // el icono salia como un garabato de 8 x 10 (medido en la captura). Arreglado el conversor, hay
    // que impedir que vuelva a pasar con NINGUNO de los 192.
    //
    // COMO SE COMPRUEBA SIN ABRIR VENTANA: cada forma del sprite empieza con un `M` (o un `m`), asi
    // que **el numero de subcaminos es el numero de `M`**. Se cuentan en el sprite y en el catalogo,
    // icono a icono, y tienen que coincidir. Un `<circle>` perdido, un `<path>` que no se convirtio o
    // una linea que se corto al generar: todo eso baja la cuenta, y esta prueba lo dice.
    let sprite = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../norma-web/kit/iconos/sprite.svg"
    ));
    let Ok(sprite) = sprite else {
        // El sprite es de OTRO repositorio (`norma-web`). Si no esta, se dice y no se miente: la
        // prueba no puede comprobar nada.
        println!("  (el sprite de norma-web no esta en esta maquina: no se puede comparar)");
        return;
    };
    let catalogo = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/iconos.slint"))
        .expect("el catalogo de iconos tiene que leerse");

    // Cuenta los subcaminos de un trazado: cada `M`/`m` empieza uno.
    let subcaminos = |d: &str| -> usize {
        d.chars()
            .filter(|c| *c == 'M' || *c == 'm')
            .count()
    };

    let mut comparados = 0;
    let mut faltan: Vec<String> = Vec::new();
    // Se recorren los simbolos del sprite: id, y sus formas (`path` y `circle`).
    let bytes: Vec<char> = sprite.chars().collect();
    let texto: String = bytes.iter().collect();
    for trozo in texto.split("<symbol").skip(1) {
        let id = trozo
            .split("id=\"")
            .nth(1)
            .and_then(|t| t.split('"').next())
            .unwrap_or("");
        let cuerpo = trozo.split('>').nth(1).and_then(|t| t.split("</symbol>").next()).unwrap_or("");
        let mut esperados = 0;
        for p in cuerpo.split("<path").skip(1) {
            if let Some(d) = p.split("d=\"").nth(1).and_then(|t| t.split('"').next()) {
                esperados += subcaminos(d).max(1);
            }
        }
        for c in cuerpo.split("<circle").skip(1) {
            if c.contains("cx=") {
                esperados += 1;
            }
        }
        if esperados == 0 || id.is_empty() {
            continue;
        }
        // Y lo que declara el catalogo para ese nombre.
        let marca = format!("nombre == \"{id}\" ? \"");
        let Some(d) = catalogo.split(&marca).nth(1).and_then(|t| t.split('"').next()) else {
            continue; // ese icono del sprite no se convirtio: otra prueba lo mira si importa
        };
        let dibujados = subcaminos(d);
        if dibujados < esperados {
            faltan.push(format!("{id}: sprite {esperados}, catalogo {dibujados}"));
        }
        comparados += 1;
    }
    println!("  subcaminos comparados en {comparados} iconos; faltan piezas en {}", faltan.len());
    assert!(
        faltan.is_empty(),
        "estos iconos del catalogo dibujan MENOS piezas que el sprite (les falta tinta): {faltan:?}"
    );
    assert!(comparados > 100, "tienen que compararse los iconos del catalogo: {comparados}");
}

#[test]
fn los_iconos_de_la_barra_existen_todos_en_el_catalogo() {
    // POR QUE EXISTE: la barra tenia CUATRO CUADRADOS en vez de iconos, y al ponerle iconos de verdad
    // el riesgo cambia de sitio: si una seccion pide un nombre que no esta en `iconos.slint`, el
    // trazado sale VACIO y **el icono es invisible** -- y ni el compilador ni el ojo lo cazan facil.
    // Se caza cruzando los `icono:` de la ventana contra el catalogo (AU-42 y AU-54).
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/ui");
    let catalogo = std::fs::read_to_string(format!("{dir}/iconos.slint")).expect("catalogo de iconos");
    let ventana = std::fs::read_to_string(format!("{dir}/ventana.slint")).expect("ventana");

    let mut cuantos = 0;
    for trozo in ventana.split("icono: \"").skip(1) {
        let nombre = trozo.split('"').next().unwrap_or("");
        assert!(!nombre.is_empty(), "un «icono:» sin nombre en ventana.slint");
        assert!(
            catalogo.contains(&format!("\"{nombre}\"")),
            "el icono «{nombre}» de la barra NO esta en iconos.slint: saldria invisible"
        );
        cuantos += 1;
    }
    assert_eq!(cuantos, 4, "la barra tiene CUATRO secciones, cada una con su icono: {cuantos}");
    for n in ["casa", "corazon", "bicicleta", "ajustes"] {
        assert!(ventana.contains(&format!("icono: \"{n}\"")), "falta el icono {n} en la barra");
    }
}
