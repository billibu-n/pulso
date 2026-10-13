# La cara: el tema elegido, y el reparto en cada pantalla

**Escrito el 2026-10-13.** El dueño eligió el **tema cálido**, y avisó de algo que cambia el diseño:
**el cliente usa Android como dispositivo principal.** *«No implica que desde escritorio no sea
viable»* — o sea: **los dos, y el móvil primero**.

---

## 1. El tema: **claro cálido**

| papel | valor | para qué |
|---|---|---|
| fondo | `#faf9f7` | crema, no blanco puro: cansa menos la vista |
| superficie | `#ffffff` | las tarjetas |
| texto | `#2b2724` | casi negro, pero cálido |
| texto apagado | `#78716c` | las etiquetas |
| línea | `#e7e2db` | los bordes, apenas visibles |
| **acento** | **`#0f766e`** | **verde azulado: lo que se puede pulsar** |
| correcto | `#15803d` | |
| aviso | `#b45309` | |

**Y la escala de las zonas**, que **no es decoración**: cada zona se distingue de la de al lado, y **el
orden se ve sin leer el nombre**.

| zona | color |
|---|---|
| recuperación | verde `#15803d` |
| resistencia | verde azulado `#0f766e` |
| tempo | violeta `#7c3aed` |
| **umbral** | **el color de acento** `#0f766e` — es la zona que más importa |
| consumo máximo | teja `#b45309` |

::: aviso
**Y esto se cambió al mirar los bocetos**: en la primera versión, las cinco zonas salían **del mismo
color**, y ahí **se perdía la lectura**. Un gráfico donde todo es igual **no informa**: solo ocupa.
:::

## 2. El reparto NO es el mismo, y esto es lo importante del aviso

**El cliente usa el móvil. Y en el móvil no hay riel que repartir** — eso ya estaba escrito en la norma
del proyecto (`reglas/09`): *«en móvil NO hay lateral que repartir; el riel es de ESCRITORIO, el reparto
es VERTICAL y la nav es CHROME»*.

| | **escritorio** (1440 px) | **móvil** (411 dp) |
|---|---|---|
| **navegación** | riel de **176 px** a la izquierda | **cromo**: barra **abajo**, 64 dp |
| **las cifras** | **4 en fila** | **2 y 2** — cuatro no caben |
| **las zonas** | barra **al lado** del nombre | barra **debajo** |
| **el panel de contexto** | columna de **320 px** | **no cabe**: se sube al contenido |
| **lo que se repliega** | el riel pierde su **columna** (AU-01) | el riel **se abre como menú** (AU-02) |
| **áreas tocables** | 32 px basta | **48 dp como mínimo** — el dedo no es un ratón |

**Y las medidas son reales, no supuestas**: el teléfono del cliente mide **1080 × 2424 px a 420 ppp**,
que son **411 × 923 dp**. Es un **teléfono normal**, no una tableta.

## 3. Por qué se hizo así y no «encogiendo el de escritorio»

**Encoger el de escritorio daría un riel de 176 px en una pantalla de 411** — o sea, **el 43 % del ancho
para la navegación**. Eso **no es un diseño: es un desperdicio**.

**Por eso hay dos repartos**, y **la norma los cubre**:
- **AU-01**: el orden de repliegue —contenido, panel inferior, panel derecho, riel—;
- **AU-02**: el riel, al perder su columna, **se vuelve menú que se abre, no se oculta**;
- **AU-03**: y **nace cerrado**, porque abrirlo por defecto tapa el contenido.

## 4. Lo que falta para Android, comprobado (no de memoria)

| qué | estado | cómo se consigue |
|---|---|---|
| **objetivo `aarch64-linux-android`** (teléfono real) | **FALTA** | `rustup target add` — **sin `sudo`** |
| **objetivo `x86_64-linux-android`** (emulador) | **FALTA** | igual |
| **el NDK** | **FALTA** | **no está en Fedora**: se instala con el SDK de Google, que ya está montado |
| **`cargo-ndk`** (4.1.2) | **FALTA** | `cargo install` — **sin `sudo`** |
| **el SDK de Android** | **instalado** | ya está, con el emulador |
| **Slint** | **soporta Android** | trae un backend propio para eso |

**Y hay que decirlo, porque es lo que más va a costar**: **Slint no dibuja gráficos de datos**. Las
barras se hacen con rectángulos, y las **líneas y áreas** hay que dibujarlas **a mano** con el elemento
de trazado. **No es un problema, pero es trabajo** — y conviene saberlo antes de empezar, no al final.

## 5. Lo que queda decidido, y lo que no

**Decidido**: el tema cálido · dos repartos (móvil y escritorio) · las zonas con escala de intensidad ·
el móvil primero.

**Sin decidir todavía**: si el panel de contexto del escritorio **sobrevive** en el móvil como una
sección más, o **desaparece** y su información sube al resumen. **Eso se ve en la aplicación, no en un
boceto.**
