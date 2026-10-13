# El patrón del cliente: medido, y qué se puede hacer con él

**Medido el 2026-10-13** sobre **21 actividades reales** (13-sep a 11-oct), **1,3 millones de lecturas**,
un dato **por segundo**.

**Aviso**: aquí hay **estructura y números agregados**, nunca datos crudos de salud. Y el análisis es
**repetible**: `analisis/explorar.py`.

---

## 1. Primero, una comprobación que hice y salió bien

En los datos había **dos campos de potencia**: `power` (la lectura instantánea) y `accumulated_power`
(un contador que **solo sube**). **Sospeché que los había mezclado** en un cálculo.

**Comprobado**: `accumulated_power` va `34, 70, 117, 150, 191…` y llega a **1.040.641** — es un
acumulador. **No se mezcló.** Y la verificación cruzada lo confirma: **la media de pulso que calculo
coincide con la que da Garmin** (115,5 contra 114 · 110,0 contra 110 · 138,6 contra 139).

::: norma
**La lección, que va a la norma**: dos campos con **el mismo nombre casi igual** y **significados
distintos** es una trampa clásica. **Un acumulador no es una medida instantánea**, y si se confunden,
los números salen plausibles y falsos. **La comprobación cruzada** —comparar mi cálculo con el que ya
da el aparato— es lo que lo caza. **Se hace siempre que se pueda.**
:::

## 2. El hallazgo: tiene un LÍMITE DE DURACIÓN a intensidad media

La medida es el **desacoplamiento pulso/potencia**: se compara la **primera mitad** de la sesión con la
**segunda**. Si el pulso **sube** y la potencia **baja**, el motor aeróbico se está cansando.

| duración de la sesión | sesiones | **desacoplamiento** | potencia media |
|---|---:|---:|---:|
| **hasta 60 min** | 10 | **+6,5 %** | 116 W |
| **60 a 120 min** | 6 | **+17,5 %** | **147 W** |
| **más de 120 min** | 1 | **−3,4 %** | 76 W |

**Y las cinco sesiones de 105 minutos, una a una:**

| fecha | potencia | desacoplamiento |
|---|---:|---:|
| 15-sep | 142 W | **+27,5 %** |
| 03-oct | 154 W | **+21,6 %** |
| 18-sep | 155 W | **+18,8 %** |
| 29-sep | 159 W | **+17,2 %** |
| 11-oct | 154 W | **+16,0 %** |

::: aviso
**Lo que esto dice, y es lo importante para él:**

- **Hasta una hora, aguanta bien** (6,5 %). Su motor responde.
- **A partir de unos 90 minutos a intensidad media (150 W), se le va el pulso arriba y la potencia
  abajo** (17 %). **Ahí está su límite.**
- **Y cuando va suave, aguanta lo que le echen**: la salida de **210 minutos a 76 W** da **−3,4 %** —
  o sea, **el pulso se le quedó por debajo de lo esperado**. **Su base aeróbica es muy buena.**

**En una frase, para él**: *«a partir de la hora y media a 150 vatios, tu pulso se te escapa. Hasta ahí,
vas sobrado. Y a ritmo suave aguantas lo que sea.»*
:::

## 3. Y lo que esto significa para sus objetivos

**Ojo con lo que está pasando**: entrena **78 % del tiempo en intensidad baja y 3 % en alta** — y
**la intensidad donde falla (150 W durante 105 minutos) no es ni suave ni explosiva: es el punto
intermedio**, justo donde aparece el desacoplamiento.

**Y ahí está la lectura**: **le falta trabajo en ese punto medio**, no más suave ni más fuerte. **Eso es
una sugerencia con fundamento**, no una opinión: sale de **sus** datos y del patrón de **todas** sus
sesiones.

## 4. El error que le está ocultando esto

**Su pulso máximo real es 179. La estimación por edad da 166.** Y sus zonas —las que él ve, y las que
usan estos cálculos— **están calculadas con la estimación**.

**Consecuencia**: **todo su reparto de zonas está desplazado**, y **el 3 % de alta intensidad que ve no
es el 3 % real**. **Cuando se corrija, el cuadro cambia** — y es lo primero que hay que arreglar.

## 5. Qué se puede hacer con esto, HOY, sin más datos

| qué | cómo | qué le dice |
|---|---|---|
| **eficiencia** | **vatios por pulsación**, sesión a sesión | si sube, **mejora**: hace lo mismo con menos pulso |
| **durabilidad** | el **desacoplamiento** de cada sesión | dónde está su límite de duración |
| **el patrón** | agrupar por duración e intensidad | **el punto exacto donde falla** |
| **reparto real de zonas** | con su **máxima real (179)** | cuánto entrena de verdad en cada zona |
| **comparación con Garmin** | su carga contra la que ya calcula Garmin | si no coinciden, **uno de los dos está mal** |

**Todo eso sale de los datos que YA están aquí.** No hace falta esperar a nada.

## 6. Lo que falta para cerrar el bucle (la retroalimentación)

**Lo que hay es un mes. Un mes son ~20 sesiones y unas 30 horas.** Con eso:

- **lo que funciona**: comparar sesiones **entre sí** — eficiencia, durabilidad, reparto;
- **lo que NO se puede todavía**: saber si **está mejorando o estancado**, porque **mejorar es una
  tendencia, y una tendencia necesita dos puntos separados en el tiempo**.

**Y para la retroalimentación completa faltan tres cosas, y ninguna es un dato de sensor:**

| qué falta | por qué | quién lo da |
|---|---|---|
| **más tiempo**: otro mes | para ver **tendencia**, no foto fija | se extrae solo, cuando haya pasado |
| **qué significa «mejorar» para él** | ganar una carrera, un tiempo, o aguantar más | **tu cliente, en una frase** |
| **su contexto**: sueño, sensación, comida, alcohol | es **«la mezcla»** que él dice que le funciona | **tu cliente, cuatro campos** |

## 7. Los cuatro campos, y por qué esos

Se piden porque **cada uno tiene un uso medible**, no por rellenar:

| campo | para qué se usa |
|---|---|
| **horas de sueño** | cruzarlo con la carga y con la eficiencia del día siguiente |
| **cómo se sintió** (1 a 5) | **es la señal que él ya da a mano**, y hoy se pierde |
| **qué comió antes** (proteína / hidratos / grasa / nada) | el **efecto térmico** se calcula con eso |
| **si tomó alcohol** | afecta al sueño, y el sueño se mide |

**Cuatro respuestas, tres de ellas una marca.** Eso **se contesta**; un diario de comidas, no.
