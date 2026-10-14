# Pulso — análisis de entrenamiento con fundamento

**Un motor que lee datos de entrenamiento, calcula, y dice qué se puede afirmar — y qué no.**

Este repositorio contiene **el motor, la documentación y los análisis** de una aplicación de
entrenamiento. Nace de un encargo real: una persona que entrena en bici, que quiere **un diagnóstico y un
plan con fundamento**, y que hasta ahora lo hacía **a mano**, mandando capturas de pantalla a un modelo de
inteligencia artificial.

Lo que sigue es **lo que hace diferente a este motor**.

---

## Este ES el repositorio del proyecto

**Escrito el 2026-10-13, y aclara una confusion que costo trabajo:** durante un tiempo hubo **dos copias**
del mismo proyecto, una aqui y otra en `app-entrenamiento/`, y **21 ficheros existian solo en la
segunda** -- o sea, **trabajo sin versionar en ningun sitio**. Se comprobaron los dos arboles fichero a
fichero, se trajo lo que faltaba, se quitaron los duplicados, y la otra copia **se borro**.

Asi que:

- **`pulso` es el proyecto.** El motor, los documentos, los analisis y los datos de ejemplo.
- **Los datos reales NO estan aqui, y no pueden estarlo.** El repositorio es **publico** y los datos son
  **de salud de una persona**. Viven al lado, en `datos/`, que esta ignorado por git.
- **`datos-de-ejemplo/` es lo que se publica**: un caso **inventado**, con la misma forma que uno real,
  para que cualquiera pueda probar el motor **sin ver los datos de nadie**.

### Los nombres que importan

| que | donde |
|---|---|
| **el motor** (Rust) | `motor/` -- **158 pruebas, cero avisos** (`pruebas.rs` 139 · `medir-interfaz.rs` 8 · `recorrido.rs` 11) |
| **la documentacion** | `docs/` -- el caso, el flujo real, la investigacion, el plan |
| **los analisis medidos** | `analisis/` |
| **el informe generado** | `informe/ejemplo-generado.md` |
| **los bocetos de la cara** | `bocetos/` |
| **los temas** (claro y oscuro) | `motor/ui/temas.slint`, con `temas/contraste.py` midiendo el contraste |
| **datos reales** | `datos/` -- **ignorado, nunca se sube** |
| **datos inventados** | `datos-de-ejemplo/` -- esto si |

---

## La idea, en tres frases

1. **Un modelo de lenguaje no calcula: redacta.** Si se le pide una cuenta, la inventa con seguridad. Y
   aquí la cuenta afecta al cuerpo de una persona.
2. **El valor no está en el informe: está en saber qué se puede afirmar.** Un resultado al que le falta un
   dato **no se inventa**: se dice **qué falta** y **cuánto afecta**.
3. **Todo lo que se afirma, se puede rastrear hasta su fuente.** Cada número sale de una fórmula citada, y
   cada fórmula tiene su porqué escrito al lado.

## Las tres respuestas

El motor **nunca devuelve solo un número**. Devuelve una de estas tres cosas:

| respuesta | cuándo | qué se le dice al usuario |
|---|---|---|
| **SE PUEDE AFIRMAR** | están los datos que la afirmación necesita | la afirmación, con **su fuente** |
| **SE PUEDE, CON RESERVA** | falta algo que cambia el número, pero se puede acotar | la afirmación, **con su margen** y **qué falta** |
| **NO SE PUEDE** | falta algo sin lo cual no tiene sentido | **por qué**, y **qué dato haría falta** |

**No hay una cuarta respuesta. Inventar no es una respuesta.**

## Qué hay dentro

```
motor/                  el motor, en Rust, con sus pruebas
  src/datos.rs          el modelo: una persona, una sesión, un dato que PUEDE faltar
  src/formulas.rs       las fórmulas, cada una con su fuente al lado
  src/esfuerzo.rs       el salto de carga, según el estudio más grande que existe
  src/fuentes.rs        cuando dos aparatos dicen cosas distintas
  src/tendencia.rs      ¿mejora o solo entrena?
  src/juicio.rs         las tres respuestas
  src/adaptador.rs      el formato de fuera, traducido al nuestro
  tests/pruebas.rs      42 pruebas, en los dos sentidos
docs/                   el contexto, los fundamentos y las decisiones
analisis/               los análisis, repetibles
datos-de-ejemplo/       datos SINTÉTICOS, para poder probarlo sin datos de nadie
informe/                un informe de ejemplo
herramientas/           comprobaciones del entorno
```

## Cómo se prueba

```bash
cd motor
cargo test          # 42 pruebas
cargo run --bin demo        # las tres respuestas, con datos inventados
cargo run --bin caso        # un ciclista de 60 años, sintético
```

**Y no basta con que dé resultados bonitos.** Cada cálculo se prueba **en los dos sentidos**:

- **que la fórmula dé bien** — con un caso calculado a mano;
- **y que el motor DETECTE lo que falta** — quitando un dato a propósito.

**Un motor que no detecta lo que le falta tranquiliza de más**, y un falso verde no se nota por el
resultado.

## Sobre los datos: aquí no hay ninguno de nadie

::: aviso
**Este repositorio es público, y por eso NO contiene datos de salud de ninguna persona.**

Los datos con los que se desarrolló pertenecen a **una persona concreta**, que **no ha dado permiso para
publicarlos**. Y no son datos cualesquiera: **37 de sus actividades llevan ruta GPS**, o sea **dónde vive
y por dónde entrena**.

**Lo que sí está** es `datos-de-ejemplo/`: **un caso sintético**, inventado, con el perfil de un ciclista
de 60 años. Existe precisamente **para poder probar el motor sin datos de nadie** — y es lo que se usa en
las pruebas y en el informe de ejemplo.
:::

## De dónde salen los fundamentos

Cada regla de este motor está respaldada, y **las fuentes están en `docs/`**:

- **El formato de los datos**: el **FIT** de Garmin, con su SDK público.
- **Las zonas de intensidad**: el **método de Karvonen**, sobre la reserva cardíaca.
- **La carga de una sesión**: el **TRIMP de Banister**.
- **El salto que causa lesiones**: el estudio más grande que existe, **5.000 corredores en 87 países**,
  publicado en el *British Journal of Sports Medicine*. **Y su conclusión descarta la relación de carga
  aguda y crónica (ACWR)**, que es la que usan la mayoría de los relojes.
- **Y por qué la frecuencia máxima NO se estima**: un trabajo de 2025 que valida **siete ecuaciones**
  contra **634 personas** y encuentra que **ninguna sirve por encima de los 40 años**.

## Estado

**En construcción.** Es la fase del motor: **calcular, decidir y justificar**. Todavía **no hay interfaz
ni aplicación instalable**.

**Y este repositorio es, a propósito, solo la parte que se puede publicar**: el motor y su fundamento. Los
datos de la persona para la que se construyó **se quedan donde tienen que quedarse**.

## Licencia

**Por decidir.** Está pendiente elegir, y la elección importa: condiciona qué puede hacer otro con esto.
