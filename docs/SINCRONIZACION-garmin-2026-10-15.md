# Sincronizar con Garmin: la vía, MEDIDA (2026-10-15)

**Escrito el 2026-10-15.** Documenta lo que se **midió** en el chat de ese día, con el dueño
**dentro del navegador** de un equipo Kali. Sustituye a las hipótesis del 2026-10-14 (huella TLS,
`_cfuvid`, `wreq`): todas quedaron **descartadas por medida**, y aquí está el porqué.

> **Estado del contrato**: `CONTRATO-garmin-sync.md` sigue vigente y **no cambia** — esta vía ES su
> «vía 1 · API interna». Lo que se precisa aquí es **por dónde pasa**: un motor de navegador.
> **La decisión de llevarlo a la aplicación es del dueño y está PENDIENTE.**

---

## 1. El acceso: lo que Garmin sirve hoy (no lo que suponíamos)

| qué | medido |
|---|---|
| Página de acceso | `https://sso.garmin.com/**portal**/sso/en-US/sign-in?clientId=GarminConnect&service=…` |
| Qué es | Una **aplicación JavaScript** (`index-d10b89ff39a6804af60e44186d1f4230.js`): **sin `<form action>`, sin `<input type=hidden>`, sin `_csrf`** |
| Qué exige | **reCAPTCHA Enterprise** (`enterprise.js?render=explicit`) — al entrar salió **reto de imágenes** («selecciona los cuadros con autobuses») |
| Consecuencia | El acceso **necesita un motor de navegador y una persona**. No es un defecto nuestro: es el diseño de Garmin |

El instrumento del repo (`motor/src/bin/sincronizacion.rs`) habla con `/sso/signin` (flujo
widget/embed) y busca `_csrf` en el HTML. **Ese HTML ya no existe**: sin JavaScript no hay formulario.

## 2. Lo que se probó y NO funciona (para no volver a pagarlo)

| vía probada | resultado | quién responde |
|---|---|---|
| `connectapi.garmin.com` + cookie `JWT_WEB` (la vía de reserva del instrumento) | **403** | ese host quiere el token DI del móvil |
| `/gc-api/` con el cliente HTTP de Playwright (`page.request`) | **403** | no es el motor del navegador |
| `/gc-api/` con `curl_cffi` **chrome** (huella de Chrome medida: `h2 t13d1516h2_…`) | **403** | `server: cloudflare` |
| `/gc-api/` con `curl_cffi` sin disfraz | **403** | `server: cloudflare` |
| `/gc-api/` con **`curl` de serie** (HTTP/1.1, lo mismo que `ureq`) | **403** | `server: cloudflare` |
| `fetch` **dentro** de la página **sin** las cabeceras de la app | **403** | control |
| **`fetch` dentro de la página CON las cabeceras** | **200 · 46.345 B** | ✅ |

**La conclusión, que es lo que importa:**

1. Las cabeceras de la app son **necesarias pero NO suficientes**.
2. Cloudflare ata la sesión al **transporte del propio navegador**: la cookie `cf_clearance` solo
   vale para quien la ganó ejecutando el reto.
3. **Imitar la huella de Chrome no sirve** (probado). Por tanto **`wreq`/BoringSSL queda descartado
   por medida**: se ahorra la obra nativa (cmake, Tokio, BoringSSL) y no hace falta romper el
   compromiso del repo. El dictamen previo (`datos/trabajo-ui/wreq-informe.md`) queda superado.
4. La pista del `_cfuvid` del 2026-10-14 **no era la causa**: un navegador real también la recibe.

## 3. La vía que SÍ funciona

La propia aplicación web de Garmin, visto en **456 peticiones suyas**:

| lo que hace la app | veces | estado |
|---|---|---|
| `POST https://connect.garmin.com/gc-api/graphql-gateway/graphql` | 27 | 200 |
| `GET /gc-api/web-gateway/snapshot/usageIndicators` · `/gc-api/userprofile-service/…` · `/gc-api/activitylist-service/…` · `/gc-api/device-service/…` | 52 rutas distintas | 200 |
| llamadas a `connectapi.garmin.com` | **0** | — |
| peticiones con `Authorization: Bearer` | **0** | — |

Las peticiones buenas llevan, además de las cookies:

```
connect-csrf-token · nk: NT · x-app-ver · x-lang · x-requested-with · referer · sec-ch-ua*
```

**Cómo se usa**: un `fetch` desde **dentro** de la página (misma pestaña, mismo origen), que es lo
que hace el instrumento `herramientas/garmin-lab/datos.py`. Ver §5.

## 4. Los datos que trae (y una pregunta del contrato, respondida)

Nueve conjuntos traídos de verdad por esa vía (crudos, en `datos/garmin-gcapi/`, **ignorados por git**):

| conjunto | tamaño | claves reales |
|---|---|---|
| **sueño** | 652 B | `dailySleepDTO` (`calendarDate`, `deepSleepSeconds`, `remSleepSeconds`, `lightSleepSeconds`, `napTimeSeconds`) · `sleepLevels` · `sleepMovement` |
| **estrés** | 316 B | **`avgStressLevel`** · `maxStressLevel` · `stressValuesArray` · `calendarDate` |
| **training status** | 2.484 B | `mostRecentTrainingStatus` · **`mostRecentVO2Max`** · `mostRecentTrainingLoadBalance` · `heatAltitudeAcclimationDTO` |
| pulso en reposo | 393 B | `allMetrics.metricsMap` → `WELLNESS_RESTING_HEART_RATE` (**con `value: null`** en los días mirados) |
| resumen del día | 2.384 B | `activeKilocalories` · `activeSeconds` · `averageMonitoringEnvironmentAltitude`… |
| dispositivos | 62.815 B | lista de **3** |
| perfil | 3.006 B | visibilidad de actividades y ajustes de cuenta |
| GraphQL (la consulta de la app) | 25.388 B | reenviada tal cual, 200 |

**`CONTRATO-garmin-sync.md` §5.1, RESPONDIDA**: la pregunta era si los diarios vienen **llenos** por
la API interna, porque en la **exportación oficial venían VACÍOS**. Medido: **sí vienen con dato**
(el sueño trae sus fases y el estrés trae `avgStressLevel`). La «vía 1» del contrato queda
justificada con medida.

## 5. Fallos concretos encontrados al comparar el dato real con el código

1. **El estrés real es `avgStressLevel`; el código espera `averageStressLevel`** (16 referencias en
   `motor/src/sincronizar.rs`, 7 en `motor/src/bin/sincronizacion.rs`). No es un descuido: se portó de
   la librería del **móvil** (`connectapi.garmin.com`), donde el campo se llama así; la **web**
   (`/gc-api/`) lo llama distinto. **Con el parser de hoy, el estrés no se leería del dato real.**
2. `dailySleepDTO` y `allMetrics.metricsMap` **sí** coinciden con lo que el código espera.
3. El pulso en reposo llega con **`value: null`**: el camino «sin dato» del parser **se va a usar**.

*(Los dos primeros se arreglan en un hito aparte, contra los JSON reales, con sus pruebas.)*

## 6. Consecuencia: hace falta un motor de navegador

De lo medido se sigue, sin margen de opinión: **el acceso y las llamadas a los datos tienen que
correr dentro de un motor de navegador** — en Android, un **WebView**. El cliente entra **una vez**
(con su reCAPTCHA, inevitable) y desde ahí el propio WebView pide `/gc-api/`.

**Lo que queda por medir ANTES de aprobarlo**: si un WebView **cabe** en la aplicación
(Rust + Slint, Android). Si no cupiera, el plan cambia. Eso es lo primero del siguiente chat.

## 7. Cómo reproducirlo (instrumentos)

En `herramientas/garmin-lab/` (copias de trabajo en el Kali, `~/garmin-lab/`):

| orden | instrumento | qué hace |
|---|---|---|
| 1 | `sonda.py` | sonda **sin credenciales**: comprueba el navegador, la página y las cookies |
| 2 | `portal.py` | inspecciona el portal y compara lo que recibe un cliente programático |
| 3 | `entrar.py` | abre el navegador y **el dueño entra a mano** (o `GARMIN_AUTO=1`) |
| 4 | `sesion.py` | saca el material de sesión y comprueba el estado |
| 5 | `mirar.py` | apunta el tráfico real de la app y prueba un `fetch` interno |
| 6 | `dentro.py` | **separa cabeceras de transporte**: es el que da el veredicto del §2 |
| 7 | `datos.py` | **trae los datos** por la vía que funciona |

Protocolo de secretos, sin excepciones: las credenciales se teclean **en la página de Garmin** y no
pasan por el chat ni por el repo; lo que se guarda en disco (cookies, cabeceras, token) va con
permisos **600** y **nunca se imprime**.
