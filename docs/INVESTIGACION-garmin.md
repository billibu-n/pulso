# Garmin: qué se puede obtener, cómo, y dónde está la traba

**Investigado el 2026-10-13** · Origen: el dueño pasó el enlace de **Garmin Connect** para saber si el
encargo se puede cumplir.

---

## 1. La conclusión, en una frase

> **Los datos que el cliente necesita EXISTEN y son suficientes. La vía oficial, en cambio, no está
> abierta para ustedes.**

Hay **dos caminos**, y **no valen lo mismo**:

| camino | qué es | ¿sirve aquí? |
|---|---|---|
| **A · el programa oficial de Garmin** | acceso de **empresa** a las APIs de Garmin | **hoy NO**: es **solo para uso empresarial** |
| **B · el usuario extrae sus propios datos** | las credenciales **del usuario**, y los datos **a su propio equipo** | **SÍ, y es el que encaja** |

## 2. Qué datos hay, exactamente (esto es la buena noticia)

Garmin publica una **matriz que compara sus dos APIs**
([matriz oficial, PDF](</home/billibu/projects/app-entrenamiento/docs/fuentes/garmin-comparison-matrix.pdf>)):

| | **Fitness / Connect API** | **GCS / Wellness API** |
|---|---|---|
| **formato** | **FIT** (medición completa del aparato) | **JSON** (15 minutos o día completo) |
| **estructura** | actividades discretas: **comienzo y fin** | igual |
| **datos** | **pulso, potencia, velocidad, ritmo, cadencia, GPS, distancia, calorías, METs, intensidad** | **pasos, calorías, distancia, sueño, pulso del día, báscula (peso, grasa corporal, masa ósea)** |

**Y lo importante**: el **FIT trae la medición completa del aparato** — o sea, **la serie segundo a
segundo**. Eso es **justo lo que el motor necesita**: no un resumen, sino **la curva**.

## 3. La traba, y está escrita por Garmin

De la **pregunta frecuente oficial** del programa
([FAQ, guardada en texto](</home/billibu/projects/app-entrenamiento/docs/fuentes/garmin-program-faq-TEXTO.md>)):

::: aviso
**«El Garmin Connect Developer Program está disponible para uso empresarial.»**

**«No hay tasas de licencia ni de mantenimiento para acceder al programa, pero es solo para uso
empresarial. El acceso a algunas métricas puede requerir el pago de una licencia o una cantidad mínima
de pedido de dispositivos para uso comercial.»**
:::

**Lo que esto significa, sin adornos**:

- **Una persona particular no entra.** Hace falta **una empresa** que acepte el acuerdo.
- **El acuerdo lo firma quien pueda obligar legalmente a una empresa** («Licensee»), no un individuo
  ([acuerdo del programa, PDF](https://www8.garmin.com/en-US/GARMINCONNECTDEVELOPERPROGRAMAGREEMENT/GARMINCONNECTDEVELOPERPROGRAMAGREEMENT_EN.pdf), 19 páginas).
- **No tiene costo**, pero **algunas métricas sí** — y esas se negocian.
- **Los plazos, si se pidiera**: confirman en **dos días hábiles**, y una integración típica lleva
  **de una a cuatro semanas**.
- **Todas las APIs usan OAuth 2.0**, y hay cinco: **Health**, **Activity**, **Training**,
  **Women's Health** y **Courses**.

## 4. El camino B, y por qué encaja mejor aquí

**El usuario extrae sus propios datos con sus propias credenciales.** Y hay herramientas hechas y
mantenidas para eso:

- **`garmin-health-data`** ([PyPI](https://pypi.org/project/garmin-health-data/)) — extrae **todo** a una
  base de datos **SQLite local**. Licencia **Apache 2.0**, y lo dice claro:
  **«las credenciales nunca salen de tu equipo»**, **«todos los datos se quedan en tu equipo»**.
- **`garmin-connect-export`** ([GitHub](https://github.com/johannesh83/garmin-connect-export)) —
  descarga los ficheros **FIT, TCX, GPX o JSON** de las actividades.

**Lo que trae `garmin-health-data`**, que es una lista de la compra casi perfecta:

| dato | detalle |
|---|---|
| **sueño** | **7 tablas**: fases, movimiento **cada minuto**, **SpO2 cada minuto**, **HRV cada 5 minutos**, respiración |
| **pulso continuo** | **cada 2 minutos** |
| **estrés** | cada 3 minutos |
| **respiración** | cada 2 minutos |
| **VO2 máximo y carga de entrenamiento** | diario, con **ACWR** (la relación de carga aguda y crónica) |
| **disposición (readiness)** | a diario |
| **actividades** | **FIT con la serie segundo a segundo**: pulso, potencia, cadencia, GPS, y **dinámica de carrera** (tiempo de contacto, oscilación vertical, longitud de zancada) |
| **fuerza** | **por serie**: repeticiones, peso y **ejercicio clasificado** |
| **perfil del usuario** | características físicas y métricas de forma |

::: norma
**Esto responde al encargo con un «sí»**: los **dos tipos de dato** que el cliente describió están
**los dos**: los **resultados del entrenamiento** (FIT, serie a serie) y **los datos de la persona**
(perfil, peso, composición corporal por la báscula de Garmin).
:::

## 5. Los riesgos del camino B, dichos claros

**No es el camino oficial, y eso tiene consecuencias que hay que poner sobre la mesa:**

1. **La herramienta usa las credenciales del usuario contra la API interna de Garmin**, no contra la API
   pública del programa. **Garmin puede cambiarla o cerrarla cuando quiera**, y entonces **la aplicación
   deja de funcionar sin aviso**.
2. **Hay medidas anti-abuso** (Cloudflare): la autenticación tiene **esperas deliberadas de 30 a 45
   segundos** y estrategias de reserva si falla. **O sea: frágil por diseño.**
3. **La potencia y los datos avanzados vienen del FIT**, no de la API: eso **sí es el fichero del
   aparato**, y es lo más sólido de todo.
4. **Hace falta la contraseña de Garmin del usuario** — que es **exactamente lo que él todavía no ha
   dado**, y con razón: hay que explicarle **dónde queda**.

**Y una ventaja que pesa mucho en este encargo**: **todo se queda en el equipo del usuario**. Para
**datos de salud**, eso no es un detalle: **es lo que hace que el diseño sea defendible**.

## 6. La recomendación

**Para el encargo tal como está hoy** —un usuario, sus datos, en su equipo—: **camino B**.

**Y hay que decírselo al cliente con estas palabras**: *la aplicación leerá sus datos **en su equipo**,
con sus credenciales, y **no mandará nada a ningún sitio** sin que él lo autorice.*

**Si algún día esto crece** —varios usuarios, o venderlo como servicio—: **entonces sí hace falta el
programa oficial**, y eso significa **empresa constituida**, **acuerdo firmado** y **de una a cuatro
semanas** de integración. **Conviene saberlo desde ahora**, porque **cambia qué se puede prometer**.

## 7. Lo que cambia en el motor que ya está hecho

**Casi nada, y eso es la prueba de que el diseño era correcto.** El motor espera un modelo **nuestro**;
**Garmin entra como un adaptador más**, y tenía que ser así desde el principio.

Lo que **sí** hay que añadir al modelo, porque Garmin lo trae y el motor aún no lo usa:
**potencia**, **HRV**, **fases del sueño**, **VO2 máximo** y **ACWR**. Y hay una decisión de fondo
nueva, que es interesante: **el ACWR y el VO2 máximo ya vienen calculados por Garmin**. ¿Se usan los
suyos, se calculan los nuestros, o **se comparan**? *(Compararlos es lo que más enseña: si no coinciden,
uno de los dos está mal.)*
