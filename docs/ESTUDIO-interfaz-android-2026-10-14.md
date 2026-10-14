# Estudio de la interfaz contra la norma de un teléfono Android

**Escrito el 2026-10-14**, a raíz de esto, del dueño:

> *«Viéndolo, se incumplen varias reglas de interfaz de usuario… Si bien se detecta visualizando, el
> motor es el que está fallando al generar estos resultados… Respeto de espacios, interfaz típica
> teléfono Android, etc.»*

Y tiene razón en lo importante: **no era un problema de la captura, era el motor**. La captura solo lo
hizo visible. Este documento mide lo que hay, lo compara con la norma de Android y deja escrito **qué
falla, por qué, y cómo se corrige**, para que no vuelva a pasar.

## Cómo se midió (AU-63: antes de tocar lo visual, se puede medir)

1. **Captura fuera de pantalla** con `motor/src/bin/capturar-ui.rs` (no abre ventana) — es lo que el
   dueño pidió ver.
2. **Geometría** con `cargo test --test medir-interfaz` (posiciones y tamaños reales).
3. **Norma oficial** de Android / Material 3, leída en la fuente (direcciones al final), no de memoria.

## Las fuentes (medidas oficiales, comprobadas el 2026-10-14)

| qué | valor oficial | dónde |
|---|---|---|
| **Top app bar (small)** | alto **64 dp**, título **22 px** (title-large) | AndroidX `TopAppBarSmallTokens.kt`; tipografía M3 |
| **Navigation bar** (con etiquetas) | alto **80 dp** (token `TallContainerHeight`; el `ContainerHeight` del mismo fichero es 64), icono **24 dp**, etiqueta **12 px** (label-medium) | AndroidX `NavigationBarTokens.kt`; tipografía M3 |
| **Indicador del activo** | forma **píldora** (`CornerFull`) **detrás del icono**, no una línea arriba | AndroidX `NavigationBarTokens.kt` |
| **Área táctil mínima** | **48 dp** | `minimumInteractiveComponentSize` (Android Developers) |
| **Radio de tarjeta** | **12 dp** | Material 3 (cards) |

## Lo que hace la aplicación, medido, contra la norma

| # | qué | Android | la aplicación | veredicto |
|---|---|---|---|---|
| 1 | iconos de la barra de abajo | 4 iconos reales, 24 dp | **4 cuadrados de color**, 18 dp | **FALLO** |
| 2 | alto de la barra de abajo | 80 dp | **64 dp** | **FALLO** |
| 3 | alto de la cabecera | 64 dp | **56 dp** | **FALLO** |
| 4 | radio de tarjeta | 12 dp | **10 dp** | **FALLO** |
| 5 | etiqueta de la barra | 12 px | **11 px** | **FALLO** |
| 6 | indicador del activo | píldora tras el icono | **línea de 2 px arriba** | **FALLO** |
| 7 | área táctil | ≥48 dp | 48 dp (medido) | cumple |
| 8 | reparto de los huecos | por igual | por igual (411/4) | cumple |

## Y tres fallos de CONTENIDO, que son los que el dueño señala como «el motor»

La captura sin datos personales enseñaba **cuatro tarjetas vacías**, **cinco barras de zona sin
nombre** y **una barra de precisión en blanco**. No es que faltara el dato: es que **el motor dibuja el
marco aunque no haya nada dentro**. Y repetía un mismo mensaje **dos veces**.

| # | el fallo | la causa, en el motor | la regla que incumple |
|---|---|---|---|
| 9 | tarjetas de cifras y barras de zona **vacías** | la vista dibuja las cuatro `Cifra` y las cinco `Zona` siempre, con listas vacías | **AU-07** (un hueco olvidado no se dibuja) y **AU-24** (el básico no dibuja lo que no le han aportado) |
| 10 | barra de precisión **en blanco** | `precision` va vacío pero el `Aviso` se dibuja igual | AU-07 |
| 11 | «Sin datos de entrenamiento todavía.» **dos veces** | `cara.rs` lo pone en `vacio` **y** en `aviso` | AU-44/AU-45 (y es ruido) |
| 12 | «frecuencia máxima» mostraba **0** sin medirla | `unwrap_or(0)` en `cara.rs` | «`Falta` no es `0`» (**ya corregido** en el commit anterior) |

**Y el origen del fallo nº 1**, que merece quedar escrito: el boceto aprobado
(`bocetos/generar-movil.py`) dibujaba el «icono» como *un cuadrado redondeado*, con este comentario
literal: *«En el boceto no hace falta dibujarlo bien»*. **El cuadro del boceto pasó a producción**, y
nadie lo comparó con la norma. Es exactamente AU-74: el diseño aprobado es una pieza y el código se
comprueba contra ella; pero **el hueco de un boceto no es una decisión de diseño**.

## Un cuarto hallazgo, que NO es un fallo de dibujo: falta la acción

El recorrido del usuario pedía **«pulsar para analizar»** (paso 5). Hoy la aplicación **calcula al
abrir** y no hay **ningún botón de acción primaria** en la pantalla. No se toca aquí porque es una
**decisión de producto** del dueño (¿se analiza solo, o se pulsa?), y cambiarla sin él sería cambiar lo
aprobado.

## Lo que se corrige en este bloque

1. **Iconos de verdad** en la barra: los 192 de Lucide que ya están en `iconos.slint` — `casa`,
   `corazon` (antes `grafico-linea`; cambiado por decisión del dueño el 2026-10-14), `bicicleta`, `ajustes` — a **24 dp**.
2. **Medidas Android**: cabecera **64**, barra de abajo **80**, radio **12**, etiqueta **12 px**, y el
   activo con **píldora detrás del icono** en vez de línea arriba.
3. **El motor no dibuja vacíos**: las cifras y las zonas se dibujan **solo si se puede diagnosticar**;
   la precisión, el aviso y el recuento **solo si tienen texto**.
4. **Un solo mensaje**: `cara.rs` deja de repetir «Sin datos…» en el aviso.

Todo con la **fuente del diseño actualizada** (AU-66: si un valor del diseño no cuadra, se corrige el
diseño) y con su **prueba de geometría** (AU-65: la disposición se comprueba).

## Lo que NO se toca, y por qué

- **La acción primaria** (punto de arriba): es decisión del dueño.
- **El tema y los colores**: no están en cuestión; su contraste ya está medido.
- **El icono del riel de escritorio**: el riel es de escritorio y hoy es texto; se puede añadir después.

## Las fuentes, con su dirección

- AndroidX `NavigationBarTokens.kt` (alto 64/80, indicador `CornerFull`):
  https://android.googlesource.com/platform/frameworks/support/+/d95b985e08c10ac8d336464be703858819ecd87f/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/NavigationBarTokens.kt
- AndroidX `TopAppBarSmallTokens.kt` (alto 64):
  https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/TopAppBarSmallTokens.kt
- Tipografía de Material 3 (title-large 22, label-medium 12):
  https://m3.material.io/components/navigation-bar/specs
- Área táctil mínima 48 dp:
  https://developer.android.com/reference/kotlin/androidx/compose/material/minimumInteractiveComponentSize.modifier

**NO CONFIRMADO**: cuál de los dos altos de la barra (64 o 80) es el que usa por defecto el componente
de Compose; el token con etiquetas es el de 80, y es el que se toma. Se dice, en vez de callarlo.
