//! **Mide lo que da de verdad la API interna de Garmin, ANTES de construir la sincronización.**
//!
//! HITO A del bloque Garmin: el instrumento de medida del contrato (`docs/CONTRATO-garmin-sync.md`
//! §6.2), hermano de `medir-huawei`. Responde las preguntas de §5: **¿por la API interna los diarios
//! vienen llenos?** (en la exportación masiva venían vacíos), **¿el login sobrevive al MFA?** y
//! **¿cuánto tarda?**
//!
//! CONTRATO (cinco líneas), como todo instrumento de este proyecto:
//! - RECIBE: las credenciales de Garmin en DOS variables de entorno (`GARMIN_USUARIO` y
//!   `GARMIN_CLAVE`), que exporta el dueño en SU terminal. El instrumento **no las escribe, no las
//!   imprime, no las registra y no las guarda en disco**: viven en memoria lo que dura la llamada.
//! - DEVUELVE, por pantalla: **cuántos hay de cada cosa** (actividades, sueño, pulso en reposo,
//!   estrés, training status), su **rango de fechas** y el **tiempo total**. Solo números.
//! - NO HACE: no vuelca ni una línea de un diario; no escribe en disco; no toca el motor de cálculo;
//!   no abre ventana; no decide nada.
//!
//! USO:
//!   cargo run -q --bin sincronizacion -- --ayuda
//!   GARMIN_USUARIO=... GARMIN_CLAVE=... cargo run -q --bin sincronizacion -- --probar
//!
//! # ESTADO (2026-10-14) — leer esto antes de tocar nada
//!
//! **HITO B: el socket YA ESTÁ.** El cliente HTTP es `ureq` 3.4.2 (con la función `cookies`, TLS
//! `rustls`: autorizado por el dueño el 2026-10-14) y el TRANSPORTE vive en `src/sincronizacion.rs`
//! —el módulo compartido con la biblioteca del motor—, que se incluye aquí con `#[path]` para que
//! este instrumento siga siendo un fichero suelto. **El error `SIN_CLIENTE_HTTP` (código 8) ya NO
//! está en el camino de red**: `--probar` con credenciales llega al login de verdad.
//!
//! `SIN_CLIENTE_HTTP` se queda declarado a propósito, con su prueba: es el hueco que existió, y
//! mientras el flujo no se haya medido contra Garmin de verdad sigue valiendo la regla de entonces
//! —**el hueco se declara, no se rellena con un remedio peor**: no se llama a `curl` por proceso
//! hijo, porque las credenciales viajarían a otro proceso, visibles en `/proc`—. El día que la
//! primera pasada real esté hecha, ese error y `SinTransporte` se borran.

use sincronizacion::{Fallo as FalloTransporte, Transporte as ClienteHttp};

/// El módulo del TRANSPORTE HTTP (hito B), compartido con la biblioteca del motor. Se incluye por
/// ruta, y no se toca `src/lib.rs`, para que este instrumento siga siendo **un fichero suelto**.
#[path = "../sincronizacion.rs"]
mod sincronizacion;

pub use sincronizacion::Secreto;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------------------------
// Códigos de salida. Distintos a propósito: el instrumento tiene que poder guionizarse, y «algo
// falló» no sirve para decidir. 1 y 2 se dejan como en el resto de los instrumentos del proyecto.
// ---------------------------------------------------------------------------------------------
const CODIGO_USO: i32 = 2;
const CODIGO_FALTAN_VARIABLES: i32 = 3;
const CODIGO_CREDENCIALES: i32 = 4;
const CODIGO_MFA: i32 = 5;
const CODIGO_SIN_RED: i32 = 6;
const CODIGO_ENDPOINT_CAMBIADO: i32 = 7;
const CODIGO_SIN_CLIENTE_HTTP: i32 = 8;
const CODIGO_DEMASIADAS_PETICIONES: i32 = 9;

/// Cuántos días se piden a los diarios. Es una decisión del instrumento, no del contrato: **siete**
/// bastan para responder «¿el diario viene lleno o vacío?» —que es la pregunta de este hito— y no
/// disparan el anti-abuso de Garmin, que es real (§5.2 de la investigación).
const VENTANA_DIAS: u64 = 7;

/// **La lista exacta de lo que hubo que añadir a `Cargo.toml`, medida —no recordada— el 2026-10-14,
/// y ya añadido ese día** (el dueño autorizó «avanza nomás con todo lo que falte»).
///
/// Medido con `cargo add --dry-run ureq --features cookies` (resuelve contra el registro y **no
/// escribe nada**): «Adding ureq v3.4.2 to dependencies», con `Cargo.toml` intacto. Lo demás sale de
/// la ficha de cada crate en crates.io. **Se conserva aquí, entera, a propósito**: es el dato que
/// decidió qué se añadía y qué no, y el aviso de `rquest` (yanked) y de `native-tls` (OpenSSL)
/// sigue valiendo para la próxima vez que haya que decidir una dependencia de red.
const DEPENDENCIAS_CANDIDATAS: &[(&str, &str)] = &[
    (
        "ureq 3",
        "**AÑADIDO el 2026-10-14** (era el RECOMENDADO): el cliente HTTP: POST de formulario, cabeceras\n             propias, redirecciones y COOKIES (feature `cookies`, que es lo que sostiene la sesión del\n             SSO). Bloqueante, sin tokio, con TLS rustls. Es el más pequeño que sirve, y esto es un\n             instrumento de línea de comandos, no un servicio.",
    ),
    (
        "reqwest",
        "alternativa si se prefiere el cliente más conocido: `blocking` + `cookies` + `json` +\n             `rustls-tls`. Hace lo mismo que `requests` en la herramienta comunitaria —esa es su\n             ventaja— pero arrastra tokio e hyper. OJO: NO usar `native-tls`, que obligaría a\n             OpenSSL en el equipo de quien compile, y esto es un repositorio público.",
    ),
    (
        "(ninguna)",
        "para el JSON: `serde_json` YA está en `Cargo.toml`; y para el reloj, `std::time::Instant`.\n             Tampoco hacen falta `regex` (el CSRF, el ticket y las variables del MFA se extraen a\n             mano, como en el resto de los instrumentos) ni `base64` (el `Basic` del intercambio DI\n             se codifica a mano, 20 líneas, con los vectores del RFC 4648 como prueba).",
    ),
    (
        "NADA HOY",
        "el `curl_cffi impersonate=\"chrome\"` de la herramienta comunitaria (huella TLS de navegador,\n             contra el WAF) **no tiene candidato sano en Rust**: a `rquest` le están *yanked* sus 152\n             versiones en crates.io (medido), y `wreq`, su sucesor, solo publica candidatas\n             (6.0.0-rc.31). `isahc` ataría el proyecto a libcurl. Decisión: se mide con `ureq`;\n             **solo si Garmin responde 403/429** se le pide al dueño el crate de suplantación, con\n             este dato delante.",
    ),
];

/// El uso completo, con el flujo portado y lo que falta. Se imprime en `--ayuda`, que **sale 0 sin
/// tocar la red**.
fn uso() -> String {
    let p = env!("CARGO_BIN_NAME");
    let mut s = String::new();
    s.push_str(&format!(
        "{p}: mide qué trae de verdad la API interna de Garmin, sin decidir nada.\n"
    ));
    s.push_str("\nUSO:\n");
    s.push_str(&format!(
        "  {p} --ayuda            esto mismo; sale 0 SIN tocar la red.\n"
    ));
    s.push_str(&format!(
        "  {p} --probar           pide las actividades y los diarios, y cuenta lo que hay.\n"
    ));
    s.push_str(&format!(
        "  {p} --rastrear         EL MISMO flujo que --probar, pero paso a paso: por cada paso, su\n\
         \x20                       número y su nombre, la URL de destino (es pública), los NOMBRES de\n\
         \x20                       los parámetros que envía (nunca sus valores), el `content-type`,\n\
         \x20                       el estado HTTP, el tamaño del cuerpo en bytes, cuántas cookies lleva\n\
         \x20                       la jarra de sesión y si la respuesta trae `set-cookie` o `location`\n\
         \x20                       (sí/no, nunca su valor).\n\
         \x20                       NUNCA imprime usuario, clave, `_csrf`, ticket, token ni valores de\n\
         \x20                       cookies. Con credenciales falsas llega al MISMO paso y muestra el\n\
         \x20                       mismo estado que con las buenas: el 500 del backend (medido el\n\
         \x20                       2026-10-14) no depende de las credenciales, sino de la forma de la\n\
         \x20                       petición — un POST de formulario sin `content-type` no da 401.\n\
         \x20                       Faltar las variables da el mismo código 3 que `--probar`.\n"
    ));
    s.push_str("\nCREDENCIALES: se leen de las variables de entorno GARMIN_USUARIO y GARMIN_CLAVE, que\n");
    s.push_str("  exporta el dueño en SU terminal. No se imprimen, no se registran y no se guardan: no\n");
    s.push_str("  aparecen ni en la salida ni en un mensaje de error.\n");
    s.push_str(
        "\nDEVUELVE: cuántos hay de cada cosa y su rango de fechas, más el tiempo total. Nada más.\n",
    );
    s.push_str("\nCÓDIGOS DE SALIDA: 0 bien · 2 uso · 3 faltan variables · 4 credenciales rechazadas ·\n");
    s.push_str("  5 MFA requerido · 6 sin red o servidor caído · 7 endpoint cambiado (forma\n");
    s.push_str("  inesperada) · 9 demasiadas peticiones (anti-abuso)\n");
    s.push_str("  (el 8 «falta el cliente HTTP» ya NO existe en el camino de red: el transporte es\n");
    s.push_str("   `ureq` 3.4.2 con `cookies`, TLS rustls, y vive en `src/sincronizacion.rs`)\n");

    s.push_str("\nEL FLUJO QUE SE PORTA, EN ORDEN (con su fuente, que es lo que lo hace comprobable):\n");
    for (nombre, url, fuente) in flujo::ENDPOINTS {
        s.push_str(&format!("  {nombre}\n      {url}\n      fuente: {fuente}\n"));
    }

    s.push_str("\nCÓMO SE HABLA POR LA RED (hito B, ya está):\n");
    s.push_str("  · cliente: `ureq` 3.4.2 bloqueante, función `cookies` (jarra EN MEMORIA dentro del\n");
    s.push_str("    `Agent`: de la sesión no se guarda nada en disco), TLS `rustls`. Sin `native-tls`:\n");
    s.push_str("    eso pediría OpenSSL en el equipo de quien compile, y este repositorio es público.\n");
    s.push_str("  · un 4xx/5xx NO aborta: se LEE (`http_status_as_error(false)`). El flujo distingue\n");
    s.push_str("    403 (permiso o `displayName`) de 429 (anti-abuso, código 9).\n");
    s.push_str("  · NO se siguen redirecciones (`max_redirects(0)`): es lo que hacen las dos\n");
    s.push_str("    referencias portadas, y evita que un fallo de redirección tape el estado real.\n");
    s.push_str("  · esperas acotadas de 30 s por paso; sin espera global, porque la anti-WAF (3-8 s)\n");
    s.push_str("    es espera DEL FLUJO y no se le puede achacar a la red.\n");

    s.push_str("\nDEPENDENCIAS: qué hubo que añadir y para qué (ya añadido el 2026-10-14):\n");
    for (crate_, para_que) in DEPENDENCIAS_CANDIDATAS {
        s.push_str(&format!("  {crate_}: {para_que}\n"));
    }
    s
}

// =============================================================================================
// EL FLUJO PORTADO
// =============================================================================================

/// **De dónde sale este flujo, medido el 2026-10-14 (no de memoria).**
///
/// La herramienta comunitaria que cita la investigación (`docs/INVESTIGACION-garmin.md` §4) **ya no
/// lleva el login dentro**:
///
/// - `johannesh83/garmin-connect-export` (rama `master`, último empujón 2025-12-22) se apoya en la
///   librería `garminconnect`: `garmin_export.py:13` (`from garminconnect import Garmin`) y
///   `garmin_export.py:146-147` (`Garmin(username, password)` + `client.login()`).
/// - El fork mantenido, `moderation/garmin-connect-export` (rama `main`), hace lo mismo con `garth`:
///   `gcexport3.py:281` (`garth.login(USERNAME, PASSWORD)`).
/// - El único que traía el login escrito a mano es el original, `kjkjava/garmin-connect-export`
///   (`gcexport.py:90-119`), y está **obsoleto**: usa `sso.garmin.com/sso/login` (línea 91) y el
///   ticket de la cookie `CASTGC` (108-110), que es el flujo viejo.
///
/// Así que lo que se porta es **el login de la librería que las dos herramientas usan de verdad**:
/// `cyberjunky/python-garminconnect`, fichero `garminconnect/client.py` (y las rutas de datos de
/// `garminconnect/__init__.py`). Cada línea del flujo lleva al lado su cita `fichero:línea`.
///
/// Un detalle de higiene que la propia librería tiene, y que aquí se copia: sus mensajes de error de
/// URL **borran los valores de la consulta** antes de registrarlos (`client.py:196-210`,
/// `_sanitize_exception_text`). Es exactamente la regla de este hito.
mod flujo {
    use super::{Fallo, Secreto, forma_del_cuerpo};
    use serde_json::{Map, Value};

    /// `client.py:311` — `self._sso = f"https://sso.{domain}"`.
    pub const SSO: &str = "https://sso.garmin.com";
    /// `client.py:754-755` — `sso_base = {sso}/sso` y `sso_embed = {sso_base}/embed`.
    pub const URL_EMBED: &str = "https://sso.garmin.com/sso/embed";
    /// `client.py:779` — `GET {sso_base}/signin`: de aquí sale el token CSRF.
    pub const URL_SIGNIN: &str = "https://sso.garmin.com/sso/signin";
    /// `client.py:895` — `POST {sso}/sso/verifyMFA/loginEnterMfaCode`: aquí se entrega el código.
    pub const URL_MFA: &str = "https://sso.garmin.com/sso/verifyMFA/loginEnterMfaCode";
    /// `client.py:954` — `{sso}/sso/verifyMFA/mfaCode`: el «pedir otro código» del navegador.
    pub const URL_MFA_PEDIR_CODIGO: &str = "https://sso.garmin.com/sso/verifyMFA/mfaCode";
    /// `client.py:170` — `DI_TOKEN_URL`.
    pub const URL_TOKEN_DI: &str = "https://diauth.garmin.com/di-oauth2-service/oauth/token";
    /// `client.py:171-173` — `DI_GRANT_TYPE`: se envia como `grant_type` y es a su vez una URL.
    pub const TIPO_DE_CONCESION_DI: &str =
        "https://connectapi.garmin.com/di-oauth2-service/oauth/grant/service_ticket";
    /// `client.py:313` — `self._connectapi = f"https://connectapi.{domain}"`.
    pub const API: &str = "https://connectapi.garmin.com";
    /// `client.py:174-179` — `DI_CLIENT_IDS`: se prueban **en este orden** hasta que uno acepte.
    pub const CLIENTES_DI: [&str; 4] = [
        "GARMIN_CONNECT_MOBILE_ANDROID_DI_2025Q2",
        "GARMIN_CONNECT_MOBILE_ANDROID_DI_2024Q4",
        "GARMIN_CONNECT_MOBILE_ANDROID_DI",
        "GARMIN_CONNECT_MOBILE_IOS_DI",
    ];
    /// `client.py:127-128` — la espera deliberada contra el WAF, **de 3 a 8 segundos**, entre el GET
    /// de la página de acceso y el POST de las credenciales (`client.py:790-793`).
    ///
    /// OJO, DIFERENCIA MEDIDA: la investigación (§5.2) dice «30 a 45 segundos». La librería que las
    /// herramientas usan HOY usa 3-8 s. Se porta el de la librería, y la diferencia se le dice al
    /// dueño: el instrumento tarda **por diseño**, no por lentitud.
    pub const ESPERA_ANTI_WAF_S: (u64, u64) = (3, 8);
    /// `__init__.py:41` — `MAX_ACTIVITY_LIMIT = 1000`: Garmin no da más de una vez.
    pub const TOPE_ACTIVIDADES: usize = 1000;

    /// `__init__.py:734-735` — `/activitylist-service/activities/search/activities`.
    ///
    /// El FIT de cada actividad es `/download-service/files/activity/{id}` (`__init__.py:745`): ese es
    /// el que trae la serie segundo a segundo, y es trabajo del hito siguiente, no de este instrumento
    /// (que cuenta, no descarga).
    pub const RUTA_ACTIVIDADES: &str = "/activitylist-service/activities/search/activities";
    /// `__init__.py:637` — `/wellness-service/wellness/dailySleepData`.
    pub const RUTA_SUENO: &str = "/wellness-service/wellness/dailySleepData";
    /// `__init__.py:697` — `/userstats-service/wellness/daily` (pulso en reposo).
    pub const RUTA_PULSO_REPOSO: &str = "/userstats-service/wellness/daily";
    /// `__init__.py:639` — `/wellness-service/wellness/dailyStress`.
    pub const RUTA_ESTRES: &str = "/wellness-service/wellness/dailyStress";
    /// `__init__.py:715` — `/metrics-service/metrics/trainingstatus/aggregated` (el que la librería
    /// usa para el training status de una fecha, `__init__.py:2478-2482`). Existe además el diario,
    /// `/metrics-service/metrics/trainingstatus/daily` (`__init__.py:709`).
    pub const RUTA_TRAINING: &str = "/metrics-service/metrics/trainingstatus/aggregated";
    /// `__init__.py:1049` — `/userprofile-service/socialProfile`: de aquí sale `displayName`, que
    /// **va dentro de la URL** del sueño y del pulso en reposo (`__init__.py:2126` y `2189`), y sin
    /// él esas dos peticiones dan 403 (`__init__.py:1111-1126`).
    pub const RUTA_PERFIL: &str = "/userprofile-service/socialProfile";

    /// El flujo, para `--ayuda`: nombre, URL y **quién lo dice**.
    pub const ENDPOINTS: &[(&str, &str, &str)] = &[
        (
            "1 · GET página del widget (deja las cookies de sesión)",
            URL_EMBED,
            "client.py:770-775",
        ),
        (
            "2 · GET página de acceso (de aquí sale el _csrf)",
            URL_SIGNIN,
            "client.py:777-788; el patrón del campo, client.py:141",
        ),
        (
            "3 · POST credenciales (username, password, embed, _csrf)",
            URL_SIGNIN,
            "client.py:795-807; la clasificación de la respuesta, client.py:812-881",
        ),
        (
            "4 · POST del código MFA (mfa-code, embed, _csrf, fromPage)",
            URL_MFA,
            "client.py:894-905; pedir otro código, client.py:954",
        ),
        (
            "5 · POST del ticket de servicio → token DI (Bearer)",
            URL_TOKEN_DI,
            "client.py:1317-1380; la reserva por cookie, client.py:1284-1306",
        ),
        (
            "6 · GET del perfil (displayName, que va dentro de la URL)",
            API,
            "__init__.py:1049 y 1111-1126",
        ),
        (
            "7 · GET de actividades (start, limit; tope 1000)",
            API,
            "__init__.py:734-735 y 2596-2636",
        ),
        (
            "8 · GET de los diarios (sueño, pulso en reposo, estrés, training status)",
            API,
            "__init__.py:637, 697, 639 y 715",
        ),
    ];

    /// `client.py:756-760` — los parámetros de la página del widget, tal cual.
    pub fn parametros_embed() -> Vec<(String, String)> {
        vec![
            ("id".into(), "gauth-widget".into()),
            ("embedWidget".into(), "true".into()),
            ("gauthHost".into(), format!("{SSO}/sso")),
        ]
    }

    /// `client.py:761-768` — los de la página de acceso: el mismo juego, con `gauthHost` apuntando al
    /// widget y los cuatro destinos de redirección al propio widget.
    pub fn parametros_signin() -> Vec<(String, String)> {
        let embed = URL_EMBED.to_string();
        vec![
            ("id".into(), "gauth-widget".into()),
            ("embedWidget".into(), "true".into()),
            ("gauthHost".into(), embed.clone()),
            ("service".into(), embed.clone()),
            ("source".into(), embed.clone()),
            ("redirectAfterAccountLoginUrl".into(), embed.clone()),
            ("redirectAfterAccountCreationUrl".into(), embed),
        ]
    }

    /// `client.py:800-805` — el cuerpo del POST de credenciales. **Aquí va la contraseña del dueño**:
    /// este texto no se imprime nunca, y por eso el campo `cuerpo` de una `Peticion` es un `Secreto`.
    pub fn campos_del_login(
        csrf: &str,
        usuario: &Secreto,
        clave: &Secreto,
    ) -> Vec<(String, Secreto)> {
        vec![
            ("username".into(), usuario.clone()),
            ("password".into(), clave.clone()),
            ("embed".into(), Secreto::nueva("true".into())),
            ("_csrf".into(), Secreto::nueva(csrf.into())),
        ]
    }

    /// `client.py:898-903` — los campos del POST del código MFA.
    ///
    /// No los envia nadie **todavía**, y es a propósito: cuando Garmin pide el segundo factor, este
    /// instrumento **para y lo dice** (código 5) en vez de pedir un código por consola, porque quien
    /// decide si eso se teclea al vuelo o se guarda la sesión es el dueño (contrato §2). Los nombres
    /// se dicen en el aviso de MFA, para que la pieza portada quede a la vista.
    pub const CAMPOS_DEL_MFA: [&str; 4] = ["mfa-code", "embed", "_csrf", "fromPage"];

    /// `client.py:1343-1348` — el cuerpo del intercambio del ticket de servicio por el token DI.
    pub fn campos_del_token(ticket: &str, client_id: &str) -> Vec<(String, String)> {
        vec![
            ("client_id".into(), client_id.into()),
            ("service_ticket".into(), ticket.into()),
            ("grant_type".into(), TIPO_DE_CONCESION_DI.into()),
            ("service_url".into(), URL_EMBED.into()),
        ]
    }

    /// `client.py:141` — `_CSRF_RE = re.compile(r'name="_csrf"\s+value="(.+?)"')`.
    ///
    /// A mano y no con `regex`: es una forma única, y una dependencia de menos en un repositorio
    /// público es una preocupación de menos.
    pub fn extraer_csrf(html: &str) -> Result<String, Fallo> {
        let aguja = "name=\"_csrf\"";
        let Some(p) = html.find(aguja) else {
            return Err(cambiado(
                "en la página de acceso, un campo oculto `name=\"_csrf\" value=\"…\"` (client.py:141)",
                html,
            ));
        };
        let resto = &html[p + aguja.len()..];
        let Some(v) = resto.find("value=\"") else {
            return Err(cambiado(
                "tras `name=\"_csrf\"`, el atributo `value=\"…\"` (client.py:141)",
                html,
            ));
        };
        let desde = &resto[v + "value=\"".len()..];
        match desde.find('"') {
            Some(fin) if fin > 0 => Ok(desde[..fin].to_string()),
            _ => Err(cambiado(
                "un valor no vacío entre comillas en el campo `_csrf` (client.py:141)",
                html,
            )),
        }
    }

    /// `client.py:877` — `re.search(r'\?ticket=(ST-[^"&\s]+)', r.text)`: la respuesta de éxito trae el
    /// ticket de servicio, y **termina en la primera comilla, ampersand o espacio**.
    pub fn extraer_ticket(html: &str) -> Result<String, Fallo> {
        let aguja = "?ticket=";
        let Some(p) = html.find(aguja) else {
            return Err(cambiado(
                "en la respuesta de éxito, un ticket de servicio `?ticket=ST-…` (client.py:877)",
                html,
            ));
        };
        let resto = &html[p + aguja.len()..];
        let fin = resto
            .find(|c: char| c == '"' || c == '&' || c.is_whitespace())
            .unwrap_or(resto.len());
        let ticket = &resto[..fin];
        if !ticket.starts_with("ST-") || ticket.len() <= 3 {
            return Err(cambiado(
                "un ticket que empiece por `ST-` (client.py:877)",
                html,
            ));
        }
        Ok(ticket.to_string())
    }

    /// `client.py:142` — `_TITLE_RE = re.compile(r"<title>(.+?)</title>")`.
    pub fn titulo(html: &str) -> String {
        let Some(p) = html.find("<title>") else {
            return String::new();
        };
        let resto = &html[p + "<title>".len()..];
        match resto.find("</title>") {
            Some(fin) => resto[..fin].trim().to_string(),
            None => String::new(),
        }
    }

    /// `client.py:144-151` — las variables que Garmin escribe en un `<script>` de la página de MFA:
    /// `var mfaMethod = "…";`. Devuelve el valor de la que se pida.
    pub fn variable_de_mfa(html: &str, nombre: &str) -> Option<String> {
        let aguja = format!("var {nombre}");
        let mut desde = 0usize;
        while let Some(p) = html[desde..].find(&aguja) {
            let absoluto = desde + p + aguja.len();
            let resto = &html[absoluto..];
            if let Some(igual) = resto.find('=') {
                let tras = resto[igual + 1..].trim_start();
                if let Some(cita) = tras.strip_prefix('"') {
                    if let Some(fin) = cita.find('"') {
                        return Some(cita[..fin].to_string());
                    }
                }
            }
            desde = absoluto;
        }
        None
    }

    /// `client.py:856-869` — **cómo se sabe que Garmin pide MFA**.
    ///
    /// La página del MFA se titula igual que la de acceso («GARMIN Authentication Application»), así
    /// que el título solo no vale: hay que mirar si trae las variables del MFA. Esa es la regla que se
    /// porta, literal.
    pub fn metodo_de_mfa(html: &str) -> Option<String> {
        let titulo = titulo(html).to_lowercase();
        let metodo = variable_de_mfa(html, "mfaMethod").unwrap_or_default();
        let parece_mfa = titulo.contains("mfa")
            || (titulo.contains("authentication application") && !metodo.is_empty());
        if parece_mfa { Some(metodo) } else { None }
    }

    /// `client.py:812-881` — **la clasificación de la respuesta del login**, en el MISMO orden que la
    /// librería, que es lo que hace que no se confunda un MFA con una contraseña mala:
    ///
    /// 1. avería del servidor → sin red (817-827); 2. contraseña mala o cuenta bloqueada (830-836);
    /// 3. cuenta de menor, que Garmin no deja entrar por web (840-848); 4. MFA (856-869); 5. cualquier
    /// otro título que no sea «Success» → endpoint cambiado (871-874); 6. el ticket (876-879).
    pub fn clasificar_login(html: &str) -> Result<String, Fallo> {
        let t = titulo(html).to_lowercase();

        for pista in [
            "bad gateway",
            "service unavailable",
            "cloudflare",
            "502",
            "503",
        ] {
            if t.contains(pista) {
                return Err(Fallo::SinRed(format!(
                    "Garmin contestó «{}»: su servidor o su filtro, no nosotros",
                    titulo(html)
                )));
            }
        }
        for pista in ["locked", "invalid", "incorrect", "account error"] {
            if t.contains(pista) {
                return Err(Fallo::CredencialesRechazadas);
            }
        }
        if t.contains("unable to sign in") || t.contains("unable to login") {
            return Err(Fallo::CuentaRestringida);
        }
        if let Some(metodo) = metodo_de_mfa(html) {
            return Err(Fallo::MfaRequerido { metodo });
        }
        if titulo(html) != "Success" {
            return Err(Fallo::EndpointCambiado {
                esperaba:
                    "una página de título «Success» con el ticket de servicio (client.py:871-879)"
                        .into(),
                llego: format!("una página de título «{}»", titulo(html)),
            });
        }
        extraer_ticket(html)
    }

    /// `client.py:1296-1306` — la reserva: si el intercambio DI no sale, la sesión se sostiene con la
    /// cookie `JWT_WEB` que deja el consumo del ticket.
    /// El PARSEO de la cookie vive en `src/sincronizacion.rs` (hito B). Aquí se queda la
    /// clasificación, que es del flujo: **sin `JWT_WEB` no hay reserva**, y eso no es «una sesión
    /// floja»: es que el endpoint ha cambiado. Las dos cosas se prueban juntas, para que no puedan
    /// separarse sin que nadie se entere.
    pub fn cookie_jwt_web(cabeceras: &[(String, String)]) -> Result<String, Fallo> {
        super::sincronizacion::cookie_jwt_web(cabeceras).ok_or_else(|| Fallo::EndpointCambiado {
            esperaba: "la cookie `JWT_WEB` al consumir el ticket (client.py:1296-1306)".into(),
            llego: format!("{} cabeceras, ninguna con JWT_WEB", cabeceras.len()),
        })
    }

    /// `client.py:190-191` — `_build_basic_auth`: `Basic base64("cliente:")`. A mano, con los vectores
    /// del RFC 4648 como prueba: es una cabecera corta, no un secreto del cliente.
    pub fn autorizacion_basica(client_id: &str) -> String {
        format!("Basic {}", base64(&format!("{client_id}:")))
    }

    /// Base64 estándar (RFC 4648), escrito a mano para no añadir dependencia.
    pub(super) fn base64(texto: &str) -> String {
        const ALFABETO: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let bytes = texto.as_bytes();
        let mut salida = String::new();
        for trozo in bytes.chunks(3) {
            let b = [
                trozo[0],
                *trozo.get(1).unwrap_or(&0),
                *trozo.get(2).unwrap_or(&0),
            ];
            let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
            salida.push(ALFABETO[(n >> 18) as usize & 63] as char);
            salida.push(ALFABETO[(n >> 12) as usize & 63] as char);
            salida.push(if trozo.len() > 1 {
                ALFABETO[(n >> 6) as usize & 63] as char
            } else {
                '='
            });
            salida.push(if trozo.len() > 2 {
                ALFABETO[n as usize & 63] as char
            } else {
                '='
            });
        }
        salida
    }

    /// `client.py:450-473` — **las cabeceras de la sesión**, tal como la librería las manda. Hay dos
    /// caminos y el instrumento tiene que saber los dos: con token DI va un `Bearer`; sin él, la
    /// sesión se sostiene con la cookie `JWT_WEB` más `DI-Backend` y el `Origin`/`Referer` de la web.
    ///
    /// **Vive en `src/sincronizacion.rs`** (hito B): aquí solo se nombra. Una sola definición, en un
    /// solo sitio, porque la usan el flujo y el transporte —y porque el día que Garmin cambie una de
    /// estas cabeceras no puede haber dos listas que se contradigan.
    pub use super::sincronizacion::cabeceras_de_sesion;

    /// `__init__.py:2596-2636` — la lista de actividades: `start` y `limit`, con el tope de 1000
    /// (línea 41).
    pub fn consulta_actividades() -> Vec<(String, String)> {
        vec![
            ("start".into(), "0".into()),
            ("limit".into(), TOPE_ACTIVIDADES.to_string()),
        ]
    }

    /// `__init__.py:2126-2127` — el sueño se pide **por nombre de usuario dentro de la URL**, con
    /// `date` y `nonSleepBufferMinutes=60`.
    pub fn ruta_sueno(display_name: &str, fecha: &str) -> (String, Vec<(String, String)>) {
        (
            format!("{RUTA_SUENO}/{display_name}"),
            vec![
                ("date".into(), fecha.into()),
                ("nonSleepBufferMinutes".into(), "60".into()),
            ],
        )
    }

    /// `__init__.py:2186-2197` — el pulso en reposo: rango de fechas y `metricId=60`. El de rango es
    /// el mismo endpoint con `fromDate`/`untilDate` distintos (`__init__.py:2199-2212`).
    pub fn ruta_pulso_reposo(
        display_name: &str,
        desde: &str,
        hasta: &str,
    ) -> (String, Vec<(String, String)>) {
        (
            format!("{RUTA_PULSO_REPOSO}/{display_name}"),
            vec![
                ("fromDate".into(), desde.into()),
                ("untilDate".into(), hasta.into()),
                ("metricId".into(), "60".into()),
            ],
        )
    }

    /// `__init__.py:2170-2176` — el estrés va **día a día**: `/dailyStress/{fecha}`.
    pub fn ruta_estres(fecha: &str) -> (String, Vec<(String, String)>) {
        (format!("{RUTA_ESTRES}/{fecha}"), Vec::new())
    }

    /// `__init__.py:2478-2482` — el training status, también día a día.
    pub fn ruta_training(fecha: &str) -> (String, Vec<(String, String)>) {
        (format!("{RUTA_TRAINING}/{fecha}"), Vec::new())
    }

    // -----------------------------------------------------------------------------------------
    // LA LECTURA DE CADA RESPUESTA: pura, para poder probarla sin red
    // -----------------------------------------------------------------------------------------

    /// Una actividad, reducida a lo que este instrumento cuenta: **día y tipo**. Ni un dato más, y
    /// desde luego no el identificador en la salida (el contrato pide contar, no listar).
    #[derive(Debug)]
    pub struct Actividad {
        pub dia: Option<String>,
        pub tipo: String,
    }

    /// `__init__.py:2596-2636` — la respuesta de actividades es una **lista JSON**; `demo.py:1818-1820`
    /// y `1168-1174` leen de cada una `activityId`, `startTimeLocal` y `activityType.typeKey`, y
    /// `typed.py:396-408` los modela con esos nombres; `garmin_export.py:87-91` usa los tres.
    pub fn actividades(cuerpo: &str) -> Result<Vec<Actividad>, Fallo> {
        let Ok(v) = serde_json::from_str::<Value>(cuerpo) else {
            return Err(cambiado(
                "las actividades: una lista JSON (__init__.py:2617-2618)",
                cuerpo,
            ));
        };
        let Some(lista) = v.as_array() else {
            return Err(cambiado(
                "las actividades: una lista JSON (__init__.py:2617-2618)",
                cuerpo,
            ));
        };
        Ok(lista
            .iter()
            .map(|a| Actividad {
                // La fecha local, recortada al día: `garmin_export.py:29-32` usa
                // `'%Y-%m-%d %H:%M:%S'` y `demo.py:1173` corta por la `T`.
                dia: a
                    .get("startTimeLocal")
                    .and_then(|s| s.as_str())
                    .map(|s| s.chars().take(10).collect::<String>())
                    .filter(|d| d.len() == 10),
                tipo: a
                    .get("activityType")
                    .and_then(|t| t.get("typeKey"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("sin tipo")
                    .to_string(),
            })
            .collect())
    }

    /// La comprobación de forma común: **un objeto JSON**, o el endpoint ha cambiado.
    fn objeto(cuerpo: &str, que: &str, fuente: &str) -> Result<Map<String, Value>, Fallo> {
        match serde_json::from_str::<Value>(cuerpo) {
            Ok(Value::Object(o)) => Ok(o),
            _ => Err(cambiado(&format!("{que}: un objeto JSON ({fuente})"), cuerpo)),
        }
    }

    /// `__init__.py:2116-2130` — el sueño trae `dailySleepDTO`, y dentro `calendarDate` y
    /// `sleepTimeSeconds` (`typed.py:184-190`, `demo.py:1066-1070`).
    ///
    /// «Con dato» quiere decir que la noche está **medida**: hay `dailySleepDTO` y trae segundos de
    /// sueño mayores que cero. Una noche sin reloj puesto viene con el resumen a cero, y eso **no es
    /// un dato**: es lo que el contrato §2 llama un diario que falta.
    pub fn sueno_tiene_dato(cuerpo: &str) -> Result<bool, Fallo> {
        let o = objeto(cuerpo, "el sueño", "__init__.py:2116-2130 y typed.py:184-238")?;
        let Some(dto) = o.get("dailySleepDTO").and_then(|d| d.as_object()) else {
            return Err(cambiado(
                "el sueño: la clave `dailySleepDTO` (__init__.py:2126-2130; demo.py:1066)",
                cuerpo,
            ));
        };
        Ok(dto
            .get("sleepTimeSeconds")
            .and_then(|s| s.as_i64())
            .map(|s| s > 0)
            .unwrap_or(false))
    }

    /// `__init__.py:2170-2176` — el estrés de un día. La forma es la de `DailyStats`
    /// (`typed.py:92-138`): `calendarDate` y `averageStressLevel`.
    ///
    /// «Con dato» = `calendarDate` está (si no, es otra respuesta) y `averageStressLevel` es un número
    /// **positivo**. El cero y el ausente se cuentan como sin dato, y la tabla dice cuántos días caen
    /// en cada lado: si esa regla no le cuadra al dueño, se corrige **con el número delante**.
    pub fn estres_tiene_dato(cuerpo: &str) -> Result<bool, Fallo> {
        let o = objeto(cuerpo, "el estrés", "__init__.py:2170-2176 y typed.py:92-138")?;
        if !o.contains_key("calendarDate") {
            return Err(cambiado(
                "el estrés: la clave `calendarDate` (typed.py:100; __init__.py:2173)",
                cuerpo,
            ));
        }
        Ok(o.get("averageStressLevel")
            .and_then(|s| s.as_i64())
            .map(|s| s > 0)
            .unwrap_or(false))
    }

    /// `__init__.py:2478-2482` — **sin claves citables**: la librería devuelve el cuerpo tal cual y
    /// `typed.py` no modela esta respuesta. Se comprueba lo único que se puede afirmar —que sea un
    /// objeto con algo dentro— y **la primera pasada real dirá cuáles son sus claves**. Aquí no se
    /// inventa una forma.
    pub fn training_tiene_dato(cuerpo: &str) -> Result<bool, Fallo> {
        let o = objeto(
            cuerpo,
            "el training status",
            "__init__.py:2478-2482 (forma no citada: la dirá la primera pasada real)",
        )?;
        Ok(!o.is_empty())
    }

    /// `__init__.py:2216-2224` — el pulso en reposo de un rango vive en
    /// `allMetrics.metricsMap.WELLNESS_RESTING_HEART_RATE`, y cada fila trae `calendarDate` y `value`;
    /// la librería **descarta las filas sin valor**, y eso es exactamente «este día no tiene dato».
    ///
    /// Devuelve los días que SÍ traen valor.
    pub fn dias_de_pulso(cuerpo: &str) -> Result<Vec<String>, Fallo> {
        let o = objeto(cuerpo, "el pulso en reposo", "__init__.py:2199-2224")?;
        if !o.contains_key("allMetrics") {
            return Err(cambiado(
                "el pulso en reposo: la clave `allMetrics` (__init__.py:2216-2217)",
                cuerpo,
            ));
        }
        let filas = o
            .get("allMetrics")
            .and_then(|a| a.get("metricsMap"))
            .and_then(|m| m.get("WELLNESS_RESTING_HEART_RATE"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(filas
            .iter()
            .filter(|f| f.get("value").map(|v| !v.is_null()).unwrap_or(false))
            .filter_map(|f| f.get("calendarDate").and_then(|d| d.as_str()))
            .map(|d| d.to_string())
            .collect())
    }

    /// Un error de forma, con lo que se esperaba y **lo que llegó, sin el cuerpo**.
    fn cambiado(esperaba: &str, cuerpo: &str) -> Fallo {
        Fallo::EndpointCambiado {
            esperaba: esperaba.to_string(),
            llego: forma_del_cuerpo(cuerpo),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Errores TIPADOS, en español, cada uno con su código de salida
// ---------------------------------------------------------------------------------------------

#[derive(Debug)]
enum Fallo {
    /// Faltan las variables de entorno. Se dicen los NOMBRES, nunca el valor.
    FaltanVariables(Vec<&'static str>),
    /// Garmin pidió el segundo factor: el flujo lo contempla, pero este instrumento no puede seguir.
    MfaRequerido { metodo: String },
    /// La contraseña no vale, o la cuenta está bloqueada.
    CredencialesRechazadas,
    /// Cuenta de menor: Garmin no la deja entrar por web.
    CuentaRestringida,
    /// No hay red, o el servidor de Garmin contestó que no está.
    SinRed(String),
    /// El anti-abuso de Garmin dijo basta.
    DemasiadasPeticiones(String),
    /// La respuesta no tiene la forma esperada: se dice qué se esperaba y qué llegó, sin el cuerpo.
    EndpointCambiado { esperaba: String, llego: String },
    /// El único hueco que queda, y **ya no está en el camino de red**: no hay transporte. Se conserva
    /// con su prueba porque la regla que lo trajo sigue valiendo: el hueco se declara, no se rellena
    /// con un remedio peor. El camino de red de hoy es `Fallo::DelTransporte`.
    SinClienteHttp,
    /// **Lo que devuelve el TRANSPORTE (hito B) cuando no puede hablar**: se conserva tal cual —con
    /// su código— para no perder de qué se trataba. Trae ya el «al pedir …» puesto.
    DelTransporte(FalloTransporte),
}

impl From<FalloTransporte> for Fallo {
    fn from(f: FalloTransporte) -> Self {
        Fallo::DelTransporte(f)
    }
}

impl Fallo {
    /// Añade **por dónde iba** a un fallo de red. No toca los demás: un error de credenciales o de
    /// forma ya dice lo suyo, y no necesita la URL.
    fn al_pedir(self, destino: &str) -> Fallo {
        match self {
            Fallo::SinRed(porque) => Fallo::SinRed(format!("{porque} (al pedir {destino})")),
            otro => otro,
        }
    }

    fn codigo(&self) -> i32 {
        match self {
            Fallo::FaltanVariables(_) => CODIGO_FALTAN_VARIABLES,
            Fallo::MfaRequerido { .. } => CODIGO_MFA,
            Fallo::CredencialesRechazadas | Fallo::CuentaRestringida => CODIGO_CREDENCIALES,
            Fallo::SinRed(_) => CODIGO_SIN_RED,
            Fallo::DemasiadasPeticiones(_) => CODIGO_DEMASIADAS_PETICIONES,
            Fallo::EndpointCambiado { .. } => CODIGO_ENDPOINT_CAMBIADO,
            Fallo::SinClienteHttp => CODIGO_SIN_CLIENTE_HTTP,
            // El código lo pone el transporte: es el mismo que el de aquí, y así no hay dos tablas.
            Fallo::DelTransporte(f) => f.codigo(),
        }
    }

    fn mensaje(&self) -> String {
        match self {
            Fallo::FaltanVariables(vars) => format!(
                "Faltan las variables de entorno: {}.\n  Las exporta el dueño en SU terminal; este \
                 instrumento no las escribe, ni las imprime, ni las guarda.",
                vars.join(" y ")
            ),
            Fallo::MfaRequerido { metodo } => format!(
                "Garmin pide el segundo factor (MFA{}): el login por sí solo NO puede seguir.\n  El \
                 flujo SÍ lo contempla: el código se entrega en {}\n  (client.py:894-905), con el \
                 cuerpo {:?} (client.py:898-903), y si hay que pedir otro, en {}\n  (client.py:954).\n  Lo que hay que \
                 decidir —y es del dueño, no del instrumento— es si el código se teclea al vuelo o si \
                 se guarda la sesión para no repetirlo: la librería puede devolver «necesito MFA» sin \
                 consumirlo (__init__.py:526-527 y client.py:927-934), y el contrato §2 ya dice que lo que se \
                 guarda es la SESIÓN, nunca la contraseña.",
                if metodo.is_empty() {
                    String::new()
                } else {
                    format!(" por «{metodo}»")
                },
                flujo::URL_MFA,
                flujo::CAMPOS_DEL_MFA,
                flujo::URL_MFA_PEDIR_CODIGO
            ),
            Fallo::CredencialesRechazadas => "Garmin rechazó el usuario o la clave (o la cuenta está \
                 bloqueada). No se muestra lo que se envió: son credenciales."
                .to_string(),
            Fallo::CuentaRestringida => "Garmin no deja entrar por web a esta cuenta (suele ser una \
                 cuenta de menor, gestionada por un adulto)."
                .to_string(),
            Fallo::SinRed(porque) => format!("No se pudo hablar con Garmin: {porque}."),
            Fallo::DemasiadasPeticiones(porque) => format!(
                "Garmin cortó por demasiadas peticiones (anti-abuso): {porque}.\n  Es lo que avisa la \
                 investigación §5.2: hay esperas deliberadas, y hay que ir despacio."
            ),
            Fallo::EndpointCambiado { esperaba, llego } => format!(
                "El endpoint ha cambiado: la respuesta no tiene la forma esperada.\n  Se esperaba: \
                 {esperaba}\n  Llegó: {llego}\n  (El cuerpo NO se vuelca: puede traer datos de salud.)"
            ),
            Fallo::SinClienteHttp => {
                let mut s = String::from(
                    "No hay cliente HTTP: el flujo está portado y probado, pero falta el socket.\n  \
                     ESTE CAMINO YA NO SE USA (hito B: el transporte es `ureq` 3.4.2, ver \
                     `src/sincronizacion.rs`).\n  Se conserva porque la regla que lo trajo sigue \
                     valiendo: el hueco se declara, no se rellena con un remedio peor. Lo que hubo \
                     que añadir fue:\n",
                );
                for (crate_, para_que) in DEPENDENCIAS_CANDIDATAS {
                    s.push_str(&format!("    · {crate_}: {para_que}\n"));
                }
                s
            }
            Fallo::DelTransporte(f) => {
                // El transporte ya dice lo suyo; aquí solo se le añade por dónde iba el flujo. Lo
                // que NO se le añade es nada del cuerpo ni de las cabeceras: no las tiene.
                format!("{}", f.mensaje())
            }
        }
    }
}

/// **Qué llegó, dicho sin volcarlo.** Ni un valor ni un texto: el TIPO de la respuesta, su tamaño y
/// —como mucho— los nombres de sus claves, que son el contrato de la API y no el dato de nadie. Es lo
/// que hace falta para distinguir «cambió el endpoint» de «cambió el dato».
fn forma_del_cuerpo(cuerpo: &str) -> String {
    let bytes = cuerpo.len();
    match serde_json::from_str::<serde_json::Value>(cuerpo) {
        Ok(serde_json::Value::Object(o)) => {
            let mut claves: Vec<&str> = o.keys().map(|k| k.as_str()).collect();
            claves.sort_unstable();
            format!(
                "un objeto JSON con {} claves ({}); {bytes} bytes",
                claves.len(),
                claves.iter().take(8).copied().collect::<Vec<_>>().join(", ")
            )
        }
        Ok(serde_json::Value::Array(a)) => {
            format!("una lista JSON de {} elementos; {bytes} bytes", a.len())
        }
        Ok(otro) => format!(
            "un JSON de tipo {}; {bytes} bytes",
            match otro {
                serde_json::Value::Null => "nulo",
                serde_json::Value::Bool(_) => "booleano",
                serde_json::Value::Number(_) => "número",
                serde_json::Value::String(_) => "texto",
                _ => "desconocido",
            }
        ),
        Err(_) => format!("{bytes} bytes que NO son JSON"),
    }
}

// ---------------------------------------------------------------------------------------------
// QUIÉN HABLA POR LA RED (hito B: el transporte ya está)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Metodo {
    Get,
    Post,
}

/// Una petición, ya montada. El cuerpo es un `Secreto` **porque el del login lleva la contraseña**, y
/// el `Debug` está escrito a mano para dejar claro qué se puede imprimir y qué no.
///
/// `Clone` lo necesita el transporte FALSO de las pruebas, que guarda lo que le pidieron para poder
/// afirmar después qué se envió. Copiar una petición copia un `Secreto`: no lo expone.
#[derive(Clone)]
struct Peticion {
    metodo: Metodo,
    url: String,
    consulta: Vec<(String, String)>,
    cabeceras: Vec<(String, String)>,
    cuerpo: Option<Secreto>,
    /// **Los NOMBRES de los campos del formulario, en orden.** Nunca los valores: uno es la
    /// contraseña y otro el `_csrf`. Se guardan aparte del cuerpo —que va codificado y en un
    /// `Secreto`— porque el rastreo tiene que poder decir **qué** se manda sin poder decir **cuánto
    /// vale**.
    campos: Vec<String>,
}

impl fmt::Debug for Peticion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Peticion")
            .field(
                "metodo",
                &match self.metodo {
                    Metodo::Get => "GET",
                    Metodo::Post => "POST",
                },
            )
            // La consulta se imprime **sin valores** (ver `descripcion_publica`): uno de ellos es el
            // ticket de servicio, que es una credencial de un solo uso.
            .field("destino", &self.descripcion_publica())
            // Las cabeceras, **sin valores tampoco**: una de ellas es `Authorization`. Se dice el
            // nombre y el esquema, que es lo único que puede estar en un mensaje.
            .field("cabeceras", &self.cabeceras_publicas())
            .field("cuerpo", &self.cuerpo)
            .finish()
    }
}

impl Peticion {
    fn get(url: &str, consulta: Vec<(String, String)>) -> Self {
        Peticion {
            metodo: Metodo::Get,
            url: url.to_string(),
            consulta,
            cabeceras: Vec::new(),
            cuerpo: None,
            campos: Vec::new(),
        }
    }

    fn post_form(url: &str, campos: &[(String, Secreto)]) -> Self {
        Peticion {
            metodo: Metodo::Post,
            url: url.to_string(),
            consulta: Vec::new(),
            cabeceras: vec![(
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            )],
            cuerpo: Some(Secreto::nueva(codificar_formulario(campos))),
            campos: campos.iter().map(|(k, _)| k.clone()).collect(),
        }
    }

    /// **CAMBIA la lista de cabeceras entera; no añade.** Es lo que quiere un paso que dice de un
    /// tirón con qué sale (el del token DI), pero es una trampa para el que solo quiere añadir una:
    /// el POST del acceso lo usó para poner el `Referer` y **se llevó por delante el `Content-Type`
    /// del formulario** que había puesto `post_form` —el `Content-Type` es una cabecera MÁS de esa
    /// lista, y `ureq`, al enviar un `String`, no añade ninguna (`AsSendBody for String` deja
    /// `content_type: None`, `send_body.rs:309-314`)—. Medido contra Garmin el 2026-10-14: un POST de
    /// formulario sin `Content-Type` no da 401 (clave mala) sino **500**, un error del backend.
    fn con_cabeceras(mut self, cabeceras: Vec<(String, String)>) -> Self {
        self.cabeceras = cabeceras;
        self
    }

    /// **AÑADE cabeceras a las que ya hay**, sin tocar las que la petición puso (p. ej. el
    /// `Content-Type` que `post_form` dejó). Es la forma correcta de sumar el `Referer` del acceso:
    /// reemplazar la lista se llevó el `Content-Type` y Garmin respondió 500 (medido 2026-10-14,
    /// rastreo-informe.md).
    fn mas_cabeceras(mut self, cabeceras: Vec<(String, String)>) -> Self {
        self.cabeceras.extend(cabeceras);
        self
    }

    /// Los nombres de lo que se envia: los de la consulta (GET) o los del formulario (POST).
    fn nombres_de_lo_que_envia(&self) -> Vec<String> {
        if self.campos.is_empty() {
            self.consulta.iter().map(|(k, _)| k.clone()).collect()
        } else {
            self.campos.clone()
        }
    }

    /// La petición dicha **para un mensaje**: la URL y los NOMBRES de los parámetros, sin sus valores.
    ///
    /// Es la regla que la propia librería aplica a sus errores (`client.py:196-210`,
    /// `_sanitize_exception_text`), y aquí no es un adorno: uno de esos valores es el ticket de
    /// servicio, que es una credencial de un solo uso. Un valor de consulta **no se escribe**.
    ///
    /// El texto lo construye el módulo del transporte ([`Destino`]): **una sola manera de decirlo**,
    /// en un solo sitio, porque aquí también lo usan los mensajes de error de la red.
    fn destino(&self) -> sincronizacion::Destino {
        sincronizacion::Destino::de(&self.url, self.consulta.iter().map(|(k, _)| k.clone()))
    }

    fn descripcion_publica(&self) -> String {
        self.destino().to_string()
    }

    /// Las cabeceras **dichas sin sus valores**: el nombre, y el esquema cuando es `Authorization`.
    /// Nació de un fallo medido: un `Debug` de la lista de cabeceras soltaba el `Bearer` entero.
    fn cabeceras_publicas(&self) -> Vec<(String, String)> {
        self.cabeceras
            .iter()
            .map(|(n, v)| {
                let dicho = if n.eq_ignore_ascii_case("authorization") {
                    sincronizacion::esquema_de_autorizacion(v)
                } else {
                    "…".to_string()
                };
                (n.clone(), dicho)
            })
            .collect()
    }
}

/// Una respuesta. Su `Debug` está escrito a mano para que **no pueda volcar el cuerpo** ni por
/// descuido (una respuesta de Garmin trae datos de salud del cliente).
struct Respuesta {
    estado: u16,
    cabeceras: Vec<(String, String)>,
    cuerpo: String,
}

impl fmt::Debug for Respuesta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Respuesta {{ estado: {}, {} cabeceras, cuerpo: {} }}",
            self.estado,
            self.cabeceras.len(),
            forma_del_cuerpo(&self.cuerpo)
        )
    }
}

impl Respuesta {
    /// La respuesta del TRANSPORTE, traducida a la de este instrumento. El cuerpo pasa de `Secreto` a
    /// `String` **aquí y solo aquí**: es el único punto donde el flujo puede leerlo, y es a propósito
    /// —así el tipo que viaja por el flujo entero no se puede imprimir.
    fn del_transporte(r: sincronizacion::Respuesta) -> Self {
        Respuesta {
            estado: r.estado,
            cabeceras: r.cabeceras,
            cuerpo: r.cuerpo.expuesto().to_string(),
        }
    }
}

/// **El único punto por donde se habla por la red.** La implementación de verdad es `Transporte`
/// (abajo, con `ureq`); las pruebas inyectan una falsa, así que **el flujo entero se puede probar sin
/// salir a internet y sin una credencial real**.
///
/// Los tres métodos de abajo tienen implementación por defecto porque **el flujo no los necesita
/// para funcionar**: solo los usa el rastreo (`--rastrear`). Un transporte que no sepa contestarlos
/// —el falso de las pruebas, por ejemplo— sigue sirviendo para todo lo demás, que es lo que evita
/// tener dos flujos: uno con rastreo y otro sin él.
trait Transporte {
    fn pedir(&self, peticion: &Peticion) -> Result<Respuesta, Fallo>;

    /// **El nombre del paso que va a pedir.** Lo dice el propio flujo, que es quien sabe por dónde
    /// va; el rastreo solo lo escribe. Un transporte que no rastree lo ignora.
    fn nombra(&self, _paso: &str) {}

    /// Cuántas cookies lleva la sesión en memoria AHORA (justo después del paso).
    fn cookies_en_la_sesion(&self) -> usize {
        0
    }

    /// **Los nombres de las cabeceras con las que salió la última petición, y su `Content-Type`.**
    /// Lo mide el cliente HTTP por dentro; un transporte que no lo sepa devuelve vacío, y el rastreo
    /// lo dice como «sin dato» en vez de inventarse una lista.
    fn salida(&self) -> sincronizacion::Salida {
        sincronizacion::Salida::default()
    }

    /// El `displayName` del cliente, en cuanto el flujo lo aprende. **No es un adorno**: ese nombre
    /// va DENTRO de la URL del sueño y del pulso en reposo (`__init__.py:2126` y `2189`), y el
    /// rastreo imprime URLs: sin esto, un informe con la traza se llevaría el nombre del cliente al
    /// repositorio.
    fn vio_el_nombre_del_cliente(&self, _nombre: &str) {}
}

/// **El transporte de verdad: `ureq` bloqueante con jarra de cookies en memoria.** Es una cáscara
/// fina sobre `sincronizacion::Transporte` (el módulo compartido con la biblioteca del motor): aquí
/// solo se traduce el `Peticion` del flujo a una llamada del cliente, y su respuesta al `Respuesta`
/// del instrumento. **Ni una línea del flujo cambia por esto**, que era el trato del hito A.
///
/// El `Option` es la RED DE SEGURIDAD, y se conserva del hito A: si algún día el cliente no se puede
/// construir, el instrumento **no se cae y no se queda mudo** —devuelve el error tipado del hueco
/// (`SIN_CLIENTE_HTTP`, código 8) diciendo que no hay cliente HTTP—. Es la misma regla que trajo el
/// código 8: el hueco se declara, no se rellena con un remedio peor.
struct Cliente(Option<ClienteHttp>);

impl Cliente {
    fn nuevo() -> Self {
        Cliente(Some(ClienteHttp::nuevo()))
    }

    /// Cuánto tardó el último paso de RED. Es lo que permite separar el tiempo de la red del tiempo
    /// de la espera anti-WAF, que es espera del flujo y no se le puede achacar a Garmin.
    fn tardanza_de_la_red(&self) -> Option<Duration> {
        self.0.as_ref().and_then(|c| c.ultima_tardanza())
    }

    /// ¿Falta el cliente HTTP? Es lo que `main` pregunta antes de medir: con el hueco, el flujo
    /// pararía en el primer paso de red, así que se dice **antes** y con su código.
    fn sin_cliente_http(&self) -> bool {
        self.0.is_none()
    }
}

impl Transporte for Cliente {
    fn pedir(&self, peticion: &Peticion) -> Result<Respuesta, Fallo> {
        let Some(cliente) = &self.0 else {
            return Err(Fallo::SinClienteHttp);
        };
        let r = match peticion.metodo {
            Metodo::Get => cliente.get(&peticion.url, &peticion.consulta, &peticion.cabeceras)?,
            Metodo::Post => {
                // El cuerpo de un POST ya viene codificado (y es un `Secreto`). Se envia tal cual.
                let cuerpo = peticion
                    .cuerpo
                    .clone()
                    .unwrap_or_else(|| Secreto::nueva(String::new()));
                cliente.post_form(&peticion.url, &cuerpo, &peticion.cabeceras)?
            }
        };
        Ok(Respuesta::del_transporte(r))
    }

    /// Cuántas cookies lleva la sesión EN MEMORIA. Solo para poder decirlo: no se guardan en disco.
    fn cookies_en_la_sesion(&self) -> usize {
        match &self.0 {
            Some(c) => c.cuantas_cookies(),
            None => 0,
        }
    }

    fn salida(&self) -> sincronizacion::Salida {
        match &self.0 {
            Some(c) => c.salida(),
            None => sincronizacion::Salida::default(),
        }
    }
}

/// **El hueco, conservado a propósito, y usado SOLO por las pruebas.** No es el transporte del camino
/// de red (lo es `Cliente` con `ureq`): el instrumento se queda con el fallo tipado que devuelve
/// `Cliente::pedir`, no con este. Así la regla que lo trajo sigue viva y probada —**el hueco se
/// declara, no se rellena con un remedio peor**— sin que quede código muerto al lado del camino bueno.
///
/// `#[cfg(test)]` por eso mismo: medido al limpiar los avisos (2026-10-14), fuera de las pruebas
/// nadie lo construía.
#[cfg(test)]
struct SinTransporte;

#[cfg(test)]
impl Transporte for SinTransporte {
    fn pedir(&self, _peticion: &Peticion) -> Result<Respuesta, Fallo> {
        Err(Fallo::SinClienteHttp)
    }
}

/// Pide al transporte y, si no se pudo, **añade por dónde iba**: la URL y los nombres de los
/// parámetros, nunca sus valores (`client.py:196-210`).
fn pedir(t: &dyn Transporte, peticion: &Peticion) -> Result<Respuesta, Fallo> {
    t.pedir(peticion).map_err(|fallo| match fallo {
        // El fallo del transporte ya dice «al pedir …» con sus propias palabras; el de aquí (el
        // hueco) es el único que necesita que se le añada el destino.
        Fallo::DelTransporte(f) => {
            Fallo::DelTransporte(con_destino(f, &peticion.descripcion_publica()))
        }
        otro => otro.al_pedir(&peticion.descripcion_publica()),
    })
}

/// Añade «al pedir …» a un fallo del transporte. **Nunca al revés**: el destino va SIN valores.
fn con_destino(f: FalloTransporte, destino: &str) -> FalloTransporte {
    match f {
        FalloTransporte::SinRed(porque) => {
            FalloTransporte::SinRed(format!("{porque} (al pedir {destino})"))
        }
        // Un estado no necesita que se le diga la URL: el número y el destino ya son el mensaje, y
        // así no se alarga. Se deja tal cual, que es lo que dice el transporte.
        otro => otro,
    }
}

/// `client.py:800-805`: el POST va como formulario, así que los valores van **codificados**. Se hace a
/// mano (espacio → `+`, lo demás percent-encoded) porque una contraseña con `&`, `+`, `=` o `%` sin
/// codificar **rompería el formulario en silencio** y parecería «clave mala».
fn codificar_formulario(campos: &[(String, Secreto)]) -> String {
    campos
        .iter()
        .map(|(k, v)| format!("{}={}", codificar(k), codificar(v.expuesto())))
        .collect::<Vec<_>>()
        .join("&")
}

/// Percent-encoding de formulario, como lo hacen `urlencode` y `requests`.
fn codificar(valor: &str) -> String {
    let mut s = String::with_capacity(valor.len());
    for b in valor.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                s.push(b as char)
            }
            b' ' => s.push('+'),
            otro => s.push_str(&format!("%{otro:02X}")),
        }
    }
    s
}

// =============================================================================================
// EL RASTREO: EL MISMO FLUJO, PASO A PASO
// =============================================================================================

/// **Las cabeceras con las que la librería portada saca ESTE POST. MEDIDAS, no leídas.**
///
/// El 2026-10-14 se levantó un servidor de eco en `127.0.0.1` y se le hizo, con la librería de verdad
/// (`cyberjunky/python-garminconnect`), **la misma llamada del paso del acceso** —`client.py:797-810`:
/// `sess.post(f"{sso_base}/signin", params=signin_params, headers={"Referer": r.url}, data={"username",
/// "password", "embed", "_csrf"}, timeout=30)`— y se apuntaron las cabeceras que llegaron. Esa
/// estrategia del widget usa `curl_cffi.Session(impersonate="chrome")` (`client.py:751`), no
/// `requests`: por eso la lista trae la huella de un navegador.
///
/// Lo que llegó (nombres, en orden alfabético): `accept`, `accept-encoding`, `accept-language`,
/// `connection`, `content-length`, `content-type`, `host`, `referer`, `sec-ch-ua`,
/// `sec-ch-ua-mobile`, `sec-ch-ua-platform`, `sec-fetch-dest`, `sec-fetch-mode`, `sec-fetch-site`,
/// `sec-fetch-user`, `upgrade`, `upgrade-insecure-requests`, `user-agent`. Y de `content-type`:
/// `application/x-www-form-urlencoded`. Los campos del cuerpo fueron `_csrf`, `embed`, `password` y
/// `username`: **los mismos que manda este instrumento**.
///
/// Se dejan FUERA tres que no son una diferencia entre los dos, y se dice por qué:
/// - `host`: lo pone cualquier cliente (también el nuestro).
/// - `cookie`: las dos partes mandan la jarra que tengan; se dice aparte, con el número de cookies.
/// - `upgrade` y `http2-settings`: son un artefacto de haber medido contra un servidor **en claro**
///   (curl_cffi intenta el `h2c`); contra Garmin, que es HTTPS, no van.
const CABECERAS_DE_LA_LIBRERIA_EN_EL_ACCESO: &[&str] = &[
    "accept",
    "accept-encoding",
    "accept-language",
    "connection",
    "content-length",
    "content-type",
    "referer",
    "sec-ch-ua",
    "sec-ch-ua-mobile",
    "sec-ch-ua-platform",
    "sec-fetch-dest",
    "sec-fetch-mode",
    "sec-fetch-site",
    "sec-fetch-user",
    "upgrade-insecure-requests",
    "user-agent",
];

/// **Las que añade NUESTRO cliente solo, después de que el middleware mire la petición.** Medido en
/// la fuente de `ureq` 3.4.2 (`src/run.rs:272-350`, `add_headers`): `Cookie` —solo si la jarra tiene
/// algo, y por eso se dice aparte con el número de cookies—, `Accept-Encoding`, `Content-Length`
/// —solo si el cuerpo tiene longitud conocida, que es el caso de un formulario—, `User-Agent`
/// (`ureq/3.4.2` por defecto) y `Accept: */*`. Ninguna se añade si la petición ya la trae.
const CABECERAS_QUE_ANADE_EL_CLIENTE: &[&str] =
    &["accept", "accept-encoding", "content-length", "user-agent"];

/// Compara dos listas de NOMBRES de cabecera (nunca valores). Pura: los mismos nombres dan el mismo
/// resultado. Devuelve `(solo_de_ella, solo_nuestras, en_las_dos)`, en minúsculas y ordenadas.
fn comparar_cabeceras(
    nuestras: &[String],
    suyas: &[&str],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let normalizar = |lista: &[String]| -> Vec<String> {
        let mut v: Vec<String> = lista.iter().map(|n| n.to_ascii_lowercase()).collect();
        v.sort();
        v.dedup();
        v
    };
    let nuestras = normalizar(nuestras);
    let mut suyas: Vec<String> = suyas.iter().map(|n| n.to_ascii_lowercase()).collect();
    suyas.sort();
    suyas.dedup();

    let solo_ella = suyas
        .iter()
        .filter(|n| !nuestras.contains(n))
        .cloned()
        .collect();
    let solo_nosotras = nuestras
        .iter()
        .filter(|n| !suyas.contains(n))
        .cloned()
        .collect();
    let en_las_dos = nuestras
        .iter()
        .filter(|n| suyas.contains(n))
        .cloned()
        .collect();
    (solo_ella, solo_nosotras, en_las_dos)
}

/// **EL RASTREO: el mismo flujo que `--probar`, pero contando cada paso.**
///
/// Es un transporte que envuelve al de verdad: el flujo no cambia ni una línea (sigue pidiendo a
/// `&dyn Transporte`), y lo único que añade es que **dice lo que sale y lo que vuelve**. Nada de esto
/// es adorno: el 500 medido contra Garmin el 2026-10-14 no se podía explicar leyendo el código —todo
/// el mundo daba por hecho que el `Content-Type` del formulario iba puesto— y sí se explica mirando
/// lo que de verdad sale por el cable.
///
/// **Lo que NUNCA escribe**: el usuario, la clave, el `_csrf`, el ticket, el token, los valores de
/// las cookies y los valores de cualquier cabecera o parámetro. Escribe nombres, números y estados
/// ([`Peticion::nombres_de_lo_que_envia`] y [`sincronizacion::Salida`] están hechos para eso), más el
/// `Content-Type`, que es una constante del protocolo y no un dato del cliente.
struct Rastro<'a> {
    interior: &'a dyn Transporte,
    /// Cuántas peticiones se han hecho: es el «paso N» de la tabla.
    hechos: Cell<usize>,
    /// El nombre del paso en curso, que pone el propio flujo con `nombra`.
    paso: RefCell<String>,
    /// El `displayName` del cliente, en cuanto el flujo lo aprende: va DENTRO de la URL del sueño y
    /// del pulso en reposo (`__init__.py:2126` y `2189`), así que la URL se imprime con él sustituido
    /// por `{displayName}`. Un informe de este repositorio no se lleva el nombre del cliente.
    nombre_del_cliente: RefCell<Option<String>>,
    /// **Dónde va el rastro.** `None`: a la pantalla, que es lo que hace el instrumento. `Some`: a un
    /// texto, que es como lo miran las pruebas —afirmar sobre un `String` es afirmar de verdad;
    /// afirmar sobre lo que ya salió por `print!`, no—.
    recogido: Option<RefCell<String>>,
}

impl<'a> Rastro<'a> {
    fn nuevo(interior: &'a dyn Transporte) -> Self {
        Rastro {
            interior,
            hechos: Cell::new(0),
            paso: RefCell::new(String::new()),
            nombre_del_cliente: RefCell::new(None),
            recogido: None,
        }
    }

    /// El rastro que se queda en un texto en vez de salir por la pantalla: es el que usan las
    /// pruebas, y prueba lo mismo (el texto se construye con el mismo código).
    #[cfg(test)]
    fn recogiendo(interior: &'a dyn Transporte) -> Self {
        Rastro {
            interior,
            hechos: Cell::new(0),
            paso: RefCell::new(String::new()),
            nombre_del_cliente: RefCell::new(None),
            recogido: Some(RefCell::new(String::new())),
        }
    }

    /// Lo que el rastro ha dicho hasta ahora (solo en las pruebas).
    #[cfg(test)]
    fn texto(&self) -> String {
        self.recogido
            .as_ref()
            .map(|s| s.borrow().clone())
            .unwrap_or_default()
    }

    /// Escribe una línea del rastro donde toque.
    fn dice(&self, texto: &str) {
        match &self.recogido {
            Some(s) => s.borrow_mut().push_str(texto),
            None => print!("{texto}"),
        }
    }

    /// La URL dicha para un informe: sin el nombre del cliente (si ya se sabe) y **sin los valores de
    /// la consulta** —el `Display` de `Destino` solo dice nombres, y uno de esos valores es el ticket
    /// de servicio—.
    fn url_publica(&self, url: &str) -> String {
        match self.nombre_del_cliente.borrow().as_ref() {
            Some(n) if !n.is_empty() => url.replace(n.as_str(), "{displayName}"),
            _ => url.to_string(),
        }
    }

    /// ¿Es la petición del ACCESO? Se reconoce por el método y la URL, no por el nombre del paso: el
    /// nombre puede cambiar y esto no.
    fn es_el_acceso(&self, peticion: &Peticion) -> bool {
        matches!(peticion.metodo, Metodo::Post) && peticion.url == flujo::URL_SIGNIN
    }

    /// La línea de un paso. Todo lo que dice es público: destino, NOMBRES, número de cookies, estado
    /// y tamaño.
    fn bloque(&self, peticion: &Peticion, respuesta: Option<&Respuesta>) -> String {
        let salida = self.interior.salida();
        let metodo = match peticion.metodo {
            Metodo::Get => "GET",
            Metodo::Post => "POST",
        };
        let envia = peticion.nombres_de_lo_que_envia();
        let mut s = String::new();
        s.push_str(&format!(
            "\n── paso {} · {}\n",
            self.hechos.get(),
            self.paso.borrow()
        ));
        s.push_str(&format!(
            "   destino         {metodo} {}\n",
            self.url_publica(&peticion.url)
        ));
        s.push_str(&format!(
            "   envia           {}\n",
            if envia.is_empty() {
                "ningún parámetro".to_string()
            } else {
                format!("{} (los NOMBRES, nunca los valores)", envia.join(", "))
            }
        ));
        s.push_str(&format!(
            "   content-type    {}\n",
            match &salida.content_type {
                Some(ct) => ct.clone(),
                None => "(NINGUNO: no lo lleva la petición, y el cliente no lo añade solo)".to_string(),
            }
        ));
        s.push_str(&format!(
            "   cabeceras       {}\n",
            if !salida.medido {
                "(sin dato: este transporte no las mide)".to_string()
            } else if salida.nombres.is_empty() {
                "(ninguna: la petición va sin cabeceras propias)".to_string()
            } else {
                salida.nombres.join(", ")
            }
        ));
        // Las que añade el cliente por su cuenta (`ureq` `run.rs:272-350`), dichas aparte para no
        // mezclar lo que manda el flujo con lo que manda la librería de red.
        let solo_del_cliente: Vec<&str> = CABECERAS_QUE_ANADE_EL_CLIENTE
            .iter()
            .copied()
            .filter(|n| !salida.nombres.iter().any(|m| m == n))
            .collect();
        if !solo_del_cliente.is_empty() {
            s.push_str(&format!(
                "   + del cliente   {} (los añade ureq: run.rs:272-350)\n",
                solo_del_cliente.join(", ")
            ));
        }
        s.push_str(&format!(
            "   cookies         {} en la jarra de la sesión (en memoria)\n",
            self.interior.cookies_en_la_sesion()
        ));
        match respuesta {
            Some(r) => s.push_str(&format!(
                "   respuesta       {} · {} bytes (ya descomprimido) · set-cookie: {} · location: {}\n",
                r.estado,
                r.cuerpo.len(),
                si_no(tiene_cabecera(&r.cabeceras, "set-cookie")),
                si_no(tiene_cabecera(&r.cabeceras, "location")),
            )),
            None => s.push_str("   respuesta       sin respuesta: no llegó a haberla\n"),
        }
        s
    }

    /// **El DIAGNÓSTICO, pegado al POST que falla.** Compara las cabeceras que salieron de verdad con
    /// las que manda la librería portada y dice, con nombres, cuál falta. Es lo que convierte «Garmin
    /// contesta 500» en «falta el `Content-Type`».
    fn diagnostico(&self) -> String {
        diagnostico_del_acceso(
            &self.interior.salida(),
            self.interior.cookies_en_la_sesion(),
        )
    }
}

/// **La comparación, como función pura**: lo que salió de verdad contra lo que manda la librería
/// portada en el POST del acceso. Pura a propósito —los mismos datos dan el mismo texto, y se prueba
/// sin red y sin credenciales—.
///
/// Compara **lo que sale de verdad por el cable**, no lo que pone la petición: a lo que el flujo puso
/// se le suman las cabeceras que añade el cliente solo (`ureq` `run.rs:272-350`). Sin eso, la
/// comparación diría que no mandamos `accept` ni `user-agent` cuando sí los mandamos, y una
/// comparación que exagera la diferencia no sirve para decidir.
fn diagnostico_del_acceso(salida: &sincronizacion::Salida, cookies_en_la_jarra: usize) -> String {
    let mut s = String::new();
    s.push_str("\n   DIAGNÓSTICO · contra la librería portada (garminconnect/client.py:795-810):\n");
    if !salida.medido {
        s.push_str(
            "      (sin dato: este transporte no mide qué cabeceras salen; el de verdad sí, con el\n\
             \x20       `middleware` de `ureq`)\n",
        );
        s.push_str(
            "      lo que ella manda en este POST, medido con un servidor de eco: accept,\n\
             \x20       accept-encoding, accept-language, connection, content-length, content-type,\n\
             \x20       referer, sec-ch-ua, sec-ch-ua-mobile, sec-ch-ua-platform, sec-fetch-dest,\n\
             \x20       sec-fetch-mode, sec-fetch-site, sec-fetch-user, upgrade-insecure-requests,\n\
             \x20       user-agent\n",
        );
        return s;
    }
    let mut nuestras = salida.nombres.clone();
    for anadida in CABECERAS_QUE_ANADE_EL_CLIENTE {
        if !nuestras.iter().any(|n| n == anadida) {
            nuestras.push(anadida.to_string());
        }
    }
    let (solo_ella, solo_nosotras, en_las_dos) =
        comparar_cabeceras(&nuestras, CABECERAS_DE_LA_LIBRERIA_EN_EL_ACCESO);
    s.push_str(&format!(
        "      ella manda y NOSOTROS NO: {}\n",
        si_hay(&solo_ella)
    ));
    s.push_str(&format!(
        "      nosotros mandamos y ELLA NO: {}\n",
        si_hay(&solo_nosotras)
    ));
    s.push_str(&format!("      coincidimos en: {}\n", si_hay(&en_las_dos)));
    s.push_str(&format!(
        "      la `cookie` no entra en la comparación: las dos mandan su jarra (nosotros, \
         {cookies_en_la_jarra} en memoria)\n"
    ));
    if salida.content_type.is_none() {
        s.push_str(
            "      → **el que explica el 500 es `content-type`**: el POST del acceso lleva el cuerpo\n\
             \x20       de un formulario y sale SIN declararlo. El paso reemplaza la lista de\n\
             \x20       cabeceras y se lleva por delante el `application/x-www-form-urlencoded` que\n\
             \x20       pone `post_form`, y `ureq`, al enviar un texto, no añade ninguna\n\
             \x20       (`send_body.rs:309-314`). El backend no lo lee como formulario y revienta con\n\
             \x20       un 500 JSON (`java.lang.StringIndexOutOfBoundsException`): el 500 NO es de las\n\
             \x20       credenciales.\n\
             \x20       Lo demás que falta es la huella de navegador de `curl_cffi`, no un requisito\n\
             \x20       del formulario: el `user-agent` coincide en nombre y no en valor (ella va como\n\
             \x20       un Chrome; nosotros, como la librería HTTP).\n",
        );
    } else {
        s.push_str(
            "      → el `content-type` SÍ sale en esta corrida: la hipótesis del formulario sin\n\
             \x20       declarar no aplica; lo que falte de la lista de arriba es lo que hay que mirar.\n",
        );
    }
    s
}

/// Una lista de nombres para un mensaje: los nombres, o «nada».
fn si_hay(nombres: &[String]) -> String {
    if nombres.is_empty() {
        "nada".to_string()
    } else {
        nombres.join(", ")
    }
}

/// Sí o no, en español y sin adornos. Nunca el valor: solo si está.
fn si_no(hay: bool) -> &'static str {
    if hay {
        "sí"
    } else {
        "no"
    }
}

/// ¿La respuesta trae esa cabecera? Se dice la PRESENCIA, jamás el valor: un `set-cookie` es una
/// credencial de sesión y un `location` puede llevar parámetros.
fn tiene_cabecera(cabeceras: &[(String, String)], nombre: &str) -> bool {
    cabeceras.iter().any(|(n, _)| n.eq_ignore_ascii_case(nombre))
}

impl Transporte for Rastro<'_> {
    fn pedir(&self, peticion: &Peticion) -> Result<Respuesta, Fallo> {
        let resultado = self.interior.pedir(peticion);
        self.hechos.set(self.hechos.get() + 1);
        let dicho = self.bloque(peticion, resultado.as_ref().ok());
        self.dice(&dicho);
        if self.es_el_acceso(peticion) {
            let falla = match &resultado {
                Ok(r) => !(200..300).contains(&r.estado),
                Err(_) => true,
            };
            if falla {
                self.dice(&self.diagnostico());
            }
        }
        resultado
    }

    fn nombra(&self, paso: &str) {
        *self.paso.borrow_mut() = paso.to_string();
        self.interior.nombra(paso);
    }

    fn cookies_en_la_sesion(&self) -> usize {
        self.interior.cookies_en_la_sesion()
    }

    fn salida(&self) -> sincronizacion::Salida {
        self.interior.salida()
    }

    fn vio_el_nombre_del_cliente(&self, nombre: &str) {
        *self.nombre_del_cliente.borrow_mut() = Some(nombre.to_string());
        self.interior.vio_el_nombre_del_cliente(nombre);
    }
}

// =============================================================================================
// EL FLUJO, PASO A PASO
// =============================================================================================

/// La sesión que se sostiene después del login: o el token DI, o la cookie `JWT_WEB`
/// (`client.py:450-473`).
struct Sesion {
    token_di: Option<String>,
    jwt_web: Option<String>,
    display_name: Option<String>,
}

/// `client.py:770-775` — la primera visita, que deja las cookies de sesión.
fn paso_embed(t: &dyn Transporte) -> Result<(), Fallo> {
    t.nombra("el widget: la primera visita, que deja las cookies de sesión (client.py:770-775)");
    let r = pedir(t, &Peticion::get(flujo::URL_EMBED, flujo::parametros_embed()))?;
    exigir_estado(&r, "la página de entrada del widget (client.py:770-775)")
}

/// `client.py:777-788` — la página de acceso, de donde sale el `_csrf`.
fn paso_csrf(t: &dyn Transporte) -> Result<String, Fallo> {
    t.nombra("la página de acceso: de aquí sale el `_csrf` (client.py:777-788)");
    let r = pedir(t, &Peticion::get(flujo::URL_SIGNIN, flujo::parametros_signin()))?;
    exigir_estado(&r, "la página de acceso (client.py:777-788)")?;
    flujo::extraer_csrf(&r.cuerpo)
}

/// `client.py:790-881` — la espera contra el WAF, el POST de las credenciales y la clasificación de
/// lo que conteste.
fn paso_credenciales(
    t: &dyn Transporte,
    csrf: &str,
    usuario: &Secreto,
    clave: &Secreto,
) -> Result<String, Fallo> {
    t.nombra("EL ACCESO: el POST de las credenciales, el paso que responde 500 (client.py:790-881)");
    espera_anti_waf();
    let peticion =
        Peticion::post_form(flujo::URL_SIGNIN, &flujo::campos_del_login(csrf, usuario, clave))
            .mas_cabeceras(vec![(
                "Referer".to_string(),
                flujo::URL_EMBED.to_string(),
            )]);
    let r = pedir(t, &peticion)?;
    exigir_estado(&r, "el acceso (client.py:795-810)")?;
    flujo::clasificar_login(&r.cuerpo)
}

/// `client.py:1317-1380` — el ticket de servicio se cambia por el token DI, probando los
/// identificadores de cliente **en orden**. Si ninguno acepta, se usa la reserva de
/// `client.py:1284-1306`: se consume el ticket y se toma la cookie `JWT_WEB` de esa respuesta.
fn paso_sesion(t: &dyn Transporte, ticket: &str) -> Result<Sesion, Fallo> {
    for (i, cliente) in flujo::CLIENTES_DI.iter().enumerate() {
        let cliente: &str = cliente;
        t.nombra(&format!(
            "el ticket → token DI, identificador {} de {} (client.py:1317-1380)",
            i + 1,
            flujo::CLIENTES_DI.len()
        ));
        let campos: Vec<(String, Secreto)> = flujo::campos_del_token(ticket, cliente)
            .into_iter()
            .map(|(k, v)| (k, Secreto::nueva(v)))
            .collect();
        let peticion = Peticion::post_form(flujo::URL_TOKEN_DI, &campos).con_cabeceras(vec![
            (
                "Authorization".to_string(),
                flujo::autorizacion_basica(cliente),
            ),
            (
                "Accept".to_string(),
                "application/json,text/html;q=0.9,*/*;q=0.8".to_string(),
            ),
            ("Cache-Control".to_string(), "no-cache".to_string()),
        ]);
        let Ok(r) = pedir(t, &peticion) else { continue };
        if r.estado == 429 {
            return Err(Fallo::DemasiadasPeticiones(
                "al cambiar el ticket por el token (client.py:1351-1354)".to_string(),
            ));
        }
        if !(200..300).contains(&r.estado) {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&r.cuerpo) {
            if let Some(token) = v.get("access_token").and_then(|t| t.as_str()) {
                return Ok(Sesion {
                    token_di: Some(token.to_string()),
                    jwt_web: None,
                    display_name: None,
                });
            }
        }
    }

    // La reserva: se consume el ticket contra la misma página del widget y se mira la cookie.
    t.nombra("el ticket → la cookie `JWT_WEB`: la reserva (client.py:1284-1306)");
    let r = pedir(
        t,
        &Peticion::get(
            flujo::URL_EMBED,
            vec![("ticket".to_string(), ticket.to_string())],
        ),
    )?;
    Ok(Sesion {
        token_di: None,
        jwt_web: Some(flujo::cookie_jwt_web(&r.cabeceras)?),
        display_name: None,
    })
}

/// `__init__.py:1049` — el `displayName` del perfil: **va dentro de la URL** del sueño y del pulso en
/// reposo, y sin él esas dos peticiones dan 403 (`__init__.py:1111-1126`).
fn paso_perfil(t: &dyn Transporte, sesion: &mut Sesion) -> Result<(), Fallo> {
    t.nombra("el perfil: de aquí sale el `displayName` (__init__.py:1049)");
    let r = pedir_api(t, sesion, flujo::RUTA_PERFIL, Vec::new())?;
    let o = match serde_json::from_str::<serde_json::Value>(&r.cuerpo) {
        Ok(serde_json::Value::Object(o)) => o,
        _ => {
            return Err(Fallo::EndpointCambiado {
                esperaba: "el perfil: un objeto JSON con `displayName` (__init__.py:1049)".into(),
                llego: forma_del_cuerpo(&r.cuerpo),
            })
        }
    };
    match o.get("displayName").and_then(|d| d.as_str()) {
        Some(d) if !d.is_empty() => {
            sesion.display_name = Some(d.to_string());
            // `displayName` va DENTRO de la URL del sueño y del pulso: el rastreo tiene que saberlo
            // para no imprimirlo. El flujo no lo necesita para nada más.
            t.vio_el_nombre_del_cliente(d);
            Ok(())
        }
        // `__init__.py:1114-1116`: un `displayName` ausente se interpola como «None» en la URL y da
        // 403. Por eso, si no viene, se dice — no se sigue con una URL inventada.
        _ => Err(Fallo::EndpointCambiado {
            esperaba: "el perfil: la clave `displayName` con valor (__init__.py:1111-1126)".into(),
            llego: forma_del_cuerpo(&r.cuerpo),
        }),
    }
}

/// Una llamada a la API con las cabeceras de la sesión.
fn pedir_api(
    t: &dyn Transporte,
    sesion: &Sesion,
    ruta: &str,
    consulta: Vec<(String, String)>,
) -> Result<Respuesta, Fallo> {
    let peticion = Peticion::get(&format!("{}{}", flujo::API, ruta), consulta).con_cabeceras(
        flujo::cabeceras_de_sesion(sesion.token_di.as_deref(), sesion.jwt_web.as_deref()),
    );
    let r = pedir(t, &peticion)?;
    exigir_estado(&r, "una llamada a la API")?;
    Ok(r)
}

/// Un estado que no es 2xx ni 429 se dice tal cual: el número y la forma, nunca el cuerpo.
fn exigir_estado(r: &Respuesta, que: &str) -> Result<(), Fallo> {
    if r.estado == 429 {
        return Err(Fallo::DemasiadasPeticiones(format!("en {que}")));
    }
    if !(200..300).contains(&r.estado) {
        return Err(Fallo::SinRed(format!(
            "{que} respondió {} ({})",
            r.estado,
            forma_del_cuerpo(&r.cuerpo)
        )));
    }
    Ok(())
}

/// `client.py:127-128` y `790-793` — la espera deliberada contra el WAF. No es lentitud: es el flujo.
/// El valor sale del reloj del sistema para no añadir el crate `rand` por esto.
///
/// **En las pruebas NO se espera**: la espera es parte del flujo contra Garmin, y `cargo test` no
/// tiene por qué tardar 3-8 s por cada escenario. Lo que se prueba es el flujo, no el reloj.
fn espera_anti_waf() {
    // Se lee SIEMPRE, aunque en las pruebas no se espere: es la constante que el flujo de verdad usa
    // —y la que la ayuda cita—, y una constante que nadie lee es una constante que miente.
    let (min, max) = flujo::ESPERA_ANTI_WAF_S;
    #[cfg(test)]
    let _ = (min, max);
    #[cfg(not(test))]
    {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(0);
        std::thread::sleep(Duration::from_secs(min + nanos % (max - min + 1)));
    }
}

// ---------------------------------------------------------------------------------------------
// El recuento: lo ÚNICO que este instrumento devuelve
// ---------------------------------------------------------------------------------------------

/// Lo que se ha visto de una cosa: cuántos días se pidieron, cuántos traían dato, qué días, y —si es
/// una lista con tipos— cuántos de cada tipo. **Un día pedido y vacío NO es un día con dato**: es
/// exactamente la pregunta del contrato §5.1 (en la exportación masiva venían vacíos).
#[derive(Default)]
struct Ficha {
    nombre: &'static str,
    pedidos: usize,
    con_dato: usize,
    dias: Vec<String>,
    por_tipo: BTreeMap<String, usize>,
}

impl Ficha {
    fn nueva(nombre: &'static str) -> Self {
        Ficha {
            nombre,
            ..Ficha::default()
        }
    }

    /// Un registro con dato, sin contar peticiones (para una lista que llega de una vez).
    fn anota(&mut self, dia: &str) {
        self.con_dato += 1;
        self.dias.push(dia.to_string());
    }

    /// Un día pedido, con dato o sin él.
    fn pide(&mut self, dia: &str, hay_dato: bool) {
        self.pedidos += 1;
        if hay_dato {
            self.anota(dia);
        }
    }

    fn rango(&self) -> Option<(String, String)> {
        let min = self.dias.iter().min()?;
        let max = self.dias.iter().max()?;
        Some((min.clone(), max.clone()))
    }

    /// Una línea de la tabla. Números y el rango. Cuando no hay nada, se dice «sin dato»: **no se
    /// dibuja un cero ni un rango vacío**, que es la lección de `medir-huawei`.
    fn linea(&self, ancho: usize) -> String {
        let mut s = format!("  {:<ancho$} ", self.nombre);
        if self.con_dato == 0 {
            s.push_str("sin dato");
            if self.pedidos > 0 {
                s.push_str(&format!(" ({} pedidos, ninguno trajo nada)", self.pedidos));
            }
        } else {
            s.push_str(&format!("{} con dato", self.con_dato));
            if self.pedidos > 0 {
                s.push_str(&format!(" de {} pedidos", self.pedidos));
            }
            if let Some((a, b)) = self.rango() {
                s.push_str(&format!(" · rango {a} .. {b}"));
            }
        }
        s.push('\n');
        if !self.por_tipo.is_empty() {
            let mut tipos: Vec<(&String, &usize)> = self.por_tipo.iter().collect();
            tipos.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            let muestra: Vec<String> = tipos
                .iter()
                .take(6)
                .map(|(t, n)| format!("{t} {n}"))
                .collect();
            s.push_str(&format!(
                "  {:<ancho$}   por tipo: {}\n",
                "",
                muestra.join(" · ")
            ));
        }
        s
    }
}

/// El resumen: la tabla y el reloj. Puro: los mismos números dan el mismo texto (como en
/// `medir-huawei`).
fn resumen(fichas: &[Ficha], tiempo: Duration) -> String {
    let mut s = String::new();
    s.push_str("--- LO QUE TRAE DE VERDAD LA API INTERNA DE GARMIN ---\n");
    for f in fichas {
        s.push_str(&f.linea(28));
    }
    s.push_str(&format!("  {:<28} {}\n", "tiempo total", segundos(tiempo)));
    s
}

/// El tiempo, con una decimal y coma, como se escribe en español.
fn segundos(t: Duration) -> String {
    format!("{:.1} s", t.as_secs_f64()).replace('.', ",")
}

// =============================================================================================
// LA MEDICIÓN
// =============================================================================================

/// **Lo que se mide, en orden**: perfil, actividades y los cuatro diarios. Devuelve la tabla con el
/// tiempo total. El transporte es el que le pasen: el de verdad (`--probar`), el rastreo
/// (`--rastrear`, que es el mismo envuelto) o el falso de las pruebas.
fn medir(t: &dyn Transporte, usuario: &Secreto, clave: &Secreto) -> Result<String, Fallo> {
    let inicio = Instant::now();

    paso_embed(t)?;
    let csrf = paso_csrf(t)?;
    let ticket = paso_credenciales(t, &csrf, usuario, clave)?;
    let mut sesion = paso_sesion(t, &ticket)?;
    paso_perfil(t, &mut sesion)?;
    let nombre = sesion.display_name.clone().unwrap_or_default();

    let mut fichas: Vec<Ficha> = Vec::new();

    // --- Actividades: de una vez, hasta el tope que deja Garmin (`__init__.py:2596-2636`) ---
    t.nombra("las actividades, de una vez (__init__.py:2596-2636)");
    let r = pedir_api(
        t,
        &sesion,
        flujo::RUTA_ACTIVIDADES,
        flujo::consulta_actividades(),
    )?;
    let lista = flujo::actividades(&r.cuerpo)?;
    let mut actividades = Ficha::nueva("actividades");
    for a in &lista {
        match &a.dia {
            // Una actividad sin fecha se cuenta como vista y sin dato: no se le inventa un día.
            Some(d) => actividades.anota(d),
            None => actividades.pedidos += 1,
        }
        *actividades.por_tipo.entry(a.tipo.clone()).or_insert(0) += 1;
    }
    // Si vino el tope, hay más: se dice, no se calla (el tope lo pone Garmin, `__init__.py:41`).
    if lista.len() >= flujo::TOPE_ACTIVIDADES {
        actividades
            .por_tipo
            .insert("(vino el tope de 1000: hay más)".to_string(), 0);
    }
    fichas.push(actividades);

    let hoy = hoy();

    // --- Sueño: día a día, que es como se pregunta «¿esta noche trae dato?» ---
    let mut sueno = Ficha::nueva("sueño");
    for i in 0..VENTANA_DIAS {
        let dia = hace_dias(i);
        let (ruta, consulta) = flujo::ruta_sueno(&nombre, &dia);
        t.nombra(&format!("el sueño del día {dia} (__init__.py:637)"));
        let r = pedir_api(t, &sesion, &ruta, consulta)?;
        sueno.pide(&dia, flujo::sueno_tiene_dato(&r.cuerpo)?);
    }
    fichas.push(sueno);

    // --- Pulso en reposo: el rango entero en UNA petición (`__init__.py:2199-2224`) ---
    let desde = hace_dias(VENTANA_DIAS - 1);
    let (ruta, consulta) = flujo::ruta_pulso_reposo(&nombre, &desde, &hoy);
    t.nombra("el pulso en reposo: el rango entero en una petición (__init__.py:2199-2224)");
    let r = pedir_api(t, &sesion, &ruta, consulta)?;
    let dias = flujo::dias_de_pulso(&r.cuerpo)?;
    let mut pulso = Ficha::nueva("pulso en reposo");
    pulso.pedidos = VENTANA_DIAS as usize;
    for d in &dias {
        pulso.anota(d);
    }
    fichas.push(pulso);

    // --- Estrés y training status: día a día (`__init__.py:2170-2176` y `2478-2482`) ---
    let mut estres = Ficha::nueva("estrés");
    let mut training = Ficha::nueva("training status");
    for i in 0..VENTANA_DIAS {
        let dia = hace_dias(i);

        let (ruta, consulta) = flujo::ruta_estres(&dia);
        t.nombra(&format!("el estrés del día {dia} (__init__.py:2170-2176)"));
        let r = pedir_api(t, &sesion, &ruta, consulta)?;
        estres.pide(&dia, flujo::estres_tiene_dato(&r.cuerpo)?);

        let (ruta, consulta) = flujo::ruta_training(&dia);
        t.nombra(&format!(
            "el training status del día {dia} (__init__.py:2478-2482)"
        ));
        let r = pedir_api(t, &sesion, &ruta, consulta)?;
        training.pide(&dia, flujo::training_tiene_dato(&r.cuerpo)?);
    }
    fichas.push(estres);
    fichas.push(training);

    Ok(resumen(&fichas, inicio.elapsed()))
}

/// La fecha de hoy, en `AAAA-MM-DD`, preguntándosela al sistema (`date`): un calendario propio es una
/// fuente de errores que aquí no hace falta.
fn hoy() -> String {
    fecha_del_sistema(&[])
}

/// Hace `n` días, en `AAAA-MM-DD`.
fn hace_dias(n: u64) -> String {
    let cuando = format!("{n} days ago");
    fecha_del_sistema(&["-d", &cuando])
}

fn fecha_del_sistema(extra: &[&str]) -> String {
    match std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .args(extra)
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        // Si no hubiera `date` (no debería), se devuelve vacío: la petición saldría con la fecha vacía
        // y el error tipado diría que la forma no es la esperada. **No se inventa una fecha.**
        _ => String::new(),
    }
}

// =============================================================================================
// PUERTA DE ENTRADA
// =============================================================================================

/// **Las credenciales, leídas del entorno.** El lector se pasa como parámetro para que las pruebas
/// puedan inyectar valores FALSOS **sin tocar el entorno real**: una prueba nunca debe poder leer la
/// clave de verdad del dueño, ni por casualidad.
fn leer_credenciales(
    leer: impl Fn(&str) -> Option<String>,
) -> Result<(Secreto, Secreto), Fallo> {
    let mut faltan: Vec<&'static str> = Vec::new();
    let mut valores: Vec<Secreto> = Vec::new();
    for nombre in ["GARMIN_USUARIO", "GARMIN_CLAVE"] {
        // Se recortan SOLO los saltos de línea finales: al pegar en la terminal se cuelan, y una
        // contraseña con un `\n` detrás falla como si fuera mala. Los espacios NO se tocan: pueden
        // formar parte de la clave.
        let valor = leer(nombre)
            .map(|v| v.trim_end_matches(['\n', '\r']).to_string())
            .map(Secreto::nueva)
            .filter(|s| !s.esta_vacio());
        match valor {
            Some(v) => valores.push(v),
            None => faltan.push(nombre),
        }
    }
    if !faltan.is_empty() {
        return Err(Fallo::FaltanVariables(faltan));
    }
    let clave = valores.pop().expect("están las dos");
    let usuario = valores.pop().expect("están las dos");
    Ok((usuario, clave))
}

fn main() {
    let argumentos: Vec<String> = std::env::args().collect();

    if argumentos
        .iter()
        .any(|a| a == "--ayuda" || a == "-h" || a == "--help")
    {
        // Sale 0 y NO toca la red: es lo que permite comprobarlo sin credenciales.
        print!("{}", uso());
        return;
    }

    // `--rastrear` es **el mismo modo**, con el transporte envuelto en el rastreo: mismo camino hasta
    // aquí (así que faltar variables da el mismo código 3) y mismo flujo después.
    let rastrear = argumentos.iter().any(|a| a == "--rastrear");

    if !argumentos.iter().any(|a| a == "--probar") && !rastrear {
        eprintln!(
            "Uso: {} --ayuda | --probar | --rastrear",
            env!("CARGO_BIN_NAME")
        );
        std::process::exit(CODIGO_USO);
    }

    let (usuario, clave) = match leer_credenciales(|n| std::env::var(n).ok()) {
        Ok(par) => par,
        Err(fallo) => {
            eprintln!("{}", fallo.mensaje());
            std::process::exit(fallo.codigo());
        }
    };

    // **HITO B: el transporte de verdad.** `ureq` bloqueante con la jarra de cookies en memoria
    // (feature `cookies`, autorizada por el dueño el 2026-10-14). Aquí se acaba el código 8: el
    // flujo llega al login de Garmin, y lo que devuelva será un error tipado de VERDAD
    // (credenciales rechazadas, MFA, sin red o anti-abuso), no el hueco de antes.
    let transporte = Cliente::nuevo();

    // La red de seguridad, que es lo único que queda del hito A: si no hubiera cliente HTTP, se dice
    // ANTES de tocar la red y con el código 8. Hoy no pasa (el cliente está), y por eso el código 8
    // ya no puede aparecer en una corrida normal.
    if transporte.sin_cliente_http() {
        let fallo = Fallo::SinClienteHttp;
        eprintln!("{}", fallo.mensaje());
        std::process::exit(fallo.codigo());
    }

    if rastrear {
        println!("--- RASTREO PASO A PASO (el mismo flujo que --probar) ---");
        println!(
            "por cada paso: destino, NOMBRES de lo que envía, content-type, estado, bytes y cookies."
        );
        println!("no se imprime NINGÚN valor: ni usuario, ni clave, ni `_csrf`, ni ticket, ni token.");
        let rastro = Rastro::nuevo(&transporte);
        let resultado = medir(&rastro, &usuario, &clave);
        match resultado {
            Ok(texto) => {
                print!("{texto}");
                eprintln!(
                    "sesión: {} cookies en memoria · último paso de red: {}",
                    transporte.cookies_en_la_sesion(),
                    match transporte.tardanza_de_la_red() {
                        Some(t) => segundos(t),
                        None => "sin dato".to_string(),
                    }
                );
            }
            Err(fallo) => {
                eprintln!("{}", fallo.mensaje());
                std::process::exit(fallo.codigo());
            }
        }
        return;
    }

    match medir(&transporte, &usuario, &clave) {
        Ok(texto) => {
            print!("{texto}");
            // Se dice, sin dar el valor: cuántas cookies sostienen la sesión y cuánto de ese tiempo
            // fue RED (el resto es la espera anti-WAF del flujo, que es por diseño).
            eprintln!(
                "sesión: {} cookies en memoria · último paso de red: {}",
                transporte.cookies_en_la_sesion(),
                match transporte.tardanza_de_la_red() {
                    Some(t) => segundos(t),
                    None => "sin dato".to_string(),
                }
            );
        }
        Err(fallo) => {
            eprintln!("{}", fallo.mensaje());
            std::process::exit(fallo.codigo());
        }
    }
}

// =============================================================================================
// UN TRANSPORTE FALSO, PARA PROBAR EL FLUJO ENTERO SIN RED Y SIN CREDENCIALES
// =============================================================================================

/// **El transporte que usan las pruebas: contesta lo que se le diga y no sale a internet.**
///
/// Existe por una razón concreta: el flujo de red NO se puede probar con las credenciales del dueño
/// (las pone él en SU terminal), así que si no hubiera un transporte falso, el flujo entero quedaría
/// sin probar. Con esto se comprueba, sin red: que el flujo llega hasta el final, que cada respuesta
/// se lee como toca, y —lo que más importa— que **por el camino no se imprime ni una credencial**.
///
/// Guarda lo que le pidieron ([`Peticion`]) para poder afirmarlo después.
#[cfg(test)]
struct TransporteFalso {
    rutas: BTreeMap<(String, String), Respuesta>,
    pedidas: std::cell::RefCell<Vec<Peticion>>,
    /// **Lo que el rastro dirá que salió.** El transporte de verdad lo MIDE (el middleware de
    /// `ureq`); el falso lo copia de aquí, para poder probar la comparación de cabeceras sin red.
    salida: sincronizacion::Salida,
}

#[cfg(test)]
impl TransporteFalso {
    fn new() -> Self {
        TransporteFalso {
            rutas: BTreeMap::new(),
            pedidas: std::cell::RefCell::new(Vec::new()),
            salida: sincronizacion::Salida::default(),
        }
    }

    /// Lo que este transporte dirá que envió. Los nombres, sin valores.
    fn salida(mut self, salida: sincronizacion::Salida) -> Self {
        self.salida = salida;
        self
    }

    /// Registra una respuesta de GET. La ruta se compara **sin la consulta**: es lo que identifica al
    /// endpoint, y los valores de la consulta son justo lo que no se quiere mirar.
    fn get(mut self, url: &str, estado: u16, cuerpo: &str) -> Self {
        self.rutas.insert(
            ("GET".to_string(), url.to_string()),
            Respuesta {
                estado,
                cabeceras: Vec::new(),
                cuerpo: cuerpo.to_string(),
            },
        );
        self
    }

    fn post(mut self, url: &str, estado: u16, cuerpo: &str, cabeceras: Vec<(String, String)>) -> Self {
        self.rutas.insert(
            ("POST".to_string(), url.to_string()),
            Respuesta {
                estado,
                cabeceras,
                cuerpo: cuerpo.to_string(),
            },
        );
        self
    }

    /// Un GET cuya respuesta trae un `Set-Cookie`: es el caso de la primera visita al widget (que
    /// deja las cookies de la sesión del SSO) y el de la reserva, que lee `JWT_WEB`.
    fn get_con_cookie(mut self, url: &str, estado: u16, cuerpo: &str, set_cookie: &str) -> Self {
        self.rutas.insert(
            ("GET".to_string(), url.to_string()),
            Respuesta {
                estado,
                cabeceras: vec![("Set-Cookie".to_string(), set_cookie.to_string())],
                cuerpo: cuerpo.to_string(),
            },
        );
        self
    }

    /// Todas las peticiones que se hicieron, en orden. Es lo que permite afirmar qué se envió.
    fn pedidas(&self) -> Vec<Peticion> {
        self.pedidas.borrow().clone()
    }

    /// Cuántas veces se pidió algo cuya URL contenga `trozo`.
    fn veces_pedido(&self, trozo: &str) -> usize {
        self.pedidas().iter().filter(|p| p.url.contains(trozo)).count()
    }

    /// Las cabeceras de la primera petición cuya URL contenga `trozo`.
    fn cabeceras_de(&self, trozo: &str) -> Vec<(String, String)> {
        for p in self.pedidas() {
            let sin_consulta = p.url.split('?').next().unwrap_or(&p.url).to_string();
            if p.url.contains(trozo) || sin_consulta.contains(trozo) {
                return p.cabeceras.clone();
            }
        }
        panic!("no se pidió nada con «{trozo}»");
    }
}

#[cfg(test)]
impl Transporte for TransporteFalso {
    fn pedir(&self, peticion: &Peticion) -> Result<Respuesta, Fallo> {
        self.pedidas.borrow_mut().push(peticion.clone());
        let metodo = match peticion.metodo {
            Metodo::Get => "GET",
            Metodo::Post => "POST",
        };
        // Cuatro maneras de reconocer la ruta, en este orden: la URL entera, la URL sin la consulta,
        // la URL construida desde la RUTA de la API (que es como se registran ellas: `flujo::RUTA_*`
        // es solo el camino) y la ruta sola. El orden importa: la primera que exista manda.
        let sin_consulta = peticion.url.split('?').next().unwrap_or(&peticion.url).to_string();
        let ruta = sin_consulta
            .split_once("://")
            .map(|(_, resto)| resto)
            .and_then(|resto| resto.find('/').map(|i| &resto[i..]))
            .unwrap_or("");
        let desde_la_ruta = format!("{}{}", flujo::API, ruta);
        for clave in [
            peticion.url.as_str(),
            sin_consulta.as_str(),
            desde_la_ruta.as_str(),
            ruta,
        ] {
            if let Some(r) = self.rutas.get(&(metodo.to_string(), clave.to_string())) {
                return Ok(Respuesta {
                    estado: r.estado,
                    cabeceras: r.cabeceras.clone(),
                    cuerpo: r.cuerpo.clone(),
                });
            }
        }
        // Una ruta que la prueba no previó NO se inventa: se dice, y así una prueba que se deja un
        // paso se cae en vez de pasar por casualidad.
        Err(Fallo::SinRed(format!(
            "la prueba no previó {metodo} {}",
            sin_consulta
        )))
    }

    fn salida(&self) -> sincronizacion::Salida {
        self.salida.clone()
    }
}

/// Una respuesta de la API con datos de salud **reconocibles**: si alguno aparece en un mensaje, es
/// que se volcó un cuerpo.
#[cfg(test)]
const SALUD_FALSA: &str = "DATO-DE-SALUD-QUE-NO-DEBE-SALIR";

/// La página de acceso, con su `_csrf`.
#[cfg(test)]
const HTML_ACCESO: &str = r#"<html><title>GARMIN Authentication Application</title>
<form><input type="hidden" name="_csrf" value="CSRF-DE-PRUEBA"></form></html>"#;

/// La respuesta de éxito del login: el ticket de servicio, que es una credencial de un solo uso.
#[cfg(test)]
const HTML_EXITO: &str = r#"<html><title>Success</title>
<a href="https://connect.garmin.com/modern/?ticket=ST-123456-FALSO&x=1">seguir</a></html>"#;

/// El perfil: de aquí sale `displayName`, que va dentro de la URL del sueño y del pulso.
#[cfg(test)]
const JSON_PERFIL: &str = r#"{"displayName":"NombreFalsoDePrueba","otro":"x"}"#;

/// Una respuesta de la API con `<clave>: SALUD_FALSA`, para poder buscar el dato falso.
#[cfg(test)]
fn json_con_salud(extra: &str) -> String {
    format!(r#"{{"marcador":"{SALUD_FALSA}"{extra}}}"#)
}

/// Las rutas de la API que el flujo pide SIEMPRE (perfil, actividades, los cuatro diarios de los
/// últimos siete días). Se registran para los días que el flujo va a pedir, que son los que da el
/// reloj del sistema: por eso se calculan con las mismas funciones que usa el flujo.
#[cfg(test)]
fn rutas_de_la_api(t: TransporteFalso) -> TransporteFalso {
    let pulso = json_con_salud(
        r#","allMetrics":{"metricsMap":{"WELLNESS_RESTING_HEART_RATE":[{"calendarDate":"2026-10-12","value":48}]}}"#,
    );
    let mut t = t
        .get(flujo::RUTA_PERFIL, 200, JSON_PERFIL)
        .get(
            flujo::RUTA_ACTIVIDADES,
            200,
            r#"[{"activityId":111,"startTimeLocal":"2026-10-01 07:31:22","activityType":{"typeKey":"running"}}]"#,
        )
        // El pulso se pide por rango: la URL base vale, y también la que lleva el displayName.
        .get(flujo::RUTA_PULSO_REPOSO, 200, &pulso)
        .get(
            &format!("{}/{}", flujo::RUTA_PULSO_REPOSO, "NombreFalsoDePrueba"),
            200,
            &pulso,
        );
    for i in 0..VENTANA_DIAS {
        let dia = hace_dias(i);
        t = t
            .get(
                &format!("{}/{}", flujo::RUTA_SUENO, "NombreFalsoDePrueba"),
                200,
                &json_con_salud(r#","dailySleepDTO":{"sleepTimeSeconds":25200}"#),
            )
            .get(
                &format!("{}/{}", flujo::RUTA_ESTRES, dia),
                200,
                &json_con_salud(r#","calendarDate":"2026-10-14","averageStressLevel":31"#),
            )
            .get(
                &format!("{}/{}", flujo::RUTA_TRAINING, dia),
                200,
                &json_con_salud(r#","loQueSea":1"#),
            );
    }
    t
}

/// Las tres primeras respuestas del login, hasta el ticket de servicio (el intercambio del token lo
/// pone cada prueba, porque ahí está la diferencia entre los dos caminos).
#[cfg(test)]
fn hasta_el_ticket() -> TransporteFalso {
    TransporteFalso::new()
        // La primera visita deja la cookie de sesión del SSO: es lo que hace que el POST del login
        // valga (y lo que sostiene el `Agent` de ureq en la vida real).
        .get_con_cookie(
            flujo::URL_EMBED,
            200,
            "<html><title>widget</title></html>",
            "CASTGC=TGT-FALSO-DE-PRUEBA; Path=/; Secure",
        )
        .get(flujo::URL_SIGNIN, 200, HTML_ACCESO)
        .post(flujo::URL_SIGNIN, 200, HTML_EXITO, Vec::new())
}

/// El camino CON TOKEN: el intercambio sale bien al primer identificador de cliente.
#[cfg(test)]
fn con_token_di() -> TransporteFalso {
    rutas_de_la_api(hasta_el_ticket().post(
        flujo::URL_TOKEN_DI,
        200,
        r#"{"access_token":"TOKEN-DI-FALSO","expires_in":3600}"#,
        Vec::new(),
    ))
}

/// El camino de RESERVA: los cuatro identificadores de cliente fallan y la sesión se sostiene con la
/// cookie `JWT_WEB` que deja el consumo del ticket (`client.py:1284-1306`).
#[cfg(test)]
fn por_cookie() -> TransporteFalso {
    rutas_de_la_api(
        hasta_el_ticket()
            // Los cuatro identificadores de cliente fallan.
            .post(flujo::URL_TOKEN_DI, 403, "", Vec::new())
            // Y la misma página del widget deja la cookie `JWT_WEB` de la reserva.
            .get_con_cookie(
                flujo::URL_EMBED,
                200,
                "<html><title>widget</title></html>",
                "JWT_WEB=JWT-FALSO-DE-PRUEBA; Path=/; Secure",
            ),
    )
}

// =============================================================================================
// PRUEBAS
// =============================================================================================

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Una credencial falsa y reconocible: si aparece en un mensaje, es que se ha colado.
    const FALSA: &str = "CLAVE-FALSA-XYZ-1234";

    fn peticion_de_login() -> Peticion {
        Peticion::post_form(
            flujo::URL_SIGNIN,
            &flujo::campos_del_login(
                "csrf-de-prueba",
                &Secreto::nueva("ana@ejemplo.cl".into()),
                &Secreto::nueva(FALSA.into()),
            ),
        )
    }

    #[test]
    fn la_ayuda_dice_los_tres_modos_y_no_toca_la_red() {
        let t = uso();
        assert!(t.contains("--ayuda"), "{t}");
        assert!(t.contains("--probar"), "{t}");
        assert!(t.contains("--rastrear"), "{t}");
        assert!(
            t.contains("GARMIN_USUARIO") && t.contains("GARMIN_CLAVE"),
            "{t}"
        );
        // Los códigos de salida van en la ayuda, que es donde se consultan. El 8 ya NO está en la
        // lista: el transporte del hito B lo sacó del camino de red, y la ayuda dice por qué.
        assert!(t.contains("9 demasiadas peticiones"), "{t}");
        assert!(t.contains("el 8 «falta el cliente HTTP» ya NO existe"), "{t}");
        assert!(t.contains("ureq"), "{t}");
        assert!(t.contains("rustls"), "{t}");
        // Y el rastreo dice, ANTES de correrlo, qué NO va a imprimir nunca y que con credenciales
        // falsas se ve lo mismo: es la promesa que la prueba de abajo comprueba de verdad.
        assert!(t.contains("NUNCA imprime usuario"), "{t}");
        assert!(t.contains("NOMBRES"), "{t}");
    }

    // -----------------------------------------------------------------------------------------
    // EL RASTREO
    // -----------------------------------------------------------------------------------------

    /// Un cuerpo de 500 como el que devolvió Garmin el 2026-10-14, con un marcador dentro: si el
    /// marcador aparece en el rastro, es que se volcó un cuerpo.
    const JSON_DEL_500: &str = r#"{"timestamp":1,"status":500,"error":"Internal Server Error","exception":"MARCADOR-DEL-CUERPO","path":"/sso/signin"}"#;

    /// Lo que sale de verdad en el POST del acceso del instrumento: el `Referer` y nada más (el
    /// `Content-Type` del formulario se pierde al reemplazar la lista de cabeceras). Es el dato que en
    /// la vida real mide el middleware de `ureq`; aquí se copia, porque la prueba no sale a la red.
    fn lo_que_sale_en_el_acceso() -> sincronizacion::Salida {
        sincronizacion::Salida {
            medido: true,
            nombres: vec!["referer".to_string()],
            content_type: None,
        }
    }

    #[test]
    fn el_rastreo_dice_cada_paso_y_el_500_del_acceso_sin_soltar_el_cuerpo() {
        // El flujo que falla como el de verdad: el POST del acceso contesta 500 con JSON del backend.
        let t = TransporteFalso::new()
            .get_con_cookie(
                flujo::URL_EMBED,
                200,
                "<html><title>widget</title></html>",
                "CASTGC=TGT-FALSO-DE-PRUEBA; Path=/; Secure",
            )
            .get(flujo::URL_SIGNIN, 200, HTML_ACCESO)
            .post(
                flujo::URL_SIGNIN,
                500,
                JSON_DEL_500,
                vec![("Set-Cookie".to_string(), "LOCALE=en; Path=/".to_string())],
            )
            .salida(lo_que_sale_en_el_acceso());
        let rastro = Rastro::recogiendo(&t);
        let fallo = medir(
            &rastro,
            &Secreto::nueva("prueba@invalido.test".into()),
            &Secreto::nueva(FALSA.into()),
        )
        .unwrap_err();
        let dicho = rastro.texto();

        // 1) El paso, numerado y con nombre, y el destino completo (es público).
        assert!(dicho.contains("paso 3"), "los pasos van numerados: {dicho}");
        assert!(
            dicho.contains("EL ACCESO: el POST de las credenciales"),
            "{dicho}"
        );
        assert!(
            dicho.contains("POST https://sso.garmin.com/sso/signin"),
            "{dicho}"
        );
        // 2) Los NOMBRES de lo que envía, y ni uno de sus valores.
        assert!(
            dicho.contains("username, password, embed, _csrf"),
            "los nombres de los campos: {dicho}"
        );
        assert!(!dicho.contains(FALSA), "salió la clave: {dicho}");
        assert!(!dicho.contains("CSRF-DE-PRUEBA"), "salió el _csrf: {dicho}");
        // 3) El content-type, el estado, los bytes y la presencia de set-cookie/location.
        assert!(
            dicho.contains("content-type    (NINGUNO"),
            "el content-type que sale: {dicho}"
        );
        assert!(dicho.contains("respuesta       500"), "{dicho}");
        assert!(dicho.contains("bytes"), "el tamaño del cuerpo: {dicho}");
        assert!(dicho.contains("set-cookie: sí"), "{dicho}");
        assert!(dicho.contains("location: no"), "{dicho}");
        assert!(dicho.contains("cookies"), "las cookies de la jarra: {dicho}");
        // 4) El cuerpo de la respuesta NO se vuelca, ni siquiera teniendo un marcador dentro.
        assert!(!dicho.contains("MARCADOR-DEL-CUERPO"), "se volcó el cuerpo: {dicho}");
        // 5) Y el diagnóstico, pegado al POST que falla, con el `content-type` señalado.
        assert!(dicho.contains("DIAGNÓSTICO"), "{dicho}");
        assert!(
            dicho.contains(
                "ella manda y NOSOTROS NO: accept-language, connection, content-type, sec-ch-ua,"
            ),
            "la comparación con la librería portada: {dicho}"
        );
        assert!(
            dicho.contains("coincidimos en: accept, accept-encoding, content-length, referer, user-agent"),
            "lo que SÍ coincide, para no exagerar la diferencia: {dicho}"
        );
        assert!(dicho.contains("nosotros mandamos y ELLA NO: nada"), "{dicho}");
        assert!(dicho.contains("el que explica el 500 es `content-type`"), "{dicho}");
        // El flujo paró donde tenía que parar, y con el mismo código que `--probar`.
        assert_eq!(fallo.codigo(), CODIGO_SIN_RED, "{}", fallo.mensaje());
    }

    #[test]
    fn el_rastreo_del_flujo_entero_no_suelta_ni_el_nombre_del_cliente_ni_un_dato_de_salud() {
        // El flujo entero, con el camino del token DI. El `displayName` va DENTRO de la URL del sueño
        // y del pulso: el rastro tiene que decir `{displayName}`, no el nombre del cliente.
        let t = rutas_de_la_api(con_token_di());
        let rastro = Rastro::recogiendo(&t);
        let _ = medir(
            &rastro,
            &Secreto::nueva("ana@ejemplo.cl".into()),
            &Secreto::nueva(FALSA.into()),
        );
        let dicho = rastro.texto();

        assert!(dicho.contains("paso 28"), "el flujo entero: {dicho}");
        assert!(dicho.contains("{displayName}"), "{dicho}");
        assert!(
            !dicho.contains("NombreFalsoDePrueba"),
            "salió el nombre del cliente: {dicho}"
        );
        assert!(!dicho.contains(FALSA), "salió la clave: {dicho}");
        assert!(!dicho.contains("TOKEN-DI-FALSO"), "salió el token: {dicho}");
        assert!(!dicho.contains(SALUD_FALSA), "salió un dato de salud: {dicho}");
        assert!(
            !dicho.contains("ST-123456-FALSO"),
            "salió el ticket de servicio: {dicho}"
        );
        // Y del camino bueno no hay diagnóstico: el acceso salió bien y no hay nada que comparar.
        assert!(!dicho.contains("DIAGNÓSTICO"), "{dicho}");
    }

    #[test]
    fn la_comparacion_de_cabeceras_dice_cual_falta_y_cual_coincide() {
        let nuestras = vec!["Referer".to_string()];
        let (solo_ella, solo_nosotras, coinciden) =
            comparar_cabeceras(&nuestras, CABECERAS_DE_LA_LIBRERIA_EN_EL_ACCESO);
        assert!(solo_ella.contains(&"content-type".to_string()), "{solo_ella:?}");
        assert!(solo_ella.contains(&"user-agent".to_string()), "{solo_ella:?}");
        assert!(solo_nosotras.is_empty(), "{solo_nosotras:?}");
        // La comparación no distingue mayúsculas: `Referer` de aquí es `referer` de allí.
        assert_eq!(coinciden, vec!["referer".to_string()]);
    }

    #[test]
    fn el_diagnostico_solo_afirma_lo_que_sus_datos_dicen() {
        // Con el `content-type` ausente: la hipótesis se afirma.
        let texto = diagnostico_del_acceso(&lo_que_sale_en_el_acceso(), 5);
        assert!(texto.contains("el que explica el 500 es `content-type`"), "{texto}");
        assert!(texto.contains("NOSOTROS NO: accept-language, connection, content-type"), "{texto}");
        // Y NO exagera la diferencia: lo que sí sale se dice que coincide. El `user-agent` y el
        // `accept` los añade `ureq`, y la comparación tiene que contarlos.
        assert!(
            texto.contains("coincidimos en: accept, accept-encoding, content-length, referer, user-agent"),
            "{texto}"
        );
        assert!(texto.contains("5 en memoria"), "las cookies, aparte: {texto}");
        // Con el `content-type` presente: NO se afirma, y se dice por qué.
        let con = sincronizacion::Salida {
            medido: true,
            nombres: vec!["referer".to_string(), "content-type".to_string()],
            content_type: Some("application/x-www-form-urlencoded".to_string()),
        };
        let texto = diagnostico_del_acceso(&con, 0);
        assert!(texto.contains("SÍ sale"), "{texto}");
        assert!(!texto.contains("el que explica el 500"), "{texto}");
        // Medido y VACÍO no es lo mismo que sin medir: si la petición no lleva ninguna cabecera, la
        // comparación se hace igual y sale que falta lo que no añade el cliente.
        let vacio = sincronizacion::Salida {
            medido: true,
            nombres: Vec::new(),
            content_type: None,
        };
        let texto = diagnostico_del_acceso(&vacio, 0);
        assert!(!texto.contains("sin dato"), "{texto}");
        assert!(texto.contains("NOSOTROS NO: accept-language"), "{texto}");
        // Y con un transporte que no mide (el falso sin `salida`): se dice «sin dato», no se inventa.
        let texto = diagnostico_del_acceso(&sincronizacion::Salida::default(), 0);
        assert!(texto.contains("sin dato"), "{texto}");
        assert!(!texto.contains("el que explica el 500"), "{texto}");
    }

    #[test]
    fn la_ayuda_dice_como_se_habla_por_la_red_y_que_no_se_siguen_redirecciones() {
        // Lo que el dueño necesita saber ANTES de correrlo, y lo que hay que poder comprobar sin
        // credenciales: el cliente, la jarra en memoria, que un 4xx no aborta y que no hay espera
        // global (para no contarle a la red la espera anti-WAF del flujo).
        let t = uso();
        assert!(t.contains("jarra EN MEMORIA"), "{t}");
        assert!(t.contains("http_status_as_error(false)"), "{t}");
        assert!(t.contains("max_redirects(0)"), "{t}");
        assert!(t.contains("403"), "{t}");
        assert!(t.contains("429"), "{t}");
        assert!(t.contains("sin espera global"), "{t}");
    }

    #[test]
    fn sin_variables_se_dice_cuales_faltan_y_se_sale_con_codigo_propio() {
        let fallo = leer_credenciales(|_| None).unwrap_err();
        let m = fallo.mensaje();
        assert_eq!(fallo.codigo(), CODIGO_FALTAN_VARIABLES);
        assert!(
            m.contains("GARMIN_USUARIO") && m.contains("GARMIN_CLAVE"),
            "{m}"
        );
    }

    #[test]
    fn el_aviso_de_variables_faltantes_no_dice_lo_que_si_habia() {
        // La clave SÍ está; el usuario no. El mensaje no puede contener la clave.
        let fallo = leer_credenciales(|n| {
            if n == "GARMIN_CLAVE" {
                Some(FALSA.to_string())
            } else {
                None
            }
        })
        .unwrap_err();
        assert!(
            !fallo.mensaje().contains(FALSA),
            "se coló la clave: {}",
            fallo.mensaje()
        );
        assert!(!format!("{fallo:?}").contains(FALSA), "{fallo:?}");
    }

    #[test]
    fn las_dos_credenciales_se_leen_y_no_se_pueden_imprimir() {
        let (u, c) = leer_credenciales(|n| {
            Some(
                if n == "GARMIN_USUARIO" {
                    "ana@ejemplo.cl"
                } else {
                    FALSA
                }
                .to_string(),
            )
        })
        .unwrap();
        assert_eq!(u.expuesto(), "ana@ejemplo.cl");
        assert_eq!(c.expuesto(), FALSA);
        assert_eq!(format!("{c:?}"), "«oculto»");
        assert_eq!(format!("{u:?}"), "«oculto»");
    }

    #[test]
    fn una_credencial_en_blanco_es_una_credencial_que_falta() {
        // `export GARMIN_CLAVE=` deja la variable puesta y vacía: eso NO es tener clave.
        let fallo = leer_credenciales(|n| {
            Some(if n == "GARMIN_USUARIO" { "ana" } else { "" }.to_string())
        })
        .unwrap_err();
        assert!(
            fallo.mensaje().contains("GARMIN_CLAVE"),
            "{}",
            fallo.mensaje()
        );
    }

    #[test]
    fn el_salto_de_linea_final_de_un_pegado_no_rompe_la_credencial() {
        let (_, c) = leer_credenciales(|_| Some(format!("{FALSA}\n"))).unwrap();
        assert_eq!(c.expuesto(), FALSA, "el \\n del pegado se tiene que quitar");
    }

    #[test]
    fn imprimir_la_peticion_de_login_no_suelta_la_contrasena() {
        let texto = format!("{:?}", peticion_de_login());
        assert!(!texto.contains(FALSA), "la contraseña salió por Debug: {texto}");
        assert!(texto.contains("«oculto»"), "{texto}");
    }

    #[test]
    fn imprimir_una_respuesta_no_vuelca_el_cuerpo() {
        let r = Respuesta {
            estado: 200,
            cabeceras: Vec::new(),
            cuerpo: r#"{"datoDeSalud":"secreto del cliente"}"#.to_string(),
        };
        let texto = format!("{r:?}");
        assert!(
            !texto.contains("secreto del cliente"),
            "se volcó el cuerpo: {texto}"
        );
    }

    #[test]
    fn las_credenciales_van_en_el_cuerpo_y_no_en_la_url() {
        // En este flujo el usuario y la clave van en el CUERPO del POST: eso es lo que evita que
        // acaben en el registro de un servidor o en un mensaje de error.
        let p = peticion_de_login();
        assert_eq!(p.url, flujo::URL_SIGNIN, "el login no lleva nada en la URL");
        assert!(p.consulta.is_empty(), "{:?}", p.descripcion_publica());
        assert!(!p.descripcion_publica().contains(FALSA));
    }

    #[test]
    fn un_mensaje_no_escribe_los_valores_de_la_consulta() {
        // Uno de esos valores es el ticket de servicio, que es una credencial de un solo uso: se dicen
        // los NOMBRES de los parámetros, nunca los valores (`client.py:196-210`).
        let p = Peticion::get(
            flujo::URL_EMBED,
            vec![("ticket".to_string(), "ST-secreto-de-verdad".to_string())],
        );
        let dicho = p.descripcion_publica();
        assert!(dicho.contains("ticket=…"), "{dicho}");
        assert!(!dicho.contains("ST-secreto-de-verdad"), "{dicho}");
        assert!(!format!("{p:?}").contains("ST-secreto-de-verdad"), "{p:?}");
    }

    #[test]
    fn el_formulario_codifica_lo_que_romperia_el_formulario() {
        // Como `urlencode`: espacio → `+`; y `&`, `+`, `=`, `@`, `%` tienen que ir escapados, o el
        // formulario se parte en silencio y Garmin contesta «clave mala».
        let campos = vec![
            (
                "username".to_string(),
                Secreto::nueva("ana+prueba@ejemplo.cl".into()),
            ),
            ("password".to_string(), Secreto::nueva("p&a=s s%".into())),
        ];
        assert_eq!(
            codificar_formulario(&campos),
            "username=ana%2Bprueba%40ejemplo.cl&password=p%26a%3Ds+s%25"
        );
    }

    #[test]
    fn el_aviso_de_mfa_dice_los_campos_que_espera_garmin() {
        // La pieza está portada aunque hoy el instrumento pare: el aviso enseña por dónde seguiría.
        let fallo = flujo::clasificar_login("<title>MFA</title>").unwrap_err();
        let m = fallo.mensaje();
        assert_eq!(flujo::CAMPOS_DEL_MFA, ["mfa-code", "embed", "_csrf", "fromPage"]);
        for campo in flujo::CAMPOS_DEL_MFA {
            assert!(m.contains(campo), "falta {campo} en el aviso: {m}");
        }
        assert!(m.contains("client.py:898-903"), "{m}");
        // Y la decisión que NO toma el instrumento, la dice: es del dueño (contrato §2).
        assert!(m.contains("contrato §2"), "{m}");
    }

    #[test]
    fn base64_sigue_los_vectores_del_rfc_4648() {
        for (entrada, esperado) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(flujo::base64(entrada), esperado, "base64({entrada:?})");
        }
        assert!(flujo::autorizacion_basica("abc").starts_with("Basic "));
    }

    #[test]
    fn saca_el_csrf_de_la_pagina_de_acceso() {
        let html = r#"<form><input type="hidden" name="_csrf" value="ABC-123"/></form>"#;
        assert_eq!(flujo::extraer_csrf(html).unwrap(), "ABC-123");
    }

    #[test]
    fn sin_csrf_se_dice_que_el_endpoint_cambio_y_no_se_vuelca_la_pagina() {
        let html = "<html><body>un cuerpo larguisimo con datos de salud del cliente</body></html>";
        let fallo = flujo::extraer_csrf(html).unwrap_err();
        assert_eq!(fallo.codigo(), CODIGO_ENDPOINT_CAMBIADO);
        let m = fallo.mensaje();
        assert!(m.contains("Se esperaba"), "{m}");
        assert!(
            m.contains("bytes que NO son JSON"),
            "tiene que decir QUÉ llegó: {m}"
        );
        assert!(
            !m.contains("datos de salud del cliente"),
            "se volcó el cuerpo: {m}"
        );
    }

    #[test]
    fn saca_el_ticket_de_servicio_y_lo_corta_donde_toca() {
        let html = r#"<title>Success</title><a href="https://connect.garmin.com/modern/?ticket=ST-123456-abcdef&x=1">"#;
        assert_eq!(flujo::extraer_ticket(html).unwrap(), "ST-123456-abcdef");
        assert!(
            flujo::extraer_ticket("?ticket=vaya").is_err(),
            "un ticket que no empieza por ST- no es un ticket"
        );
    }

    #[test]
    fn reconoce_el_mfa_por_las_variables_de_la_pagina_y_no_por_el_titulo() {
        // La página del MFA se titula igual que la de acceso: hay que mirar las variables.
        let html = r#"<title>GARMIN Authentication Application</title>
            <script>var mfaMethod = "email"; var customerGuid = "x";</script>"#;
        assert_eq!(flujo::metodo_de_mfa(html).as_deref(), Some("email"));
        match flujo::clasificar_login(html).unwrap_err() {
            Fallo::MfaRequerido { metodo } => assert_eq!(metodo, "email"),
            otro => panic!("tenía que ser MFA, fue {otro:?}"),
        }
        assert_eq!(
            flujo::clasificar_login(html).unwrap_err().codigo(),
            CODIGO_MFA
        );
        // Y la de acceso SIN MFA no se confunde con un MFA.
        let acceso =
            r#"<title>GARMIN Authentication Application</title><input name="_csrf" value="x">"#;
        assert!(flujo::metodo_de_mfa(acceso).is_none());
    }

    #[test]
    fn clasifica_igual_que_la_libreria_y_en_el_mismo_orden() {
        // 1. avería del servidor → sin red.
        assert!(matches!(
            flujo::clasificar_login("<title>Bad Gateway</title>").unwrap_err(),
            Fallo::SinRed(_)
        ));
        // 2. contraseña mala o cuenta bloqueada → credenciales.
        let f = flujo::clasificar_login("<title>Account Locked</title>").unwrap_err();
        assert!(matches!(f, Fallo::CredencialesRechazadas));
        assert_eq!(f.codigo(), CODIGO_CREDENCIALES);
        // 3. cuenta restringida.
        assert!(matches!(
            flujo::clasificar_login("<title>Unable to sign in</title>").unwrap_err(),
            Fallo::CuentaRestringida
        ));
        // 5. un título que no es «Success» → el endpoint ha cambiado, y se dice cuál llegó.
        let f = flujo::clasificar_login("<title>Otra cosa</title>").unwrap_err();
        assert!(f.mensaje().contains("Otra cosa"), "{}", f.mensaje());
        // 6. éxito, con su ticket.
        assert_eq!(
            flujo::clasificar_login(r#"<title>Success</title>?ticket=ST-9-z"#).unwrap(),
            "ST-9-z"
        );
    }

    #[test]
    fn la_cookie_jwt_web_se_lee_de_las_cabeceras() {
        let cabeceras = vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            (
                "Set-Cookie".to_string(),
                "JWT_WEB=eyJhbGciOi.abc.def; Path=/; Secure".to_string(),
            ),
        ];
        assert_eq!(
            flujo::cookie_jwt_web(&cabeceras).unwrap(),
            "eyJhbGciOi.abc.def"
        );
        assert!(flujo::cookie_jwt_web(&[]).is_err());
    }

    #[test]
    fn las_cabeceras_de_sesion_tienen_los_dos_caminos() {
        // Las construye el módulo del transporte (hito B); el flujo las nombra desde su `flujo`.
        let con_token = sincronizacion::cabeceras_de_sesion(Some("TOKEN"), None);
        assert!(con_token
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer TOKEN"));
        let con_cookie = flujo::cabeceras_de_sesion(None, Some("JWT"));
        assert_eq!(
            con_cookie,
            sincronizacion::cabeceras_de_sesion(None, Some("JWT")),
            "el flujo y el módulo tienen que decir LO MISMO: es una sola lista de cabeceras"
        );
        assert!(con_cookie.iter().any(|(k, _)| k == "DI-Backend"));
        assert!(con_cookie
            .iter()
            .any(|(k, v)| k == "Cookie" && v == "JWT_WEB=JWT"));
        // El agente nativo, en los dos caminos (`client.py:165-169`).
        assert!(con_cookie.iter().any(|(k, _)| k == "X-Garmin-User-Agent"));
    }

    #[test]
    fn la_forma_del_cuerpo_se_describe_sin_volcar_ni_un_valor() {
        let json = r#"{"sleepTimeSeconds":25200,"nota":"secreto del cliente"}"#;
        let forma = forma_del_cuerpo(json);
        assert!(forma.contains("2 claves"), "{forma}");
        assert!(
            forma.contains("sleepTimeSeconds"),
            "las claves sí se dicen: {forma}"
        );
        assert!(!forma.contains("secreto del cliente"), "{forma}");
        assert!(!forma.contains("25200"), "un valor no se dice: {forma}");

        assert!(forma_del_cuerpo("[1,2,3]").contains("lista JSON de 3 elementos"));
        assert!(forma_del_cuerpo("<html>hola</html>").contains("NO son JSON"));
    }

    #[test]
    fn las_actividades_se_leen_sin_guardar_el_identificador() {
        let cuerpo = r#"[{"activityId":111,"startTimeLocal":"2026-10-01 07:31:22","activityType":{"typeKey":"running"}},
                         {"activityId":222,"startTimeLocal":"2026-10-09T18:00:00","activityType":{"typeKey":"strength_training"}},
                         {"activityId":333,"activityType":{"typeKey":"walking"}}]"#;
        let a = flujo::actividades(cuerpo).unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(a[0].dia.as_deref(), Some("2026-10-01"));
        assert_eq!(a[1].dia.as_deref(), Some("2026-10-09"));
        assert_eq!(a[1].tipo, "strength_training");
        assert!(a[2].dia.is_none(), "sin fecha no se inventa un día");
        // Y una respuesta que no es lista es un endpoint cambiado, no un pánico.
        assert_eq!(
            flujo::actividades(r#"{"activities":[]}"#).unwrap_err().codigo(),
            CODIGO_ENDPOINT_CAMBIADO
        );
    }

    #[test]
    fn el_sueno_cuenta_como_dato_solo_si_la_noche_esta_medida() {
        assert!(flujo::sueno_tiene_dato(r#"{"dailySleepDTO":{"sleepTimeSeconds":25200}}"#).unwrap());
        // Una noche sin reloj viene con el resumen a cero: NO es un dato.
        assert!(!flujo::sueno_tiene_dato(r#"{"dailySleepDTO":{"sleepTimeSeconds":0}}"#).unwrap());
        assert!(!flujo::sueno_tiene_dato(r#"{"dailySleepDTO":{}}"#).unwrap());
        // Sin la clave, no es «vacío»: es otra respuesta.
        let f = flujo::sueno_tiene_dato(r#"{"otraCosa":1}"#).unwrap_err();
        assert_eq!(f.codigo(), CODIGO_ENDPOINT_CAMBIADO);
        assert!(f.mensaje().contains("dailySleepDTO"), "{}", f.mensaje());
    }

    #[test]
    fn el_estres_cuenta_los_dias_con_medida_y_dice_cuantos_pidio() {
        assert!(flujo::estres_tiene_dato(r#"{"calendarDate":"2026-10-14","averageStressLevel":31}"#).unwrap());
        // Un día con la clave pero sin medida no es un día con dato.
        assert!(!flujo::estres_tiene_dato(r#"{"calendarDate":"2026-10-14","averageStressLevel":0}"#).unwrap());
        assert!(!flujo::estres_tiene_dato(r#"{"calendarDate":"2026-10-14"}"#).unwrap());
        assert!(flujo::estres_tiene_dato(r#"{"averageStressLevel":31}"#).is_err());
    }

    #[test]
    fn el_pulso_en_reposo_cuenta_solo_las_filas_con_valor() {
        let cuerpo = r#"{"allMetrics":{"metricsMap":{"WELLNESS_RESTING_HEART_RATE":[
            {"calendarDate":"2026-10-12","value":48},
            {"calendarDate":"2026-10-13","value":null},
            {"calendarDate":"2026-10-14","value":50}]}}}"#;
        assert_eq!(
            flujo::dias_de_pulso(cuerpo).unwrap(),
            vec!["2026-10-12".to_string(), "2026-10-14".to_string()],
            "__init__.py:2219-2224 descarta las filas sin valor"
        );
        // Una respuesta sin `allMetrics` no es «sin dato»: es otra respuesta.
        assert_eq!(
            flujo::dias_de_pulso(r#"{"loQueSea":1}"#).unwrap_err().codigo(),
            CODIGO_ENDPOINT_CAMBIADO
        );
        // Y sin la serie (pero con `allMetrics`) es un rango vacío, no un error.
        assert!(flujo::dias_de_pulso(r#"{"allMetrics":{"metricsMap":{}}}"#)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn el_training_status_se_comprueba_solo_como_objeto_no_vacio() {
        // No hay claves citables para esta respuesta: se comprueba lo único que se puede afirmar.
        assert!(flujo::training_tiene_dato(r#"{"loQueSea":1}"#).unwrap());
        // Un objeto vacío es «ese día no hay dato» —y así lo enseña la tabla—, no un cambio de forma.
        assert!(!flujo::training_tiene_dato("{}").unwrap());
        // Lo que sí es un cambio de forma es que no venga un objeto.
        assert_eq!(
            flujo::training_tiene_dato("[]").unwrap_err().codigo(),
            CODIGO_ENDPOINT_CAMBIADO
        );
    }

    #[test]
    fn sin_cliente_http_el_fallo_dice_que_falta_y_que_habria_que_pedir() {
        // El hueco se conserva a propósito (ver la cabecera del fichero): ya no es el transporte del
        // camino de red —ese es `Cliente`, con `ureq`— pero la regla que lo trajo sigue probada.
        let fallo = SinTransporte
            .pedir(&Peticion::get("https://sso.garmin.com/x", Vec::new()))
            .unwrap_err();
        assert_eq!(fallo.codigo(), CODIGO_SIN_CLIENTE_HTTP);
        let m = fallo.mensaje();
        assert!(m.contains("ureq"), "{m}");
        assert!(m.contains("reqwest"), "{m}");
        // Y el hallazgo medido que decide no pedirlo todo de golpe: `rquest` está yanked.
        assert!(
            m.contains("yanked"),
            "el dato de rquest tiene que seguir dicho: {m}"
        );
        // Pero tiene que quedar claro que ESTE camino ya no se usa.
        assert!(m.contains("YA NO SE USA"), "{m}");
    }

    #[test]
    fn la_medicion_para_en_el_primer_paso_de_red_sin_hacer_nada_mas() {
        // Con el hueco (un `Cliente` sin cliente HTTP dentro), el instrumento devuelve el error
        // tipado, no un pánico —y ese es el camino que `main` corta ANTES de medir, con el código 8—.
        let fallo = medir(
            &Cliente(None),
            &Secreto::nueva("ana@ejemplo.cl".into()),
            &Secreto::nueva(FALSA.into()),
        )
        .unwrap_err();
        assert_eq!(fallo.codigo(), CODIGO_SIN_CLIENTE_HTTP);
        // Y el camino bueno SÍ tiene cliente: `main` no puede cortar por el hueco en una corrida
        // normal. Es la prueba de que el código 8 desapareció del camino de red.
        assert!(
            !Cliente::nuevo().sin_cliente_http(),
            "el transporte de verdad tiene que estar puesto"
        );
    }

    #[test]
    fn se_cuentan_los_registros_por_tipo_y_su_rango() {
        let mut f = Ficha::nueva("actividades");
        f.anota("2026-10-01");
        f.anota("2026-10-09");
        f.anota("2026-10-09");
        *f.por_tipo.entry("running".to_string()).or_insert(0) += 2;
        *f.por_tipo.entry("cycling".to_string()).or_insert(0) += 1;
        let t = f.linea(28);
        assert!(t.contains("3 con dato"), "{t}");
        assert!(t.contains("rango 2026-10-01 .. 2026-10-09"), "{t}");
        assert!(t.contains("running 2"), "{t}");
        assert!(t.contains("cycling 1"), "{t}");
    }

    #[test]
    fn un_diario_vacio_se_dice_que_esta_vacio_y_no_se_dibuja_un_cero() {
        // La pregunta del contrato §5.1: en la exportación venían vacíos. Aquí se distingue «pedido y
        // vacío» de «con dato», que es justo lo que hay que medir.
        let mut f = Ficha::nueva("estrés");
        f.pide("2026-10-13", false);
        f.pide("2026-10-14", false);
        let t = f.linea(28);
        assert!(t.contains("sin dato"), "{t}");
        assert!(t.contains("2 pedidos, ninguno trajo nada"), "{t}");
        assert!(!t.contains("rango"), "vacío no tiene rango: {t}");
        // Y con un día bueno de tres, se dice 1 de 3.
        f.pide("2026-10-12", true);
        let t = f.linea(28);
        assert!(t.contains("1 con dato de 3 pedidos"), "{t}");
        assert!(t.contains("rango 2026-10-12 .. 2026-10-12"), "{t}");
    }

    #[test]
    fn el_resumen_da_el_tiempo_con_coma() {
        let mut a = Ficha::nueva("actividades");
        a.anota("2026-10-14");
        let t = resumen(&[a], Duration::from_millis(12_400));
        assert!(
            t.contains("LO QUE TRAE DE VERDAD LA API INTERNA DE GARMIN"),
            "{t}"
        );
        assert!(t.contains("actividades"), "{t}");
        assert!(t.contains("12,4 s"), "el tiempo, en español: {t}");
    }

    #[test]
    fn los_endpoints_portados_dicen_su_fuente() {
        // Cada endpoint tiene que traer de dónde se copió: sin cita, no vale.
        for (nombre, url, fuente) in flujo::ENDPOINTS {
            assert!(url.starts_with("https://"), "{nombre}");
            assert!(fuente.contains(':'), "{nombre} sin cita: {fuente}");
        }
        assert!(flujo::ENDPOINTS
            .iter()
            .any(|(_, u, _)| u.contains("sso.garmin.com")));
        assert!(flujo::ENDPOINTS
            .iter()
            .any(|(_, u, _)| u.contains("connectapi.garmin.com")));
    }

    // -----------------------------------------------------------------------------------------
    // EL FLUJO ENTERO, SIN RED Y CON CREDENCIALES FALSAS
    //
    // El flujo de red NO se prueba con las credenciales del dueño (las pone él en SU terminal): con
    // el transporte falso se prueba TODO lo demás, que es donde estaban los fallos posibles.
    // -----------------------------------------------------------------------------------------

    #[test]
    fn el_flujo_entero_llega_al_final_y_ninguna_credencial_sale_por_el_camino() {
        let t = con_token_di();
        let resumen = medir(
            &t,
            &Secreto::nueva("prueba@invalido.test".into()),
            &Secreto::nueva(FALSA.into()),
        )
        .expect("el flujo tiene que llegar hasta el final con el transporte falso");

        // El recuento: lo ÚNICO que el instrumento devuelve.
        assert!(resumen.contains("actividades"), "{resumen}");
        assert!(resumen.contains("1 con dato"), "{resumen}");
        assert!(resumen.contains("rango 2026-10-01 .. 2026-10-01"), "{resumen}");
        assert!(resumen.contains("7 con dato"), "los siete diarios: {resumen}");
        assert!(resumen.contains("pulso en reposo"), "{resumen}");

        // **La regla que más importaba**: por el camino no puede salir ni la clave falsa, ni el
        // cuerpo de una respuesta, ni el token.
        assert!(!resumen.contains(FALSA), "salió la clave: {resumen}");
        assert!(
            !resumen.contains(SALUD_FALSA),
            "salió un dato de salud: {resumen}"
        );
        assert!(!resumen.contains("TOKEN-DI-FALSO"), "{resumen}");

        // Y el flujo hizo los deberes, contados a mano: widget + página de acceso + POST de las
        // credenciales + POST del token + perfil + actividades + pulso por rango = 7, y 3 más por
        // cada uno de los 7 días (sueño, estrés y training status) = 21. Total, 28.
        let pedidas = t.pedidas();
        assert_eq!(pedidas.len(), 28, "pasos de red");
        assert!(!pedidas.is_empty(), "el transporte falso tiene que haber visto algo");

        // Ninguna petición soltó la clave falsa ni la puso en una URL.
        for p in &pedidas {
            assert!(!p.url.contains(FALSA), "la clave fue en una URL: {}", p.url);
            assert!(
                !format!("{p:?}").contains(FALSA),
                "la clave salió por el Debug de una petición"
            );
            assert!(
                !p.descripcion_publica().contains("ST-123456-FALSO"),
                "el ticket de servicio salió en un mensaje: {}",
                p.descripcion_publica()
            );
            // Las llamadas a la API llevan el agente nativo (`client.py:165-169`). OJO, medido: los
            // pasos del SSO (`sso.garmin.com`) NO lo llevan —van como el navegador del flujo—, así
            // que la comprobación es sobre la API y no sobre todo el camino.
            if p.url.contains(flujo::API) {
                assert_eq!(
                    p.cabeceras
                        .iter()
                        .find(|(k, _)| k == "User-Agent")
                        .map(|(_, v)| v.as_str()),
                    Some(sincronizacion::AGENTE),
                    "sin agente nativo en {}",
                    p.url
                );
                assert!(p.cabeceras.iter().any(|(k, _)| k == "X-Garmin-User-Agent"));
            }
        }
        // Y los pasos del SSO, en cambio, llevan el `Referer` del widget en el POST del login.
        let del_login = t
            .pedidas()
            .into_iter()
            .find(|p| matches!(p.metodo, Metodo::Post) && p.url == flujo::URL_SIGNIN)
            .expect("el POST del login se hizo");
        assert!(
            del_login
                .cabeceras
                .iter()
                .any(|(k, v)| k == "Referer" && v == flujo::URL_EMBED),
            "{:?}",
            del_login.cabeceras
        );

        // Las llamadas a la API llevan el `Bearer` del camino con token.
        let de_la_api = t.cabeceras_de(flujo::RUTA_PERFIL);
        assert!(
            de_la_api
                .iter()
                .any(|(k, v)| k == "Authorization" && v == "Bearer TOKEN-DI-FALSO"),
            "{de_la_api:?}"
        );
        // Y el `Debug` de esa petición dice el esquema, no el token.
        let con_bearer = t
            .pedidas()
            .into_iter()
            .find(|p| p.url.contains(flujo::RUTA_PERFIL))
            .expect("el perfil se pidió");
        let dicho = format!("{con_bearer:?}");
        assert!(dicho.contains("Bearer …"), "{dicho}");
        assert!(!dicho.contains("TOKEN-DI-FALSO"), "se volcó el token: {dicho}");
    }

    #[test]
    fn si_el_intercambio_del_token_falla_la_sesion_sigue_por_la_cookie_jwt_web() {
        // `client.py:1284-1306`: el camino de reserva. Se prueba entero: los CUATRO identificadores
        // de cliente se intentan en orden, y después la sesión se sostiene con `JWT_WEB`.
        let t = por_cookie();
        let resumen = medir(
            &t,
            &Secreto::nueva("prueba@invalido.test".into()),
            &Secreto::nueva(FALSA.into()),
        )
        .expect("la reserva tiene que sostener el flujo");
        assert!(resumen.contains("actividades"), "{resumen}");
        assert_eq!(
            t.veces_pedido(flujo::URL_TOKEN_DI),
            4,
            "se prueban los cuatro identificadores de cliente, en orden (client.py:174-179)"
        );
        // Y la cookie se leyó de las cabeceras: la sesión va por `JWT_WEB` + las cabeceras de la web.
        let de_la_api = t.cabeceras_de(flujo::RUTA_PERFIL);
        assert!(
            de_la_api
                .iter()
                .any(|(k, v)| k == "Cookie" && v == "JWT_WEB=JWT-FALSO-DE-PRUEBA"),
            "{de_la_api:?}"
        );
        assert!(de_la_api.iter().any(|(k, _)| k == "DI-Backend"));
        assert!(de_la_api.iter().any(|(k, _)| k == "NK"));
        assert!(de_la_api.iter().any(|(k, _)| k == "Origin"));
    }

    #[test]
    fn con_credenciales_malas_el_flujo_devuelve_el_error_tipado_de_credenciales_y_no_el_8() {
        // Lo que el dueño va a ver con una clave que no vale: código 4 (credenciales rechazadas), no
        // el 8 del hueco. La clasificación es la de la librería (`client.py:812-881`).
        let t = TransporteFalso::new()
            .get(flujo::URL_EMBED, 200, "<html><title>widget</title></html>")
            .get(flujo::URL_SIGNIN, 200, HTML_ACCESO)
            .post(
                flujo::URL_SIGNIN,
                200,
                "<html><title>Account Locked</title></html>",
                Vec::new(),
            );
        let fallo = medir(
            &t,
            &Secreto::nueva("prueba@invalido.test".into()),
            &Secreto::nueva(FALSA.into()),
        )
        .unwrap_err();
        assert_eq!(fallo.codigo(), CODIGO_CREDENCIALES);
        assert!(
            fallo.codigo() != CODIGO_SIN_CLIENTE_HTTP,
            "el hueco ya no está en el camino de red"
        );
        assert!(
            !fallo.mensaje().contains(FALSA),
            "el mensaje no puede traer la clave: {}",
            fallo.mensaje()
        );
    }

    #[test]
    fn un_429_por_el_camino_para_con_el_codigo_del_anti_abuso() {
        // Con el transporte de verdad, un 429 llega como fallo tipado del transporte y conserva su
        // código (9). Se comprueba la conversión, que es el punto donde se podría perder.
        let del_transporte: Fallo = sincronizacion::Fallo::DemasiadasPeticiones(429).into();
        assert_eq!(del_transporte.codigo(), CODIGO_DEMASIADAS_PETICIONES);
        assert!(
            del_transporte.mensaje().contains("anti-abuso"),
            "{}",
            del_transporte.mensaje()
        );
        // Un 403, en cambio, es «sin red» para el transporte y el flujo decide qué hacer con él.
        let sin_red: Fallo = sincronizacion::Fallo::Estado(403).into();
        assert_eq!(sin_red.codigo(), CODIGO_SIN_RED);
        assert!(sin_red.mensaje().contains("403"), "{}", sin_red.mensaje());
    }

    #[test]
    fn un_fallo_de_red_dice_por_donde_iba_pero_sin_los_valores_de_la_consulta() {
        // El «al pedir …» lo añade el instrumento, y el destino va SIN valores: uno de ellos es el
        // ticket de servicio, que es una credencial de un solo uso.
        let t = TransporteFalso::new(); // sin ninguna ruta: todo falla
        let peticion = Peticion::get(
            flujo::URL_EMBED,
            vec![("ticket".to_string(), "ST-123456-FALSO".to_string())],
        );
        let fallo = pedir(&t, &peticion).unwrap_err();
        let m = fallo.mensaje();
        assert!(m.contains("al pedir"), "{m}");
        assert!(m.contains("ticket=…"), "{m}");
        assert!(!m.contains("ST-123456-FALSO"), "salió el ticket: {m}");
        assert_eq!(fallo.codigo(), CODIGO_SIN_RED);
    }

    #[test]
    fn las_dependencias_candidatas_dicen_crate_y_para_que() {
        // El dato que el orquestador necesita para pedir autorización tiene que seguir en el binario.
        let texto = uso();
        assert!(texto.contains("ureq"), "{texto}");
        assert!(texto.contains("reqwest"), "{texto}");
        assert!(texto.contains("serde_json"), "{texto}");
        assert!(texto.contains("yanked"), "{texto}");
    }
}
