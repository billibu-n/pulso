# El recorrido del usuario, paso a paso (y qué pasa de verdad en cada uno)

**Escrito el 2026-10-13, a petición del dueño**, que lo pidió así:

> *«Hagamos una simulación de la experiencia del usuario desde el inicio y en cada sección, y luego
> veamos si la aplicación cumple. Primero, inicia: debe tener una interfaz amigable y poder realizar un
> diagnóstico que seguramente no podrá realizar, por lo que tendría que añadir o incluir cuentas para
> poder vincular la data con el proyecto. Luego de vincular, debería poder arrancar la herramienta y
> funcionar y arrojar los resultados.»*

**Y POR QUÉ ESTE DOCUMENTO ES LO QUE FALTABA**, dicho sin adornos: el motor tiene **86 pruebas en
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

## Y qué pasa DE VERDAD en cada paso, medido hoy

Esto **no es una opinión**: cada fila está comprobada ejecutando la aplicación.

| # | el paso | qué pasa hoy | veredicto |
|---|---|---|---|
| **0** | abrir | hay **13 binarios de desarrollo** (`demo`, `probar_fit`, `caso`...) y **uno que abre la ventana**. **Ninguno se llama «la aplicación»**, y **no hay forma de saber cuál lanzar** | **NO CUMPLE** |
| **1** | primer arranque sin datos | **calcula y enseña resultados que no ha medido**: dice `169±3` y **«Precisión alta»** con un umbral **escrito a mano en el código**, y a la vez **«Sin datos todavía»**. **Se contradice** | **NO CUMPLE** |
| **2** | ofrecer vincular | **no ofrece nada.** No hay pantalla, ni aviso, ni botón | **NO EXISTE** |
| **3** | vincular cuenta | **no existe.** Ni cuentas, ni autorización, ni conexión | **NO EXISTE** |
| **4** | traer los datos | **solo por línea de órdenes**, con una ruta de carpeta. **El usuario tendría que saber dónde están sus ficheros** | **NO CUMPLE** |
| **5** | datos de la persona | **escritos en el código** (`edad = 60`, `umbral = 169`...). **El usuario no puede cambiarlos desde la aplicación** | **NO CUMPLE** |
| **6** | arrancar el análisis | **no hay nada que pulsar.** El cálculo ocurre al abrir | **NO CUMPLE** |
| **7** | ver resultados | **se ven cuatro cifras, cinco zonas y dos avisos**. El motor produce **625 resultados** | **A MEDIAS** |
| **8** | volver otro día | **no recuerda nada**: cada arranque vuelve a leer los 102 ficheros desde cero | **NO EXISTE** |

**De ocho pasos: uno a medias, y siete que no cumplen o no existen.**

---

## Las tres causas, y ninguna es «falta un detalle»

**1 · No hay recorrido, hay piezas.** El motor sabe calcular y la ventana sabe pintar. **Nadie escribió
el camino entre el usuario y el motor.** Por eso cada paso por separado «parecía» bien.

**2 · Los datos de la persona están escritos en el código.** Eso significa que **la aplicación no es
usable por nadie más que por quien la compiló** — y que **los resultados que enseña no son los suyos:
son los que yo puse**. Es el fallo más grave de la lista, porque **no se ve**: la pantalla enseña
números con buena cara.

**3 · Y no hay ninguna comprobación del recorrido.** Las 86 pruebas comprueban **el motor**, y las 5 de
medición comprueban **la geometría**. **Cero pruebas comprueban que un usuario pueda hacer algo.** Es
exactamente lo que la norma nueva prohíbe (AU-65): **si solo se puede validar mirándolo, no está
terminado.**

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
7. **Recordar entre arranques.**

**Y CADA PASO, CON SU COMPROBACIÓN** — porque si no, vuelve a pasar lo mismo: **ocho pasos que parecen
bien y no lo están**.
