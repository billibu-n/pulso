# Las piezas de la aplicación, y el contrato entre ellas

**Escrito el 2026-10-14**, porque hasta ahora **no existía**. Este documento es la respuesta a una
crítica del dueño que es correcta y hay que dejar escrita:

> *«Toda tarea debe ser realizada por partes… como funciona cualquier cosa, por componentes, y luego se
> acoplan sin el miedo de dañar e interferir en el funcionamiento de uno. Pongámonos en el caso de un
> auto: el motor, la transmisión… Todos son piezas independientes que se acoplan entre sí.»*

**Y LO QUE PASÓ, dicho sin adornos**: se construyeron **el motor, la transmisión y las ruedas**, cada uno
por separado y bien —el motor con 183 pruebas— y **se descubrió al final que no había bancada**. La pieza
que acopla todo, `pulso.rs` (entonces `ventana.rs`), fue **la última y la más frágil**: se escribió de una vez, no compilaba, y
hubo que delegar el arreglo. **Eso no es mala suerte: es haber empezado a ejecutar sin los planos.**

---

## 1. La regla que gobierna este documento, y de dónde sale

**LAS DEPENDENCIAS APUNTAN HACIA DENTRO.** El motor **no conoce** la base de datos. La base de datos
**conoce** el motor. Quien tiene que cambiar cuando cambia el mundo exterior es **el borde**, nunca el
centro.

**Es la arquitectura de puertos y adaptadores**, y **no la invento yo**. La formuló **Alistair Cockburn**
en **«Hexagonal Architecture (Ports & Adapters)»**, *HaT Technical Report 2005.02*, del **4 de septiembre
de 2005** — <https://alistair.cockburn.us/hexagonal-architecture/>. **Consultado y verificado el
2026-10-14.** Y dice, literal:

> *«The asymmetry to exploit is not that between left and right sides of the application but between
> **inside** and **outside**. **The rule to obey is that code pertaining to the inside part should not
> leak into the outside part.**»*

**Traducido**: la regla no es «izquierda contra derecha», es **dentro contra fuera**. **El código de
dentro no se filtra hacia fuera.**

**Y el artículo dice TRES cosas más que dan razón a este proyecto, y las tres estaban ya aquí sin
saberlo:**

1. **Describe el problema exacto que este proyecto vivió**: *«one of the great bugaboos… has been
   infiltration of business logic into the user interface code»* — **lógica de negocio metida en la
   interfaz**. Y explica la consecuencia: *«the system can't neatly be tested with automated test suites
   because part of the logic needing to be tested is dependent on oft-changing visual details»*.
   **Eso es literalmente lo que pasaba: el recorrido del usuario no se podía probar sin abrir la
   ventana.**
2. **Y avisa de por qué no basta con prometerlo**: *«having no mechanism to detect when a violation of
   that promise occurs, the organization finds a few years later that the new layer is cluttered with
   business logic and the old problem has reappeared»*. **Una promesa sin mecanismo que la detecte se
   rompe sola** — y por eso aquí cada contrato lleva **su portero** (ver §3).
3. **Y da el criterio de terminado de una arquitectura de estas**: *«the ultimate benefit of a ports and
   adapters implementation is the ability to run the application in a **fully isolated mode**»*.
   **Eso es exactamente `cara.rs`**: el recorrido del usuario **se comprueba aislado, sin ventana.**

> **Y su orden de desarrollo recomendado es el que NO seguimos**, y merece quedar escrito: el artículo
> describe empezar **con las pruebas**, luego la interfaz, luego el almacén de verdad — y en su ejemplo
> el primer paso es *«FIT + App + constant-as-mock-database»*. **Nosotros empezamos por el motor (bien) y
> dejamos el acoplamiento para el final (mal).** La arquitectura no es un dibujo: es **un orden de
> trabajo**.

**Lo que esto significa en la práctica, y es lo que da la garantía que pide el dueño**: se puede
**tirar y rehacer una pieza del borde sin tocar el motor**, y el motor **no se entera**. Ese es el
«sin miedo a dañar» — no es confianza, es **dirección de las flechas**.

---

## 2. Las piezas, de dentro hacia fuera

Cada una dice: **qué es**, **qué NO es**, **quién depende de quién**, y **cómo se comprueba sola**.

### CAPA 1 · EL CENTRO: lo que se puede afirmar

| pieza | qué es | qué NO es |
|---|---|---|
| **`datos.rs`** | el modelo: `Persona`, `Sesion`, `Muestra`, `Dato<T>` | no lee ficheros, no formatea, no decide |
| **`rango.rs`**, **`juicio.rs`**, **`fuentes.rs`** | **cuánto vale un número y de dónde sale**: el margen, la confianza, la procedencia | no calcula fisiología |
| **`formulas*.rs`**, **`umbral.rs`**, **`esfuerzo.rs`**, **`patron.rs`**, **`hrv.rs`**, **`tendencia.rs`**, **`potencia.rs`**, **`avisos.rs`** | **el cálculo**, cada uno con su fórmula citada | **no saben qué es un FIT, ni un JSON, ni una pantalla** |

**La prueba de que esta capa está bien separada**: `cargo test --test pruebas` → **150 pruebas que no
tocan ningún fichero de fuera**. Se construyen personas y sesiones **a mano** y se comprueba el cálculo.
**Si alguna de estas pruebas necesitara un FIT, la capa estaría mal.**

### CAPA 2 · LOS ADAPTADORES: cómo entra el mundo

| pieza | qué es | de quién depende |
|---|---|---|
| **`fit.rs`** | lee **FIT** de Garmin y lo traduce a `Sesion` | **de la capa 1** (habla su idioma) |
| **`adaptador.rs`** | lee el **JSON** del perfil | de la capa 1 |
| **`profile.rs`** | **guarda y lee los datos de la persona**, junto al programa | de la capa 1 |
| *(pendiente)* **`huawei.rs`** | leerá la **exportación de Huawei** | de la capa 1 |

**La regla que las hace intercambiables**: un adaptador **traduce y no decide**. Si un dato no viene,
devuelve **`Dato::Falta`** — **nunca un cero**. Un cero es un dato falso, y esa es la regla que sostiene
todo el proyecto.

**Y aquí está la prueba de que el dueño tiene razón**: el día que Huawei cambie su formato, **cambia
`huawei.rs` y no se toca una línea del motor**. Eso es una pieza del auto, no una cirugía.

### CAPA 3 · EL ALMACÉN: lo que se recuerda

| pieza | qué es | por qué está separada |
|---|---|---|
| **`memoria.rs`** | guarda el resultado del análisis con una **huella** de lo leído | **medido: 5,57 s → 0,032 s** |
| **`analisis.rs`** | **el análisis de la carpeta entera, en un solo sitio** | para poder **nombrarlo** (guardarlo) y **probarlo** (sin abrir ventana) |

**Las dos existen por el mismo motivo**: antes el análisis vivía **dentro de `main`**, y entonces
**no se podía guardar** (no había forma de nombrarlo) ni **probar** (había que abrir la ventana). **Un
dato que no se puede nombrar no se puede guardar, y uno que no se puede probar no se cree.**

### CAPA 4 · LA DECISIÓN: qué se enseña

| pieza | qué es | la regla que cumple |
|---|---|---|
| **`cara.rs`** | `lo_que_ensena(&Persona, Option<&Real>) -> Ensenado` — **función pura** | **aquí se decide QUÉ se enseña; en `.slint`, CÓMO se ve** |

**Es la pieza que hace comprobable el recorrido del usuario**: **11 pruebas** recorren los pasos del
usuario **sin abrir una ventana**. Sin esta separación, comprobar que la aplicación sirve exigiría
**mirarla** — y eso la norma lo prohíbe (AU-65).

### CAPA 5 · EL BORDE: el mundo

| pieza | qué es | quién decide |
|---|---|---|
| **`ventana.slint`** | el aspecto: los dos temas, el armazón, las piezas | **decisión de diseño, con fuente** |
| **`bin/pulso.rs`** | **une**: lee argumentos, carga, pide lo que hay que enseñar, y lo escribe | **no calcula y no decide nada** |
| **`temas/contraste*.py`** | mide el contraste WCAG de los dos temas y de los controles de serie | **mide, no opina** |

**`pulso.rs` es la bancada de pruebas del auto**: sujeta el motor, pero **no es el motor**. Y su
tamaño lo delata: **si `pulso.rs` crece, es que se está colando lógica dentro.**

---

## 3. El contrato entre las piezas, y cómo se comprueba que se respeta

**Un contrato que no se comprueba no es un contrato: es un deseo.** Estos son los que ya existen, con
**su portero**:

| contrato | quién lo comprueba |
|---|---|
| **el motor no conoce el mundo exterior** | `cargo test --test pruebas`: 150 pruebas **sin tocar un fichero de fuera** |
| **lo que se enseña se puede obtener sin ventana** | `cargo test --test recorrido`: 11 pruebas |
| **sin datos de la persona, NO se enseña ningún número** | la prueba del paso 1 del recorrido, y **una que busca los números prohibidos en el texto** |
| **`Falta` no es cero** | pruebas del perfil: un campo vacío vuelve como `Falta` |
| **la memoria no miente y no relee lo que ya tiene** | pruebas de `fit::listar` + `analisis::decidir` + la memoria corrupta, y `medir-carga` sobre datos reales (0 ficheros releidos) |
| **lo que va a la pantalla son 4 cifras y 5 zonas** | la ventana indexa 0..3 y 0..4: si un array llega corto, **falla** |
| **los dos temas cumplen WCAG** | `temas/contraste.py` y `contraste-widgets.py`, que **salen con error** si bajan del mínimo |
| **el esquema de los controles sigue a nuestro tema** | una prueba que lee `controles_en_oscuro` |

---

## 4. Lo que este documento cambia a partir de ahora

**Cuatro reglas de trabajo, y son la respuesta a la crítica:**

1. **Antes de escribir código de una pieza, se escribe su contrato** — qué recibe, qué devuelve, y
   **qué NO hace**. Si el contrato no cabe en cinco líneas, la pieza es demasiado grande.
2. **Las dependencias apuntan hacia dentro.** Si un módulo del centro necesita saber qué es un FIT,
   **está mal puesto**.
3. **Cada pieza trae sus pruebas, y se ejecutan solas.** Una pieza que necesita las demás para
   probarse **no está terminada**.
4. **La pieza que acopla se diseña al principio, no al final.** Es el error de hoy: `pulso.rs` fue
   lo último y lo más frágil.

**Y una consecuencia práctica**: cuando una pieza se pueda **tirar y rehacer** sin tocar las demás,
estará bien hecha. **Eso es lo que se puede comprobar, y por eso es lo que se exige.**

---

## 5. Lo que falta en este mapa, dicho claro

- **`huawei.rs` YA EXISTE** (2026-10-14): lee la exportación de Huawei y desbloquea la regla del agua
  corporal. El formato está en `docs/HUAWEI-exportacion.md`, el contrato en `docs/CONTRATO-huawei.md` y
  sus 12 pruebas en `tests/huawei.rs`. **Falta conectarla**: que `Hoy` lleve el agua **y su fecha**.
  **Abre el ZIP tal cual sale del correo** (dependencia `zip` autorizada por el dueño: minimizar el trabajo
  del cliente), así que él no descomprime nada.
- **La fecha de nacimiento YA ESTÁ en `Persona`** (`datos.rs`), y la edad se calcula al usarla. Lo que
  falta es **exponerla en la pantalla de Datos**: el campo está en el modelo y en el fichero, pero no en
  el formulario.
- **Ninguna pieza del borde sabe elegir una carpeta.** Hoy la ruta entra por línea de órdenes. **Medido**:
  Slint 1.18.1 **no trae `FileDialog` ni `FolderDialog`**; el selector hay que **escribirlo** (sin
  dependencia nueva) o autorizar `rfd`.
- **El estado del arte está REVISADO fila a fila** (2026-10-14): `docs/ESTADO-DEL-ARTE.md` se reescribió
  tras abrir las 41 direcciones y comprobar cada DOI. Aparecieron **fuentes falsas** (DOI inexistentes,
  un autor inventado, un artículo que era de otro tema) y se corrigieron o se dejaron **SIN FUENTE
  LOCALIZADA**. Quedan **14 filas sin fuente** y las citas numéricas de las URLs que bloquean a los
  programas **sin verificar**: 403 no es URL muerta.
