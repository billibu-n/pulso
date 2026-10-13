# Lo que dicen los estudios MODERNOS sobre la frecuencia maxima estimada

**Recogido el 2026-10-13.** Nace de una pregunta del dueno: *«quizá eso ya esté contemplado en
trabajos más recientes»*, sobre la anomalía que se le señalo al cliente (su maxima estimada por
edad se queda corta).

**Respuesta corta: no solo no se ha corregido, sino que un trabajo de 2025 lo empeora.**

---

## 1. Estudio de 2025: NINGUNA ecuacion funciona por encima de los 40 anos

**Titulo**: *Maximum heart rate prediction equations fail key external validation test*
**Autores**: Portuguez Molina, P. y Aragon-Vargas, L. F.
**Revista**: Pensar en Movimiento, vol. 23, n. 1, pp. 20-44, 2025
**DOI**: 10.15517/pensarmov.v23i1.64714
**Enlace**: https://www.scielo.sa.cr/scielo.php?script=sci_abstract&pid=S1659-44362025000100020

**Que hicieron**: validaron **7 ecuaciones** de prediccion contra un conjunto de datos
**independiente**: **634 personas** (474 hombres, 160 mujeres), de **18 a 85 anos**, con prueba de
esfuerzo real en cinta.

**Que encontraron**, con sus palabras:

> *«Ninguna de las ecuaciones analizadas para la muestra completa acepto las dos hipotesis
> (pendiente = 1 y constante = 0).»*

> *«Al hacer el analisis por sexo, seis de las ecuaciones cumplieron las dos hipotesis para las
> mujeres, pero ninguna para los hombres; y al hacerlo por grupo de edad, 4 de las ecuaciones
> cumplieron las hipotesis para el grupo de 40 anos o menos, pero NO para los mayores de 40.»*

> *«La frecuencia cardiaca maxima parece dificil de predecir mediante una sola ecuacion. Se
> recomienda que, cuando se necesite una medida valida, se use una prueba de esfuerzo.»*

**Lo que esto cambia para nuestro caso**: el cliente tiene **60 anos**. No es que su formula
concrete tenga error: es que **ninguna de las 7 probadas sirve a esa edad**. La conclusion del
estudio es exactamente lo que el motor ya dice: **hay que medirla**.

## 2. Estudio de 2025: las zonas por porcentaje fijo son inexactas

**Titulo**: *Zone 2 Intensity: A Critical Comparison of Individual Variability in Different
Submaximal Exercise Intensity Boundaries*
**Publicado en**: PMC (PMC11986187)

**Que hicieron**: **50 ciclistas** (30 hombres, 20 mujeres) hicieron prueba de rampa y prueba
escalonada en laboratorio, midiendo potencia, pulso, **lactato en sangre**, ventilacion y uso de
sustratos.

**Que encontraron**:

> *«Variabilidad sustancial en los marcadores de Z2, con coeficientes de variacion del **6 % al
> 29 %** segun el parametro.»*

> *«El **umbral ventilatorio 1** y la **maxima oxidacion de grasas** mostraron una fuerte
> concordancia, mientras que los **porcentajes fijos de la frecuencia maxima** y los umbrales de
> lactato en sangre mostraron **amplias diferencias individuales**.»*

> *«Los marcadores estandarizados para Z2, como los porcentajes fijos de la frecuencia maxima,
> ofrecen simplicidad practica pero pueden reflejar de forma inexacta la respuesta metabolica,
> afectando potencialmente a los resultados del entrenamiento.»*

**Lo que esto cambia para nuestro caso**: usar un **porcentaje fijo** de la frecuencia maxima
—que es lo que hace el motor cuando estima— **no refleja la fisiologia de esa persona concreta**.
Y lo que **si** lo refleja es un **umbral medido**. El cliente **tiene uno**: 169.

## 3. La conclusion, y es la que ordena el diseno

::: norma
**Lo que dicen los estudios modernos, en una frase**: **la frecuencia maxima NO se estima, se mide.**
Y **por encima de los 40 anos, ninguna formula lo hace bien**.

**Consecuencia para el motor, y hay que cambiarlo**:

1. **El umbral de lactato (169) MANDA sobre la formula.** En ciclismo las zonas se fijan sobre el
   umbral, y ademas **es un dato medido** del propio cliente.
2. **La formula por edad baja de categoria**: deja de ser el calculo principal y pasa a ser
   **un ultimo recurso**, con su aviso — que es lo que ya hace.
3. **Y con el umbral, las zonas cambian**: su zona de umbral empieza en **159**, no en 152. Eso es
   **prescribir bien la intensidad**, y es la diferencia entre un plan que funciona y uno que no.
:::

## 4. Y sobre el entrenador privado del cliente

El dueno apunta que **el cliente contrata un entrenador privado**, y que *«ellos justifican su
informacion con datos y estudios veridicos»*.

**Y es exactamente lo que estos dos estudios respaldan**: un entrenador que trabaja bien
**no usa porcentajes fijos de la frecuencia maxima** — usa **umbrales medidos**, y **pruebas de
esfuerzo**. Es tambien lo que recomienda la literatura.

**Consecuencia practica, y es una oportunidad**: si el cliente **ya tiene entrenador**, lo mas
probable es que **ese entrenador tenga sus umbrales medidos**. Pedirlos **es mas facil que
medirlos**, y **con ellos el motor pasa de estimar a saber**.
