# AUTORREVISIÓN — Pulso · cierre semana 1 y 2

**Fecha de la pasada**: 2026-10-17, 12:10–12:20 (reloj del equipo).
**Quién**: el agente de la sesión, con el cliente NO disponible. Por eso esto es una **autorevisión
documentada** y no una sesión de pruebas con el cliente: donde no hay medida, se dice.
**Alcance**: las 4 pantallas, los 7 botones, el desplegable, el interruptor, el grid de «Datos», el
botón «Sincronizar», el informe HTML y los 3 CSV.
**Cómo se midió**: con los instrumentos del proyecto, no «mirando». Cada fila lleva el comando que
la sostiene; lo que no tiene comando está en la sección «lo que NO se pudo medir».

---

## 1. Qué funciona — y con qué se comprueba

| qué | medida de hoy | comando |
|---|---|---|
| **Las 4 pantallas** (Resumen, Zonas, Análisis, Datos) | las 4 se pintan y la barra reparte 411/4 con área táctil ≥ 48 | `cargo test --test medir-interfaz` |
| **La app arranca y pinta** sin abrir ventana | render en memoria con el MISMO camino de pintado que la app | `cargo test --test medir-estados` |
| **Hover de los 7 botones** (Guardar, Buscar, Sincronizar + 4 del selector) | «Guardar mis datos»: **1.708 px** cambian al pasar el puntero (el 2026-10-15 eran **0**) | `cargo test --test medir-estados -- --nocapture` |
| **Hover de la barra** (no se toca, se re-verifica) | **7.700 px** al pasar por «Zonas»; **0 px** al apartar el puntero (una señal que no se apaga miente) | idem |
| **Cursor** en los controles | `mouse-cursor: pointer` declarado en los 7 botones y en los 4 huecos de la barra | `grep -c mouse-cursor motor/ui/*.slint` |
| **Desplegable del sexo** | **1072 × 48** px (era 32) | `cargo test --test medir-interfaz el_desplegable -- --nocapture` |
| **Interruptor del tema** | **1072 × 48** px (era 20) | idem |
| **Grid de «Datos»** | las 4 filas de campos empiezan a la misma altura, desfase **≤ 1 px** (era 20) | `cargo test --test medir-interfaz el_grid -- --nocapture` |
| **Aire muerto entre «Guardar» y «Buscar»** | **18 px** (eran 63). Queda el canto de los dos marcos (4+4) más el hueco de 14: no es aire muerto | `cargo test --test medir-interfaz sin_mensaje -- --nocapture` |
| **«Sincronizar» responde** | el handler está conectado y la frase del resumen es la del contrato (números reales, en una línea, sin fechas) | `cargo test --test pruebas el_resumen_de_garmin` |
| **Informe HTML** | 4.096 bytes, con tabla, UTF-8, **sin JS ni CDN**; se genera desde los 102 FIT | `bash smoke-test.sh` |
| **3 CSV** | actividades 10.616 B (102 filas) · potencia 97.341 B (6.313 filas) · zonas 474 B (5 filas) | `bash smoke-test.sh` |
| **El informe no inventa números** | la guarda del motor da «No: todos los números… salieron de un cálculo» | `bash smoke-test.sh` |
| **El conjunto** | **430 pruebas en verde, 0 avisos, SMOKE TEST: VERDE** | `bash smoke-test.sh` |
| **Un dato que falta no se convierte en cero** | CSV: `Dato::Falta` → celda **vacía**; una prueba lo fija | `cargo test --lib un_dato_que_falta` |

---

## 2. Qué NO se pudo medir (y por qué) — el hueco se declara, no se tapa

| qué | por qué no | qué haría falta |
|---|---|---|
| **El cursor, por API** | Slint 1.18 no expone el cursor en su API pública (medido el 2026-10-15): `MouseCursorInner` vive en `i-slint-core`, que no se reexporta. Se comprueba **por su declaración**, no por su efecto | una versión de Slint que lo exponga, o una captura del sistema operativo |
| **El estado «pulsado» de los 7 botones** | el instrumento provoca hover, no pressed, sobre los botones de Datos; el halo nuevo no dibuja un estado distinto al pulsar | extender `auditar-estados` con pressed por botón y su portero |
| **El hover de los 4 botones del selector** | el selector es una CAPA que se superpone; hoy solo se mide el botón «Guardar» (los 7 comparten el mismo patrón) | un caso de prueba que abra la capa y provoque hover |
| **El foco de teclado más allá del paso 12** | el instrumento se queda en 12 TAB (hueco abierto del 2026-10-15) | extender la batería de foco a los botones |
| **Que LibreOffice/Excel abran los CSV** | no hay hoja de cálculo en este equipo; se comprueba la forma (cabecera, comas, UTF-8, comillas RFC 4180) | abrirlos en un equipo con LibreOffice |
| **Que el navegador pinte el HTML** | no se abre navegador en esta sesión; se comprueba que el documento es completo, sin dependencias externas y con el texto del informe | abrirlo con doble clic |
| **Que «Sincronizar» traiga datos de Garmin** | la RED está fuera de alcance (ADR): el handler pide, no promete; sin credenciales no hay sincronización real | una sesión con la cuenta del cliente |

---

## 3. Defectos que se encontraron AL REVISAR (no estaban en la lista)

1. **El halo de hover metido en el layout reservaba 48 px de alto.** Lo dejó la corrección del
   2026-10-15 y era invisible: el marco de «Buscar» medía **100 px** en vez de 56 y el hueco entre
   botones **66 px**. Se midió al escribir la prueba del aire muerto, y se corrigió sacando el halo
   del layout (posicionado absoluto) y poniendo el toque ENCIMA, con la misma acción que el botón.
   **Lección**: una corrección sin prueba de geometría puede dejar el mismo defecto con otro disfraz.
2. **`informe_html` no escapaba el texto** en su primera versión: un nombre de fichero con `<`
   rompía la página. Lo cazó su propia prueba (`un_nombre_de_fichero_con_menor_no_rompe_la_pagina`).
3. **La exportación de zonas usa el nombre en minúsculas** del motor («resistencia»): no es un fallo,
   pero si el cliente espera «Resistencia» en la hoja, se cambia en el motor, no en el CSV.

---

## 4. Deudas que quedan (documentadas, no arregladas)

| deuda | por qué no se arregla ahora |
|---|---|
| Pulsado de los botones sin medir | requiere extender el instrumento; el hover ya está |
| Coordenadas de `auditar-estados` obsoletas | el layout cambió (spacing 10→14 y líneas que ya no se dibujan): las capturas de hover de botones salen a 0 px **porque el puntero cae en otro sitio**, no porque el hover falle. Se comprobó con la prueba de píxeles, que pregunta la posición por nombre |
| Foco de teclado más allá del TAB 12 | deuda anterior, sin dueño todavía |
| Tipografías de «Datos» (15/11/11 vs 14/12/12) | defecto 7 de la auditoría, fuera de T002/T003 |
| Los campos de texto no dan hover propio | defecto 8 de la auditoría |
| Ejemplo con «Sin datos…» y tarjetas a la vez | defecto 12: es la captura de ejemplo, no el motor |

---

## 5. Veredicto

**Se puede enseñar**: las cuatro pantallas se pintan, los controles llegan al mínimo táctil, los
botones dan señal (medida en píxeles), el botón de sincronizar responde con su frase, y la entrega
—informe HTML y tres CSV— se genera desde los datos reales sin inventar un número.

**No se puede afirmar todavía**: que se vea bien en el teléfono del cliente, que el teclado llegue a
todo, y que los CSV abran en su hoja de cálculo. Eso pide una sesión con él, y es lo que queda para
después de Farellones.

*Autorevisión hecha con instrumentos del propio proyecto. Donde no hubo medida, se dice; no se
rellena el hueco con una impresión.*
