# Las sondas que decidieron los umbrales (2026-10-13)

**Qué es esto.** Ocho programas pequeños que se escribieron **para elegir con números** los cortes del
motor de avisos, en vez de elegirlos a ojo. **No son la aplicación**: son los instrumentos con los que
se decidió qué número separa de verdad, en **sus** datos.

**Por qué se guardan aquí y no en `/tmp`.** Estaban en el directorio temporal, que en este equipo es
**una partición pequeña** y ya rompió compilaciones una vez por llenarse. **Un instrumento que se pierde
es un número que ya no se puede volver a comprobar** — y estos números deciden avisos sobre el cuerpo de
una persona. El código pesa **40 KB**; los binarios compilados pesaban **1,6 GB**. Se guarda el código.

**No están en el espacio de trabajo de Cargo**, así que no se compilan con el proyecto y no estorban.
Para volver a usarlos: copiar el `.rs` que interese a `motor/src/bin/` y ejecutarlo allí.

| sonda | qué decidió |
|---|---|
| `lineas-base.rs` | **qué corte separa de verdad** una sesión normal de una cargada. Y por qué se prefiere la **MAD** (desviación absoluta mediana) a la desviación típica: **su distribución tiene colas** (una deriva de −23,6 % y otra de +34,7 %), y la típica se deja arrastrar por un solo valor extremo |
| `deriva.rs`, `deriva-dos.rs` | **la deriva con dos lectores distintos, sesión a sesión.** Si los dos no coinciden, **uno de los dos está mal** — y esa comprobación sale gratis |
| `umbral-fase.rs` | **cuántas sesiones caen EXACTAMENTE en el corte.** Es lo que decide si el corte tiene **una sola dirección**: si muchas caen justo encima, el aviso salta por azar |
| `corte-fase.rs`, `final-fase.rs`, `bordes.rs` | los bordes y el final de la fase de pedaleo: dónde empieza y acaba de verdad el ciclo, en sus ficheros |

**Y una regla que estos programas dejan escrita**: un umbral **no se elige, se mide**. Un corte que
separa bien sus 102 sesiones no tiene por qué servir para otro cliente — y por eso cada sonda dice
**sobre cuántas sesiones** se midió.
