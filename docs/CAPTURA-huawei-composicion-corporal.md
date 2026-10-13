# La captura del cliente: qué añade, y qué choca

**Analizada el 2026-10-13.** Este documento nace de una captura de una aplicación de salud y de una
**báscula de composición corporal**.

::: aviso
**Los valores de las tablas son DE EJEMPLO, no de ninguna persona.** Lo que se conserva es **el método**
—cómo se comprueba una báscula, y qué pasa cuando dos aparatos dicen cosas distintas—, porque **las
medidas corporales de una persona concreta no se publican**.
:::

---

## 1. Lo primero, y es una sorpresa de fondo

**No es de Garmin. Es de Huawei.** Y eso significa que el cliente tiene **dos ecosistemas**, con datos en
cada uno:

| ecosistema | qué mide | de dónde salen los datos que ya tengo |
|---|---|---|
| **Garmin** | **el entrenamiento**: pulso, potencia, cadencia, VO2, umbral | sus 21 actividades, en FIT |
| **Huawei** | **el cuerpo**: peso, grasa, músculo, agua, hueso | **esta captura, y solo como imagen** |

**Consecuencia**: los datos de composición corporal **no están en ninguna parte accesible**. La captura es
**una foto**: no se puede consultar la de la semana pasada, ni ver una tendencia. **Hay que decidir si se
pide acceso a Huawei Health o si se le pregunta a él.** Y **hay una API de Huawei** (su portal de
desarrolladores responde), pero eso es otra integración, con su propio trámite.

## 2. Lo que la captura añade al modelo

Estos campos **no los tenía**, y son los que el motor necesita para la parte de «la persona»:

| dato | valor | para qué sirve en el motor |
|---|---|---|
| **peso** | **70,2 kg** | la carga relativa; y la potencia por kilo |
| **IMC** | **22,1** (saludable) | referencia general |
| **grasa corporal** | **19,1 %** | el dato que de verdad dice la composición |
| **masa muscular** | **31,0 kg** | seguir si el entrenamiento la conserva |
| **grasa visceral** | **9,8** (nivel) | indicador de salud metabólica |
| **tasa metabólica basal** | **1 540 kcal/día** | **el gasto en reposo**: base para la alimentación |
| **agua corporal** | **55,0 %** | referencia de hidratación |
| **masa ósea** | **3,01 kg** | referencia |
| **proteínas** | **21,6 %** | referencia |
| **masa libre de grasa** | **56,8 kg** | la parte que produce potencia |

**Y hay un dato que vale mucho para lo que él pide**: la **tasa metabólica basal, 1 540 kcal/día**. Es
**la energía que gasta en reposo**, y es **la base de cualquier cálculo de alimentación**. Sin ella, hablar
de comida es hablar a ciegas.

## 3. Y comprobé que la báscula es coherente consigo misma

Antes de dar por buenos los números, **se comprueban**. Y cuadran:

| comprobación | resultado |
|---|---|
| **el IMC** | con 70,2 kg y 178 cm sale **22,1** — y la báscula dice **22,1**. **Cuadra.** |
| **la suma de las partes** | agua 38,6 + proteínas 15,2 + grasa 13,4 + hueso 3,01 = **70,2 kg** — **exactamente el peso**. |
| **el porcentaje de grasa** | 13,4 ÷ 70,2 = **19,1 %** — y la báscula dice **19,1 %**. **Cuadra.** |

::: norma
**La lección, y es la misma del acumulador de potencia**: **un dato se comprueba antes de usarlo.** Que la
báscula cuadre **consigo misma** no garantiza que sea exacta —para eso haría falta otra referencia—, pero
**descarta que los números estén rotos**. Y esa comprobación **cuesta un minuto**.
:::

**Y un detalle que conviene decirle al cliente**: la propia aplicación de Huawei se desmarca. Al pie de su
captura dice, con sus palabras: *«los datos y los resultados se proporcionan solo con fines de referencia.
No deben considerarse como diagnósticos profesionales»*. **La báscula no es un laboratorio.**

## 4. El choque: el pulso en reposo, y esto SÍ cambia las zonas

Aquí está el problema importante, y **es de interpretación, no de dato**:

| fuente | pulso en reposo |
|---|---|
| **Garmin** (su perfil) | **48** |
| **Huawei** (la báscula, esa tarde) | **71** |

**23 pulsaciones de diferencia.** Y **las dos pueden ser correctas**, porque **no miden lo mismo**:

- **el 48** es el **más bajo observado**, probablemente **mientras duerme**. Es el valor que usa el
  **método de Karvonen**, y es el correcto para calcular zonas. Un ciclista entrenado con un consumo de
  oxígeno de 57 **tiene** ese pulso de reposo.
- **el 71** es **una medida puntual, sentado y despierto**. Eso **no es** el pulso de reposo en el sentido
  fisiológico: es **el pulso en un momento del día**, y depende de lo que hiciera antes, del café, del
  calor o del estrés.

**Y lo que cuesta confundirlos, medido:**

| zona | con reposo **48** | con reposo **71** | diferencia |
|---|---:|---:|---:|
| **Z1** | 114 ppm | 125 ppm | **+12** |
| **Z2** | 127 ppm | 136 ppm | **+9** |
| **Z3** | 140 ppm | 147 ppm | **+7** |
| **Z4** | 159 ppm | 163 ppm | **+3** |

: Las zonas según el pulso de reposo que se use, con la frecuencia máxima real de 179.

::: aviso
**Hasta 12 pulsaciones de diferencia en la zona suave**, solo por elegir mal qué «pulso de reposo» se usa.

**Y esto es un defecto del motor, no del dato**: hoy el motor tiene **un solo campo** llamado
`pulso_reposo`, **y no distingue** si es el de por la mañana, el de una medición puntual o el mínimo
nocturno. **Un campo, tres significados** — exactamente el problema que el acumulador de potencia ya
enseñó una vez en este proyecto.

**Lo que hay que hacer**: que el motor **sepa cuál es cuál**, y que **use el nocturno para las zonas**,
diciendo cuál usó. Y si solo tiene el puntual, **que lo advierta**.
:::

## 5. Lo que esto cambia en el trabajo

1. **El motor necesita más campos de persona**: composición corporal, tasa metabólica basal, y **el pulso
   de reposo con su origen declarado**.
2. **Hay una decisión nueva**: **Huawei entra como segunda fuente**, o **se le pregunta a él** y se
   escribe a mano. Empezar preguntando es más barato; la integración se decide después.
3. **Y una cosa que hay que aclarar con el cliente**: **cuál de los dos pulsos de reposo es el suyo**. No
   para elegir uno, sino para **saber qué significa cada número** — porque **las zonas dependen de ello**.
