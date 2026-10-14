# Especificación de la pantalla «Resumen» (auditoría y propuesta)

**Escrito el 2026-10-14**, después de que el dueño viera la aplicación y dijera, con razón, que el
trabajo era insuficiente. Sus palabras son el encargo de este documento:

> *«Está el texto corrido, los botones no centrados, el gráfico no sé qué representa, debería estar
> desde el inicio y ahí pintar… Hay muchos errores en el modelo o en la interfaz, debemos entregar algo
> legítimo y limpio… hiciste una corrección mínima cuando hay varios detalles.»*

## Por qué me estaba costando (diagnóstico, sin excusas)

No es la herramienta ni el método. Es esto:

1. **Parcheaba síntomas en vez de construir contra un objetivo.** Arreglé «los iconos», luego «los
   vacíos», luego «la altura», cada uno por separado. Sin **una especificación de la pantalla**, eso no
   puede quedar coherente. El boceto **no es una especificación**: es un marcador.
2. **Mezclé dos capas en el mismo cambio.** Ocultar contenido (interfaz) movió la barra (disposición) y
   tuve que perseguirla. Son dos problemas distintos y se arreglan en dos pasos, no en uno.
3. **Miré la captura para CONFIRMAR, no para JUZGAR.** Me pregunté «¿están los iconos?» en vez de
   «¿esto se entiende, está alineado, dice algo?». El dueño lo vio en diez segundos.
4. **El gráfico está mal desde el MODELO, no desde el dibujo.** La barra de cada zona mide su **techo**
   respecto al techo mayor (67 %…100 %). Eso no representa nada: ningún ajuste de dibujo lo arregla.

**La documentación no es el problema.** Lo que faltaba es la pantalla **como pieza de diseño** (AU-66 y
AU-74), y eso es lo que empieza a poner este documento.

## Auditoría: TODO lo que está mal en la captura

| # | lo que se ve | por qué está mal | dónde nace |
|---|---|---|---|
| 1 | «Precisión aceptable: las zonas salen del estimado a partir de la frecuencia máxima, que es una estimación.» | **texto corrido**: es un párrafo, y la pantalla no se explica (AU-44/AU-45) | `cara.rs` `texto_precision` |
| 2 | «Faltan tus datos: con la edad y el umbral ya se calculan las zonas. Se rellenan en Datos.» | dos frases; un vacío se dice en UNA línea | `cara.rs` `que_falta` |
| 3 | las 5 barras de zona, casi todas llenas, sin eje ni unidad | **la barra no representa nada**: es el techo de la zona dividido por el techo mayor | `cara.rs`: `nivel = hasta/tope*100` |
| 4 | «hasta 114», «114 – 127» | mezcla dos notaciones y no dice la unidad (ppm) | `cara.rs` `tramo` |
| 5 | 3 de 4 tarjetas con un «—» de 26 px a la izquierda | tarjetas casi vacías; el guion no informa y desequilibra | `cara.rs` + `ventana.slint` |
| 6 | cifras y filas alineadas a la izquierda con la tarjeta ancha | el dueño las ve como botones mal centrados | `ventana.slint` `Cifra`/`Zona` |
| 7 | un hueco grande entre el último bloque y la barra | el contenido corto no se reparte; el hueco se ve | `armazon.slint` |
| 8 | los iconos de la barra, tenues y pequeños | los elegí a ojo, sin verlos a tamaño real | `iconos.slint` |
| 9 | el título dice «Pulso», no la sección | el top app bar de Android dice dónde estás | `ventana.slint` |
| 10 | no hay **acción primaria** | el recorrido pide «pulsar para analizar»; hoy calcula al abrir y no hay botón | producto |

## Lo que cada bloque debe ser (propuesta, para decidir)

**1 · El cromo de arriba.** Título de la **sección** (Resumen/Zonas/Sesiones/Datos), no el nombre de la
app; 64 dp; 22 px. La app se llama en la sección Resumen, si acaso.

**2 · Las cifras.** Cada tarjeta con **su número centrado** y su etiqueta debajo; cuando un dato no
existe, **no se dibuja la tarjeta** (AU-91), en vez de un «—» de 26 px. Cuatro tarjetas solo si hay
cuatro; si hay una, una. Y el margen va con el número (eso ya está).

**3 · El gráfico de zonas — el fallo de fondo.** Hoy la barra es un número inventado. Propuesta:
**cada zona es una BANDA sobre un eje común de pulsaciones** (de la mínima a la máxima), con el eje
marcado en ppm. Así se ve **qué zona es ancha y dónde cae**, que es lo que un ciclista lee.
- **Opción A (sin sesiones)**: las 5 bandas sobre el eje. Ya se puede dibujar con edad/umbral.
- **Opción B (con sesiones, después)**: añadir **tiempo en zona** por banda, que Garmin ya trae en
  `TimeInZone`. Necesita el modelo, y va en su propio paso.
- El **modelo** debe entregar el **tramo de cada zona en ppm** (inicio y fin), no un «nivel» 0–100.

**4 · Lo que se dice en una línea.** La precisión y lo que falta, en **una línea corta y con su icono**
(AU-45). El párrafo se va a la ayuda, no a la pantalla.

**5 · La barra de abajo.** Iconos a 24 dp, etiqueta 12, **todo centrado en su hueco**, indicador de
píldora. Y **ver los 4 iconos a tamaño real** antes de darlos por buenos.

**6 · El hueco.** O el contenido se reparte, o la pantalla no reserva alto de más. Nada de un vacío
grande entre el contenido y la barra.

## Lo que hay que decidir (dueño)

1. **¿El gráfico está desde el inicio?** El dueño dice que sí («debería estar desde el inicio y ahí
   pintar»). Eso choca con AU-91 (no dibujar vacíos): la salida es que el gráfico **existe** en cuanto
   hay edad/umbral (bandas), y **se pinta** cuando llegan las sesiones. Sin edad/umbral, una línea.
   ¿Se acepta?
2. **¿Hay botón «Analizar»?** Hoy se calcula al abrir. El recorrido pedía pulsarlo.
3. **¿Qué cifras van arriba y en qué orden?** El boceto dice pulso máximo, umbral, vatios/pulso, deriva;
   el motor enseña umbral, frecuencia máxima, vatios/pulso, deriva.

## El plan (y por qué en una conversación nueva)

Este chat lleva **más de 250 000 tokens reenviados en cada turno**; buena parte de por qué me cuesta es
**este contexto**. El método de la casa lo dice: **un bloque = un chat**. Propuesta:

1. **Nueva conversación** que arranque leyendo este documento y `ESTADO.md` §6-nonies.
2. Implementar **un bloque de la especificación a la vez**, contra este documento, no contra el boceto.
3. Cada bloque con **su prueba de geometría** y **su captura**, y **ver la captura como diseñador**
   (¿se entiende?) además de medirla (¿cabe?). Las dos cosas.
4. Empezar por el **gráfico** (es el error de modelo) y las **cifras** (alineación), que es donde más se
   nota.

**Lo que NO se hace**: otro parche suelto. Si no cabe en la especificación, no se toca.
