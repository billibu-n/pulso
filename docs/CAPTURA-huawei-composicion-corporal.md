# La captura del cliente: qué añade, y qué choca

**Analizada el 2026-10-13.** Es una captura de **Salud de Huawei**, tomada el **13 de octubre a las
16:38**, de una **báscula de composición corporal** («Desde HUAWEI Scale 3»).

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
| **peso** | **69,45 kg** | la carga relativa; y la potencia por kilo |
| **IMC** | **21,9** (saludable) | referencia general |
| **grasa corporal** | **18,6 %** | el dato que de verdad dice la composición |
| **masa muscular** | **30,6 kg** | seguir si el entrenamiento la conserva |
| **grasa visceral** | **10,0** (nivel) | indicador de salud metabólica |
| **tasa metabólica basal** | **1 533 kcal/día** | **el gasto en reposo**: base para la alimentación |
| **agua corporal** | **55,3 %** | referencia de hidratación |
| **masa ósea** | **2,99 kg** | referencia |
| **proteínas** | **21,8 %** | referencia |
| **masa libre de grasa** | **56,6 kg** | la parte que produce potencia |

**Y hay un dato que vale mucho para lo que él pide**: la **tasa metabólica basal, 1 533 kcal/día**. Es
**la energía que gasta en reposo**, y es **la base de cualquier cálculo de alimentación**. Sin ella, hablar
de comida es hablar a ciegas.

## 3. Y comprobé que la báscula es coherente consigo misma

Antes de dar por buenos los números, **se comprueban**. Y cuadran:

| comprobación | resultado |
|---|---|
| **el IMC** | con 69,45 kg y 178 cm sale **21,9** — y la captura dice **21,9**. **Cuadra.** |
| **la suma de las partes** | agua 38,39 + proteínas 15,15 + grasa 12,92 + hueso 2,99 = **69,45 kg** — **exactamente el peso**. |
| **el porcentaje de grasa** | 12,92 ÷ 69,45 = **18,6 %** — y la captura dice **18,6 %**. **Cuadra.** |

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
