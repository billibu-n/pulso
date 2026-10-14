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
//! **El flujo está portado; falta el socket.** En `motor/Cargo.toml` **no hay cliente HTTP** y este
//! hito tiene prohibido añadirlo sin autorización del dueño (contrato §6.2). Por eso `--probar` llega
//! hasta la comprobación de variables y **para en el primer paso de red** con un error TIPADO
//! (`SIN_CLIENTE_HTTP`, código 8) que trae la lista de dependencias candidatas.
//!
//! Lo que sí queda hecho, y probado, es **el flujo entero**: endpoints, parámetros, sesión, cookies,
//! MFA, la lectura de cada respuesta y el recuento (ver `mod flujo`). Cuando el dueño autorice la
//! dependencia, el único trabajo que queda es **implementar `Transporte` para el cliente elegido**:
//! ni una línea del flujo cambia.
//!
//! **Lo que NO se hizo, y por qué**: no se llama a `curl` por proceso hijo, que sería hablar por la
//! red sin añadir dependencia. Las credenciales viajarían a otro proceso (visibles en `/proc`, y en
//! el historial si algún día alguien las pasara por argumento). Con una contraseña de Garmin y datos
//! de salud, eso no se hace: se pide la dependencia y se espera.

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

/// **La lista exacta de lo que habría que añadir a `Cargo.toml`, medida —no recordada— el 2026-10-14.**
///
/// Medido con `cargo add --dry-run ureq --features cookies` (resuelve contra el registro y **no
/// escribe nada**): «Adding ureq v3.4.2 to dependencies», con `Cargo.toml` intacto. Lo demás sale de
/// la ficha de cada crate en crates.io.
const DEPENDENCIAS_CANDIDATAS: &[(&str, &str)] = &[
    (
        "ureq 3",
        "el cliente HTTP: POST de formulario, cabeceras propias, redirecciones y COOKIES (feature\n             `cookies`, que es lo que sostiene la sesión del SSO). Bloqueante, sin tokio, con TLS\n             rustls. Es el más pequeño que sirve, y esto es un instrumento de línea de comandos, no\n             un servicio. RECOMENDADO.",
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
    s.push_str("\nCREDENCIALES: se leen de las variables de entorno GARMIN_USUARIO y GARMIN_CLAVE, que\n");
    s.push_str("  exporta el dueño en SU terminal. No se imprimen, no se registran y no se guardan: no\n");
    s.push_str("  aparecen ni en la salida ni en un mensaje de error.\n");
    s.push_str(
        "\nDEVUELVE: cuántos hay de cada cosa y su rango de fechas, más el tiempo total. Nada más.\n",
    );
    s.push_str("\nCÓDIGOS DE SALIDA: 0 bien · 2 uso · 3 faltan variables · 4 credenciales rechazadas ·\n");
    s.push_str("  5 MFA requerido · 6 sin red o servidor caído · 7 endpoint cambiado (forma\n");
    s.push_str("  inesperada) · 8 falta el cliente HTTP · 9 demasiadas peticiones (anti-abuso)\n");

    s.push_str("\nEL FLUJO QUE SE PORTA, EN ORDEN (con su fuente, que es lo que lo hace comprobable):\n");
    for (nombre, url, fuente) in flujo::ENDPOINTS {
        s.push_str(&format!("  {nombre}\n      {url}\n      fuente: {fuente}\n"));
    }

    s.push_str("\nDEPENDENCIAS QUE FALTAN (crate y para qué) — ninguna está añadida:\n");
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
    /// `client.py:171-173` — `DI_GRANT_TYPE`: se envía como `grant_type` y es a su vez una URL.
    pub const TIPO_DE_CONCESION_DI: &str =
        "https://connectapi.garmin.com/di-oauth2-service/oauth/grant/service_ticket";
    /// `client.py:313` — `self._connectapi = f"https://connectapi.{domain}"`.
    pub const API: &str = "https://connectapi.garmin.com";
    /// `client.py:468` — la cabecera `DI-Backend` de la sesión por cookie.
    pub const DI_BACKEND: &str = "connectapi.garmin.com";
    /// `client.py:466-467` — `Origin` y `Referer` de la sesión por cookie.
    pub const ORIGEN_WEB: &str = "https://connect.garmin.com";
    /// `client.py:165` — `NATIVE_API_USER_AGENT`.
    pub const AGENTE: &str = "GCM-Android-5.23";
    /// `client.py:166-169` — `X-Garmin-User-Agent`.
    pub const AGENTE_GARMIN: &str = "com.garmin.android.apps.connectmobile/5.23; ; \
         Google/sdk_gphone64_arm64/google; Android/33; Dalvik/2.1.0";
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
    /// No los envía nadie **todavía**, y es a propósito: cuando Garmin pide el segundo factor, este
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
    pub fn cookie_jwt_web(cabeceras: &[(String, String)]) -> Result<String, Fallo> {
        for (nombre, valor) in cabeceras {
            if nombre.eq_ignore_ascii_case("set-cookie") && valor.starts_with("JWT_WEB=") {
                let desde = &valor["JWT_WEB=".len()..];
                let fin = desde.find(';').unwrap_or(desde.len());
                if fin > 0 {
                    return Ok(desde[..fin].to_string());
                }
            }
        }
        Err(Fallo::EndpointCambiado {
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
    pub fn cabeceras_de_sesion(token: Option<&str>, jwt_web: Option<&str>) -> Vec<(String, String)> {
        let mut h = vec![
            ("User-Agent".to_string(), AGENTE.to_string()),
            (
                "X-Garmin-User-Agent".to_string(),
                AGENTE_GARMIN.to_string(),
            ),
            ("Accept".to_string(), "application/json".to_string()),
        ];
        if let Some(token) = token {
            h.push(("Authorization".to_string(), format!("Bearer {token}")));
            return h;
        }
        h.push(("NK".to_string(), "NT".to_string()));
        h.push(("Origin".to_string(), ORIGEN_WEB.to_string()));
        h.push(("Referer".to_string(), format!("{ORIGEN_WEB}/modern/")));
        h.push(("DI-Backend".to_string(), DI_BACKEND.to_string()));
        if let Some(jwt) = jwt_web {
            h.push(("Cookie".to_string(), format!("JWT_WEB={jwt}")));
        }
        h
    }

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

/// **Una credencial.** No implementa `Display` —para que no pueda acabar imprimiéndose por descuido— y
/// su `Debug` dice «oculto». Como el cuerpo de una petición de login es un `Secreto`, imprimir la
/// petición entera con `{:?}` tampoco suelta la contraseña. Es la regla del hito, escrita en el tipo.
#[derive(Clone)]
struct Secreto(String);

impl Secreto {
    fn nueva(valor: String) -> Self {
        Secreto(valor)
    }
    /// El valor, para el único sitio que lo necesita: construir el cuerpo del formulario.
    fn expuesto(&self) -> &str {
        &self.0
    }
    fn esta_vacio(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Debug for Secreto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "«oculto»")
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
    /// El único hueco: no hay cliente HTTP en `Cargo.toml` y añadirlo necesita autorización.
    SinClienteHttp,
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
                    "No hay cliente HTTP en `motor/Cargo.toml`, y este hito tiene prohibido añadirlo \
                     sin autorización del dueño.\n  El flujo está portado y probado; falta el socket. \
                     Lo que habría que añadir:\n",
                );
                for (crate_, para_que) in DEPENDENCIAS_CANDIDATAS {
                    s.push_str(&format!("    · {crate_}: {para_que}\n"));
                }
                s
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
// EL HUECO: quién habla por la red
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Metodo {
    Get,
    Post,
}

/// Una petición, ya montada. El cuerpo es un `Secreto` **porque el del login lleva la contraseña**, y
/// el `Debug` está escrito a mano para dejar claro qué se puede imprimir y qué no.
struct Peticion {
    metodo: Metodo,
    url: String,
    consulta: Vec<(String, String)>,
    cabeceras: Vec<(String, String)>,
    cuerpo: Option<Secreto>,
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
            .field("cabeceras", &self.cabeceras)
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
        }
    }

    fn con_cabeceras(mut self, cabeceras: Vec<(String, String)>) -> Self {
        self.cabeceras = cabeceras;
        self
    }

    /// La petición dicha **para un mensaje**: la URL y los NOMBRES de los parámetros, sin sus valores.
    ///
    /// Es la regla que la propia librería aplica a sus errores (`client.py:196-210`,
    /// `_sanitize_exception_text`), y aquí no es un adorno: uno de esos valores es el ticket de
    /// servicio, que es una credencial de un solo uso. Un valor de consulta **no se escribe**.
    fn descripcion_publica(&self) -> String {
        if self.consulta.is_empty() {
            return self.url.clone();
        }
        let nombres = self
            .consulta
            .iter()
            .map(|(k, _)| format!("{k}=…"))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}?{nombres}", self.url)
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

/// **El único punto por donde hay que hablar por la red.** Cuando el dueño autorice la dependencia se
/// implementa esto con el cliente elegido y **no se toca ni una línea del flujo**.
trait Transporte {
    fn pedir(&self, peticion: &Peticion) -> Result<Respuesta, Fallo>;
}

/// Hoy no hay cliente HTTP: el hueco se declara, no se rellena con un remedio peor.
struct SinTransporte;

impl Transporte for SinTransporte {
    fn pedir(&self, _peticion: &Peticion) -> Result<Respuesta, Fallo> {
        Err(Fallo::SinClienteHttp)
    }
}

/// Pide al transporte y, si no se pudo, **añade por dónde iba**: la URL y los nombres de los
/// parámetros, nunca sus valores (`client.py:196-210`).
fn pedir(t: &dyn Transporte, peticion: &Peticion) -> Result<Respuesta, Fallo> {
    t.pedir(peticion)
        .map_err(|fallo| fallo.al_pedir(&peticion.descripcion_publica()))
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
    let r = pedir(t, &Peticion::get(flujo::URL_EMBED, flujo::parametros_embed()))?;
    exigir_estado(&r, "la página de entrada del widget (client.py:770-775)")
}

/// `client.py:777-788` — la página de acceso, de donde sale el `_csrf`.
fn paso_csrf(t: &dyn Transporte) -> Result<String, Fallo> {
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
    espera_anti_waf();
    let peticion =
        Peticion::post_form(flujo::URL_SIGNIN, &flujo::campos_del_login(csrf, usuario, clave))
            .con_cabeceras(vec![(
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
    for cliente in flujo::CLIENTES_DI {
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
fn espera_anti_waf() {
    let (min, max) = flujo::ESPERA_ANTI_WAF_S;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    std::thread::sleep(Duration::from_secs(min + nanos % (max - min + 1)));
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
/// tiempo total. Con `SinTransporte` para en el primer paso de red y devuelve el error tipado.
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
        let r = pedir_api(t, &sesion, &ruta, consulta)?;
        sueno.pide(&dia, flujo::sueno_tiene_dato(&r.cuerpo)?);
    }
    fichas.push(sueno);

    // --- Pulso en reposo: el rango entero en UNA petición (`__init__.py:2199-2224`) ---
    let desde = hace_dias(VENTANA_DIAS - 1);
    let (ruta, consulta) = flujo::ruta_pulso_reposo(&nombre, &desde, &hoy);
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
        let r = pedir_api(t, &sesion, &ruta, consulta)?;
        estres.pide(&dia, flujo::estres_tiene_dato(&r.cuerpo)?);

        let (ruta, consulta) = flujo::ruta_training(&dia);
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

    if !argumentos.iter().any(|a| a == "--probar") {
        eprintln!("Uso: {} --ayuda | --probar", env!("CARGO_BIN_NAME"));
        std::process::exit(CODIGO_USO);
    }

    let (usuario, clave) = match leer_credenciales(|n| std::env::var(n).ok()) {
        Ok(par) => par,
        Err(fallo) => {
            eprintln!("{}", fallo.mensaje());
            std::process::exit(fallo.codigo());
        }
    };

    // Hoy el transporte es el que no hay: el flujo se detiene en el primer paso de red con el error
    // tipado que explica qué falta. Cuando el dueño autorice la dependencia, ESTA línea cambia por la
    // implementación real de `Transporte` y nada más.
    let transporte = SinTransporte;

    match medir(&transporte, &usuario, &clave) {
        Ok(texto) => print!("{texto}"),
        Err(fallo) => {
            eprintln!("{}", fallo.mensaje());
            std::process::exit(fallo.codigo());
        }
    }
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
    fn la_ayuda_dice_los_dos_modos_y_no_toca_la_red() {
        let t = uso();
        assert!(t.contains("--ayuda"), "{t}");
        assert!(t.contains("--probar"), "{t}");
        assert!(
            t.contains("GARMIN_USUARIO") && t.contains("GARMIN_CLAVE"),
            "{t}"
        );
        // Los códigos de salida van en la ayuda, que es donde se consultan.
        assert!(t.contains("8 falta el cliente HTTP"), "{t}");
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
        let con_token = flujo::cabeceras_de_sesion(Some("TOKEN"), None);
        assert!(con_token
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer TOKEN"));
        let con_cookie = flujo::cabeceras_de_sesion(None, Some("JWT"));
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
    }

    #[test]
    fn la_medicion_para_en_el_primer_paso_de_red_sin_hacer_nada_mas() {
        // Hoy no hay transporte: el instrumento devuelve el error tipado, no un pánico.
        let fallo = medir(
            &SinTransporte,
            &Secreto::nueva("ana@ejemplo.cl".into()),
            &Secreto::nueva(FALSA.into()),
        )
        .unwrap_err();
        assert_eq!(fallo.codigo(), CODIGO_SIN_CLIENTE_HTTP);
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
