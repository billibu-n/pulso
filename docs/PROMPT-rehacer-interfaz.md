# PROMPT para rehacer la interfaz de `pulso` desde la norma

**Para el dueño**: esto es el encargo que se le pasa a un modelo de razonamiento que **escribe código
pero no ve**. El bucle visual lo cierra otro paso (ver el final). Está también en
`pulso/docs/PROMPT-rehacer-interfaz.md`.

---

## Arranque del chat nuevo (primer mensaje para pegar)

> Estoy retomando `pulso`. Quiero **rehacer la capa de interfaz desde la norma**, no parchearla. El
> encargo completo está en `pulso/docs/PROMPT-rehacer-interfaz.md` y el contrato en
> `pulso/docs/ESPECIFICACION-interfaz-resumen.md`. **Tú haces de ojos**: cada vez que se genere una
> captura (`datos/capturas/` o `/tmp`), la miras con visión, la comparas con la especificación y
> devuelves hallazgos concretos; el modelo que escribe código razona, pero **no ve**. Empezamos por la
> **fase 0** (congelar la especificación) y seguimos fase por fase, cada una con `cargo test`, la
> geometría (`--test medir-interfaz`) y su captura. Nada de parches sueltos.

---

## El encargo (texto para pegar)

Eres un ingeniero de interfaz. Vas a **rehacer la capa de interfaz** de la aplicación de escritorio
`pulso` (Rust + Slint), en el repositorio `~/projects/pulso`, en una rama nueva `ui-desde-cero`.

**El motor de cálculo NO se tira.** Se conserva entero (tiene más de 200 pruebas). Solo se toca cuando
la interfaz necesita un dato que hoy no entrega, y entonces **primero se escribe su contrato** (5 líneas:
qué recibe, qué devuelve y qué NO hace) y **su prueba**, y después el código.

### Lo primero: LEER (no escribir de memoria)

1. `~/.dsh/skills/norma-app/SKILL.md` — la norma universal. Aplica sobre todo: **AU-41 a AU-48** (una
   sola forma; lo que explica NO va en pantalla; un vacío se dice en UNA línea con su icono), **AU-63,
   AU-65, AU-66, AU-67** (medir antes de tocar; la disposición se comprueba; el diseño manda; una medida
   en un sitio) y **AU-89 a AU-93** (medidas de Android; un marcador de boceto no es producción; un
   vacío no se dibuja; no repetir mensajes; anclar el armazón a la ventana).
2. `pulso/docs/ESPECIFICACION-interfaz-resumen.md` — la auditoría de los 10 defectos y lo que debe ser
   cada bloque. **Es el contrato de esta tarea.**
3. `pulso/docs/ESTUDIO-interfaz-android-2026-10-14.md` — las medidas oficiales (Material 3) y su fuente.
4. `pulso/docs/SLINT-lo-que-hay-que-saber.md` — las trampas de Slint; cada una costó un fallo.
5. `pulso/docs/ARQUITECTURA-piezas-y-contrato.md` — las cinco capas; las dependencias apuntan hacia dentro.
6. `pulso/motor/ui/*.slint` y `pulso/motor/src/cara.rs` — lo que hay hoy (se reescribe, pero se lee).

### Reglas duras (no negociables)

- **NUNCA abras una ventana.** Para ver la interfaz se **captura fuera de pantalla**:
  `cargo run -q --bin capturar-ui -- <salida.ppm>`, y se convierte con `magick`. Los modos `--ayuda`,
  `--texto`, `--elegir` y `capturar-ui` devuelven antes de `run()`.
- **Toda medida se prueba**: `cargo test --test medir-interfaz` consulta la **geometría** real. Un cambio
  visual sin prueba de geometría **no está hecho** (AU-63/AU-65).
- **El repositorio es PÚBLICO**: ni un dato del cliente. `datos/` está ignorado por git; las capturas van
  a `datos/capturas/` y **no se suben**.
- **Un bloque = un commit**, solo con sus ficheros. NUNCA `git add -A`. Sin `sudo`.
- **Español** en comentarios, informes y textos de pantalla.
- **No cambies lo aprobado sin decirlo**: las 4 secciones de navegación, los dos temas, y que **no se
  llama «diagnóstico»** a nada.

### Qué hay que hacer, en fases (una por commit)

**Fase 0 · CONGELAR la especificación.** Completa `ESPECIFICACION-interfaz-resumen.md` con **los números
que faltan**: alto de cada bloque, márgenes, alineación (qué va centrado y qué a la izquierda),
tipografía (title-large 22 / body 14 / label 12), y **el estado vacío de cada bloque**. **No escribas
interfaz todavía.** La especificación tiene que permitir dibujar la pantalla **sin verla**.

**Fase 1 · El armazón.** Rehace `motor/ui/armazon.slint` y `temas.slint`: top app bar **64** con el
título de la **sección** a **22 px**; barra de abajo **80** con **iconos reales** de `iconos.slint` a
**24 px**, etiqueta **12**, **indicador de píldora**; radio **12**; margen **16**; área táctil **≥48**; y
el armazón **anclado a 411×880** (nada de huecos entre el contenido y la barra).

**Fase 2 · El Resumen** (lo que más se ve):
- **Las cifras**: cada tarjeta con **su número CENTRADO** y su etiqueta; **si el dato no existe, la
  tarjeta NO se dibuja** (nada de un «—» de 26 px).
- **El gráfico de zonas** — el error de fondo. Hoy la barra es `hasta/tope*100`, un número inventado.
  Debe ser **cada zona una BANDA sobre un EJE COMÚN en ppm** (de la mínima a la máxima), con el eje y su
  unidad. **El modelo entrega el tramo de cada zona** (inicio y fin en ppm), no un nivel 0–100. Si hay
  sesiones, se añade **tiempo en zona** (Garmin ya lo trae en `TimeInZone`) en su propio paso.
- **Los textos**: la precisión y lo que falta, **en UNA línea corta y con su icono**. El párrafo se va a
  la ayuda, nunca a la pantalla.
- El gráfico **existe en cuanto hay edad/umbral** y **se pinta** cuando llegan las sesiones.

**Fase 3 · Las otras pantallas** (Zonas, Sesiones, Datos), reutilizando los componentes de 1 y 2. Ajustes
y Datos usan los controles de serie (decisión del dueño) con su marco por fuera.

### Cómo se comprueba (la definición de TERMINADO)

Por cada fase, **las cuatro cosas**:
1. `cargo test` — todo verde y **0 avisos**.
2. `cargo test --test medir-interfaz` — la geometría, contra **los números de la especificación**.
3. `cargo run -q --bin capturar-ui -- /tmp/ui.ppm && magick /tmp/ui.ppm /tmp/ui.png`.
4. **Entregar la captura** y escribir **qué debe contener**. Tú no ves: **no afirmes «se ve bien»**.
   Enumera lo que la captura tiene que enseñar y deja que **quien vea** lo confirme.

### Trampas de Slint que ya costaron un fallo (leer el doc, no repetirlas)

- Dentro de un layout **no se ponen `x` ni `y`**.
- Una propiedad sin `in` es **privada** y el setter no existe (el compilador no avisa).
- `visible: false` **no libera el hueco**; un `if` sí.
- `Text {};` **con punto y coma no compila**.
- Un `Timer` **viene encendido**.
- `ScrollView` no se arrastra sin `mouse-drag-pan-enabled: true`.
- **Un solo punto de entrada** en `build.rs`.
- Para medir hace falta información de depuración.
- Los controles de serie **no aceptan nuestros colores**; hay que fijar `Palette.color-scheme`.

### Si tu entorno puede llamar a modelos de VISIÓN (subagentes)

Si tienes la herramienta para lanzar **subagentes** con un modelo que **vea** (o un MCP de visión),
**úsala** en el paso 4: pásale la **ruta del PNG** y **la especificación**, y pídele que conteste a las
tres preguntas (¿se entiende qué representa cada cosa? ¿está todo alineado y centrado? ¿sobra texto o
algún hueco?). Con tres reglas:

- **El que mira NO es el que escribe.** Ni el mismo modelo en el mismo hilo: se enamora de su código.
- Lo que devuelve la visión es un **indicio, no un veredicto**: **se confirma midiendo** (geometría). Un
  modelo de visión pequeño **da falsos «está bien»**; la norma lo dice: **la visión solo descarta**.
- Si **no** tienes visión a mano, **deja la captura y di exactamente qué hay que mirar**; no la declares
  buena. El visto bueno final de la **estética** lo da una persona que mira.

### El bucle visual lo cierra OTRO (esto es para quien orquesta)

Tú escribes a ciegas. Por cada fase, **otra persona o paso con visión** mira la captura y contesta a
tres preguntas: **¿se entiende qué representa cada cosa? ¿está todo alineado y centrado? ¿sobra texto o
algún hueco?** Solo entonces la fase se da por buena. **Medir dice «cabe»; ver dice «se entiende».** Hacen
falta las dos, y por eso el que escribe no puede ser el que valida.
