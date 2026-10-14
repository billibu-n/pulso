# ESTADO: dónde estamos y por dónde se sigue

**Escrito el 2026-10-13**, y **actualizado el mismo día**, porque el contexto se estaba llenando y **una compactación a mitad de tarea
es catastrófica**. Este fichero existe para que **quien retome no tenga que reconstruir nada**: dice qué
está hecho, qué está medido, **qué falta**, y **dónde está cada cosa**.

> **Si acabas de llegar: lee esto entero antes de tocar nada.** Y después
> `docs/RECORRIDO-del-usuario.md`, que es el plan.

---

## 1. Qué es este proyecto, en tres frases

**Un motor que lee datos de entrenamiento, calcula, y dice qué se puede afirmar — y qué no.**
El cliente es **un ciclista de 60 años** que hasta ahora mandaba capturas a un modelo de lenguaje.
**La aplicación es el motor con una cara**, en **Rust + Slint**, y tiene que llegar a **su teléfono**.

## 2. Qué está HECHO y VERIFICADO

| qué | estado | dónde |
|---|---|---|
| **el motor** | **188 pruebas en verde**, cero avisos | `motor/src/` |
| **la biomecánica** | fase de pedaleo **por pierna** (difieren en 80 de 83), potencia/cadencia **sentado·de pie**, respiración, carga | `motor/src/bin/biomecanica.rs` |
| **lo que calcula** | **625 resultados** con su margen y su fuente | `motor/src/bin/inventario.rs` |
| **el informe** | por plantillas, reproducible, con guarda anti-invención | `motor/src/informe.rs` |
| **la interfaz** | existe y **se MIDE**: 411x880, cabecera 56, nav 64, huecos repartidos | `motor/ui/` |
| **los dos temas** | claro cálido y **negro OLED**, con contraste medido | `motor/ui/temas.slint` + `temas/contraste.py` |
| **los iconos** | **187** de Lucide, licencia ISC respetada | `motor/ui/iconos.slint` + `herramientas/iconos-a-slint.py` |
| **Android** | **comprobado**: binario ARM aarch64 para Android | ver `docs/SLINT-…`, §10.6 |
| **la norma** | **82 reglas**, 15 patrones de fallo | `~/.dsh/skills/norma-app/SKILL.md` |
| **los controles** | **DECIDIDO**: los de serie, con su precio medido | `docs/SLINT-…` §12 · `motor/ui/sonda-borde.slint` |
| **8 pruebas de medición** | 6 de geometría + **2 de los controles de serie** | `cargo test --test medir-interfaz` |
| **11 pruebas del RECORRIDO** | recorren los pasos del usuario y **exigen que sin datos NO se enseñe ningún número** | `cargo test --test recorrido` |
| **el encargo de Farellones** | medido punto por punto: qué se puede y qué no | `docs/ENCARGO-farellones-2026.md` |
| **la memoria entre arranques** | **5,40 s -> 0,001 s** (6.586x), con el resultado idéntico | `motor/src/memoria.rs` + `motor/src/analisis.rs` |

## 3. Qué FALTA, en orden, y por qué ese orden

**Los ocho pasos del recorrido del usuario, medidos: dos cumplen ya, uno a medias y cinco que faltan.**
El detalle está en `docs/RECORRIDO-del-usuario.md`. El resumen:

| # | el paso | qué falta |
|---|---|---|
| ~~**1**~~ | ~~**la decisión de los controles**~~ | **HECHO el 2026-10-13**: se usan **los de serie**. Con 4 reglas medidas (§12 del doc de Slint) y **una obligatoria**: `Palette.color-scheme` se fija desde nuestro tema. |
| **2** | una **puerta de entrada única** | hoy hay 13 binarios de desarrollo y ninguno se llama «la aplicación» |
| **3** | **los datos de la persona, editables** | hoy están **escritos en el código**: los resultados **no son del cliente** |
| **4** | **el primer arranque sin datos** | hoy dice `169±3` **y** «sin datos» a la vez: **se contradice** |
| **5** | **pulsar para analizar** | hoy calcula al abrir, y **tarda 5,4 s sin decir nada** |
| **6** | **vincular la cuenta** | no existe. Y necesita **credenciales de Garmin** |
| ~~**7**~~ | ~~**recordar entre arranques**~~ | **HECHO el 2026-10-13**: medido **5,40 s -> 0,001 s**. Falta conectarlo al abrir (lo hace la otra tarea en curso). |

## 4. Dónde está cada cosa (para no buscarla)

```
~/projects/pulso/                    ← ESTE ES EL REPOSITORIO. Público, sin datos de salud.
├── motor/                el motor (Rust) y la interfaz (Slint)
│   ├── src/              14 módulos: datos, rango, formulas_v2, umbral, patron, hrv,
│   │                     contexto, informe, fit, tendencia, juicio, fuentes, adaptador
│   ├── src/bin/          18: pulso (abre), y los INSTRUMENTOS de medida
│   │                     (inventario, biomecanica, potencia, avisos, que-traen,
│   │                      pulso-reposo, volcar-campos, medir-carga...)
│   ├── tests/pruebas.rs 155 pruebas · huawei.rs 12 · medir-interfaz.rs 8 · recorrido.rs 11
│   └── ui/               temas.slint · armazon.slint · ventana.slint · iconos.slint · raiz.slint
│                         sonda-borde.slint  <- el experimento de los controles, CONSERVADO
├── docs/                10 documentos, y LAS FUENTES en docs/slint-fuentes/ (32 páginas)
├── herramientas/        traer-slint.sh · html-a-texto.py · iconos-a-slint.py
├── datos/               **1.003 MB DEL CLIENTE** — ignorado por git, NUNCA se sube.
│                        102 FIT de entrenamiento + **1.360 ficheros de Garmin**
│                        (sueño, pulso, estrés, training status: los diarios vienen VACIOS)
├── datos-de-ejemplo/    caso inventado, esto sí se publica
└── temas/                contraste.py (los dos temas) · contraste-widgets.py (los CONTROLES)
```

**Y los otros dos repositorios:**

- `~/projects/mimo-nuevo` — la norma DEL PROYECTO, `PATRONES-DE-FALLO.md` (15), `TASKS.json` (86).
- `~` — la norma universal, en `.dsh/skills/norma-app/SKILL.md`.

## 5. Las trampas ya conocidas, para no volver a caer

**De Slint** (todas comprobadas — el detalle en `docs/SLINT-lo-que-hay-que-saber.md`):

1. **No se puede cambiar de estado desde Rust.** Se gobierna con **una propiedad** y condiciones `when`.
2. **`private` es el default de una propiedad**: sin `in` explícito, **el setter no existe y el
   compilador no avisa**.
3. **Dentro de un layout, `x` e `y` no se ponen**: lo coloca el layout. Ponerlo a un hijo y a otro no
   **cambia el tamaño del padre**.
4. **Los widgets de serie no aceptan nuestros colores** — **ni un borde**: `Unknown property border-width
   in Button`. En el tema negro su borde da **1,14:1** y no se ve. Salida medida: **marco por fuera**
   (ver §12 del doc de Slint).
5. **`visible: false` no libera el hueco.**
6. **`Text {};` con punto y coma no compila.**
7. **`to-float()` usa el separador del IDIOMA**: `"1.5"` da **0** en un equipo alemán.
8. **`Timer` viene encendido** (`running: true`): consume batería si no se apaga.
9. **`ScrollView` no se desplaza con el dedo** sin `mouse-drag-pan-enabled: true`.
10. **Un solo punto de entrada** en `build.rs`, o el segundo fichero pisa al primero.
11. **Para medir hace falta información de depuración** (`with_debug_info(true)`).

**Del método** (y son las que más costaron):

12. **Un modo que solo comprueba NO puede abrir ventana.** Si la comprobación se ve, no se puede hacer
    en cualquier momento — y **aparece en la pantalla del dueño**.
13. **`pkill` con un patrón que aparece en su propia orden se mata a sí mismo.** Matar **por PID**.
14. **`/tmp` es una partición pequeña aquí**: llenarla rompe compilaciones con un error que **no es del
    código**.
15. **Los binarios de depuración ocupan cientos de MB**: «Finished in 0.2s» puede significar **que no
    recompiló**. Forzar con `touch`.

**Y cuatro que costaron el 2026-10-13, al decidir los controles:**

16. **El esquema de los widgets NO lo decide nuestro tema, lo decide el SISTEMA.** Si no coinciden, el
    texto del control sale a **1,05:1** — **ilegible**. Se arregla con **una línea obligatoria**
    (`Palette.color-scheme = …` en la raíz). Sin ella la pantalla se rompe en el equipo de cualquiera
    que tenga el tema contrario al suyo.
17. **Los controles de serie miden 27 px de alto por su contenido, y su mínimo son 32.** El mínimo
    táctil del proyecto son **48 dp**: **hay que pedirlo**, no viene puesto.
18. **Dimensionar un envoltorio con `self.preferred-height` NO funciona**: vale **cero** en ese momento.
    Medido: el marco salió de **288x4** y el botón de **284x27**, saliéndose. La forma correcta es **no
    calcular**: `padding` en el layout y la altura pedida **al hijo**.
19. **`match_id` casa con el id INTERNO del elemento, no con el `accessible-id`.** Para buscar por
    `accessible-id` hay que **filtrar** sobre `find_all()`. Y `group` **no existe** como papel de
    accesibilidad: el catálogo tiene **`groupbox`** y **`region`**.
20. **El tiempo de abrir NO está en calcular, está en LEER.** Medido con 102 ficheros: leer **5,50 s**,
    contar 549.842 muestras **0,000 s**, calcular la deriva de 102 sesiones **0,00 s**. Optimizar el
    cálculo habría sido perder el tiempo: lo que había que evitar era **volver a leer**. Con la
    memoria: **5,40 s -> 0,001 s**, y el fichero que lo consigue pesa **9.473 bytes**.
21. **La huella de los ficheros se hace por TAMAÑO, no por fecha** — y es a propósito: la fecha cambia
    al **copiar** un fichero aunque el contenido sea el mismo, así que con la fecha, copiar sus datos
    invalidaría la memoria sin motivo. Y un fichero de memoria **corrupto o de otra versión** no puede
    impedir abrir: se recalcula y ya. **Una memoria que no se puede invalidar es una forma de mentir.**

## 6. Cómo se comprueba que algo funciona (el método, no la fe)

```bash
cd ~/projects/pulso/motor
cargo test                              # 188 pruebas, en verde, 0 avisos
python3 ../temas/contraste-widgets.py   # el contraste de los controles de serie (y su trampa)
cargo run -q --release --bin biomecanica -- ../datos/garmin_files/storage   # lo que traen los FIT
cargo run -q --release --bin pulso-reposo -- ../datos/garmin_files/storage  # los TRES pulsos
cargo run -q --bin inventario -- ../datos/garmin_files/storage   # cuenta los 625 resultados
cargo run -q --bin pulso -- ../datos/garmin_files/storage --texto   # lo que enseña, SIN ABRIR
cargo run -q --release --bin medir-carga -- ../datos/garmin_files/storage   # mide la memoria
```

**Y dos reglas del método, que están en la norma (AU-63 y AU-70):**

- **lo visual se MIDE, no se mira** — y si no se puede medir, **eso es la tarea**;
- **nunca abrir la ventana en la pantalla del dueño** para comprobar algo. Se mide.

## 6-ter. La conexión, HECHA (2026-10-13, noche)

**Las tres piezas están conectadas a `pulso.rs`** (antes `ventana.rs`), y comprobado con la salida literal:

```
--- LO QUE LA VENTANA RECIBE Y ENSEÑA ---
  NO SE PUEDE DIAGNOSTICAR TODAVIA
  Faltan tus datos: con la edad y el umbral ya se calculan las zonas. Se rellenan en Datos.
  102 sesiones · 549842 lecturas · 1259082 intervalos de HRV
```

**CERO cifras calculadas sin datos de la persona.** Ese era el fallo más grave que tenía la aplicación:
enseñaba `169±3` con un umbral escrito a mano **y** decía «sin datos» a la vez.

**Y medido, con el comando que lo mide** (`medir-carga`, y un control con la caché de disco ya caliente):

| | |
|---|---|
| leer los 102 ficheros | **5,57 s** |
| con la memoria | **0,032 s** (proceso entero, no solo la lectura) |
| el fichero de memoria | **9.473 bytes**, en vez de releer los **34 MB** de FIT |

**OJO CON UN NÚMERO QUE ESTABA MAL EN ESTE FICHERO**: donde decía «1.003 MB» son los 102 FIT. **1.003 MB
es TODO `datos/`**, con la base de datos de Garmin dentro. Los FIT pesan **34 MB**. Corregido también en
el comentario del código.

**Lo que NO se pudo ejecutar de punta a punta**: **pulsar «Guardar»** — necesita abrir la ventana, y eso
está prohibido. La rama aplicar → guardar → **recalcular con los datos nuevos** está compilada y
revisada, pero **no probada de punta a punta**. Es lo primero que hay que probar mañana, con la ventana.

**Y la EDAD sigue mal**: dice `60` y tiene **62**. No se arregla escribiendo 62 — eso **caduca el 25 de
enero** — sino **guardando la fecha de nacimiento** y calculando la edad al usarla. Eso exige un campo
nuevo en `Persona` (`datos.rs`), que es lo único que queda por añadir al modelo.

---

## 6-bis. El encargo del cliente (Farellones, 29 de noviembre de 2026)

**Llegó el 2026-10-13, redactado por otra IA**, con requisitos, sugerencias y supuestos mezclados. Se
comprobó **punto por punto contra sus datos**, y está todo en `docs/ENCARGO-farellones-2026.md`.
**Quedan 47 días.**

**Lo que hay que saber de un vistazo:**

- **Sus ficheros traen MUCHO más de lo que se usaba**: TSS (18 sesiones por encima de 100), FTP **213 W**
  detectado por Garmin, potencia, cadencia, **potencia y cadencia sentado/de pie**, respiración, y
  **la fase de pedaleo de cada pierna** (difieren en 80 de 83 sesiones).
- **El reparto de potencia izquierda/derecha NO está en el FIT**: lo calcula Garmin en su servidor. Lo
  que sí es por pierna es **la fase**.
- **Tres trampas de nombre**, documentadas con su fuente, que casi hacen acusar al cliente de un
  desequilibrio inventado: `left_right_balance` trae **el número de serie del sensor**;
  `avg_power_position` es **postura** (sentado/de pie) y **no lado**; `avg_*_pco` vale **0 siempre**.
- **Sueño, pulso en reposo, estrés: 165 ficheros de cada y TODOS vacíos.** No hay serie.
- **El pulso en reposo se separó en TRES datos distintos**, que era la causa del conflicto (48 dormido
  de Garmin, 71 despierto de Huawei, y el mínimo en entrenamiento). **Este cliente empieza a entrenar
  con el pulso ya alto** (mediana del mínimo: 90 ppm), así que de un entrenamiento **no** se saca.
- **La composición corporal tiene camino y no es teclear**: **exportación oficial de Huawei**
  (Centro de Privacidad → Request Your Data → Health), que trae peso, grasa, **agua, músculo y masa
  ósea**. Mejor que Health Sync + Terra API, que era lo que proponía el encargo.
- **La edad estaba mal**: dice `60` y **tiene 62** (nació el 1964-01-25). **No se arregla escribiendo
  62** — eso caduca el 25 de enero — sino guardando la **fecha** y calculando la edad al usarla.

**Y lo que NO se construye**: el «agente de IA en bucle». Ninguna de sus tres reglas lo necesita (son
«si X, entonces Y»), y esto decide sobre el cuerpo de una persona. Sus pautas se implementan **como
suyas**, no como hallazgos médicos.

---

## 6-quater. Tres deudas medidas, con su número (2026-10-13)

Salieron del módulo de potencia, y las tres están **medidas**, no supuestas. **Ninguna rompe nada hoy**;
las tres harían daño el día que se use el dato para lo que no es.

**1 · `fit::Declarado.duracion_s` guarda el tiempo EQUIVOCADO, y de forma NO DETERMINISTA.**
El código acepta `total_elapsed_time` **o** `total_timer_time`, **el que llegue primero** (`fit.rs:193`).
O sea: **el valor depende del orden de los campos dentro del fichero**, no de una decisión. Y la cuenta
del TSS de Coggan usa el **tiempo en movimiento**, no el transcurrido. Medido: con la transcurrida la
comprobación cuadra en **31 de 83** sesiones; con la de movimiento, en **55**. *Arreglo*: que `Declarado`
lleve **las dos**, con dos nombres distintos (AU-88).

**2 · `TimeInZone` viene uno por VUELTA y uno por SESIÓN.** Si se coge el primero —el de la vuelta 0, que
es lo natural— **el reparto suma de mediana un 84,7 % MENOS que la sesión, y el número sigue pareciendo
bueno**. Hay que filtrar por `reference_mesg == "session"`. *Y esta es la lección*: el fallo no da error,
da **un número plausible y falso** (AU-85).

**3 · `TimeInZone` trae DIEZ posiciones y Coggan define SIETE.** 28 de las 83 sesiones con potencia tienen
tiempo en las posiciones 8 a 10 (**1.322 s**). **No se reparten a ojo**: se cuentan aparte. La
comprobación de que las zonas suman la sesión cuadra en **69 de 83** con las siete, y en **79 de 83**
sumando las diez.

**Y un dato de su forma física que el encargo va a necesitar**: la **carga semanal BAJA un 58,8 %**
(377 → 155 TSS) entre el 2026-04-27 y el 2026-10-05, con el fondo en **84** la semana del 8 de junio.
Con **47 días** para Farellones, eso **no es un detalle**: es la diferencia entre llegar y no llegar, y
es exactamente lo que el cliente querrá saber. Ojo: esa tendencia **no incluye** las 19 sesiones sin TSS.

---

## 6-quinquies. El bloque del 2026-10-14: la puerta de entrada, la decisión de la rodilla, y las investigaciones

**Cinco cosas cerradas, y una en marcha.**

**1 · `--ayuda` ya NO abre ventana, y hay UNA puerta de entrada.** El defecto conocido está corregido:
`--ayuda` / `-h` / `--help` se comprueba **antes de crear la interfaz** y devuelve 0 sin abrir nada.
**Medido**: `cargo run -q -- --ayuda` imprime el uso y sale con `exit=0`, sin tocar ninguna ventana.
Y el binario de la aplicación **se llama `pulso`**: `ventana.rs` → `pulso.rs`, y `default-run = "pulso"`
hace que **`cargo run` a secas sea LA aplicación**. Los otros 17 son **instrumentos de medida**, no la
aplicación.
**Medido**: **188 pruebas en verde**, **0 avisos** (2 de `pide_ayuda` + 12 de `huawei` + 5 del selector).

**2 · DECISIÓN DEL DUEÑO (2026-10-14): el aviso de la rodilla NO dice «diagnóstico».** Se le preguntó,
porque el encargo pedía dos cosas que no caben a la vez («di que no es un diagnóstico» y «que el texto
nunca diga diagnóstico»). Eligió la opción recomendada: **decir el sentido, no el término**. El aviso de
la rodilla usa `avisos::NO_ES_VALORACION_MEDICA` («es una observación de sus propios datos, no una
valoración médica»), y la palabra «diagnóstico» solo se usa como nombre del informe global
(«NO SE PUEDE DIAGNOSTICAR TODAVÍA»). Coincide con lo ya implementado y con la prueba `es_observacion`.
**Es una línea, no un cambio de código.**

**3 · La exportación de Huawei: INVESTIGADA, y su lector ESCRITO.** El formato está en
`docs/HUAWEI-exportacion.md` (claves, unidades y las fuentes: el XLSX del propio export, tres parsers
independientes y la API de Huawei). El contrato se escribió ANTES (`docs/CONTRATO-huawei.md`) y la pieza
es `motor/src/huawei.rs`, con **12 pruebas propias** (`tests/huawei.rs`) que no necesitan la aplicación.
Lo que respeta: un campo ausente es `Falta` y NUNCA 0; **no se filtra por `type`** (el mismo dato viene
como 8 y como 10006), se filtra por `key == WEIGHT_BODYFAT_BROAD`; `value` es un texto con JSON dentro
(doble codificación); y la fecha sale de `startTime` (ms UTC) **más la zona del propio registro**, no la
del equipo.
**Y abre el ZIP tal cual sale del correo**: el dueño autorizó la dependencia `zip` (2026-10-14: «la idea
es minimizar el trabajo del cliente»); se lee **en memoria** y no se escribe nada en disco. Una carpeta ya
descomprimida también vale, y un fichero que no sea un ZIP se dice con un error tipado.

**4 · EL ESTADO DEL ARTE, REVISADO FILA A FILA Y CORREGIDO.** Las 41 direcciones (30 únicas) se abrieron,
y cada DOI se comprobó contra Crossref y PubMed. **El resultado no es bonito**: aparecieron **fuentes
FALSAS** —el DOI de Morton no existe (el bueno es `10.1007/s00421-005-0088-2`), el de Vanhatalo 2016 da
404 y su título no consta, el de Williams 2017 tampoco, el artículo del caldo de huesos es en realidad un
estudio de supermercados, y el de la asimetría entre piernas es de ortesis plantares—. **Los DOI críticos
los comprobé yo** en la API de Crossref, no solo el obrero. `docs/ESTADO-DEL-ARTE.md` queda reescrito:
las fuentes que se sostienen, corregidas, y **14 filas marcadas SIN FUENTE LOCALIZADA antes que dejar una
falsa**.
**Y un hallazgo de método**: **403 NO es URL muerta**; **19 de 25 URLs de las secciones 3-6 bloquean a los
programas** (Cloudflare), así que sus citas numéricas quedan **sin verificar**. El primer intento con el
carril OBREROS (`openrouter/nvidia/nemotron-3-super-120b-a12b:free`) estuvo **25 minutos y escribió CERO
ficheros**; se interrumpió y se relanzó con `deepseek-flash`. **La ruta del modelo iba en la llamada y
era la correcta: falló el modelo, no la ruta.** Los informes están en
`docs/ESTADO-DEL-ARTE-revision-1-2.md` y `-3-6.md`.

**EN MARCHA (no cerrado, y hay que retomarlo):**
- **Elegir la carpeta con un botón** (paso 5): **EN CURSO**. **Medido**: Slint 1.18.1 **no trae
  `FileDialog` ni `FolderDialog`** (búsqueda sobre los 13 crates instalados). **Decisión: selector
  propio, sin dependencia nueva** (autorizar `rfd` es la otra vía, y no se toma sin el dueño). **YA
  HECHO**: el contrato (`docs/CONTRATO-carpeta.md`) y el **núcleo probado** (`motor/src/carpeta.rs`, 5
  pruebas): lista subcarpetas sin ocultas ni ficheros, sube hasta la raíz, no revienta con una carpeta
  ilegible, y **calcula lo que el panel tiene que dibujar** (`abrir` devuelve la lista con **cuántos FIT
  tiene cada carpeta**, que es la intuición). **FALTA la capa 5**: el panel que lo dibuja y su conexión
  a `pulso.rs` para que al elegir se vuelva a analizar.

---

## 6-sexies. La memoria INCREMENTAL: solo se lee lo nuevo (2026-10-14)

**EL PROBLEMA QUE RESOLVIÓ, medido**: la memoria guardaba **solo el resultado**. Si el ciclista añadía
la sesión de hoy, el total ya no cuadraba con los ficheros y **se releían los tres meses de histórico**
para sumar **un** fichero. Justo lo que el dueño pidió evitar:

> *«por cada activación de la aplicación, no tenga que releer de nuevo toda la información de hace meses
> y ir sumando la información más reciente»*.

**Lo que se hizo**: guardar **una ficha por fichero** (nombre, tamaño y **su aporte** al total), y al
arrancar **leer solo las fichas nuevas o cambiadas**. El contrato se escribió ANTES:
docs/CONTRATO-memoria-incremental.md. La decisión de reutilizar es por **nombre y tamaño** (no por
fecha), y es una **función pura** (analisis::decidir) con sus pruebas, porque es el corazón de la pieza.

**Medido** con medir-carga sobre sus 102 ficheros:

| | |
|---|---|
| primera vez (lee todo) | **5,48 s**, 102 leídas |
| segunda, sin cambios | **0,001 s**, **0 ficheros releídos** |
| llega UNA sesión nueva | **0,086 s**, **1 leída**, y el total es idéntico |
| el fichero de memoria | **40.015 bytes** (antes 9.473): se paga disco a cambio de no releer |

**Y la coherencia de las FECHAS**, que es lo que pidió: la memoria deja de ser un total sin contexto.
Cada **aporte** guarda **cuándo empezó su sesión** (inicio_s); el resultado lleva **desde** y **hasta**;
y el fichero de memoria lleva **cuándo se calculó** (guardado_en_s). El arranque lo dice sin abrir
ventana: el modo --texto imprime **«periodo ... <primera sesión> .. <última sesión>»**. Sin fecha
conocida queda en «no lo sé» y se dice, no se rellena con la del fichero ni con la del equipo.

**Lo que NO se cambió**: el **tamaño** (no la fecha) sigue decidiendo si un fichero se reutiliza, y una
memoria **corrupta o de otra versión** (ahora la **2**) sigue sin poder impedir abrir.

---

## 7. Lo que NO está resuelto, dicho claro

- **La decisión de los controles** ya **NO** bloquea: se usan los de serie (paso 1, hecho).
- **Los datos de la persona están en el código**: **los resultados no son del cliente todavía.** Y
  están repetidos **a mano en 5 sitios de producción** (`pulso.rs`, `inventario.rs`, `informe.rs`,
  `sin_datos.rs`, `demo.rs`) y **en 4 de prueba**. Existe **un solo camino** para leerlos de un fichero:
  `adaptador::leer_perfil` — y **no existe guardar nada en disco** (no hay `Serialize` en el proyecto).
- **Cuatro páginas de Slint sin traer**: tipos de propiedad, **propiedades y bindings** (el hueco más
  grave: no se sabe qué pasa al asignar a mano a un binding), operadores y pureza.
- **13 iconos de Simple Icons** sin convertir: son de relleno, no de trazo.
- **Nada probado en un segundo proyecto**: las 82 reglas **solo se han usado en uno**.
- **`~` es un repositorio local** con 32 ficheros de caché sin decidir.
