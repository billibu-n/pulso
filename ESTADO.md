# ESTADO: dónde estamos y por dónde se sigue

**Escrito el 2026-10-13**, porque el contexto se estaba llenando y **una compactación a mitad de tarea
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
| **el motor** | **120 pruebas en verde**, cero avisos | `motor/src/` |
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
│   ├── src/bin/          18: ventana (abre), y los INSTRUMENTOS de medida
│   │                     (inventario, biomecanica, potencia, avisos, que-traen,
│   │                      pulso-reposo, volcar-campos, medir-carga...)
│   ├── tests/pruebas.rs  101 pruebas · medir-interfaz.rs  8 · recorrido.rs  11
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
cargo test                              # 120 pruebas (101 motor + 8 medicion + 11 recorrido)
python3 ../temas/contraste-widgets.py   # el contraste de los controles de serie (y su trampa)
cargo run -q --release --bin biomecanica -- ../datos/garmin_files/storage   # lo que traen los FIT
cargo run -q --release --bin pulso-reposo -- ../datos/garmin_files/storage  # los TRES pulsos
cargo run -q --bin inventario -- ../datos/garmin_files/storage   # cuenta los 625 resultados
cargo run -q --bin ventana -- ../datos/garmin_files/storage --texto   # lo que enseña, SIN ABRIR
cargo run -q --release --bin medir-carga -- ../datos/garmin_files/storage   # mide la memoria
```

**Y dos reglas del método, que están en la norma (AU-63 y AU-70):**

- **lo visual se MIDE, no se mira** — y si no se puede medir, **eso es la tarea**;
- **nunca abrir la ventana en la pantalla del dueño** para comprobar algo. Se mide.

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

## 7. Lo que NO está resuelto, dicho claro

- **La decisión de los controles** ya **NO** bloquea: se usan los de serie (paso 1, hecho).
- **Los datos de la persona están en el código**: **los resultados no son del cliente todavía.** Y
  están repetidos **a mano en 5 sitios de producción** (`ventana.rs`, `inventario.rs`, `informe.rs`,
  `sin_datos.rs`, `demo.rs`) y **en 4 de prueba**. Existe **un solo camino** para leerlos de un fichero:
  `adaptador::leer_perfil` — y **no existe guardar nada en disco** (no hay `Serialize` en el proyecto).
- **Cuatro páginas de Slint sin traer**: tipos de propiedad, **propiedades y bindings** (el hueco más
  grave: no se sabe qué pasa al asignar a mano a un binding), operadores y pureza.
- **13 iconos de Simple Icons** sin convertir: son de relleno, no de trazo.
- **Nada probado en un segundo proyecto**: las 82 reglas **solo se han usado en uno**.
- **`~` es un repositorio local** con 32 ficheros de caché sin decidir.
