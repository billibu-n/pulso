//! **El TRANSPORTE HTTP del bloque Garmin: quién habla por la red.**
//!
//! HITO B del bloque Garmin. El flujo (endpoints, parámetros, sesión, cookies, MFA y la lectura de
//! cada respuesta) está portado y probado en el instrumento `src/bin/sincronizacion.rs`; lo que
//! faltaba era **el socket**: hasta el 2026-10-14 `motor/Cargo.toml` no tenía cliente HTTP y el
//! instrumento paraba en el error tipado `SIN_CLIENTE_HTTP` (código 8). Ese hueco es este módulo.
//!
//! ## QUÉ HACE Y QUÉ NO
//!
//! - HACE: pedir por HTTP (GET con parámetros de consulta, POST de formulario), **sostener la sesión
//!   con una jarra de cookies EN MEMORIA**, y devolver **(estado, cuerpo)** con el cuerpo envuelto en
//!   un [`Secreto`].
//! - NO HACE: no decide nada del flujo (eso es del instrumento), no escribe en disco, no registra
//!   nada en ningún sitio, y **no puede imprimir un cuerpo**: el cuerpo es un [`Secreto`], que no
//!   implementa `Display` y cuyo `Debug` dice «oculto».
//!
//! ## POR QUÉ ASÍ (medido el 2026-10-14, no recordado)
//!
//! - **`ureq` 3.4.2 con la función `cookies`** (autorizada por el dueño ese día). Bloqueante, sin
//!   `tokio`; TLS `rustls` + `rustls-webpki-roots` por defecto. **`native-tls` NO se enciende**: eso
//!   obligaría a OpenSSL en el equipo de quien compile, y este repositorio es público.
//! - La jarra la trae la función `cookies` (`ureq` 3.4.2 `Cargo.toml:70-73` → `cookie_store`), y
//!   **vive en memoria** dentro del `Agent` (`src/agent.rs:96` y `src/run.rs:194`): se devuelve sola
//!   en cada petición del mismo `Agent`. No hay que parsear `Set-Cookie` a mano para SOSTENER la
//!   sesión (sí para LEERLA, que es lo que hace el instrumento con `JWT_WEB`).
//! - **`http_status_as_error(false)`** (`ureq` `src/config.rs:491`; en ureq 3 un 4xx/5xx es un `Err`
//!   por defecto, `src/error.rs:13`). Aquí un 403, un 429 o un 500 **tienen que poder leerse**: un
//!   error de ureq no trae el estado, y el flujo necesita distinguir «clave mala» de «anti-abuso».
//!   **Eso es lo que se midió el 2026-10-14**: con credenciales falsas el POST del login contesta
//!   **500** con un cuerpo JSON, y el instrumento pudo decir «respondió 500 (un objeto JSON con 6
//!   claves …)» **sin volcar el cuerpo**. Con el comportamiento por defecto de ureq eso habría sido un
//!   `Err` sin estado y el mensaje no habría podido decir nada.
//! - **`max_redirects(0)`**: **no se siguen redirecciones**. Es lo que hacen las dos referencias
//!   (la librería `python-garminconnect` y el `gcexport.py` original: el instrumento lo cita en su
//!   `--ayuda`), y no es un detalle: en ureq 3 seguir una redirección de un POST **puede fallar**
//!   (`Error::RedirectFailed`, `src/run.rs:608`) y, si fallara, el error taparía el estado real que
//!   hace falta ver. Se pide, se mira el estado y se decide.
//! - **Esperas acotadas** (30 s para resolver el nombre, conectar, enviar y recibir): sin esto, una
//!   red que no contesta deja el instrumento colgado para siempre. Deliberadamente NO hay espera
//!   global: la espera anti-WAF del flujo (3-8 s) es del instrumento, y un tope global la contaría.
//!
//! ## LA REGLA DE HIGIENE, ESCRITA EN LOS TIPOS
//!
//! 1. El **cuerpo** de una respuesta es un [`Secreto`]: ni `Display` ni `Debug` pueden soltarlo, y
//!    [`Respuesta`] tiene el `Debug` escrito a mano para que un `{:?}` de descuido no lo vuelque (una
//!    respuesta de Garmin trae datos de salud del cliente).
//! 2. Un mensaje de error dice **la URL y el NOMBRE de los parámetros, nunca sus valores**
//!    ([`Destino`]): uno de esos valores es el ticket de servicio, que es una credencial de un solo
//!    uso. Es la regla que la propia librería portada aplica a sus errores (`client.py:196-210`,
//!    `_sanitize_exception_text`).
//! 3. Ningún texto de este módulo incluye el valor de una cabecera: de `Authorization` solo se puede
//!    decir el **esquema** ([`esquema_de_autorizacion`]).
//!
//! ## DÓNDE VIVE CADA COSA
//!
//! Lo que el instrumento NECESITA en producción es el cliente y su traducción de errores. Las
//! funciones PURAS que son la regla del módulo (`es_exito`, `revisar_estado`, `valores_de_cabecera`,
//! `codigos`) están marcadas `#[cfg(test)]`: **se comprueban en la suite** —que es exactamente lo
//! que pide el hito— y no se arrastran al binario, donde el flujo ya tiene las suyas. Así el
//! instrumento compila con **0 avisos** y no queda aquí una segunda tabla de estados que pueda
//! contradecir a la del flujo.

use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// =============================================================================================
// LAS CONSTANTES DE LA SESIÓN, CON SU FUENTE
// =============================================================================================

/// `client.py:165` — `NATIVE_API_USER_AGENT`. Es el agente de la app Android de Garmin: la API
/// interna espera este valor, no el de un navegador.
pub const AGENTE: &str = "GCM-Android-5.23";

/// `client.py:166-169` — `X-Garmin-User-Agent`, el segundo agente que manda la app.
pub const AGENTE_GARMIN: &str = "com.garmin.android.apps.connectmobile/5.23; ; Google/sdk_gphone64_arm64/google; Android/33; Dalvik/2.1.0";

/// `client.py:466-467` — `Origin` y `Referer` de la sesión por cookie (la web de Connect).
pub const ORIGEN_WEB: &str = "https://connect.garmin.com";

/// `client.py:468` — la cabecera `DI-Backend` de la sesión por cookie.
pub const DI_BACKEND: &str = "connectapi.garmin.com";

/// `client.py:473` — el valor de la cabecera `NK` en la reserva por cookie.
pub const NK: &str = "NT";

/// `client.py:450-457` — `Accept` de la API interna, en el camino con token.
pub const ACEPTA_JSON: &str = "application/json";

/// `client.py:450-473` — `Accept` del camino por cookie: es más ancho porque pasa por la web.
pub const ACEPTA_WEB: &str = "application/json,text/html;q=0.9,*/*;q=0.8";

/// La espera máxima de cada paso de red: resolver, conectar, enviar y recibir. **Es un tope para no
/// quedarse colgado, no una espera del flujo**: la espera anti-WAF (3-8 s) la hace el instrumento, y
/// por eso aquí NO hay espera global que la contaría.
pub const ESPERA_MAXIMA: Duration = Duration::from_secs(30);

// =============================================================================================
// LA CREDENCIAL, EN UN TIPO QUE NO SE PUEDE IMPRIMIR
// =============================================================================================

/// **Un texto que no se imprime.** No implementa `Display` —para que no pueda acabar en la salida por
/// descuido— y su `Debug` dice «oculto». Aquí se usa para el **cuerpo** de una respuesta (y el
/// instrumento lo usa además para la contraseña y para el cuerpo del POST del login).
#[derive(Clone)]
pub struct Secreto(String);

impl Secreto {
    pub fn nueva(valor: String) -> Self {
        Secreto(valor)
    }

    /// El valor, para los pocos sitios que de verdad lo necesitan: comparar, parsear, o enviarlo.
    /// Todo lo demás pasa por los tipos de arriba.
    pub fn expuesto(&self) -> &str {
        &self.0
    }

    pub fn esta_vacio(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Debug for Secreto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "«oculto»")
    }
}

// =============================================================================================
// LOS ERRORES DEL TRANSPORTE, TIPADOS Y CON SU CÓDIGO
// =============================================================================================

/// Los códigos de salida que el transporte puede devolver. **Son los mismos que los del instrumento**
/// (`src/bin/sincronizacion.rs`), porque son suyos: aquí se declaran para poder comprobar —en la
/// suite— que ninguno se ha movido, y así el dueño no lee un número que signifique otra cosa.
#[cfg(test)]
pub mod codigos {
    /// Todo salió bien.
    pub const BIEN: i32 = 0;
    /// Garmin respondió 429: anti-abuso.
    pub const DEMASIADAS_PETICIONES: i32 = 9;
    /// No hay red, o el servidor contestó un estado que no se puede leer.
    pub const SIN_RED: i32 = 6;
}

/// Un fallo del transporte.
#[derive(Debug)]
pub enum Fallo {
    /// Un estado HTTP que **no** es 2xx (4xx y 5xx se leen, no se convierten en error).
    Estado(u16),
    /// El anti-abuso de Garmin (429). Tiene su propio código: no es «sin red».
    DemasiadasPeticiones(u16),
    /// No se pudo hablar: DNS, conexión, TLS, espera agotada o protocolo.
    SinRed(String),
}

impl Fallo {
    pub fn codigo(&self) -> i32 {
        match self {
            Fallo::Estado(_) => 6,
            Fallo::DemasiadasPeticiones(_) => 9,
            Fallo::SinRed(_) => 6,
        }
    }

    pub fn mensaje(&self) -> String {
        match self {
            // MEDIDO el 2026-10-14: con credenciales falsas, el POST del login contesta 500 con un
            // cuerpo JSON. El mensaje dice el número y **nada del cuerpo** —que es lo correcto—, pero
            // se le añade el aviso de que «sin red» es el código de esta familia, para que nadie
            // confunda «Garmin contestó y dijo que no» con «no hubo conexión».
            Fallo::Estado(estado) => format!(
                "Garmin respondió {estado} (no es un 2xx). No se muestra el cuerpo: puede traer datos \
                 de salud.\n  OJO: esto NO es «no hay red» —hubo respuesta—, pero comparte su código \
                 de salida (6): el contrato de códigos es del instrumento y no se cambia por aquí."
            ),
            Fallo::DemasiadasPeticiones(estado) => format!(
                "Garmin respondió {estado}: demasiadas peticiones (anti-abuso).\n  Es lo que avisa la \
                 investigación §5.2: hay esperas deliberadas, y hay que ir despacio."
            ),
            // Se dice el motivo tal como lo da el sistema. **No se dice nada de la petición**: ni la
            // URL con valores ni una cabecera (ver `Destino`).
            Fallo::SinRed(porque) => format!("No se pudo hablar con Garmin: {porque}."),
        }
    }
}

/// Traduce un error de `ureq` a un fallo tipado. **Ninguna rama incluye el valor de una cabecera ni
/// el cuerpo de nada**: lo que se dice es el motivo del sistema, que es lo que hace falta para
/// distinguir «no hay red» de «el certificado no vale».
///
/// La lista de variantes está en `ureq` `src/error.rs:9-70` (versión 3.4.2).
fn de_ureq(e: ureq::Error) -> Fallo {
    match e {
        // `http_status_as_error(false)` NO convierte los estados en errores, así que esta rama es la
        // red de seguridad por si alguna vez alguien la enciende.
        ureq::Error::StatusCode(estado) => {
            if estado == 429 {
                Fallo::DemasiadasPeticiones(estado)
            } else {
                Fallo::Estado(estado)
            }
        }
        ureq::Error::Timeout(cual) => Fallo::SinRed(format!("se agotó la espera ({cual:?})")),
        ureq::Error::HostNotFound => {
            Fallo::SinRed("no se pudo resolver el nombre del servidor".to_string())
        }
        ureq::Error::ConnectionFailed => Fallo::SinRed("no se pudo abrir la conexión".to_string()),
        ureq::Error::RedirectFailed => Fallo::SinRed(
            "la respuesta era una redirección y no se siguen redirecciones (a propósito)".to_string(),
        ),
        // El error de E/S dice el motivo del sistema; no lleva credenciales.
        ureq::Error::Io(io) => Fallo::SinRed(format!("error de entrada/salida: {io}")),
        otro => Fallo::SinRed(format!("{otro}")),
    }
}

// =============================================================================================
// LO QUE SE PUEDE DECIR DE UNA PETICIÓN (SIN SUS VALORES)
// =============================================================================================

/// **Una petición dicha para un mensaje**: la URL y los NOMBRES de los parámetros.
///
/// Existe para que la regla de higiene sea una sola, en un solo sitio: un valor de consulta **no se
/// escribe nunca** (uno de ellos es el ticket de servicio, credencial de un solo uso), y una
/// cabecera tampoco (una de ellas es `Authorization`). El instrumento construye el mismo texto con
/// esta función, así que no hay dos maneras de decirlo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destino {
    pub url: String,
    pub parametros: Vec<String>,
}

impl Destino {
    pub fn de(url: &str, nombres: impl IntoIterator<Item = String>) -> Self {
        Destino {
            url: url.to_string(),
            parametros: nombres.into_iter().collect(),
        }
    }
}

impl fmt::Display for Destino {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.parametros.is_empty() {
            return write!(f, "{}", self.url);
        }
        let nombres: Vec<String> = self.parametros.iter().map(|k| format!("{k}=…")).collect();
        write!(f, "{}?{}", self.url, nombres.join("&"))
    }
}

/// De una cabecera `Authorization` se dice **el esquema, nunca el valor**: `Bearer …`. Se probó
/// primero sin esta función y un `Debug` de la lista de cabeceras soltaba la credencial entera.
pub fn esquema_de_autorizacion(valor: &str) -> String {
    match valor.split_once(' ') {
        Some((esquema, _)) => format!("{esquema} …"),
        None => "…".to_string(),
    }
}

// =============================================================================================
// LA RESPUESTA
// =============================================================================================

/// Lo que devuelve el transporte: **el estado, las cabeceras y el cuerpo**. El cuerpo es un
/// [`Secreto`] a propósito (una respuesta de Garmin trae datos de salud) y por eso el `Debug` está
/// escrito a mano: un `{:?}` de descuido dice el ESTADO y el TAMAÑO, nunca el contenido.
pub struct Respuesta {
    pub estado: u16,
    pub cabeceras: Vec<(String, String)>,
    pub cuerpo: Secreto,
}

impl fmt::Debug for Respuesta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Respuesta {{ estado: {}, {} cabeceras, cuerpo: {} }}",
            self.estado,
            self.cabeceras.len(),
            // Solo el tamaño: ni un valor, ni una clave del JSON.
            self.cuerpo.expuesto().len()
        )
    }
}

/// Un estado que el flujo SÍ puede leer: el transporte no lo convierte en error. Función pura, para
/// poder probarla sin red (ver «DÓNDE VIVE CADA COSA» en la cabecera).
#[cfg(test)]
pub fn es_exito(estado: u16) -> bool {
    (200..300).contains(&estado)
}

/// **El estado, traducido a lo que el flujo tiene que hacer con él.** Es la única regla del
/// transporte sobre los códigos, y es pura: por eso se prueba sin red.
///
/// - 2xx → `Ok`, se sigue.
/// - 429 → anti-abuso: parar y decirlo (código 9), que es distinto de «no hay red».
/// - 401/403 → credenciales o permiso: se devuelve el estado tal cual. En este flujo un 403 también
///   significa «`displayName` mal», y quien decide eso es el instrumento, no el transporte.
/// - Cualquier otro 4xx/5xx → `Fallo::Estado`, con el número a la vista.
#[cfg(test)]
pub fn revisar_estado(estado: u16) -> Result<(), Fallo> {
    if es_exito(estado) {
        return Ok(());
    }
    if estado == 429 {
        return Err(Fallo::DemasiadasPeticiones(estado));
    }
    Err(Fallo::Estado(estado))
}

/// El valor de una cabecera, buscada **sin distinguir mayúsculas** (las cabeceras HTTP no las
/// distinguen). Devuelve también las repetidas: `Set-Cookie` viene una vez por cookie.
#[cfg(test)]
pub fn valores_de_cabecera<'a>(cabeceras: &'a [(String, String)], nombre: &str) -> Vec<&'a str> {
    cabeceras
        .iter()
        .filter(|(n, _)| n.eq_ignore_ascii_case(nombre))
        .map(|(_, v)| v.as_str())
        .collect()
}

/// La cookie `JWT_WEB` de una respuesta, o `None`. **Es leerla, no sostenerla**: sostener la sesión
/// lo hace la jarra del `Agent`.
///
/// Hace falta porque es uno de los dos caminos del flujo (`client.py:1284-1306`): si el intercambio
/// del ticket por el token DI no sale, la sesión se sostiene con esta cookie, y el instrumento tiene
/// que verla para poder usar el camino de reserva. La librería portada la busca en `Set-Cookie`;
/// aquí se busca en las cabeceras ya recogidas, que es lo mismo sin volver a parsear.
pub fn cookie_jwt_web(cabeceras: &[(String, String)]) -> Option<String> {
    for (nombre, valor) in cabeceras {
        if nombre.eq_ignore_ascii_case("set-cookie") {
            if let Some(resto) = valor.strip_prefix("JWT_WEB=") {
                let fin = resto.find(';').unwrap_or(resto.len());
                let jwt = &resto[..fin];
                if !jwt.is_empty() {
                    return Some(jwt.to_string());
                }
            }
        }
    }
    None
}

// =============================================================================================
// LAS CABECERAS DEL FLUJO, COMO FUNCIONES PURAS
// =============================================================================================

/// `client.py:450-473` — **las cabeceras de la sesión de la API**, los dos caminos:
///
/// - **Con token DI**: `Authorization: Bearer <token>` + `Accept: application/json`.
/// - **Sin token (reserva)**: `NK`, `Origin`, `Referer` de la web y `DI-Backend`, más la cookie
///   `JWT_WEB` si se tiene.
///
/// En los dos caminos van los agentes nativos (`client.py:165-169`).
///
/// **El token NO se puede perder de vista**: el `Debug` de esta lista de cabeceras soltaría el
/// `Bearer`. Por eso el `Authorization` se construye aquí y en ningún sitio más, y para imprimir una
/// petición se usa `esquema_de_autorizacion` (o, mejor, no se imprime).
///
/// La usa el FLUJO del instrumento, que la nombra desde su `mod flujo`: una sola definición, para que
/// el día que Garmin cambie una de estas cabeceras no haya dos listas que se contradigan.
pub fn cabeceras_de_sesion(token: Option<&str>, jwt_web: Option<&str>) -> Vec<(String, String)> {
    let mut h = vec![
        ("User-Agent".to_string(), AGENTE.to_string()),
        ("X-Garmin-User-Agent".to_string(), AGENTE_GARMIN.to_string()),
    ];
    if let Some(token) = token {
        h.push(("Accept".to_string(), ACEPTA_JSON.to_string()));
        h.push(("Authorization".to_string(), format!("Bearer {token}")));
        return h;
    }
    // `client.py:450-473`, camino de reserva, en el orden de la librería.
    h.push(("Accept".to_string(), ACEPTA_WEB.to_string()));
    h.push(("NK".to_string(), NK.to_string()));
    h.push(("Origin".to_string(), ORIGEN_WEB.to_string()));
    h.push(("Referer".to_string(), format!("{ORIGEN_WEB}/modern/")));
    h.push(("DI-Backend".to_string(), DI_BACKEND.to_string()));
    if let Some(jwt) = jwt_web {
        h.push(("Cookie".to_string(), format!("JWT_WEB={jwt}")));
    }
    h
}

/// `client.py:795-807` — las cabeceras del POST del login. Además del `Content-Type` (que lo pone el
/// cliente al enviar un formulario, `ureq` `src/request.rs:600-602`), la librería manda un `Referer`
/// con la propia página del widget: es parte del flujo del SSO, no un adorno.
///
/// Está `#[cfg(test)]` porque el POST del login del instrumento construye su `Referer` al lado de la
/// petición (donde se lee de un tirón qué se manda y con qué), y esta función es la que la suite
/// compara contra esa lista.
#[cfg(test)]
pub fn cabeceras_del_login() -> Vec<(String, String)> {
    vec![
        (
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        ),
        (
            "Referer".to_string(),
            "https://sso.garmin.com/sso/embed".to_string(),
        ),
    ]
}

// =============================================================================================
// EL TRANSPORTE
// =============================================================================================

/// **El cliente bloqueante con su jarra de cookies EN MEMORIA.**
///
/// Un `Transporte` = un `ureq::Agent` (que trae la jarra cuando está encendida la función `cookies`)
/// + el reloj de la última petición, que solo sirve para poder decir **cuánto tardó la red** sin
/// contar la espera anti-WAF del flujo.
pub struct Transporte {
    agente: ureq::Agent,
    /// Cuándo empezó el último paso de red. Va en un `Mutex` (y no en un `Cell`) solo para que el
    /// transporte siga siendo `Sync`, como el `Agent` que envuelve: no se comparte entre hilos hoy,
    /// pero no hay razón para cerrarle la puerta.
    ultima: Mutex<Option<Instant>>,
    /// **Lo que llevaba la última petición al salir**, medido dentro del propio cliente: lo llena el
    /// `middleware` de `ureq`, que corre justo ANTES de que el cliente añada por su cuenta el
    /// `User-Agent`, la `Cookie` de la jarra, el `Accept`, el `Accept-Encoding` y el
    /// `Content-Length` (`ureq` `src/run.rs:272-350`, dentro de `add_headers`).
    ///
    /// Existe porque hay cosas que **no se pueden suponer leyendo el flujo**: un `Content-Type` que
    /// no sale no se nota en el código del paso —lo puso otro, y otro lo puede haber borrado—, y el
    /// 500 medido contra Garmin el 2026-10-14 salió exactamente de ahí. Se mide donde se pone.
    salida: Arc<Mutex<Salida>>,
}

/// **Lo que llevaba una petición al salir de aquí.** Los NOMBRES de sus cabeceras y, aparte, el
/// `Content-Type`: es el único valor de cabecera que este proyecto necesita poder decir, y decirlo no
/// es un riesgo —es una constante pública del protocolo, no un dato del cliente—.
///
/// `medido` existe por una razón medida: **«no lo he mirado» y «lo he mirado y venía vacío» no son lo
/// mismo**, y confundirlos hace que un informe diga «este transporte no mide las cabeceras» sobre un
/// transporte que sí las mide (le pasó al primer rastreo, el 2026-10-14: los dos GET del SSO no
/// llevan cabeceras propias y el rastro lo contó como que no se habían medido).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Salida {
    pub medido: bool,
    pub nombres: Vec<String>,
    pub content_type: Option<String>,
}

impl fmt::Debug for Transporte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Ni la configuración ni la jarra: solo que existe y cuántas cookies lleva.
        write!(
            f,
            "Transporte {{ cliente: ureq 3.4.2, cookies en memoria: {} }}",
            self.cuantas_cookies()
        )
    }
}

impl Transporte {
    /// Un transporte nuevo, con la jarra **vacía y en memoria**. No toca la red: se puede construir
    /// sin credenciales y sin conexión (lo hacen las pruebas).
    pub fn nuevo() -> Self {
        let salida: Arc<Mutex<Salida>> = Arc::new(Mutex::new(Salida::default()));
        let visto = Arc::clone(&salida);
        let config = ureq::Agent::config_builder()
            // Un 4xx/5xx NO es un error: se lee. El flujo tiene que distinguir 403 de 429.
            .http_status_as_error(false)
            // No se siguen redirecciones (ver la cabecera del módulo: es lo que hacen las dos
            // referencias, y evita que `RedirectFailed` tape el estado real).
            .max_redirects(0)
            .timeout_resolve(Some(ESPERA_MAXIMA))
            .timeout_connect(Some(ESPERA_MAXIMA))
            .timeout_send_request(Some(ESPERA_MAXIMA))
            .timeout_send_body(Some(ESPERA_MAXIMA))
            .timeout_recv_response(Some(ESPERA_MAXIMA))
            .timeout_recv_body(Some(ESPERA_MAXIMA))
            // El mirón: **no cambia la petición**, solo apunta sus cabeceras antes de que el cliente
            // añada las suyas. Es de solo lectura a propósito —un middleware puede modificar la
            // petición, y este no lo hace: si algún día lo hiciera, dejaría de medir lo que el flujo
            // manda y pasaría a mandar él—.
            .middleware(
                move |peticion: ureq::http::Request<ureq::SendBody>,
                      siguiente: ureq::middleware::MiddlewareNext| {
                    let mut nombres = Vec::new();
                    let mut content_type = None;
                    for (nombre, valor) in peticion.headers() {
                        let nombre = nombre.as_str().to_ascii_lowercase();
                        if nombre == "content-type" {
                            content_type = valor.to_str().ok().map(|v| v.to_string());
                        }
                        nombres.push(nombre);
                    }
                    nombres.sort();
                    nombres.dedup();
                    if let Ok(mut sitio) = visto.lock() {
                        *sitio = Salida {
                            medido: true,
                            nombres,
                            content_type,
                        };
                    }
                    siguiente.handle(peticion)
                },
            )
            .build();
        Transporte {
            agente: ureq::Agent::new_with_config(config),
            ultima: Mutex::new(None),
            salida,
        }
    }

    /// **Los nombres de las cabeceras que llevaba la última petición, y su `Content-Type` si lo
    /// llevaba.** Es lo que el instrumento necesita para poder decir «el `Content-Type` no salió»
    /// como un hecho medido y no como una lectura del código.
    pub fn salida(&self) -> Salida {
        self.salida
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|e| e.into_inner().clone())
    }

    /// Cuántas cookies hay en la jarra. **Para poder afirmar que la sesión se sostiene**: es la
    /// prueba de que el login dejó cookies y de que el paso siguiente las va a devolver.
    pub fn cuantas_cookies(&self) -> usize {
        self.agente.cookie_jar_lock().iter().count()
    }

    /// GET con parámetros de consulta. Devuelve el estado y el cuerpo, pase lo que pase con el
    /// estado: los 4xx/5xx **se leen**.
    pub fn get(
        &self,
        url: &str,
        consulta: &[(String, String)],
        cabeceras: &[(String, String)],
    ) -> Result<Respuesta, Fallo> {
        let mut peticion = self.agente.get(url).query_pairs(consulta.to_vec());
        for (nombre, valor) in cabeceras {
            peticion = peticion.header(nombre, valor);
        }
        self.anota_reloj();
        let respuesta = peticion.call().map_err(de_ureq)?;
        Ok(self.recoge(respuesta))
    }

    /// POST de formulario. **El cuerpo ya viene codificado** —y es un `Secreto`, porque el del login
    /// lleva la contraseña—: aquí solo se envía.
    ///
    /// Se manda con `.send(cuerpo)` y no con `.send_form(...)` a propósito: `send_form` volvería a
    /// codificar los valores, y este flujo ya los codifica una sola vez, en el instrumento (que es
    /// donde está probado que `&`, `+`, `=` y `%` no rompen el formulario en silencio).
    pub fn post_form(
        &self,
        url: &str,
        cuerpo: &Secreto,
        cabeceras: &[(String, String)],
    ) -> Result<Respuesta, Fallo> {
        let mut peticion = self.agente.post(url);
        for (nombre, valor) in cabeceras {
            peticion = peticion.header(nombre, valor);
        }
        self.anota_reloj();
        let respuesta = peticion
            .send(cuerpo.expuesto().to_string())
            .map_err(de_ureq)?;
        Ok(self.recoge(respuesta))
    }

    /// **Cuánto tardó el último paso de red.** Es el número que el instrumento necesita para
    /// responder «¿cuánto tarda?» sin contarle la espera anti-WAF, que es espera del flujo.
    pub fn ultima_tardanza(&self) -> Option<Duration> {
        let cuando = *self.ultima.lock().unwrap_or_else(|e| e.into_inner());
        cuando.map(|t| t.elapsed())
    }

    fn anota_reloj(&self) {
        *self.ultima.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
    }

    /// Traduce la respuesta de `ureq` a la del instrumento: estado, cabeceras y cuerpo **como
    /// `Secreto`**. Una cabecera cuyo valor no sea UTF-8 válido se cae en vez de mentir: no se
    /// inventa un valor, y desde luego no se imprime.
    fn recoge(&self, respuesta: ureq::http::Response<ureq::Body>) -> Respuesta {
        let estado = respuesta.status().as_u16();
        let cabeceras = respuesta
            .headers()
            .iter()
            .filter_map(|(n, v)| Some((n.as_str().to_string(), v.to_str().ok()?.to_string())))
            .collect();
        let mut cuerpo = respuesta.into_body();
        // `read_to_string` no falla por bytes no válidos: `ureq` los sustituye (pérdida deliberada,
        // `src/body/mod.rs:292` y `body/lossy.rs`). El flujo lee JSON o comprueba formas; un byte
        // raro tiene que producir «el endpoint ha cambiado», no un error de red.
        let texto = cuerpo.read_to_string().unwrap_or_default();
        Respuesta {
            estado,
            cabeceras,
            cuerpo: Secreto::nueva(texto),
        }
    }
}

/// **El transporte que NO hay: el hueco, declarado.** Existe para que el caso «no hay cliente HTTP»
/// —que antes era el error 8— tenga una salida tipada y no un pánico. `get` con él devuelve el
/// fallo, nunca una respuesta inventada.
///
/// Está `#[cfg(test)]` porque **hoy solo lo usa la prueba** de abajo: en el camino de red el
/// instrumento se queda con el fallo tipado de `Cliente`, no con este hueco. Medido al limpiar los
/// avisos (2026-10-14): fuera de las pruebas, nadie lo construía.
#[cfg(test)]
pub struct SinTransporte;

#[cfg(test)]
impl SinTransporte {
    pub fn get(
        &self,
        _url: &str,
        _consulta: &[(String, String)],
        _cabeceras: &[(String, String)],
    ) -> Result<Respuesta, Fallo> {
        Err(Fallo::SinRed(
            "no hay cliente HTTP: el hueco `SIN_CLIENTE_HTTP` del instrumento".to_string(),
        ))
    }
}

// =============================================================================================
// PRUEBAS: TODO LO QUE SE PUEDE PROBAR SIN RED (el flujo con credenciales reales es del dueño)
// =============================================================================================

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Una credencial falsa y reconocible: si aparece en un mensaje, es que se ha colado.
    const FALSA: &str = "CLAVE-FALSA-XYZ-1234";

    #[test]
    fn el_transporte_se_construye_sin_red_y_con_la_jarra_vacia() {
        let t = Transporte::nuevo();
        assert_eq!(t.cuantas_cookies(), 0, "la jarra empieza vacía y en memoria");
        assert!(t.ultima_tardanza().is_none(), "todavía no ha pedido nada");
    }

    #[test]
    fn sin_token_la_sesion_va_por_cookie_y_lo_dice() {
        let h = cabeceras_de_sesion(None, Some("JWT-FALSO"));
        assert!(h.iter().any(|(k, v)| k == "NK" && v == NK));
        assert!(h.iter().any(|(k, v)| k == "DI-Backend" && v == DI_BACKEND));
        assert!(h.iter().any(|(k, v)| k == "Origin" && v == ORIGEN_WEB));
        assert!(h
            .iter()
            .any(|(k, v)| k == "Referer" && v == "https://connect.garmin.com/modern/"));
        assert!(h.iter().any(|(k, v)| k == "Cookie" && v == "JWT_WEB=JWT-FALSO"));
        assert!(!h.iter().any(|(k, _)| k == "Authorization"));
        // Los dos agentes nativos van SIEMPRE (`client.py:165-169`).
        assert!(h.iter().any(|(k, v)| k == "User-Agent" && v == AGENTE));
        assert!(h.iter().any(|(k, _)| k == "X-Garmin-User-Agent"));
    }

    #[test]
    fn con_token_la_sesion_va_con_bearer_y_no_lleva_las_de_la_web() {
        let h = cabeceras_de_sesion(Some("TOKEN-FALSO"), Some("JWT-FALSO"));
        assert!(h
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer TOKEN-FALSO"));
        assert!(h.iter().any(|(k, v)| k == "Accept" && v == ACEPTA_JSON));
        // Con token NO se manda la sesión por cookie: son dos caminos distintos (`client.py:450-473`).
        assert!(!h.iter().any(|(k, _)| k == "Cookie"));
        assert!(!h.iter().any(|(k, _)| k == "DI-Backend"));
        assert!(!h.iter().any(|(k, _)| k == "NK"));
    }

    #[test]
    fn el_login_manda_el_referer_del_widget_y_el_tipo_de_formulario() {
        let h = cabeceras_del_login();
        assert!(h
            .iter()
            .any(|(k, v)| k == "Referer" && v == "https://sso.garmin.com/sso/embed"));
        assert!(h
            .iter()
            .any(|(k, v)| k == "Content-Type" && v == "application/x-www-form-urlencoded"));
    }

    #[test]
    fn de_una_autorizacion_solo_se_dice_el_esquema() {
        let dicho = esquema_de_autorizacion(&format!("Bearer {FALSA}"));
        assert_eq!(dicho, "Bearer …");
        assert!(!dicho.contains(FALSA), "{dicho}");
        assert_eq!(esquema_de_autorizacion("Basic abc"), "Basic …");
        assert_eq!(esquema_de_autorizacion("sin-esquema"), "…");
    }

    #[test]
    fn la_respuesta_se_puede_imprimir_sin_volcar_el_cuerpo() {
        let r = Respuesta {
            estado: 200,
            cabeceras: vec![("Set-Cookie".to_string(), "JWT_WEB=abc".to_string())],
            cuerpo: Secreto::nueva(r#"{"datoDeSalud":"secreto del cliente"}"#.to_string()),
        };
        let texto = format!("{r:?}");
        assert!(!texto.contains("secreto del cliente"), "se volcó: {texto}");
        assert!(!texto.contains("datoDeSalud"), "{texto}");
        assert!(texto.contains("estado: 200"), "{texto}");
        // Y el `Debug` de un cuerpo, solo, tampoco lo suelta.
        assert_eq!(format!("{:?}", r.cuerpo), "«oculto»");
    }

    #[test]
    fn un_cuerpo_secreto_no_se_puede_imprimir_ni_con_display_ni_con_debug() {
        // `Secreto` NO implementa `Display`: la única forma de sacarlo es `.expuesto()`, que se ve
        // en el código al leerlo. Esto se comprueba con el `Debug`.
        let s = Secreto::nueva(FALSA.to_string());
        assert_eq!(format!("{s:?}"), "«oculto»");
        assert!(!format!("{s:?}").contains(FALSA));
        assert_eq!(s.expuesto(), FALSA);
        assert!(Secreto::nueva("  ".to_string()).esta_vacio());
        assert!(!Secreto::nueva(FALSA.to_string()).esta_vacio());
    }

    #[test]
    fn los_estados_se_clasifican_igual_que_en_el_flujo() {
        // 2xx: se sigue.
        for estado in [200, 201, 204, 299] {
            assert!(revisar_estado(estado).is_ok(), "{estado}");
            assert!(es_exito(estado), "{estado}");
        }
        // 429: anti-abuso, con su propio código (no es «sin red»).
        let f = revisar_estado(429).unwrap_err();
        assert_eq!(f.codigo(), codigos::DEMASIADAS_PETICIONES);
        assert!(f.mensaje().contains("anti-abuso"), "{}", f.mensaje());
        // 403: se dice el número; quien decide el flujo es el instrumento.
        let f = revisar_estado(403).unwrap_err();
        assert!(matches!(f, Fallo::Estado(403)));
        assert_eq!(f.codigo(), codigos::SIN_RED);
        assert!(f.mensaje().contains("403"), "{}", f.mensaje());
        // 500 y 302 también se leen y se dicen.
        assert!(revisar_estado(500).is_err());
        assert!(revisar_estado(302).is_err(), "una redirección no es un éxito");
        assert!(!es_exito(302));
    }

    #[test]
    fn el_mensaje_de_un_estado_no_lleva_el_cuerpo() {
        // MEDIDO el 2026-10-14: con credenciales falsas el login contesta 500 con un cuerpo JSON. El
        // mensaje dice el número y nada más.
        let f = revisar_estado(500).unwrap_err();
        assert!(f.mensaje().contains("500"), "{}", f.mensaje());
        assert!(
            f.mensaje().contains("datos de salud"),
            "tiene que decir POR QUÉ no se muestra: {}",
            f.mensaje()
        );
        // Y avisa de lo que se midió: un estado NO es «no hay red», aunque comparta código.
        assert!(f.mensaje().contains("NO es «no hay red»"), "{}", f.mensaje());
    }

    #[test]
    fn el_destino_dice_los_nombres_de_los_parametros_y_ningun_valor() {
        // Uno de esos valores es el ticket de servicio: una credencial de un solo uso.
        let d = Destino::de(
            "https://sso.garmin.com/sso/embed",
            vec!["ticket".to_string(), "id".to_string()],
        );
        let dicho = d.to_string();
        assert_eq!(dicho, "https://sso.garmin.com/sso/embed?ticket=…&id=…");
        assert!(!dicho.contains("ST-"), "{dicho}");
        assert!(!dicho.contains(FALSA), "{dicho}");
        // Sin parámetros, solo la URL.
        assert_eq!(
            Destino::de("https://connectapi.garmin.com/x", Vec::new()).to_string(),
            "https://connectapi.garmin.com/x"
        );
    }

    #[test]
    fn la_cookie_jwt_web_se_lee_de_las_cabeceras_y_no_se_confunde() {
        let cabeceras = vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            (
                "set-cookie".to_string(),
                "CASTGC=TGT-123; Path=/; Secure".to_string(),
            ),
            (
                "Set-Cookie".to_string(),
                "JWT_WEB=eyJhbGciOi.abc.def; Path=/; Secure".to_string(),
            ),
        ];
        assert_eq!(
            cookie_jwt_web(&cabeceras).as_deref(),
            Some("eyJhbGciOi.abc.def")
        );
        // Sin la cookie, no hay reserva: `None`, no un valor inventado.
        assert!(cookie_jwt_web(&[]).is_none());
        assert!(
            cookie_jwt_web(&[("Set-Cookie".to_string(), "JWT_WEB=; Path=/".to_string())]).is_none()
        );
        // Y las cabeceras repetidas se ven todas (Set-Cookie viene una vez por cookie).
        assert_eq!(valores_de_cabecera(&cabeceras, "SET-COOKIE").len(), 2);
    }

    #[test]
    fn un_destino_que_no_existe_da_un_fallo_de_red_y_no_un_panico() {
        // La prueba NO habla con Garmin: pide a un puerto cerrado de la propia máquina. Lo que se
        // comprueba es que un fallo de red se convierte en un error TIPADO, sin pánico.
        let t = Transporte::nuevo();
        let fallo = t
            .get("http://127.0.0.1:9/nada", &[], &[])
            .expect_err("el puerto 9 no escucha");
        assert_eq!(fallo.codigo(), codigos::SIN_RED);
        assert!(fallo.mensaje().contains("Garmin"), "{}", fallo.mensaje());
        assert!(
            t.ultima_tardanza().is_some(),
            "el reloj del último paso de red tiene que quedar puesto aunque falle"
        );
    }

    #[test]
    fn los_codigos_del_transporte_no_chocan_con_los_del_instrumento() {
        // El transporte declara los códigos que el instrumento ya tenía: si alguien mueve uno, esta
        // prueba lo dice antes de que el dueño lea un código equivocado en su terminal.
        assert_eq!(codigos::BIEN, 0);
        assert_eq!(codigos::SIN_RED, 6);
        assert_eq!(codigos::DEMASIADAS_PETICIONES, 9);
        assert_eq!(Fallo::Estado(500).codigo(), codigos::SIN_RED);
        assert_eq!(
            Fallo::DemasiadasPeticiones(429).codigo(),
            codigos::DEMASIADAS_PETICIONES
        );
    }

    #[test]
    fn el_hueco_sin_transporte_se_declara_y_no_se_rellena_con_un_remedio_peor() {
        let fallo = SinTransporte
            .get("https://sso.garmin.com/sso/embed", &[], &[])
            .unwrap_err();
        assert!(
            fallo.mensaje().contains("SIN_CLIENTE_HTTP"),
            "{}",
            fallo.mensaje()
        );
    }
}
