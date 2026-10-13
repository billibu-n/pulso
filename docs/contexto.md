---
title: "Contexto justificado: aplicación de entrenamiento, sensores y análisis"
author: "Emilio Bergez"
---

# Introducción

Este documento fija **el contexto técnico** de la aplicación que se va a construir: **qué datos entran,
qué se hace con ellos y por qué**, con las fuentes que lo respaldan. Se escribe **antes** de una línea de
código, y por una razón concreta: el encargo pide **resultados justificados y coherentes**, y eso solo se
consigue si **el método está decidido antes que el programa**.

La petición del cliente, en sus palabras: contrata un entrenador personal, y **a partir de los datos que
obtiene de sus sensores** quiere un **diagnóstico** y un **plan de entrenamiento** para mejorar de forma
continua. Hoy lo hace **a mano**: entrega sus datos a un modelo de inteligencia artificial y le pide
tablas y un análisis. Lo que falta es **que eso ocurra dentro de una aplicación**, bien hecho.

# Los dos tipos de dato, y por qué no se mezclan

Toda la aplicación se apoya en **dos entradas**, y **confundirlas es el primer error**:

| entrada | qué es | quién la produce | ejemplo |
|---|---|---|---|
| **la sesión** | lo que pasó durante el entrenamiento | **los sensores** | pulsaciones, potencia, duración, distancia, series |
| **la persona** | cómo está el usuario | **él mismo**, con datos médicos o de otras herramientas | peso, porcentaje de musculatura, edad, etnia |

**Por qué la distinción importa**: **la primera se mide y la segunda se declara**. Los sensores
**pueden fallar o faltar**; los datos de la persona **pueden estar desactualizados o no existir**. Y el
resultado —el diagnóstico y el plan— **hereda los errores de las dos**. Un plan construido sobre una
báscula de hace tres meses, o sobre una sesión con el sensor de pulso mal puesto, **parece correcto y no
lo es**.

::: aviso
**El problema central del encargo no es calcular: es decidir qué hacer cuando falta un dato.** El propio
cliente lo dice: *«un resultado al que le falta data puede arrojar resultados con márgenes de error no
menores»*. Eso **no se arregla con un modelo de inteligencia artificial**: se arregla **declarando qué
falta y cuánto afecta**. Un modelo al que le falta un dato **lo inventa con seguridad**, que es la peor
combinación posible en algo que afecta al cuerpo de una persona.
:::

# Con qué se mide: los formatos de los sensores

Lo que el cliente llama «su `.json`» procede, en la mayoría de los casos, de **un formato estándar de la
industria**: el **FIT**, el formato que usan Garmin y buena parte de los relojes y medidores de potencia
([Garmin, s. f.](https://developer.garmin.com/fit/overview/)). Otros formatos habituales son **TCX** y
**GPX**, más simples y orientados a la ruta.

**Por qué esto importa para el diseño**: si el origen es un formato **conocido y documentado**, la
aplicación **no debería leer el `.json` como un texto cualquiera**, sino **entender sus campos**: qué es
`power`, qué es `heart_rate`, qué es un `lap` (una vuelta, un tramo). **Un mismo número significa cosas
distintas según el campo**, y confundirlos produce un análisis con números plausibles y conclusiones
falsas.

**Y en el lado de la persona**, los datos de salud y actividad tienen sus propios estándares de
interoperabilidad —**HealthKit** en el ecosistema de Apple y **Health Connect** en Android—, que definen
los **tipos de dato** y las **unidades** con las que se intercambian
([Health Connect, s. f.](https://pub.dev/documentation/health_connector/2.4.0/topics/Health%20Data%20Types-topic.html)).
**Aplicación independiente no significa aislada**: significa que **no depende del navegador**, y que
**habla los formatos que el resto del mundo habla**.

# Qué dicen las fuentes sobre el entrenamiento

Aquí está el fundamento de que un plan **se pueda justificar**, en lugar de inventarse.

**Cuánto ejercicio, y de qué tipo.** La Organización Mundial de la Salud publica las directrices de
referencia sobre actividad física y comportamiento sedentario
([OMS, 2020](https://www.who.int/es/publications/b/55501)), y el **American College of Sports Medicine**
es la referencia para **prescribir** ejercicio: sus guías y sus tomas de posición sobre entrenamiento de
fuerza ([ACSM, s. f.](https://acsm.org/wp-content/uploads/2026/03/Pronouncement-ppt-deck_resistance-training-ps.pdf))
definen **qué dosis** de ejercicio corresponde a **qué objetivo**. **Un plan que no se pueda comparar con
estas guías no es un plan: es una opinión.**

**La intensidad, que es donde se mide de verdad.** La intensidad del esfuerzo se estima a partir de la
**frecuencia cardíaca**, y para eso hacen falta dos referencias: la **frecuencia máxima** y la **de
reposo**. La **reserva de frecuencia cardíaca** —la diferencia entre ambas— es la base del **método de
Karvonen** para fijar zonas ([Karvonen, 1957, citado en *Hungarian Review of Sports Medicine*](https://real-j.mtak.hu/11265/9/650.2002.08.04.pdf)),
y para estimar la máxima se usan **ecuaciones basadas en la edad**, que **tienen error** y por eso se
comparan entre sí antes de elegir una.

**La carga acumulada.** El **TRIMP** (*training impulse*) es la forma clásica de resumir **cuánto
esfuerzo** ha supuesto una sesión, combinando **duración e intensidad**
([Banister, citado en Hourcade, 2019](https://theses.hal.science/tel-02117612v1/file/va_Hourcade_Jean-Christophe.pdf)).
Es lo que permite comparar **sesiones distintas** y ver **si el plan está progresando o estancado**.

**Y la recuperación.** La **variabilidad de la frecuencia cardíaca** se usa como indicador de **estado y
recuperación**, y es el dato que suele decidir **si hoy toca entrenar fuerte o no**. Es, además, **uno de
los más frágiles**: depende de cómo se mide, cuándo y con qué.

::: norma
**La consecuencia de todo esto para el diseño**: un diagnóstico **justificado** tiene que poder decir
**de dónde sale cada afirmación** —«esta zona sale de esta fórmula», «esta carga sale de este modelo»,
«este consejo se corresponde con esta guía»—. **Si no se puede citar, no es un diagnóstico: es una
sugerencia.**
:::

# Por qué hacía falta un modelo de inteligencia artificial (y qué no debe hacer)

El cliente usaba un modelo porque **quería un análisis redactado y razonado**, no solo números. Eso tiene
sentido: **la parte difícil no es calcular, es explicar**.

Pero ahí hay **dos peligros**, y conviene tenerlos escritos desde el principio:

**El primero: el modelo no sabe lo que no le han dado.** Si falta el dato de sueño, **no lo echa de
menos**: escribe un plan como si lo tuviera. Por eso **la aplicación tiene que saber qué le falta
ANTES de preguntar nada**, y **decírselo al usuario** en vez de disimularlo.

**El segundo: los datos de salud son los más sensibles que existen.** Mandarlos a un servicio externo
**es una decisión con consecuencias**, no un detalle técnico: hay que decidir **qué sale del dispositivo,
qué se queda, por cuánto tiempo**, y **decírselo al usuario con claridad**.

::: aviso
**Y la consecuencia práctica, que ordena todo el resto**: la aplicación tiene que poder decir **qué
sabe, qué no sabe, y cuánto le afecta no saberlo**. Un plan con **tres datos** no vale lo mismo que un
plan con **diez**, y **el usuario tiene derecho a saber en cuál de los dos casos está**.
:::

# Lo que hay que decidir, y no se puede decidir solo

| decisión | por qué es del cliente y no mía |
|---|---|
| **Qué datos son obligatorios y cuáles opcionales** | porque decide **qué se le puede prometer** al usuario |
| **Si los datos de salud salen del dispositivo** | porque es **su responsabilidad**, no una preferencia técnica |
| **Qué se hace cuando falta un dato** | porque hay tres respuestas honestas y **ninguna es «inventarlo»**: decir que no se puede, dar un resultado con su margen, o pedir el dato |
| **Quién responde del plan** | porque un plan de entrenamiento **afecta al cuerpo de una persona** |

# Conclusión

El encargo no es «una aplicación que lea un `.json`». Es **un sistema que decide qué hacer con datos
incompletos, y que justifica lo que afirma**. Los formatos de los sensores están **estandarizados**, el
fundamento del entrenamiento está **publicado**, y las fórmulas de intensidad y carga **tienen nombre y
autor**. Lo que falta —y es lo que hay que construir— es **la parte que decide, avisa y explica**.

Eso es lo que este documento deja fijado **antes** de escribir código: **qué entra, con qué se mide, qué
se puede afirmar, y qué no**.
