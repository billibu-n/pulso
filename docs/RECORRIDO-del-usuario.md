# El recorrido del usuario, paso a paso (y qué pasa de verdad en cada uno)

**Escrito el 2026-10-13, a petición del dueño**, que lo pidió así:

> *«Hagamos una simulación de la experiencia del usuario desde el inicio y en cada sección, y luego
> veamos si la aplicación cumple. Primero, inicia: debe tener una interfaz amigable y poder realizar un
> diagnóstico que seguramente no podrá realizar, por lo que tendría que añadir o incluir cuentas para
> poder vincular la data con el proyecto. Luego de vincular, debería poder arrancar la herramienta y
> funcionar y arrojar los resultados.»*

**Y POR QUÉ ESTE DOCUMENTO ES LO QUE FALTABA**, dicho sin adornos: el motor tiene **186 pruebas en
verde**, calcula **625 resultados con su margen y su fuente**, y **el recorrido del usuario no existe**.
Se construyeron **las piezas**, y **nadie escribió el camino que las une**. Eso explica por qué costaba
tanto «cumplir la tarea»: **no había tarea escrita**, solo piezas sueltas.

> **Un recorrido escrito es también una norma**: dice qué tiene que pasar, en qué orden, y **qué se
> considera terminado**. Sin él, cada paso se juzga por separado y **el conjunto nunca se mira.**

---

## Los pasos, tal como el dueño los describió

| # | el paso | qué tiene que pasar |
|---|---|---|
| **0** | **Instalar y abrir** | la aplicación arranca y **se explica sola** lo justo para empezar |
| **1** | **Primer arranque, sin datos** | **NO puede diagnosticar**, y **lo dice** — no se inventa nada |
| **2** | **Ofrecer vincular** | como no hay datos, **ofrece conectar una cuenta** para traerlos |
| **3** | **Vincular la cuenta** | se conecta con el servicio (Garmin, reloj, báscula) y **autoriza** |
| **4** | **Traer los datos** | los ficheros llegan, y **se ve cuántos y de cuándo** |
| **5** | **Los datos de la persona** | los que **no los da ningún aparato**: edad, peso, umbral, sueño... |
| **6** | **Arrancar el análisis** | se pulsa y **el motor calcula** |
| **7** | **Ver los resultados** | las zonas, la deriva, la eficiencia, **con su margen y su fuente** |
| **8** | **Volver otro día** | la aplicación **recuerda** lo de antes y **solo trae lo nuevo** |

**El paso 8 no lo pidió el dueño, y se añade**: una aplicación que **hay que reconfigurar cada vez**
no se usa dos veces. Y es el paso que **decide si esto sirve o es una demostración**.

---

## Las decisiones del dueño que gobiernan este recorrido (2026-10-13)

**Son suyas, están tomadas, y no se vuelven a preguntar.** Se anotan aquí porque **este documento es el
plan**, y un plan sin sus decisiones se reinterpreta cada vez que alguien lo lee.

| decisión | qué significa, y por qué |
|---|---|
| **Los controles son los DE SERIE de Slint** | *«debe tener la ventaja de no romperse»*. Traen probado el foco, el teclado y la accesibilidad. **El precio, medido**: no llevan nuestros colores ni aceptan un borde. Ver §12 de `SLINT-lo-que-hay-que-saber.md`. |
| **Los datos de la persona se guardan JUNTO AL PROGRAMA** | El fichero **viaja con la aplicación**: si copia la carpeta a otro equipo, o al teléfono, **sus datos van con ella**. Es lo más fácil de entender para él: *sus datos están donde está el programa*. **Con reserva medida**: si la carpeta es de solo lectura (una instalación del sistema), se cae a la carpeta de configuración del usuario **y se dice dónde se guardó**. |
| **Manda SIEMPRE nuestro tema, nunca el del sistema** | Los controles de serie se adaptan al esquema del sistema, y **si no coincide con el nuestro el texto queda ilegible** — medido: **1,05 : 1** de contraste, con mínimo 4,5 : 1. Se fija desde nuestro estado en la raíz de la ventana. **Cambiar de tema es cambiar UNA propiedad**, y los controles la siguen. |

**Y una consecuencia de la segunda que hay que tener presente**: **el perfil MANDA sobre el aparato.**
Si el usuario declara su frecuencia máxima, **manda él** — no se le sobreescribe con lo que diga el FIT.
El aparato **aporta lo que el usuario no puede saber** (lo que midió entrenando); el usuario **aporta lo
que el aparato no puede saber** (su edad, su umbral medido en una prueba). **Cuando los dos hablan del
mismo dato, gana el que lo midió mejor, y eso se dice en pantalla.**

---

## Y qué pasa DE VERDAD en cada paso, medido hoy

Esto **no es una opinión**: cada fila está comprobada ejecutando la aplicación.

> **ESTA TABLA SE ACTUALIZA, y por eso lleva fecha.** Un documento que dice cómo está algo y no se
> corrige cuando cambia **es peor que no tenerlo**: quien lo lea trabajará sobre algo que ya no es
> verdad. **Última revisión: 2026-10-13, por la tarde.**

| # | el paso | qué pasa hoy | veredicto |
|---|---|---|---|
| **0** | abrir | hay **13 binarios de desarrollo** (`demo`, `probar_fit`, `caso`...) y **uno que abre la ventana**. **Ninguno se llama «la aplicación»**, y **no hay forma de saber cuál lanzar** | **NO CUMPLE** |
| **1** | primer arranque sin datos | **calcula y enseña resultados que no ha medido**: dice `169±3` y **«Precisión alta»** con un umbral **escrito a mano en el código**, y a la vez **«Sin datos todavía»**. **Se contradice** | **EN ARREGLO** (tarea en curso) |
| **2** | ofrecer vincular | **no ofrece nada.** No hay pantalla, ni aviso, ni botón | **NO EXISTE** |
| **3** | vincular cuenta | **no existe.** Ni cuentas, ni autorización, ni conexión | **NO EXISTE** |
| **4** | traer los datos | **solo por línea de órdenes**, con una ruta de carpeta. **El usuario tendría que saber dónde están sus ficheros** | **NO CUMPLE** |
| **5** | datos de la persona | **escritos en el código** (`edad = 60`, `umbral = 169`...), repetidos **a mano en 5 sitios de producción**. **El usuario no puede cambiarlos desde la aplicación** | **EN ARREGLO** (tarea en curso) |
| **6** | arrancar el análisis | **no hay nada que pulsar.** El cálculo ocurre al abrir | **NO CUMPLE** |
| **7** | ver resultados | **se ven cuatro cifras, cinco zonas y dos avisos**. El motor produce **625 resultados** | **A MEDIAS** |
| **8** | volver otro día | **HECHO Y MEDIDO**: se guarda el resultado con una huella de los ficheros. **5,40 s → 0,001 s** (6.586 veces). Falta conectarlo al abrir | **CASI** |

**De ocho pasos: dos en arreglo, uno casi, uno a medias, y cuatro que no cumplen o no existen.**

**Y LO QUE SE APRENDIÓ AL REVISAR ESTA TABLA**, que es una lección de método: **los pasos no se
arreglan de uno en uno, y el que parecía el más difícil era el más fácil.** El paso 8 —«recordar entre
arranques»— parecía el más costoso, y resultó ser **el más barato de todos**: se midió, y el tiempo no
estaba donde todo el mundo suponía (**leer** 5,50 s frente a **calcular** 0,000 s), así que la solución
fueron **nueve números guardados** en vez de una base de datos.

---

## Las tres causas, y ninguna es «falta un detalle»

**1 · No hay recorrido, hay piezas.** El motor sabe calcular y la ventana sabe pintar. **Nadie escribió
el camino entre el usuario y el motor.** Por eso cada paso por separado «parecía» bien.

**2 · Los datos de la persona están escritos en el código.** Eso significa que **la aplicación no es
usable por nadie más que por quien la compiló** — y que **los resultados que enseña no son los suyos:
son los que yo puse**. Es el fallo más grave de la lista, porque **no se ve**: la pantalla enseña
números con buena cara.

**3 · Y no hay ninguna comprobación del recorrido.** Las **94** pruebas comprueban **el motor**, y las
**8** de medición comprueban **la geometría y los controles**. **Cero pruebas comprueban que un usuario
pueda hacer algo.** Es exactamente lo que la norma nueva prohíbe (AU-65): **si solo se puede validar
mirándolo, no está terminado.**

---

## Lo que hay que construir, en orden

**Por orden de dependencia, no de facilidad:**

1. **Una sola puerta de entrada** que se llame como la aplicación y **diga qué hacer** si no sabe.
2. **Los datos de la persona, editables por ella** — es lo que hace que los resultados sean **suyos**.
3. **El primer arranque sin datos**: que **diga «no puedo diagnosticar todavía»** y **no enseñe nada
   calculado a mano**.
4. **El paso de vincular**: explicar **qué se gana** y **qué se necesita**.
5. **Los conectores**: al menos **importar ficheros** (que es lo que hoy funciona por línea de
   órdenes), y después la cuenta.
6. **Pulsar para analizar** y **verlo**, en vez de calcular al abrir.
7. ~~**Recordar entre arranques.**~~ **HECHO el 2026-10-13**: guardar el resultado con la huella de los
   ficheros. **Medido: 5,40 s → 0,001 s.** Lo que falta es **conectarlo al abrir**, que es una línea.

**Y CADA PASO, CON SU COMPROBACIÓN** — porque si no, vuelve a pasar lo mismo: **ocho pasos que parecen
bien y no lo están**.

---

## Y una cosa que el recorrido NO pide, pero decide si esto sirve

**El orden de arriba es el correcto por dependencia, y no es el orden en que él lo va a usar.**

El cliente **ya tiene sus datos** (102 sesiones, 1.003 MB) y lo que quiere es **ver qué dicen**. El
recorrido, en cambio, empieza por «vincular la cuenta» — que es el paso **más caro y el que menos
falta le hace ahora mismo**, porque sus ficheros ya están en su equipo.

**Por eso el paso 4 («traer los datos») tiene un atajo que hay que hacer PRIMERO**: **poder elegir la
carpeta donde están sus ficheros**. Eso ya funciona por línea de órdenes y **no necesita ni cuenta ni
autorización ni internet**; solo necesita **un botón y un diálogo de carpeta**.

**La consecuencia práctica**: los pasos 2 y 3 (vincular la cuenta) se pueden dejar **para el final**,
cuando él lo pida. Y mientras tanto **la aplicación le sirve de verdad**, que es lo que decide si la
sigue usando o la abandona en la segunda semana.
