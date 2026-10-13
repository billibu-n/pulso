# El flujo real: qué hace hoy el cliente, y qué le falta

**Escrito el 2026-10-13**, con lo que el dueño ha visto de cómo lo usa.

---

## 1. Lo que hace hoy, paso a paso

```
  SENSORES                DATOS PROPIOS            LA IA                LO QUE RECIBE
  ────────                ─────────────            ─────                ─────────────
  sus ejercicios   ──┐
  (pulso, potencia,  │
   cadencia, GPS)    │
                     ├──→  capturas de     ──→   el modelo     ──→   tabla
  edad, altura,    ──┤     pantalla +            redacta             gráfico
  peso, etc.         │     lo que él                                análisis
                     │     añade a mano                            sugerencias
  cómo durmió,     ──┘                                           (sueño, alimentación,
  qué comió, cómo                                                 mejora continua)
  se sintió
```

**Y hay algo que hay que ver en ese dibujo**: **las tres entradas NO son lo mismo**, y **hoy entran todas revueltas**, como imágenes.

## 2. Las tres entradas, y por qué se distinguen

| entrada | qué es | quién la produce | cómo entra **hoy** |
|---|---|---|---|
| **los sensores** | pulso, potencia, cadencia, GPS | **el aparato** | captura de pantalla |
| **los datos de la persona** | edad, altura, peso | **él los declara** | escritos en el mensaje |
| **su contexto** | cómo durmió, qué comió, cómo se sintió | **él lo cuenta** | escrito en el mensaje |

::: aviso
**La tercera es la que casi nadie aprovecha, y aquí está claro que es la que MÁS importa.** Fíjate en
lo que dice el dueño: el cliente **añade información** antes de pasársela a la IA. **Ese trabajo es
suyo, lo hace a mano, y hoy se pierde**: no queda registrado, no se puede comparar con el de la semana
pasada, y **no se puede analizar**.

**Y es justo lo que él dice que le funciona**: *«mezcla alimentación, horas de sueño, hábitos»*. **Eso
no está en los sensores. Está en su cabeza** — y hay que sacarlo de ahí.
:::

## 3. Y ahora se ve qué hacía de verdad la IA

**No analizaba: ordenaba y redactaba.** El análisis lo hacía **él**, al elegir **qué capturas mandar** y
**qué contar**. La IA:
- colocaba los números en **una tabla**,
- dibujaba **un gráfico**,
- y escribía **un texto con sugerencias**.

**Consecuencia directa para el diseño**: **el valor no está en el texto**. Está en **la tabla, el gráfico
y las sugerencias** — y las tres se pueden **calcular**, en vez de **redactar**. Un cálculo **se puede
repetir** y **se puede explicar**; un texto de un modelo, **no**.

## 4. Lo que el cliente quiere, en sus tres palabras

| lo que pide | cómo se consigue |
|---|---|
| **tabla** | el motor ya produce los números con su fuente |
| **gráfico** | se dibuja de esos números; **un gráfico no calcula, solo enseña** |
| **análisis y sugerencias** | **reglas con fundamento**, no texto inventado |

**Y el «análisis» es donde está la trampa**: si lo escribe un modelo, **suena bien y no se puede
comprobar**. Si sale de reglas, **se puede rastrear hasta la fórmula** — y eso es **exactamente** lo que
el cliente pide cuando dice que quiere resultados **«justificados y coherentes»**.

## 5. Lo que esto añade al trabajo, y **no** es poco

**Hay que construir el mecanismo para que su contexto entre como DATO y no como texto.** Y aquí está la
dificultad real, que conviene decir: **nadie registra lo que come**. Preguntarle cada día **no lo va a
hacer**.

**Así que la pregunta de diseño no es «¿qué campos pongo?», sino «¿qué es tan corto que lo va a
contestar de verdad?»**. Y la respuesta, mirando lo que él ya hace a mano, es: **cuatro cosas contadas
en un minuto**:

| lo que se le pregunta | cómo se responde | por qué esa y no otra |
|---|---|---|
| **horas de sueño** | **ya lo mide Garmin** | no hay que preguntar nada |
| **cómo se sintió** | una marca, de 1 a 5 | **es la señal que él ya da a mano hoy** |
| **qué comió antes de entrenar** | una lista corta: proteína / hidratos / grasa / nada | porque **el efecto térmico se calcula con eso**, y es lo que él menciona |
| **si tomó alcohol** | sí o no | porque **afecta al sueño**, y el sueño ya se mide |

**Cuatro respuestas, y tres de ellas son una marca.** Eso **sí se contesta**. Un diario de comidas
detallado, **no** — y por eso **no se construye**.

## 6. Lo que cambia en el motor

**Poco, y es buena señal otra vez.** El motor ya tiene la pieza que resuelve el problema: **el juez de
datos**. Cuando falte el contexto, **no hay que inventarlo**: el motor dirá **«no se puede afirmar»** y
**«para afirmarlo hace falta saber cómo durmió»**.

**Lo que hay que añadirle**:
1. **El contexto como parte del modelo**, con sus tres o cuatro campos.
2. **Reglas que crucen las entradas**: *«esta semana la carga subió y el sueño bajó»*. **Ese cruce es
   exactamente la «mezcla» que él dice que le funciona**, y hoy no existe en ningún sitio.
3. **El generador de informes**: tabla, gráfico y sugerencias **desde los fundamentos del motor**.

## 7. Y una pregunta para el cliente que vale más que cualquier dato

**¿Tiene una prueba de esfuerzo?** Porque:
- a los **60 años**, con **ciclismo competitivo**, **la fórmula por edad puede quedarse corta**, y
  entonces **todas sus zonas salen bajas** (medido: el suelo de la zona fuerte, **8 pulsaciones por
  debajo**);
- y **no es solo para rendir mejor**: es **la forma de saber que puede exigir el corazón con
  tranquilidad**. Eso **lo dice un médico**, no una aplicación.

## 8. Y el reparto queda así, ya con las tres entradas

```
   SENSORES ──→  ┌─────────────────────────────────────────┐
                 │  EL MOTOR                               │      ┌── TABLA
   PERSONA  ──→  │  · calcula, con su fuente               │ ──→  ├── GRÁFICO
                 │  · decide qué se puede afirmar          │      └── ANÁLISIS
   CONTEXTO ──→  │  · dice qué falta, y para qué           │         y sugerencias
                 │  · cruza las tres entradas              │         CON FUNDAMENTO
                 └─────────────────────────────────────────┘
                                        │
                                        └──→ sólo si hace falta: un modelo
                                             redacta, SIN ver los datos crudos
                                             y SIN calcular nada
```
