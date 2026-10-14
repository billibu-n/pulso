# CONTRATO · los datos que `cara.rs` entrega a la interfaz (Fase 2)

**Qué es este documento.** Los **seis contratos** de los huecos 1, 2, 3, 4, 5 y 8 de
`docs/ESPECIFICACION-interfaz-resumen.md` §E13, escritos **antes del código**, como manda la regla 1 de
`docs/ARQUITECTURA-piezas-y-contrato.md:150-163`: *«antes de escribir código de una pieza, se escribe
su contrato — qué recibe, qué devuelve, y qué NO hace. Si el contrato no cabe en cinco líneas, la pieza
es demasiado grande»*.

**Lo que este contrato NO cubre, y se dice para no volver a buscarlo**: el **hueco 6** (tiempo en zona,
la correspondencia 7 ↔ 5) **falta la decisión del dueño** y no se toca; el **hueco 7** (la lista de
sesiones) **es otro hito** y no se toca. La interfaz (`motor/ui/*.slint`) y `motor/tests/medir-interfaz.rs`
**no se tocan aquí**: los edita otro agente en paralelo.

**Unidad de todo lo de abajo**: **ppm** (pulsaciones por minuto), que es la unidad del eje y de las
zonas. Todos los números van en `f64`, **sin formatear**; el redondeo y la coma decimal son de la
interfaz.

---

## Contrato 1 · Los tramos de zona, en números y con su margen

**Qué recibe**: las cinco zonas del umbral tal como las devuelve `umbral::zonas_del_umbral`
(`(ZonaCiclismo, Rango, Rango)`, `umbral.rs:132-147`), que ya traen **valor y margen** de inicio y de fin.

**Qué devuelve**: `Ensenado.zonas_ppm: Vec<ZonaEnsenada>`, **cinco** elementos en orden de esfuerzo, y
cada uno con `nombre: String`, `inicio_ppm: f64`, `fin_ppm: f64`, `margen_inicio_ppm: f64` y
`margen_fin_ppm: f64`. Con umbral 169 y margen 3: `recuperación` es `hasta 137` porque su inicio y su
fin **coinciden** (136,89 y 136,89).

**LO QUE LA PRUEBA ENSEÑÓ AL CONTRATO (2026-10-14), y se anota porque es una corrección**: los
**inicios NO son todos distintos**. `recuperación` y `resistencia` **comparten suelo** (`0,81 · umbral`)
y lo que comparten es la verdad: una zona empieza donde acaba la otra. Lo que no puede repetirse es el
**par (inicio, fin)** — cada zona tiene que ser un tramo distinto. Escribir «inicios únicos» en el
contrato habría sido escribir una regla falsa, y la prueba la cazó antes de que llegara al código.

**Qué NO hace**: **no formatea texto** (el `«137 – 152»` de hoy sigue saliendo del campo viejo
`zonas`), **no calcula niveles 0-100** (el nivel es DIBUJAR, y por eso no entra en el contrato), y **no
decide el color ni el orden de pintado** de ninguna banda.

---

## Contrato 2 · «Esta cifra no existe», como valor explícito

**Qué recibe**: las cuatro cifras que hoy se calculan en `cara.rs:141-153` — el umbral con su margen,
la frecuencia máxima medida, la eficiencia y la deriva —, cada una de las cuales **puede no existir**.

**Qué devuelve**: `Ensenado.cifras_atadas: Vec<Cifra>`, con `Cifra { etiqueta: String, dato: Option<String> }`.
`dato: None` significa **«esta cifra no existe»** y es lo único que autoriza a la interfaz a **sacar la
tarjeta del árbol**. `dato: Some(t)` con `t` vacío significa otra cosa: **existe y su texto es vacío**.

**Qué NO hace**: **no manda el guion `«—»`** (AU-91 y el anexo A, cambio 7: la cadena de 26 px está
prohibida), **no confunde «vacío» con «no medido»** (AU-88: son dos estados, y por eso el tipo es
`Option` y no `String`), y **no rellena con un cero ni con una cadena vacía** lo que no se midió.

---

## Contrato 3 · El rango del eje

**Qué recibe**: las cinco zonas numéricas del contrato 1.

**Qué devuelve**: `Ensenado.eje: Eje { inicio_ppm: f64, fin_ppm: f64 }`, donde `inicio_ppm` es la
**mínima** de las cinco zonas y `fin_ppm` la **máxima**. Con umbral 169: `137` y `203`. Lleva un
`es_valido()` que dice si el eje se puede pintar.

**Qué NO hace**: **no elige la escala a ojo** (la escala es una afirmación, y por eso sale de los
datos y no de la pantalla), **no decide las marcas ni el paso de la rejilla** (eso es dibujar), y **no
usa el margen** para ensancharse: el eje son **los cinco tramos**, y el margen viaja en cada tramo.

---

## Contrato 4 · La precisión, en UNA frase corta

**Qué recibe**: la referencia del umbral (`umbral::Referencia`: medido, estimado de la máxima, o de
población) y el margen del umbral, del que sale lo ancha que es (`Rango::como_va`, `rango.rs:84-95`).

**Qué devuelve**: **dos campos**: `Ensenado.precision_corta: String`, **una sola frase y una sola
línea** (`«Precisión alta: el umbral está medido.»`), que es **lo que va a la pantalla**; y el párrafo
completo, que **se conserva** en `Ensenado.precision` (el campo de hoy, `cara.rs:184-195`) **para la
ayuda**, que es donde AU-44/AU-45 lo mandan.

**Qué NO hace**: **no explica en la pantalla** (AU-44/AU-45: la pantalla no se explica), **no dice de
dónde sale el dato dos veces** (la frase y el párrafo no repiten el mismo texto: la frase nombra la
precisión y el origen, el párrafo desarrolla), y **no inventa un grado de precisión**: la palabra sale
de `como_va()`.

---

## Contrato 5 · Lo que falta, en UNA frase

**Qué recibe**: los datos de la persona que **de verdad desbloquean el cálculo** (el umbral de lactato
y la edad, `cara.rs:258-261`), no todos los que faltan.

**Qué devuelve**: `Ensenado.que_falta_corto: String`, **una sola frase y una sola línea**, diciendo
**qué falta y dónde se rellena**. El campo de hoy, `que_falta` (`cara.rs:43`), **se conserva** con su
texto de dos líneas para no romper la interfaz que ya lo consume.

**Qué NO hace**: **no da pena ni se explica** (AU-44/AU-45), **no enumera los ocho datos** cuando con
uno ya funciona, y **no aparece cuando no falta nada**: si se puede diagnosticar, es cadena vacía.

---

## Contrato 6 · La cifra y su etiqueta, atadas

**Qué recibe**: las cuatro cifras (contrato 2) con la etiqueta que le corresponde a cada una.

**Qué devuelve**: **una sola lista**, `Vec<Cifra>`, donde cada elemento **lleva su etiqueta dentro**:
`Cifra { etiqueta, dato }`. Son **cuatro** elementos, siempre en el mismo orden (umbral, frecuencia
máxima, vatios por pulso, deriva).

**Qué NO hace**: **no son dos listas paralelas** (AU-73: lo que existe dos veces acaba divergiendo, y
quitar una cifra desalinearía las etiquetas), **no indexa por posición** desde fuera (la interfaz
recorre la lista, no cuenta), y **no ordena ni filtra** las cifras: el orden lo fija el motor.

---

## Los campos nuevos de `Ensenado`, y qué consume la interfaz en la Fase 2

| campo nuevo | contrato | tipo | qué consume la UI |
|---|---|---|---|
| `zonas_ppm` | 1 | `Vec<ZonaEnsenada>` | las bandas sobre el eje, con su margen |
| `cifras_atadas` | 2 y 6 | `Vec<Cifra>` | las cuatro tarjetas; `None` = se saca del árbol |
| `eje` | 3 | `Eje` | el mínimo y el máximo del eje |
| `precision_corta` | 4 | `String` | la frase de pantalla; `precision` (viejo) va a la ayuda |
| `que_falta_corto` | 5 | `String` | la frase de pantalla; `que_falta` (viejo) se conserva |

**Los campos viejos NO se borran en este hito, y esto es una deuda con fecha**: `zonas` (con su
`nivel` 0-100 y su tramo en texto), `cifras`, `etiquetas`, `precision` y `que_falta` siguen
alimentando a la interfaz de hoy. **Dejar `zonas` y `zonas_ppm` a la vez es duplicar un dato, y AU-73
dice que lo que existe dos veces acaba divergiendo**: por eso la Fase 2 **borra** los viejos en cuanto
la interfaz lea los nuevos, y ese borrado es parte del cierre de la fase, no una mejora opcional.

## Cómo se comprueba (la prueba va ANTES del código)

Cada contrato tiene su prueba en `motor/tests/recorrido.rs`, y **las pruebas se escribieron y se
ejecutaron antes de implementar**, para verlas fallar (el resultado está en
`datos/trabajo-ui/contrato-cara-informe.md`):

| contrato | pruebas |
|---|---|
| 1 | `las_zonas_van_en_ppm_CON_su_margen_y_el_umbral_manda`, `dos_zonas_seguidas_no_pueden_ser_el_mismo_tramo` |
| 2 | `una_cifra_que_no_existe_es_None_y_NO_una_cadena_vacia`, `vacio_y_no_medido_NO_se_confunden`, `sin_frecuencia_maxima_medida_la_cifra_NO_EXISTE_y_no_es_un_cero_ni_un_guion` (cambiada) |
| 3 | `el_eje_va_del_suelo_de_la_primera_zona_al_techo_de_la_ultima`, `sin_datos_no_hay_eje_que_valga` |
| 4 | `la_precision_en_pantalla_es_UNA_frase_y_el_parrafo_va_a_la_ayuda` |
| 5 | `lo_que_falta_en_pantalla_es_UNA_frase_y_sigue_diciendo_donde` |
| 6 | `cada_cifra_lleva_su_etiqueta_dentro_y_no_en_una_lista_paralela` |
