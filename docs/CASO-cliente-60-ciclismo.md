# El caso concreto: 60 años, ciclismo competitivo, y la pregunta del informe

**Escrito el 2026-10-13**, con lo que el dueño ha contado del cliente.

---

## 1. Lo que pide el cliente, dicho claro

Un **ciclista competitivo de 60 años**, en **buen estado físico**, que **busca superarse**. Hoy lo hace
**a mano**: manda **capturas de pantalla** de sus datos a un modelo de inteligencia artificial, y el
modelo le devuelve **un informe con tablas, sugerencias y mejoras**.

Y dice algo que es **la clave de todo**: su mejora **constante** sale de **mezclar alimentación, horas de
sueño y hábitos de ejercicio**. No es solo entrenar más: es **el conjunto**.

**Y menciona un detalle muy concreto**: que **ciertos alimentos le restan eficiencia**, porque
**consumen parte de la energía que podría ir al ejercicio**.

## 2. Y tiene razón en ese detalle: existe, y se mide

No es una intuición. **Digerir cuesta energía**, y esa energía **sale del mismo presupuesto** que usa el
ejercicio. Se llama **efecto térmico de los alimentos**, y **no cuesta lo mismo según qué se coma**:

| lo que comes | cuánta energía se va en digerirlo |
|---|---|
| **proteína** | **entre el 20 % y el 30 %** |
| **hidratos de carbono** | entre el 5 % y el 10 % |
| **grasa** | entre el 0 % y el 3 % |

**O sea: de una comida de 500 calorías, con proteína se van hasta 150 en digerirla; con grasa, casi
ninguna.** Eso es **exactamente** lo que el cliente nota — *«consumen parte de la energía»* — y **tiene
nombre y números**.

::: aviso
**Pero hay que decir la otra mitad, porque aquí es fácil pasarse**: **ningún alimento tiene «calorías
negativas»**. Digerir **nunca cuesta más de lo que aporta**. Lo que existe es **un coste distinto según
el alimento** — y ese coste **se puede calcular**, que es justo lo que hay que hacer en vez de repetir
folclore.
:::

**Y la consecuencia práctica, que es lo que se puede convertir en un consejo justificado**: **lo que se
come en las horas anteriores a una sesión cambia la energía disponible**. Eso **no es una opinión: es
una cuenta.**

## 3. Su edad cambia el motor, y es lo primero que hay que arreglar

Aquí hay un problema **de precisión**, y es serio.

El motor estima la frecuencia cardíaca máxima con la ecuación de **Tanaka** (`208 − 0,7 × edad`), que se
prefiere a la clásica `220 − edad`. **Pero las dos tienen un error individual grande.** Hay un estudio
hecho sobre **una población muy grande y sana** —el **HUNT Fitness Study**
([PubMed](https://pubmed.ncbi.nlm.nih.gov/22376273/))— y una revisión reciente sobre **la exactitud de
las fórmulas por edad** ([ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S1443950624011661)).

**Lo que hay que entender del error**: no es que la fórmula «se equivoque un poco». Es que **dos personas
de 60 años pueden tener frecuencias máximas muy distintas**, y **la fórmula no puede saber cuál es cuál**.

**Para él, la cuenta es esta**: a los 60 años, Tanaka da **208 − 42 = 166**. Y ese número **gobierna las
zonas de entrenamiento enteras**.

::: aviso
**Y aquí está el problema de verdad**: **un ciclista competitivo de 60 años suele tener una frecuencia
máxima MÁS ALTA que la media de su edad**, porque está entrenado. Si el motor usa 166 y su máxima real
es 180, **entonces TODAS sus zonas salen bajas** — y el motor le estaría diciendo que entrena más suave
de lo que entrena.

**Un error de 14 pulsaciones no es un detalle: es prescribir la intensidad equivocada.**
:::

**La solución es la que ya dice el motor, y ahora con más razón**: **medirla**. Una **prueba de esfuerzo**
—que a su edad y con ciclismo competitivo **es razonable que ya tenga hecha, o que le venga bien
hacerse**— da el número **real**. Y mientras no la haya, **el motor tiene que decir, bien alto, que sus
zonas son una estimación**.

**Y hay una segunda razón, de seguridad**: a partir de cierta edad, **una prueba de esfuerzo no es solo
para rendir mejor**. Es **la forma de saber que se puede exigir el corazón con tranquilidad**. Eso
**no lo decide una aplicación**: lo decide un médico.

## 4. El sueño: lo que él intuye también está medido

Sobre el sueño hay estudios concretos, y **el más útil para este caso** compara **dormir más y dormir
menos** y mira **cómo cambia el rendimiento en resistencia y los índices de pulso**
([PubMed](https://pubmed.ncbi.nlm.nih.gov/34711770/)). Y hay trabajo sobre **ciclismo de ultra-resistencia
y privación de sueño** ([Wiley](https://onlinelibrary.wiley.com/doi/full/10.1111/jsr.70295)).

**Lo que esto le da al motor**: **una razón para cruzar las dos cosas** — *«esta semana has dormido menos
y la carga ha subido»* — en vez de mirarlas por separado. **Y eso es exactamente lo que él dice que
funciona: la mezcla.**

## 5. Tu pregunta, que es la buena: ¿hace falta una IA, o basta un algoritmo?

**Respuesta corta: para el cálculo, un algoritmo. Y una IA no debe tocar ni un número.**

La razón es la que ya vimos en este proyecto: **un modelo de lenguaje no calcula, redacta**. Si le pides
una cuenta, **la inventa con seguridad**. Y aquí la cuenta **afecta al cuerpo de una persona**.

**Así que el reparto honesto es este:**

| qué | quién lo hace | por qué |
|---|---|---|
| **medir, calcular, comparar** | **el motor** | es determinista: los mismos datos dan lo mismo **siempre** |
| **decidir qué se puede afirmar** | **el motor** | es lo que ya hace, con las tres respuestas |
| **elegir la sugerencia** | **el motor** | sale de reglas con fundamento, no de una opinión |
| **redactar el informe** | **el motor, con plantillas** | así **el informe es reproducible** |
| **redactar mejor, con más soltura** | *un modelo* | **solo con los datos ya calculados** |

**Y la respuesta larga, que es la importante**: **lo que él hacía con Gemini no era análisis, era
redacción.** El análisis **lo hacía él** —al elegir qué capturas mandar y cómo leer la respuesta—. Lo que
la IA aportaba era **un texto ordenado y una tabla**.

**Eso se puede hacer mejor, y sin IA**: el motor **ya produce** los fundamentos con su fuente. Un
**informe generado desde ahí** sale **igual de bien redactado**, **con la ventaja de que es
reproducible**: los mismos datos dan **el mismo informe**, y **cada afirmación se puede rastrear hasta su
fórmula**.

::: norma
**La regla que propongo, y es de diseño**: **si un modelo de lenguaje entra en este sistema, entra AL
FINAL y SIN CALCULAR.** Recibe **hechos ya calculados** y **solo los redacta**. **Nunca ve los datos
crudos, nunca produce un número, y nunca afirma nada que el motor no haya afirmado antes.**
:::

**Y hay una razón de coste y de privacidad que refuerza lo mismo**: si el modelo **no ve los datos de
salud**, entonces **los datos de salud no salen del dispositivo**. Se le manda **el resultado**, no las
pulsaciones. **Eso resuelve por diseño el problema que quedó abierto** (la decisión H3 de la revisión).

## 6. Cómo queda el reparto, en una imagen

```
   DATOS                    EL MOTOR                         EL INFORME
   ─────                    ────────                         ──────────
   actividades   ──┐
   sueño           ├──→   calcula (con fuente)   ──→   tablas  ──→  informe
   alimentación  ──┤      decide qué se puede         textos         (reproducible)
   persona       ──┘      dice qué falta              sugerencias
```

**Y el paso de «alimentación» está sin construir**, porque **hoy no sabemos qué datos de alimentación
tiene**. Es la pieza que falta para responder a lo que él pide.

## 7. Lo que hace falta preguntarle, y **no** es la contraseña

1. **¿Tiene una prueba de esfuerzo?** Es **el dato que más cambia el resultado** en su caso, por su edad
   y porque compite. Con ella, las zonas dejan de ser una estimación.
2. **¿Qué datos de alimentación puede dar, y cómo?** Porque **«mezcla alimentación»** es lo que dice que
   le funciona, y **sin datos no se puede analizar**. ¿Registra comidas en alguna aplicación? ¿Tiene
   diario? ¿O es «como sano y ya»?
3. **¿Qué quiere decir con «buen estado físico»?** Peso, grasa corporal, VO2 máximo, potencia en bici:
   **cuáles tiene y con qué número**.
4. **¿Cuántas horas duerme, y lo mide?** Garmin ya lo mide, así que **probablemente sí**.
5. **¿Qué busca exactamente este año?** Porque «superarse» puede ser **ganar una carrera**, **mejorar un
   tiempo**, o **mantenerse competitivo a los 60** — y **cada uno pide un plan distinto**.

## 8. Y lo que hay que construir, en orden

1. **El lector de FIT** (el formato está publicado) — **se puede hacer ya**.
2. **El motor del salto**: comparar cada sesión con el mes anterior — **se puede hacer ya**.
3. **El modelo de la alimentación**, cuando se sepa qué datos hay.
4. **El generador de informes**, desde los fundamentos del motor — **reproducible**.
5. **Y solo si hace falta**, el modelo de lenguaje **al final, para redactar**, con los hechos ya hechos.
