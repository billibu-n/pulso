# Las piezas de la aplicación, y el contrato entre ellas

**Escrito el 2026-10-14**, porque hasta ahora **no existía**. Este documento es la respuesta a una
crítica del dueño que es correcta y hay que dejar escrita:

> *«Toda tarea debe ser realizada por partes… como funciona cualquier cosa, por componentes, y luego se
> acoplan sin el miedo de dañar e interferir en el funcionamiento de uno. Pongámonos en el caso de un
> auto: el motor, la transmisión… Todos son piezas independientes que se acoplan entre sí.»*

**Y LO QUE PASÓ, dicho sin adornos**: se construyeron **el motor, la transmisión y las ruedas**, cada uno
por separado y bien —el motor con 158 pruebas— y **se descubrió al final que no había bancada**. La pieza
que acopla todo, `ventana.rs`, fue **la última y la más frágil**: se escribió de una vez, no compilaba, y
hubo que delegar el arreglo. **Eso no es mala suerte: es haber empezado a ejecutar sin los planos.**

---

## 1. La regla que gobierna este documento, y de dónde sale

**LAS DEPENDENCIAS APUNTAN HACIA DENTRO.** El motor **no conoce** la base de datos. La base de datos
**conoce** el motor. Quien tiene que cambiar cuando cambia el mundo exterior es **el borde**, nunca el
centro.

**Es la arquitectura de puertos y adaptadores**, y **no la invento yo**: la formuló **Alistair Cockburn**
(«Hexagonal Architecture», 2005) y su regla central es exactamente esa — *la aplicación se comunica con
el exterior por puertos, y cada puerto tiene uno o varios adaptadores intercambiables*. La formulación
de **Robert C. Martin** («The Clean Architecture», 2012) la generaliza con **la Regla de la Dependencia**:
*las dependencias del código fuente solo pueden apuntar hacia dentro*.

> **Y esta cita está pendiente de verificar.** El documento del estado del arte lo está comprobando
> (§6, arquitectura). **Si la cita no se sostiene, esta sección se corrige**: en este proyecto una
> regla sin fuente es una opinión con buena letra.

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

**La prueba de que esta capa está bien separada**: `cargo test --test pruebas` → **139 pruebas que no
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
| **`bin/ventana.rs`** | **une**: lee argumentos, carga, pide lo que hay que enseñar, y lo escribe | **no calcula y no decide nada** |
| **`temas/contraste*.py`** | mide el contraste WCAG de los dos temas y de los controles de serie | **mide, no opina** |

**`ventana.rs` es la bancada de pruebas del auto**: sujeta el motor, pero **no es el motor**. Y su
tamaño lo delata: **si `ventana.rs` crece, es que se está colando lógica dentro.**

---

## 3. El contrato entre las piezas, y cómo se comprueba que se respeta

**Un contrato que no se comprueba no es un contrato: es un deseo.** Estos son los que ya existen, con
**su portero**:

| contrato | quién lo comprueba |
|---|---|
| **el motor no conoce el mundo exterior** | `cargo test --test pruebas`: 139 pruebas **sin tocar un fichero de fuera** |
| **lo que se enseña se puede obtener sin ventana** | `cargo test --test recorrido`: 11 pruebas |
| **sin datos de la persona, NO se enseña ningún número** | la prueba del paso 1 del recorrido, y **una que busca los números prohibidos en el texto** |
| **`Falta` no es cero** | pruebas del perfil: un campo vacío vuelve como `Falta` |
| **la memoria no miente**: si los ficheros cambian, no se usa | 5 pruebas de `memoria.rs`, incluida la de la memoria corrupta |
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
4. **La pieza que acopla se diseña al principio, no al final.** Es el error de hoy: `ventana.rs` fue
   lo último y lo más frágil.

**Y una consecuencia práctica**: cuando una pieza se pueda **tirar y rehacer** sin tocar las demás,
estará bien hecha. **Eso es lo que se puede comprobar, y por eso es lo que se exige.**

---

## 5. Lo que falta en este mapa, dicho claro

- **`huawei.rs` no existe.** Es la pieza que desbloquea la regla del agua corporal, y está en
  investigación (formato del ZIP, nombres de campo, unidades).
- **La fecha de nacimiento** no está en `Persona`. La edad está escrita a mano y **caduca el 25 de
  enero**.
- **Ninguna pieza del borde sabe elegir una carpeta.** Hoy la ruta entra por línea de órdenes.
- **Y la cita de Cockburn está sin verificar.** Si no se sostiene, se corrige: en este proyecto una
  regla sin fuente es una opinión con buena letra.
