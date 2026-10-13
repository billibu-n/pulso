# Slint: lo que hay que saber (destilado de las fuentes oficiales)

**Escrito el 2026-10-13.** Existe porque **el proyecto tenía nueve documentos y ninguno trataba del
lenguaje con el que está construido**. Se estaba escribiendo Slint **adivinando**, y eso costó tres
intentos para una unidad, dos para una bandera, y una tarde entera corrigiendo una pantalla a ciegas.

**Las fuentes**, todas oficiales:
[Posicionamiento y disposición](https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/) ·
[Tipos numéricos](https://docs.slint.dev/latest/docs/slint/reference/property-types/numeric-types/) ·
[Componentes de serie](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/overview/) ·
[Botón](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/basic-widgets/button/) ·
[Lista completa de la referencia](https://docs.slint.dev/latest/docs/slint/reference/overview/)

> **Cómo leer esto**: cada punto lleva **el error que lo originó**. No es un resumen de la
> documentación — es **lo que este proyecto necesitaba saber y no sabía**.

---

## 1. La regla que más desalineado causa

> **«Los valores por defecto de `x` y `y` CENTRAN el elemento dentro de su padre.»**
> — *Posicionamiento y disposición*

**Y ESTO ES LO QUE YO HACÍA MAL.** Dentro de un `VerticalLayout` ponía `x: 20px` y **no ponía `y`**,
pensando que el layout colocaría la vertical. **Resultado: el layout coloca `y`, y `x` centra el
elemento** — un texto que debería estar a la izquierda **sale centrado**.

**La regla práctica**: dentro de un layout, **`x` y `y` NO se ponen**. Si hace falta separar del borde,
se usa **`padding`** (del layout) o **`spacing`** (entre hijos). **`x` suelto dentro de un layout es
siempre un error.**

*Y esto lo cazó la medición*: los sitios del armazón salían donde el diseño decía, pero **los textos de
dentro no**. La geometría de los bloques cuadraba y el contenido estaba descolocado.

---

## 2. Por qué la barra de abajo salía con los rótulos PEGADOS

> **«Cuando no se especifica una alineación, los elementos se estiran.»**
> **«Un valor de `0` significa que el elemento NO se estira. Todos se estiran igual si todos tienen
> factor `1`.»**
> — *Posicionamiento y disposición*

**Yo creía que `horizontal-stretch: 1` repartía por igual. Y reparte — pero solo entre los que lo
tienen.** En la barra de abajo, los cuatro huecos **no lo tenían**, así que **el ancho lo decidía el
texto**: «Resumen» 32 px, «Sesiones» 46, «Datos» 31. **Medido.**

**La regla práctica**, y es la **AU-68** de la norma: **una fila de cosas que deben alinearse se reparte
por igual**. En Slint eso es **`horizontal-stretch: 1` en TODOS los hijos** — o, mejor, **el ancho
calculado** (`parent.width / 4`), que **no depende de que alguien se acuerde de poner el stretch**.

---

## 3. `width` no es lo mismo que `min-width`

> **«Cualquier elemento con `width` y `height` especificados tiene un TAMAÑO FIJO dentro de un
> layout.»** Las de constraint (`min-width`, `max-width`, `preferred-width`) **no fijan: acotan.**
> — *Posicionamiento y disposición*

**Confundirlas fue otro de mis intentos fallidos**: puse `min-width: 60px` esperando que el hueco
midiera 60, y midió **lo que quiso el contenido**.

**La regla práctica**: para **fijar**, `width`. Para **acotar**, `min-width`/`max-width`. Y si el
tamaño tiene que ser **una proporción del padre**, se calcula: `parent.width / 4`.

---

## 4. La unidad: `px` o `phx`, y **el número suelto no vale**

| tipo | qué es | cuándo |
|---|---|---|
| **`px`** | **píxel lógico**, se escala con la pantalla | **el recomendado, y el que se usa aquí** |
| `phx` | píxel físico | casi nunca |
| `%` | **relativo al padre** | cuando el tamaño depende del espacio |

**Y esto costó tres intentos con los iconos**: `stroke-width` es **una medida** (lleva unidad:
`2px`), y `viewbox-width` es **un sistema de coordenadas** (**número sin unidad**: `24.0`). Poner la
unidad donde no va da *«Cannot convert length to float»*, y quitarla donde sí va da *«Cannot convert
float to length»*. **El compilador dice cuál es cuál — pero solo si se prueba.**

**Regla práctica** (AU-72): **no adivinar la unidad: probar en un fichero de tres líneas.** Treinta
segundos, contra veinte minutos de suposiciones.

---

## 5. Lo que Slint **ya trae hecho** (y yo estaba construyendo a mano)

```slint
import { Button, LineEdit, ScrollView, Spinner, VerticalBox } from "std-widgets.slint";
```

| lo que hay | para qué sirve aquí |
|---|---|
| **`Button`** | **«Vincular cuenta», «Analizar»** — con `text`, `clicked`, `enabled`, `primary` |
| **`LineEdit`** | **los datos de la persona**: edad, peso, umbral |
| **`SpinBox`** | números con límites: edad, horas de sueño |
| **`ScrollView`** | el contenido largo de una pantalla estrecha |
| **`ListView`** | **la lista de sesiones** |
| **`Spinner`** | **el «cargando» de los 5 segundos de arranque** |
| **`Switch`** | el tema claro/oscuro |
| **`ProgressIndicator`** | traer 102 ficheros |
| **`ComboBox` / `RadioGroup`** | la comida antes de entrenar (4 opciones) |

**`Button` tiene además `primary`**, que es **el botón de la acción principal** — justo lo que pide la
norma: **una acción, una forma, y la principal se distingue**.

> **Y ESTO ES UNA LECCIÓN, no un atajo**: estaba construyendo **un botón a mano con un `Rectangle` y un
> `TouchArea`**. Funcionaba, y **le faltaba todo lo demás**: el estado pulsado, el foco del teclado, el
> deshabilitado, y **la accesibilidad**. **Lo que trae el marco está probado; lo que uno escribe, no.**

---

## 6. Los números se escriben solos, y **en español**

```slint
property<float> pi: 3.14159;
property<string> a: pi.to-fixed(2);              // "3,14"  <-- CON COMA, si la configuración es española
property<string> b: pi.to-string-unlocalized();  // "3.14159" <-- siempre con punto
```

> **«Ambas funciones formatean el número usando el separador decimal de la configuración regional
> actual.»** — *Tipos numéricos*

**Y YO ESCRIBÍ UNA FUNCIÓN EN RUST PARA CAMBIAR EL PUNTO POR LA COMA.** No hacía falta: **Slint ya
sabe hacerlo, y mejor, porque respeta la configuración del equipo del usuario**.

**Regla práctica**: **el formateo de números es de la vista, no del motor.** El motor calcula; **la
pantalla decide cómo se escribe**. Y si el motor formatea, **está haciendo trabajo de la vista** — que
es mezclar capas.

---

## 7. Las tres cosas que se aprendieron a golpes, y que no están en un índice

**a) `@children` no puede ir dentro de un `if`.** El armazón se tuvo que **partir en dos componentes**,
y la condición se movió a quien decide. *(Error: «The @children placeholder cannot appear in a
conditional element».)*

**b) Un solo punto de entrada.** Compilar **dos ficheros por separado** hace que **el segundo pise al
primero**: Slint genera **un módulo con lo último**. El síntoma es engañoso —**compila bien y luego el
programa no encuentra el tipo**—. Se resuelve con un `ui/raiz.slint` que **importa todo** y se compila
solo ese. *(AU-71.)*

**c) Para MEDIR hace falta información de depuración.** Sin ella, la consulta de geometría **no
existe**, y el aviso lo dice con estas palabras: *«The use of the ElementHandle API requires the
presence of debug info»*. Se activa en `build.rs` con `CompilerConfiguration::new().with_debug_info(true)`.
*(Y es lo que permite que la interfaz se compruebe sin abrirla — AU-65.)*

---

## 8. Lo que TODAVÍA no se ha mirado, y hay que mirar

**Lo digo para que no se confunda «documentado» con «entendido»:**

| falta mirar | para qué |
|---|---|
| **[Estados y transiciones](https://docs.slint.dev/latest/docs/slint/guide/language/coding/states/)** | las pantallas del recorrido (sin datos → vinculado → analizado) |
| **[Repetición y modelos](https://docs.slint.dev/latest/docs/slint/guide/language/coding/repetition-and-data-models/)** | la lista de sesiones y los gráficos |
| **[Temporizadores](https://docs.slint.dev/latest/docs/slint/reference/timer/)** | quitar el «se queda detenido» del arranque |
| **[Flickable](https://docs.slint.dev/latest/docs/slint/reference/gestures/flickable/)** | la pantalla estrecha, que se desplaza |
| **[Path](https://docs.slint.dev/latest/docs/slint/reference/elements/path/)** | **los gráficos de líneas y áreas** — lo que la norma avisa que hay que hacer a mano |
| **[Paleta](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/globals/palette/)** | **cómo se integran los componentes de serie con NUESTROS dos temas** |

**Y el último es el que más importa**: los componentes de serie traen **su propio aspecto**, y
**hay que ver si respetan el tema cálido y el negro OLED** — o si hay que ajustarlos. **Un botón que
no sigue el tema rompe los dos temas de golpe.**

---

## 9. Los gráficos: `Path`, y dos formas de dibujarlos

> **«Un trazado se puede definir de DOS maneras: usando comandos SVG como una cadena, o usando
> elementos de trazado en el marcado `.slint`.»**
> **«Si `width` y `height` no son cero, la figura entera se AJUSTA a esos límites, escalando.»**
> — *Path*

**Los dos caminos, y cuándo cada uno:**

| forma | cuándo conviene |
|---|---|
| **`commands: "M0 0 L10 10 ..."`** (cadena SVG) | **viene de fuera**: los iconos del kit, que ya son SVG |
| **`MoveTo` / `LineTo` / `CubicTo` / `ArcTo`** | **se genera aquí**: una curva calculada, un gráfico de datos |

**Y DOS FUNCIONES que resuelven el gráfico de líneas**:

- **`point-at(t)`** devuelve un punto **a lo largo del trazado** (`t` de 0 a 1);
- **`angle-at(t)`** devuelve **el ángulo** en ese punto.

**Para qué sirven aquí**: para **poner una marca o una etiqueta SOBRE la curva** — el punto de la
deriva, la etiqueta de un mes en la tendencia. Hacerlo a mano sería **recalcular la curva por fuera**,
que es justo lo que no hay que hacer.

**Y `fit`** controla cómo se ajusta la figura a la caja: **es lo que decide si un gráfico se estira o
se respeta la proporción**.

> **Lo que esto cambia para el proyecto**: la norma avisa de que **Slint no dibuja gráficos de datos**
> y hay que hacerlos a mano. **Es verdad a medias**: no trae un elemento «gráfico», **pero sí trae las
> piezas** — el trazado, el escalado por caja, y las funciones para recorrerlo. **Lo que hay que
> escribir es la lista de puntos; el dibujo lo hace `Path`.**

---

## 10. Estudiar el lenguaje a fondo (2026-10-13) — y lo que apareció

**Se trajeron 32 páginas de la documentación oficial** a `docs/slint-fuentes/` (texto, con fecha y
dirección), y se estudiaron **por temas**. Aquí va lo que **cambia decisiones** — no un resumen: **lo
que hay que saber antes de escribir la aplicación**.

### 10.1 · **NO se puede cambiar de estado desde Rust.** Y es lo más importante de todo

**Verificado en la API**: los estados de Slint **solo existen en `private_unstable_api`**. En la API
pública de Rust **no hay `set_state` ni nada equivalente** — los estados son una **construcción del
compilador**, ligada a condiciones `when`.

> **Consecuencia para el diseño, y hay que decidirlo AHORA, no al final**: el recorrido del usuario
> tiene varias pantallas (sin datos → vinculado → analizado), y **quien decide en cuál está es Rust**
> —porque depende de si hay datos, de si hay cuenta, de si ya se analizó—.
>
> **La forma que sí funciona**: declarar **una propiedad** (`in-out property <int> pantalla`) y escribir
> los estados **condicionados a ella**. Así **Rust manda por la propiedad**, y **Slint hace la
> transición**. Si esto se descubre al final, hay que rehacer las pantallas.

### 10.2 · Las trampas del `Timer`

**`running` vale `true` por defecto.** Un temporizador que no se apaga **consume CPU para siempre** — y
en un móvil, **batería**. Se apaga al soltarlo.

*Para qué se necesita aquí*: **el «se queda detenido» de los cinco segundos de arranque**. Un `Timer`
que muestre progreso **mientras se leen los 102 ficheros**.

### 10.3 · Las listas: no reinstalar el modelo

> *«Es más eficiente operar sobre el modelo y enviar los cambios… que reinstalar la propiedad con un
> modelo distinto.»*

**Para la lista de sesiones** (102 y subiendo): se opera sobre el modelo, **no se reemplaza**.

### 10.4 · Android: lo accionable, y una condición

| dato | valor |
|---|---|
| **mínimo de Android** | **API 26 (Android 8.0)** — por debajo, **falla** |
| **el crate** | tiene que ser **`cdylib`** |
| **el punto de entrada** | `android_main(app: slint::android::AndroidApp)` |
| **la característica** | `backend-android-activity-06` |
| **variables** | `ANDROID_HOME` y `ANDROID_NDK_ROOT` |
| **empaquetar** | `cargo apk` o `x build --platform android --format apk` |
| **guardar estado** | **no lo da Slint**: `StateSaver::store(&[u8])` del ciclo de vida de Android |

**Y una advertencia honesta**: **la documentación de Slint NO trae una página del ciclo de vida de
Android.** Lo accionable sale de `android-activity` y de ejemplos. **No hay una guía que seguir paso a
paso** — eso hay que construirlo y **probarlo en el teléfono**.

### 10.5 · Herramientas de depuración que sí existen

| variable | para qué |
|---|---|
| `SLINT_DEBUG_PERFORMANCE` | **rendimiento**: `refresh_lazy`/`refresh_full_speed` + `console`/`overlay` |
| `SLINT_SLOW_ANIMATIONS` | ver las animaciones despacio |
| `SLINT_SCALE_FACTOR` | probar otra densidad de pantalla |
| `slint-viewer --screenshot` | **una captura sin abrir ventana** — sirve para lo visual |
| `SLINT_LIVE_PREVIEW=1` | ver los cambios del `.slint` al guardar |

**`slint-viewer --screenshot` merece mirarse**: sería **otra forma de ver la interfaz sin abrirla en la
pantalla del dueño**.
