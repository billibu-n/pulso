# Especificación de la interfaz (fase 0: CONGELADA)

**Escrito el 2026-10-14**, después de que el dueño viera la aplicación y dijera, con razón, que el
trabajo era insuficiente. Sus palabras son el encargo de este documento:

> *«Está el texto corrido, los botones no centrados, el gráfico no sé qué representa, debería estar
> desde el inicio y ahí pintar… Hay muchos errores en el modelo o en la interfaz, debemos entregar algo
> legítimo y limpio… hiciste una corrección mínima cuando hay varios detalles.»*

**FASE 0 · cerrada el 2026-10-14.** Este documento deja de ser una auditoría con propuestas y pasa a ser
**el CONTRATO de las fases 1 a 3** (armazón, Resumen, y Zonas/Sesiones/Datos). Regla de uso, y es
literal: **quien construya no dibuja nada que no esté aquí, y nada que esté aquí se cambia sin cambiar
antes este documento** (AU-66, `norma-app` regla de la sección 9-ter).

**Cómo se cita.** Cada número de esta especificación sale de una fuente, y se cita como `doc:línea`:

| abreviatura | documento | qué es |
|---|---|---|
| **estudio:N** | `docs/ESTUDIO-interfaz-android-2026-10-14.md` | las medidas oficiales de Android / Material 3 y su dirección |
| **auditoría:N** | este mismo documento (las tablas de abajo) | los defectos medidos en la captura |
| **slint:N** | `docs/SLINT-lo-que-hay-que-saber.md` | las trampas del lenguaje, cada una con el fallo que costó |
| **estado:N** | `ESTADO.md` | las decisiones del dueño y lo medido |
| **arq:N** | `docs/ARQUITECTURA-piezas-y-contrato.md` | las cinco capas y el contrato entre piezas |
| **cara.rs:N** | `motor/src/cara.rs` | **lo que el motor entrega hoy** (se reescribe la vista, no esto) |
| **ventana.slint:N** | `motor/ui/ventana.slint` | la vista de hoy, que **se tira**: se cita para saber qué se está tirando |
| **armazon.slint:N** | `motor/ui/armazon.slint` | el armazón de hoy, que **se rehace** |
| **AU-N** | `~/.dsh/skills/norma-app/SKILL.md` | la norma universal de aplicaciones |

**Y lo que NO está medido se dice: al final hay una sección de huecos del motor** (§E13). Un número sin
fuente es una opinión con buena letra (AU-49, AU-50, AU-82).

---

## 0. Los números de la pantalla, en UN solo sitio (AU-67, `slint:186`)

**MEDIDO HOY en la aplicación**: el archivo `motor/ui/armazon.slint:27-52` **ya declara estas medidas una
sola vez**, en el `global Medidas`, y `motor/tests/medir-interfaz.rs` las comprueba contra la geometría
real. **Eso se conserva tal cual.** Esta tabla es su fuente de verdad declarada, y quien construya la cita
desde aquí:

| medida | valor | fuente |
|---|---|---|
| **lienzo (ventana)** | **411 × 880** | `armazon.slint:36-37` · prueba: `motor/tests/medir-interfaz.rs:88-89` |
| **barra superior** (top app bar small) | alto **64** | `estudio:24` (AndroidX `TopAppBarSmallTokens.kt`) |
| **título de la barra superior** | **22 px** (title-large), **a la izquierda** | `estudio:24`, `estudio:94` |
| **barra inferior** (navigation bar con etiquetas) | alto **80** (token `TallContainerHeight`) | `estudio:25` |
| **icono de la barra inferior** | **24** | `estudio:25` |
| **etiqueta de la barra inferior** | **12 px** (label-medium) | `estudio:25` |
| **píldora del activo** | `CornerFull` **DETRÁS del icono**, no una línea arriba | `estudio:26` |
| **área táctil mínima** | **48** | `estudio:27` (`minimumInteractiveComponentSize`) |
| **radio de tarjeta** | **12** | `estudio:28` (Material 3, cards) |
| **margen de la pantalla** | **16** | `armazon.slint:49` · `estado:469-491` |
| **hueco entre bloques del contenido** | **14** | `armazon.slint:117` y `armazon.slint:177` |
| **tipografía** | title-large **22** · body **14** · label **12** | `estudio:24-25`, `estudio:94`; encargo del dueño |
| **secciones de navegación** | **4**: Resumen, Zonas, Sesiones, Datos | `ventana.slint:516-520` · `estado:280` |
| **temas** | **2**: claro cálido y **negro OLED** | `estado:26` · `motor/ui/temas.slint` |

**Las cuatro secciones, con su icono** — el icono es **dato**, no decoración, y sale del catálogo de
187 iconos de Lucide (`estado:28`, `ventana.slint:516-520`):

| # | sección | icono (`iconos.slint`) | qué es |
|---|---|---|---|
| 0 | **Resumen** | `casa` | lo que se puede afirmar hoy, en cifras, zonas y una línea |
| 1 | **Zonas** | `grafico-linea` | las cinco zonas **con su tramo en ppm** y, si hay sesiones, el tiempo en cada una |
| 2 | **Sesiones** | `bicicleta` | **las sesiones, una por fila** |
| 3 | **Datos** | `ajustes` | los datos de la persona, la carpeta de sus ficheros y el tema |

**Y NADA SE LLAMA «DIAGNÓSTICO»** (`estado:280`, decisión del dueño del 2026-10-14): la palabra se usa
**solo** como nombre del informe global —el texto que dice «NO SE PUEDE DIAGNOSTICAR TODAVÍA»— y en el
campo `puede_diagnosticar` de `cara.rs:42`, que es interno. En pantalla no aparece. El aviso de la
rodilla dice el sentido, no el término: `avisos::NO_ES_VALORACION_MEDICA` (`estado:280`).

---

## 1. Por qué me estaba costando (diagnóstico, sin excusas) — *sin cambios*

No es la herramienta ni el método. Es esto:

1. **Parcheaba síntomas en vez de construir contra un objetivo.** Arreglé «los iconos», luego «los
   vacíos», luego «la altura», cada uno por separado. Sin **una especificación de la pantalla**, eso no
   puede quedar coherente. El boceto **no es una especificación**: es un marcador (AU-90, `estudio:56-60`).
2. **Mezclé dos capas en el mismo cambio.** Ocultar contenido (interfaz) movió la barra (disposición) y
   tuve que perseguirla. Son dos problemas distintos y se arreglan en dos pasos, no en uno.
3. **Miré la captura para CONFIRMAR, no para JUZGAR.** Me pregunté «¿están los iconos?» en vez de
   «¿esto se entiende, está alineado, dice algo?». El dueño lo vio en diez segundos.
4. **El gráfico está mal desde el MODELO, no desde el dibujo.** La barra de cada zona mide su **techo**
   respecto al techo mayor (67 %…100 %). Eso no representa nada: ningún ajuste de dibujo lo arregla.

**La documentación no es el problema.** Lo que faltaba es la pantalla **como pieza de diseño** (AU-66 y
AU-74), y eso es lo que este documento pone.

---

## 2. Auditoría: TODO lo que está mal en la captura — *sin cambios*

| # | lo que se ve | por qué está mal | dónde nace |
|---|---|---|---|
| 1 | «Precisión aceptable: las zonas salen del estimado a partir de la frecuencia máxima, que es una estimación.» | **texto corrido**: es un párrafo, y la pantalla no se explica (AU-44/AU-45) | `cara.rs` `texto_precision` |
| 2 | «Faltan tus datos: con la edad y el umbral ya se calculan las zonas. Se rellenan en Datos.» | dos frases; un vacío se dice en UNA línea | `cara.rs` `que_falta` |
| 3 | las 5 barras de zona, casi todas llenas, sin eje ni unidad | **la barra no representa nada**: es el techo de la zona dividido por el techo mayor | `cara.rs`: `nivel = hasta/tope*100` |
| 4 | «hasta 114», «114 – 127» | mezcla dos notaciones y no dice la unidad (ppm) | `cara.rs` `tramo` |
| 5 | 3 de 4 tarjetas con un «—» de 26 px a la izquierda | tarjetas casi vacías; el guion no informa y desequilibra | `cara.rs` + `ventana.slint` |
| 6 | cifras y filas alineadas a la izquierda con la tarjeta ancha | el dueño las ve como botones mal centrados | `ventana.slint` `Cifra`/`Zona` |
| 7 | un hueco grande entre el último bloque y la barra | el contenido corto no se reparte; el hueco se ve | `armazon.slint` |
| 8 | los iconos de la barra, tenues y pequeños | los elegí a ojo, sin verlos a tamaño real | `iconos.slint` |
| 9 | el título dice «Pulso», no la sección | el top app bar de Android dice dónde estás | `ventana.slint` |
| 10 | no hay **acción primaria** | el recorrido pide «pulsar para analizar»; hoy calcula al abrir y no hay botón | producto |

**Y la comprobación objetiva de los defectos 3, 5 y 9 está medida, no opinada** (`estudio:34-54`): la
barra de abajo medía 64 (norma 80), la cabecera 56 (norma 64), el radio 10 (norma 12), la etiqueta 11
(norma 12), el icono 18 (norma 24); con la persona sin datos salían **cuatro tarjetas y cinco barras
vacías** y **una barra de precisión en blanco**; y «Sin datos de entrenamiento todavía.» salía **dos
veces** (`estudio:45-53`).

---

## 3. Lo que cada bloque debe ser — *contrato, ya decidido*

Lo que sigue es la parte que se congela. **Ningún bloque existe si no está en esta tabla**, y ningún
bloque que esté aquí se dibuja de otra forma.

### 3.0 · Las reglas que gobiernan TODOS los bloques

| regla | qué obliga | fuente |
|---|---|---|
| **Un vacío NO se dibuja, ni su marco** | sin dato, la tarjeta no existe: se saca del árbol con `if`, **nunca** con `visible: false` | AU-91, AU-80, `slint:376-379` |
| **Un vacío se dice en UNA línea, con su icono** | no un párrafo, y no dos veces lo mismo | AU-45, AU-92 |
| **Lo que se explica NO va en pantalla** | va a la ayuda y se pide | AU-44 |
| **La pantalla no repite un mensaje** | si el recuento ya dice «sin datos», el aviso no lo repite | AU-92, `estudio:53` |
| **Prohibido el «—» de 26 px** | un guion grande no informa y desequilibra la fila | auditoría:35, `ventana.slint:68` |
| **`Palette.color-scheme` se fija desde la raíz** | si no, el texto del control de serie sale a 1,05 : 1 (mínimo 4,5 : 1) | AU-84, `slint:436-457`, `ventana.slint:533-541` |
| **Una medida se escribe UNA vez** | los números viven en `global Medidas` y las pruebas los comprueban contra su fuente | AU-67, AU-73, `slint:186` |
| **Nada de `x`/`y` dentro de un layout** | si hace falta separar, `padding` y `spacing` | `slint:19-35` |
| **Toda propiedad que rellene el motor lleva `in`** | sin `in`, el setter no existe y el compilador no avisa | `slint:338-354` |
| **La disposición se COMPRUEBA** | `cargo test --test medir-interfaz`, con los números de esta especificación | AU-65, `slint:127-133` |

---

### 3.1 · El cromo de arriba (top app bar)

| qué | valor | fuente |
|---|---|---|
| alto | **64** | `estudio:24` · `armazon.slint:44` |
| fondo | `sup` del tema | `armazon.slint:154` |
| título | **la SECCIÓN** (Resumen/Zonas/Sesiones/Datos), **no** el nombre de la app | auditoría:39, 44-45 · `estado:469-491` |
| tamaño del título | **22 px** (title-large), peso 700 | `estudio:24` · `armazon.slint:158` |
| alineación | **a la izquierda**, con `padding` de **16** desde el borde | `armazon.slint:160` |
| vertical | centrado en los 64 | `armazon.slint:160` |
| papeles de accesibilidad | `banner` con `accessible-id: "cabecera"` | `armazon.slint:151-152` |

**Comportamiento**: el título **cambia con la sección** y no hay botón de volver en la raíz (no hay a
dónde volver). **La app se llama «Pulso», y su nombre NO va en la barra** (auditoría:39, 44-45). Si el
título de una sección no cabe, **no se recorta el texto**: se acorta el nombre en la tabla de secciones
(§0), que es una decisión y no un accidente de la disposición (AU-68).

### 3.2 · La barra de navegación de abajo

| qué | valor | fuente |
|---|---|---|
| alto | **80** | `estudio:25` · `armazon.slint:45` |
| huecos | **4**, **repartidos por igual**: `parent.width / 4` (= 102,75 en 411) | AU-68 · `slint:37-53` · `armazon.slint:196` |
| icono | **icono real** de `iconos.slint` a **24** | `estudio:25`, `estudio:34-35`, AU-90 |
| iconos concretos | `casa`, `grafico-linea`, `bicicleta`, `ajustes` | `estado:472-476` · `ventana.slint:516-520` |
| etiqueta | **12 px** (label-medium), **centrada** en su hueco | `estudio:25` · `armazon.slint:224-225` |
| indicador | **píldora** detrás del icono: 64 × 32, radio 16, acento al 18 % | `estudio:26` · `armazon.slint:205-209` |
| área táctil | **≥ 48**, por hueco entero | `estudio:27` · `armazon.slint:199` |
| orden vertical de cada hueco | píldora+icono arriba, etiqueta debajo, **2 de hueco**, todo centrado | `armazon.slint:200-228` |
| papeles | `navigation`, `accessible-id: "navegacion"` | `armazon.slint:184-185` |
| huecos entre sí | `spacing: 4` y `padding` lateral 4: **sin esto sale «ResumenZonasSesionesDatos»** | `slint:37-53` · `armazon.slint:191-193` |

**Comportamiento**: una acción **una sola forma** (AU-41): el mismo icono, el mismo tamaño y el mismo
color para la misma sección en toda la aplicación. El estado inactivo usa `muted`; el activo, `ac`
(`armazon.slint:216-227`). **No hay huecos vacíos esperando una quinta sección**: si un día hay tres
secciones, el divisor es 3 (AU-08: instalar o quitar no reajusta la geometría a mano).

### 3.3 · Contenido, y el ancla a la ventana (AU-93)

| qué | valor | fuente |
|---|---|---|
| margen del contenido | **16** por los cuatro lados | `armazon.slint:175` · `estado:469-491` |
| hueco entre bloques | **14** | `armazon.slint:177` |
| alto del contenido | **`alto − cabecera − barra = 880 − 64 − 80 = 736`**, declarado como **altura MÍNIMA** | AU-93 · `armazon.slint:172` |
| papel | `main`, `accessible-id: "contenido"` | `armazon.slint:167-168` |

**Por qué la altura mínima está declarada, y no se confía en el layout**: al dejar de dibujar los vacíos
(AU-91), el armazón **encogió y la barra de abajo subió al centro de la pantalla**; se cazó **midiendo**,
no mirando (`estado:486-487`, `armazon.slint:170-172`). **Consecuencia para toda fase: cualquier bloque
que se añada o se quite NO puede mover la barra.**

**Si el contenido no cabe** en 736: se desplaza **la página**, no se reparte la ventana (AU-05). El
contenido se envuelve en un `ScrollView` con `mouse-drag-pan-enabled: true` — **sin esa línea no se
arrastra con el dedo en el móvil** (`slint:380-392`).

### 3.4 · La acción primaria — **DECISIÓN PENDIENTE, y no se inventa**

Hoy **calcula al abrir** y **no hay ningún botón** (auditoría:40, `estudio:62-67`, `estado:488-491`).
El recorrido pide «pulsar para analizar» (`RECORRIDO-del-usuario.md:31`, paso 6).

**Lo que la especificación fija, para cuando se decida**: si hay acción primaria, es **un `Button` de
serie con `primary: true`**, con **marco nuestro por fuera**, a **ancho completo** y **≥ 48** de alto
(`slint:485-497`, `ventana.slint:376-395`). **Lo que NO se hace**: inventarla. Queda fuera de las fases
1-3 mientras el dueño no la decida.

---

### 3.5 · Las cifras del Resumen — **el contrato**

**Cada tarjeta lleva su número CENTRADO y su etiqueta debajo. Si el dato no existe, LA TARJETA NO SE
DIBUJA** (encargo del dueño; AU-91, `estudio:45-51`).

**Hoy** (`ventana.slint:48-80`): una tarjeta `Cifra`, alto **66**, radio 12, borde 1 px `linea` al 45 %,
`padding` 12, `spacing` 3, número a **26 px** y etiqueta a **12 px** — **y el número sale a la IZQUIERDA**,
que es el defecto 6 de la auditoría (auditoría:36).

| qué | valor | fuente |
|---|---|---|
| disposición | **2 columnas** por fila, `spacing: 12`; hasta 2 filas | `ventana.slint:170-179` |
| alto de la tarjeta | **66** con número+etiqueta; **52** sin margen — **y NO se cambia**: hay una prueba que cuenta las tarjetas **por su alto** | `ventana.slint:55`, `ventana.slint:219`, `medir-interfaz.rs:447-468` |
| radio / borde | **12** / 1 px de `linea` al 45 % | `estudio:28` · `ventana.slint:56-60` |
| relleno interior | **12**, `spacing: 3` | `ventana.slint:64-65` |
| **el número** | **CÉNTRADO horizontal y verticalmente** | auditoría:36, 47-48 |
| tipografía del número | **22 px** (title-large) — **no 26**. **El alto de la tarjeta sigue siendo 66**: solo cambia el tamaño del texto, porque hay una prueba que localiza las tarjetas **por su altura** (66) y las filas de zona **por la suya** (40) | encargo del dueño; `estudio:24` · `medir-interfaz.rs:447-468` |
| tipografía de la etiqueta | **12 px** (label), `muted`, **CENTRADA** | `estudio:25` · `ventana.slint:76` |
| margen (±) | **va pegado al número**, en la misma cadena — **no** en una nota al pie | `ventana.slint:45-47` |
| acento | el número en `ac` **solo si el dato es de los que el cliente mira primero** | `ventana.slint:70-72` |
| el margen de la cifra | **el motor lo entrega dentro del número**: `169±3` (`cara.rs:142`) | `cara.rs:142` |

**La lista de cifras, en orden, con el dato que las hace existir** (`cara.rs:141-159`):

| # | cifra | etiqueta | cuándo EXISTE | cuándo NO se dibuja |
|---|---|---|---|---|
| 1 | umbral `valor±margen` | «umbral (pulsaciones)» | siempre que se pueda diagnosticar | si no hay umbral: **no existe la pantalla de cifras entera** |
| 2 | frecuencia máxima | «frecuencia máxima» | **solo si está MEDIDA** | **sin máxima medida, la tarjeta NO se dibuja** |
| 3 | vatios por pulso | «vatios por pulso» | **solo si hay eficiencia media** | sin sesiones: no se dibuja |
| 4 | deriva % | «deriva %» | **solo si hay deriva media** | sin sesiones: no se dibuja |

**EL «—» ESTÁ PROHIBIDO, y esto es una corrección, no un gusto**: hoy el motor manda `"—"` cuando el
dato falta (`cara.rs:143-152`) **y la vista lo pinta igual** — y el «—» a 26 px es el defecto 5 de la
auditoría (auditoría:35). **Lo que se hace en su lugar**: la tarjeta **se saca del árbol**. Y **el motor
deja de mandar el guion**: manda «esta cifra no existe» (§E13, hueco 2). **El cero también está
prohibido**: un `0` donde no se midió nada es un dato falso con buena cara (AU-85, `estado:440-447`).

**Regla de composición**: con 4 cifras, dos filas de dos; con 3, dos arriba y una abajo (en la columna de
la izquierda, para que la rejilla no baile); con 2 o 1, una fila. **Nunca una tarjeta vacía para completar
la rejilla** (AU-07, AU-24, `estudio:51`).

### 3.6 · El gráfico de zonas — **el fallo de fondo, y su contrato**

**Hoy**: `nivel = hasta / tope * 100` (`cara.rs:166-181`), o sea **el ancho de la barra es el techo de la
zona dividido por el techo mayor** (67 %…100 %), **que no representa nada** (auditoría:33, `ventana.slint:113-118`).

**El contrato** (encargo del dueño; auditoría:51-57):

| qué | cómo queda |
|---|---|
| **el eje** | **UN eje común en ppm**, de la **mínima a la máxima** de las cinco zonas, **con su unidad escrita** («ppm») |
| **cada zona** | una **BANDA** sobre ese eje: **su inicio y su fin** en ppm, dibujados **en su sitio del eje** |
| lo que se lee | **qué zona es ancha y dónde cae**, que es lo que un ciclista lee |
| el dato del motor | **el TRAMO de cada zona (inicio y fin en ppm)** — **prohibido un nivel 0–100** |
| la unidad | **ppm**, y se dice: «hasta 114» y «114 – 127» **sin unidad** son el defecto 4 (auditoría:34) |
| el tiempo en zona | **si hay sesiones**, en **su propio paso**: tiempo por banda (Garmin `TimeInZone`) |
| cuándo EXISTE | **en cuanto hay edad o umbral**; se **pinta** cuando llegan las sesiones |
| cuándo NO se dibuja | sin edad **y** sin umbral: **no hay gráfico ni su marco**; en su lugar, **una línea** (§3.7) |

**Estado del paso «tiempo en zona», medido** (`estado:223-240`, `potencia.rs:497-524`): el lector **ya
filtra `reference_mesg == "session"`**, y **eso no es un detalle**: cogiendo el primer `TimeInZone` —el de
la vuelta 0— el reparto suma **una media del 84,7 % MENOS** que la sesión **y el número sigue pareciendo
bueno**. Y viene con **diez posiciones mientras Coggan define siete**: 28 de las 83 sesiones con potencia
tienen tiempo en las posiciones 8 a 10 (**1.322 s**), que **se cuentan aparte porque no se pueden nombrar
con la tabla que el módulo cita** (`potencia.rs:643-663`, `potencia.rs:680-698`).
**Además, y hay que decirlo**: las zonas de **Garmin son de 7** y las de este motor, **5**
(`umbral.rs:151-193`). **El emparejamiento entre las dos tablas NO está decidido**, así que el tiempo en
zona **no se pinta hasta que se decida por escrito** (AU-66 y AU-85: un nombre plausible con otro
contenido es la mentira más difícil de ver).

**Medidas del bloque** (`ventana.slint:180-212`): tarjeta de alto **268** con radio 12, borde 1 px,
`padding` 14, `spacing` 4, título «Zonas de entrenamiento» a **14 px** (body) **a la izquierda**, y cinco
filas de **40** de alto. **El alto de la TARJETA pasa a ser el que pida el contenido** (con el eje, deja
de ser un número fijo) y la prueba lo comprueba contra la suma declarada, no contra 268. **Pero el alto de
la FILA de zona sigue siendo 40 y NO se cambia**: la prueba de los vacíos localiza las filas de zona
**por su altura** (`medir-interfaz.rs:447-468`), así que un alto distinto deja ese portero ciego.

**Cada fila (zona) se reutiliza en la sección Zonas** (§3.9). Composición de la fila:

| pieza | alineación | tipografía | fuente |
|---|---|---|---|
| nombre de la zona | **izquierda** | 12 (label), `muted` | `ventana.slint:99-103` |
| tramo | **derecha** | 12 (label), `fg` | `ventana.slint:104-107` |
| tiempo en zona (cuando exista) | **derecha, a la izquierda del tramo**, y **solo si hay dato** | 12 (label) | `estado:223-240` |
| barra | **ancho completo de la tarjeta**, 10 de alto, radio 5 | `ventana.slint:109-118` |
| pista de la barra | **el mismo color de la zona, muy suave** (α 0,18): **no gris**, porque el gris iguala las cinco zonas | `ventana.slint:90-91`, `ventana.slint:204-209` |

---

### 3.7 · Los textos del Resumen: la precisión y lo que falta

**UNA línea corta, con su icono. El párrafo va a la AYUDA, nunca a la pantalla** (encargo del dueño;
AU-44, AU-45).

**Hoy, y esto es lo que se corrige**: `texto_precision` es un párrafo de dos frases
(«Precisión aceptable: las zonas salen del estimado a partir de la frecuencia máxima, que es una
estimación.» — auditoría:31, `cara.rs:184-195`) y `que_falta` son **dos frases** («Faltan tus datos: con
la edad y el umbral ya se calculan las zonas. **Se rellenan en Datos.**» — auditoría:32,
`cara.rs:258-273`).

| línea | dónde | cómo | fuente |
|---|---|---|---|
| **precisión** | tarjeta de aviso del Resumen | **el texto se recorta a UNA frase**; el detalle entero se va a la ayuda | auditoría:59-60 |
| **lo que falta** | **bloque de una línea, con su icono** | una frase que dice **qué falta y dónde se rellena** | `cara.rs:253-255` |
| icono de la línea | **real, del catálogo**, no un cuadrado de color | `iconos.slint` (187 iconos) | AU-90, `estudio:56-60` |
| el punto de hoy | `Aviso` (ventana.slint:124-149) pinta **un círculo de color de 10 px** | **se sustituye por el icono** | `ventana.slint:136-139` |

**Estado del bloque de aviso**: alto **44**, radio 12, fondo `pista`, `padding` lateral 12, `spacing` 8,
texto 12 px con ajuste de palabra **a la izquierda** (`ventana.slint:123-149`).

**Y LO QUE NO SE DIBUJA**: si el texto está vacío, **el bloque no existe** — hoy ya es así para la
precisión (`ventana.slint:213`) y el aviso (`ventana.slint:243`) y **eso se conserva**. **Sin datos, el
aviso va vacío a propósito** (`cara.rs:198-218`): anunciar lo que aparecerá es explicarse, y además
nombra dos cosas que no existen (deriva y eficiencia).

### 3.8 · El bloque del periodo y el recuento

| qué | valor | fuente |
|---|---|---|
| alto | **52** (fijo: la línea del periodo no absorbe el hueco sobrante) | `ventana.slint:214-219` |
| radio / borde | **12** / 1 px `linea` | `ventana.slint:220-223` |
| relleno | lateral **14**, **centrado** verticalmente, `spacing: 2` | `ventana.slint:224-228` |
| el periodo | **12 px**, `muted`, **a la izquierda**, con ajuste de palabra | `ventana.slint:229-236` |
| el recuento | **12 px** (era 13: **se corrige a label**), `muted`, a la izquierda | `ventana.slint:237-240` |
| el periodo no se dibuja si no hay fechas | `if root.periodo != ""` | `ventana.slint:229`, `cara.rs:238-248` |

**El texto lo escribe Rust y la vista NO calcula** (`cara.rs:56-63`, `ventana.slint:2-6`). El recuento
dice «N sesiones · N lecturas · N intervalos de HRV», y **sin datos dice UNA sola frase**: «Sin datos de
entrenamiento todavía.» (`cara.rs:123-129`). **Y esa frase no se repite en ningún otro bloque** (AU-92,
`estudio:53`).

---

### 3.9 · La sección ZONAS (fase 3)

**Qué es**: el mismo gráfico del Resumen, **en grande y con todo el detalle**, más la tabla de las cinco
zonas. **Reutiliza la fila de zona de §3.6**, componente a componente (encargo del dueño: las fases 3
reutilizan los componentes de 1 y 2).

| bloque | alto | márgenes | alineación | tipografía |
|---|---|---|---|---|
| cabecera de la sección | **64** | `padding` 16 | título **a la izquierda** | **22** |
| título «Zonas» dentro del contenido | auto | `padding` 14 | **a la izquierda** | **14** (body) 700 |
| gráfico con el eje | el que pida el contenido | `padding` 14 | bandas **sobre el eje**, la unidad **bajo el eje**, centrada | etiquetas **12** |
| tabla de las 5 zonas | 5 × **40** | `padding` 14, `spacing` 4 | nombre **izq.** / tramo **der.** | **12** |
| barra inferior | **80** | — | centrada en su hueco | **12** |

**Estado vacío — UNA línea con su icono**, y qué dato lo provoca:

| falta | qué se dibuja | qué NO se dibuja |
|---|---|---|
| **ni edad ni umbral** | **una línea**: lo que falta y dónde se rellena (`cara.rs:258-273`, en UNA frase) | **ni el gráfico ni el eje ni las cinco filas** |
| **hay edad/umbral, pero no hay sesiones** | las **5 bandas sobre el eje** (el gráfico **existe**) | **ni tiempo en zona ni ninguna columna de tiempo** |
| **hay sesiones pero el tiempo en zona está sin decidir** | las bandas, y **nada más** | la columna de tiempo: **no se inventa el emparejamiento de 7 ↔ 5** |

**Prohibido**: cinco barras **sin nombre** (fue el fallo real, `estudio:45-47`), y una barra que
represente un **nivel 0–100** (auditoría:33, 57).

### 3.10 · La sección SESIONES (fase 3)

**Qué es**: **las sesiones, una por fila**. Es la pantalla del paso 7 del recorrido, «ver los
resultados» (`RECORRIDO-del-usuario.md:31`).

| bloque | valor | fuente |
|---|---|---|
| lista | **`ListView`** y **no `ScrollView`**: el `ScrollView` **instancia todos los hijos**, y con 102 sesiones eso se nota | `slint:387-389`, `slint:98` |
| alto de la fila | **56** (dos líneas de texto + aire), **área táctil ≥ 48** | `estudio:27` |
| fila: línea 1 | **fecha y tipo**, **14 px** (body), **a la izquierda**, en `fg` | `estudio:94` |
| fila: línea 2 | **duración y desnivel**, **12 px** (label), **a la izquierda**, en `muted` | `estudio:25` |
| el dato de la derecha | si lo hay, **alineado a la DERECHA** de la fila, **12 px** | AU-68 |
| separador | 1 px de `linea` al **45 %**, como el borde de las tarjetas | `ventana.slint:58-60` |
| relleno de la fila | lateral **16**, vertical **8** | `armazon.slint:49` |
| barra inferior | **80**, **centrada en su hueco** | `estudio:25` |

**Estado vacío**: **UNA línea con su icono** — «Sin datos de entrenamiento todavía.» **una sola vez en
toda la pantalla** (`cara.rs:128`, AU-92). Con la lista vacía, **no se dibuja ni una fila ni su
separador** (AU-91). **Y el recuento del periodo (§3.8) no se repite aquí** (AU-92).

### 3.11 · La sección DATOS (fase 3) — controles de SERIE, con marco por fuera

**Decisión del dueño, tomada el 2026-10-13 y NO se vuelve a preguntar**: *«utilicemos los de Slint para
esta tarea, debe tener la ventaja de no romperse»* (`slint:415-417`, `estado:44-45`).

**El precio está medido, y es lo que obliga a las dos reglas de abajo**: los controles **no aceptan
nuestros colores ni un borde** (`slint:418-435`, errores literales `Unknown property background in
Button` / `border-width in Button`), y **su esquema lo decide el SISTEMA**: con el sistema en el tema
contrario, el texto del control sale a **1,05 : 1** y **1,12 : 1**, con mínimo **4,5 : 1** (`slint:436-457`).

| pieza | cómo queda | fuente |
|---|---|---|
| **`Palette.color-scheme`** | **fijado desde NUESTRO tema, en la raíz**, con un **binding** (no un `init`: el usuario cambia de tema cuando quiere) | AU-84 · `slint:436-457` · `ventana.slint:533-541` |
| **marco por fuera** | un `Rectangle` de 1–2 px con el color `linea` **por fuera del control**, nunca reescribirlo | `slint:458-484` · `ventana.slint:281-300` |
| **el hueco del marco** | lo pone el **`padding` del layout**; la altura **se le pide al HIJO** (`height: hijo.preferred-height`) | `estado:118-121` |
| **lo que NO se hace** | dimensionar el envoltorio con `self.preferred-height`: **vale CERO** en ese momento (medido: marco `288x4`, botón `284x27`, saliéndose) | `estado:118-121` |
| **mínimo táctil** | **`height: max(48px, self.preferred-height)`** en cada control: el estilo trae 32 y el proyecto pide 48 | `slint:485-497` · `ventana.slint:294-297` |
| **ancho completo** | `width: parent.width` o `horizontal-stretch: 1`: el estilo trae `horizontal-stretch: 0` | `slint:485-497` |
| **tema** | un `Switch` — «Tema oscuro»: **cambiar de tema es cambiar UNA propiedad** | `ventana.slint:441-445` |

**Los bloques de la sección** (`ventana.slint:303-446`):

| bloque | alto | márgenes | alineación | tipografía |
|---|---|---|---|---|
| título «Sus datos» | auto | `padding` 16 | **izquierda** | **14** (body) 700 *(hoy 15)* |
| línea de ayuda: «Un campo vacío es “no lo sé”, no un cero» | auto | — | **izquierda** | **12** (label) *(hoy 11)* |
| los campos, en **dos columnas** | **≥ 48** cada control | `spacing` 10 / 8 | etiqueta **izq.**; el valor, dentro del control | etiqueta **12** *(hoy 11)* |
| los 10 campos | edad, pulso en reposo, pulso máximo, umbral de lactato, VO₂ máx · peso, altura, grasa, masa muscular + sexo (`ComboBox`) | — | — | — |
| guardar | **≥ 48** | marco por fuera, `padding` 2 | texto **centrado** por el estilo | — |
| buscar mis ficheros | **≥ 48** | marco por fuera, `padding` 2 | — | — |
| el tema (`Switch`) | **≥ 48** | — | a la izquierda | — |
| lo que pasó al guardar | auto | — | **izquierda** | **12** |

**Los nombres de campo, y esto es contrato, no adorno** (`ventana.slint:27-41`): un campo vacío es **«no
lo sé»**, **nunca un cero**; el sexo lleva **tres** opciones y el «No lo sé» **no se quita** (un
`RadioGroup` no se puede desmarcar; se usa `ComboBox` porque el `RadioGroup` **no deja escribir su
índice desde fuera**, `ventana.slint:356-374`). Y `accessible-id` en cada campo, que es lo que permite
medirlos sin abrir ventana (`ventana.slint:267-270`).

**Estado vacío**: cada campo vacío se dibuja **normal, con su pista** («ppm, durando»…) — **un campo
vacío no es un vacío**: es una pregunta sin responder, y esos **sí** se dibujan. Lo que no se dibuja es
**el bloque del periodo** si no hay fechas (§3.8) ni **la línea de mensaje** si no se ha guardado nada
(`ventana.slint:400-408`).

### 3.12 · La capa del selector de carpeta (superpuesta)

**Decisión del dueño**: **capa superpuesta, NO una quinta sección** — la navegación de 4 está aprobada y
medida (`estado:301-307`). Hoy se dibuja **la última** en `ventana.slint`, para quedar **por encima** de
los dos armazones (`ventana.slint:590-602`), y **sin la capa, sus elementos no existen** (medido:
`estado:296-300`).

| qué | valor | fuente |
|---|---|---|
| tamaño | **100 % × 100 %** (cubre la ventana) | `selector.slint:34-35` |
| relleno | **16** | `selector.slint:38` |
| botones | **marco por fuera** y **≥ 48** de alto | `selector.slint:79-120` |
| cabecera de la capa | **14** (body) *(hoy 15)*, **a la izquierda** | `selector.slint:44` |
| la ruta y el recuento de FIT | **12** (label) *(hoy 11)*, **a la izquierda** | `selector.slint:50-70` |
| subir | **solo si se puede subir** (`if`): no se dibuja un botón que no lleva a nada | `selector.slint:79` |
| cerrar | **siempre la misma forma** en toda la aplicación | AU-48 |

**Comportamiento**: se abre desde **Datos** («Buscar mis ficheros»), **vuelve a analizar al elegir sin
reiniciar**, y se cierra con su botón (`estado:296-306`).

---

## 4. Lo que hay que decidir (dueño) — *dos de tres, resueltas en la fase 0*

1. **¿El gráfico está desde el inicio?** **ACEPTADO, y ya está en el contrato** (§3.6): el gráfico
   **existe** en cuanto hay edad o umbral (las bandas sobre el eje) y **se pinta** cuando llegan las
   sesiones. Sin edad ni umbral, **una línea**. (Encargo del dueño; resuelve el choque con AU-91.)
2. **¿Hay botón «Analizar»?** **SIGUE ABIERTO**, y no se inventa: §3.4 (§7, `estado:488-491`).
3. **¿Qué cifras van arriba y en qué orden?** **RESUELTO**: las cuatro de la tabla de §3.5, en ese
   orden — umbral, frecuencia máxima, vatios por pulso, deriva (`cara.rs:141-159`, que es lo que el motor
   ya entrega). **El boceto decía pulso máximo, umbral, vatios/pulso, deriva; manda el motor**, porque es
   el que sabe qué dato existe.

---

## 5. El plan, por fases (y la definición de TERMINADO)

1. **Fase 0 (esta)**: la especificación congelada, con sus medidas y sus huecos declarados.
2. **Fase 1 · El armazón**: cabecera 64 / título de la sección a 22, barra 80 con iconos reales a 24 y
   etiqueta 12, píldora, radio 12, margen 16, táctil ≥ 48, armazón **anclado a 411 × 880** y altura mínima
   declarada.
3. **Fase 2 · El Resumen**: las cifras centradas y **sin tarjeta si el dato no existe**; el gráfico de
   **bandas sobre el eje en ppm**; los textos **en una línea con su icono**.
4. **Fase 3 · Zonas, Sesiones y Datos**, reutilizando los componentes de las fases 1 y 2; Datos con los
   controles de serie, marco por fuera y `Palette.color-scheme`.

**Cada fase se cierra con LAS CUATRO COSAS** (`PROMPT-rehacer-interfaz.md:86-91`): `cargo test` verde y
**0 avisos** · `cargo test --test medir-interfaz` **contra los números de este documento** · la captura
sin abrir ventana (`cargo run -q --bin capturar-ui`) · y **decir qué debe contener la captura**, porque
**medir dice «cabe»; ver dice «se entiende»**, y **el que escribe no puede ser el que valida**.

**Lo que NO se hace**: otro parche suelto. **Si no cabe en esta especificación, no se toca.**

---

## E13 · Huecos: lo que la interfaz necesita y el motor HOY no entrega

**Esto es un contrato pendiente, no una queja.** Cada línea dice **qué falta**, **qué función de
`cara.rs` habría que tocar**, y **por qué el motor no puede seguir como está**. Según la casa, **primero
el contrato (5 líneas: qué recibe, qué devuelve, qué NO hace) y su prueba, y después el código**
(`arq:150-163`).

| # | lo que la UI necesita | qué entrega HOY | dónde se arregla | por qué |
|---|---|---|---|---|
| 1 | **el tramo de cada zona en ppm**: `(nombre, inicio_ppm, fin_ppm)` | **un nivel 0–100 inventado**: `nivel = hasta / tope * 100` — `cara.rs:166-181`. **El tramo SÍ se calcula** (`umbral::zonas_del_umbral`, `umbral.rs:132-147`, devuelve `(ZonaCiclismo, Rango, Rango)`) **pero `cara.rs` lo convierte en TEXTO** (`cara.rs:98-105`) y **tira los números** | `cara.rs::lo_que_ensena` → el campo `zonas` (`cara.rs:49-50`) | sin el número no hay banda posible sobre el eje: **el dato existe y se está perdiendo al formatearlo** |
| 2 | **«esta cifra no existe», en vez de `"—"`** | **la cadena `"—"`**: `cara.rs:143-152` (`map_or("—".to_string(), …)`) | `cara.rs::lo_que_ensena` → `cifras` (`cara.rs:141-153`) | para que la tarjeta **no se dibuje** hace falta que el motor diga **cuál falta**; una cadena vacía no basta, porque «vacío» y «no medido» se confunden (AU-88) |
| 3 | **el rango del eje** (mínima y máxima de las cinco zonas, en ppm) | **nada**: se deriva del `tope` (`cara.rs:166`) | `cara.rs::lo_que_ensena` | el eje **es el gráfico**: sin mínimo y máximo explícitos, cada pantalla elegiría su escala, y la escala es una afirmación |
| 4 | **la precisión en UNA frase corta** (+ su párrafo para la ayuda) | un **párrafo de dos frases**: `cara.rs:184-195` | `cara.rs::lo_que_ensena` → `precision` (`cara.rs:52`) | AU-44/AU-45: la pantalla no se explica, y el texto de hoy es el defecto 1 de la auditoría |
| 5 | **lo que falta, en UNA frase corta** | **dos frases**: `cara.rs:262-266` (y `cara.rs:43` dice «una o dos líneas») | `cara.rs::que_falta` (`cara.rs:258-273`) | defecto 2 de la auditoría: es la línea más larga de la pantalla |
| 6 | **el tiempo en zona por banda, con la correspondencia 7 ↔ 5 DECIDIDA** | **nada llega a la cara**: el lector existe (`potencia.rs::leer_declarado_de_potencia` filtra `reference_mesg == "session"`, `potencia.rs:497-524`) y devuelve 7 zonas (`potencia.rs::tiempo_en_zonas`, `potencia.rs:654-671`), pero **`cara.rs` no lo mira** | `cara.rs` (campo nuevo) + **contrato escrito** de la correspondencia | Garmin define 10 posiciones, Coggan 7 (`potencia.rs:643-663`) y **este motor 5** (`umbral.rs:151-193`): emparejar a ojo es **AU-85**, el fallo de `avg_power_position` |
| 7 | **qué sesiones hay** (para la sección Sesiones: fecha, tipo, duración, desnivel) | `Real` trae **el recuento y las fechas del periodo** (`cara.rs:123-129`, `cara.rs:238-248`), **no la lista** | `cara.rs` (campo nuevo) y `analisis::Real` | la pantalla **no calcula** (`ventana.slint:2-6`): si la lista se monta en Slint, la pantalla puede mentir sin que una prueba lo vea |
| 8 | **la etiqueta de cada cifra, atada a su dato** | **dos listas paralelas** `cifras` y `etiquetas` (`cara.rs:46-48`) | `cara.rs` → una lista de pares | con dos listas, **quitar una cifra desalinea las etiquetas**: es AU-73 (lo que existe dos veces acaba divergiendo) |

**Y un hueco de producto, que NO es del motor**: **la acción primaria** (§3.4, auditoría:40,
`estado:288-291`). Hoy **calcula al abrir** y tarda **5,4 s sin decir nada** (`estado:44`).

**Lo que NO es un hueco, y se dice para no volver a buscarlo**: el umbral con su margen (`169±3`,
`cara.rs:142`), el periodo en una línea (`cara.rs:238-248`), el recuento (`cara.rs:123-129`) y la
decisión de los controles de serie con su precio medido (`slint:415-505`) **ya están entregados y
decididos**. Y `TimeInZone` **no llega vacío**: llega **sin usar** (hueco 6).

---

## 6. Anexo A · Cambios respecto al texto previo de este documento (todo anotado)

**Regla que se siguió**: no se borró nada aprobado; se completó, y **lo contradictorio se corrigió
diciendo qué decía antes**. Las secciones 1, 2 y el plan se conservan **literalmente** (solo se renumeran).

| # | qué cambió | antes decía | ahora dice | por qué |
|---|---|---|---|---|
| 1 | **Título y estado** | «Especificación de la pantalla «Resumen» (auditoría y propuesta)» | «Especificación de la interfaz (fase 0: CONGELADA)», con tabla de fuentes y `doc:línea` | el encargo pide **el contrato de las fases 1-3**, y una propuesta no es un contrato |
| 2 | **Alcance** | solo la pantalla Resumen | **las cuatro secciones** + la capa del selector | las fases 1-3 construyen Zonas, Sesiones y Datos, y **no había nada escrito de ellas** |
| 3 | **Medidas** | no existían en el documento | **§0**: tabla única de medidas con su fuente (411×880, 64, 80, 24, 12, 16, 14, 48, 22/14/12) | el encargo: «alto y márgenes de cada bloque, con la fuente citada» |
| 4 | **«¿El gráfico está desde el inicio?»** | pregunta abierta (punto 1 de «lo que hay que decidir») | **ACEPTADO**: existe con edad/umbral, se pinta al llegar las sesiones | lo fijó el encargo del dueño |
| 5 | **«¿Qué cifras van arriba y en orden?»** | pregunta abierta (punto 3) | **RESUELTO**: umbral, frecuencia máxima, vatios por pulso, deriva | es lo que el motor entrega (`cara.rs:141-159`), y manda el motor |
| 6 | **Las cifras, tipografía** | «su número centrado» sin tamaño | **22 px** (title-large), **no 26** como hoy | encargo del dueño (22/14/12) |
| 7 | **El «—» de 26 px** | «en vez de un «—» de 26 px» | **prohibido**, y **el motor deja de mandarlo**: la tarjeta se saca del árbol | AU-91 + hueco 2 de §E13 |
| 8 | **El gráfico de zonas** | «Opción A» y «Opción B (después)» | **un solo contrato**: bandas sobre eje en ppm, con eje y unidad; el tiempo en zona **en su paso**, y **no se pinta hasta decidir la correspondencia 7↔5** | las opciones eran una duda; y el emparejamiento sin decidir es AU-85 |
| 9 | **El apartado «5 · La barra de abajo»** | «Iconos a 24 dp, etiqueta 12… **ver los 4 iconos a tamaño real**» | **§3.2 con la tabla completa** (píldora 64×32 radio 16, reparto 411/4, táctil 48, `spacing` 4) y el defecto **ya corregido y medido** | lo de «verlos a tamaño real» ya se hizo: son iconos reales de Lucide (`estado:472-476`) |
| 10 | **El apartado «6 · El hueco»** | «O el contenido se reparte, o la pantalla no reserva alto de más» | **§3.3**: altura **mínima declarada** de 736 y **ScrollView** con `mouse-drag-pan-enabled` | AU-93: se resolvió **midiendo**, no repartiendo |
| 11 | **Texto corrido (defecto 1) y dos frases (defecto 2)** | se describían como defectos | **§3.7**: una frase en pantalla, el párrafo **a la ayuda**, con su icono | AU-44/AU-45, y el encargo lo pide explícito |
| 12 | **Los avisos** | `Aviso` con su punto de color | **el punto se sustituye por un icono real del catálogo** | AU-90: un marcador de boceto no es producción |
| 13 | **Ajustes y Datos** | no se decía nada | **§3.11**: controles de serie, `Palette.color-scheme` en la raíz y **marco por fuera** con el hueco por `padding` | decisión del dueño (`estado:44-45`) con su precio medido (`slint:415-505`) |
| 14 | **El selector de carpeta** | no aparecía | **§3.12**: capa superpuesta, 100 %×100 %, relleno 16, botones ≥ 48 | decisión del dueño (`estado:301-307`) |
| 15 | **Tipografías corregidas a la escala 22/14/12** | 26 (cifra), 15 (títulos), 13 (recuento), 11 (etiquetas de campo y del selector) | **22 / 14 / 12** | el dueño fijó **tres** tamaños; los demás eran del boceto |
| 16 | **§E13, los huecos** | no existía | **ocho huecos** con la función de `cara.rs` que hay que tocar | el encargo lo pide como entregable, y `arq:150-163` exige contrato antes que código |
| 17 | **Anexo A (esta tabla)** | no existía | los 17 cambios, anotados | la regla del encargo: «anota TODO cambio respecto al texto previo» |
| 18 | **El plan** | «Este chat lleva más de 250 000 tokens… propuesta: nueva conversación» | **§5**: las tres fases con **las cuatro cosas** de la definición de TERMINADO | el bloque se cerró: el plan pasa a ser por fases, no por conversaciones |

**Lo que NO se cambió, y a propósito**: §1 (por qué costaba), §2 (la auditoría de los 10 defectos) y las
tres causas del «diagnóstico sin excusas». **Son el motivo del documento y siguen siendo ciertos.**

## Anexo B · Cómo se comprueba esto (y qué NO se puede comprobar así)

| se comprueba | con |
|---|---|
| que la ventana mide 411 × 880 | `motor/tests/medir-interfaz.rs:88-89` |
| que la cabecera mide 64 y la barra 80 | `motor/tests/medir-interfaz.rs:96-105` |
| que los cuatro huecos están repartidos por igual (411/4) | `motor/tests/medir-interfaz.rs:109-155` |
| que sin poder diagnosticar **no hay tarjetas vacías** | `motor/tests/medir-interfaz.rs:447-455` |
| que el periodo está **y no deja hueco** cuando no lo hay | `motor/tests/medir-interfaz.rs:406-436` |
| que el botón de serie trae su accesibilidad y **el marco no lo recorta** | `motor/tests/medir-interfaz.rs:286-374` |
| que el selector existe, y su botón llega a 48 | `motor/tests/medir-interfaz.rs:240-279` |
| que ningún elemento **se sale** de la ventana | la prueba de geometría, con el contenido largo |

**Lo que NO se puede medir así, y se dice antes de prometer nada** (AU-75, `slint:127-133`): **el color,
el contraste y lo que se dibuja dentro de un elemento**. Para eso están `temas/contraste.py` y
`temas/contraste-widgets.py`, **que salen con error si un par baja del mínimo** (`arq:143`). Y la
pregunta «¿se entiende?» **no la contesta ninguna prueba**: la contesta **alguien que mire la captura**,
y por eso **el que escribe no valida** (`PROMPT-rehacer-interfaz.md:118-123`).
