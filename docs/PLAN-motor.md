# El motor: qué es, y en qué orden se construye

**Decidido el 2026-10-13** · el dueño: *«hacer funcionar el motor y luego instalarlo o darle la
accesibilidad al usuario y la interfaz»*.

---

## 1. Qué es «el motor», exactamente

**El motor es el programa que piensa.** Recibe datos, calcula, decide **qué se puede afirmar**, y
**produce un resultado justificado**. **No dibuja nada.**

Esa separación no es una manía de arquitectura: es lo que permite **construirlo y probarlo hoy**, sin
Slint, sin Android y **sin la interfaz**. Y es lo que hace que, cuando llegue la interfaz, **lo difícil
ya esté resuelto y probado**.

| pieza | qué hace | ¿dibuja? |
|---|---|---|
| **el motor** | leer, calcular, decidir, justificar | **no** |
| **el instalable** | empaquetarlo para que corra en el teléfono | no |
| **la interfaz** | enseñarlo y dejar tocarlo | **sí** |

## 2. El orden, y por qué ese

```
MOTOR  →  INSTALABLE  →  INTERFAZ
```

**Por qué el motor primero**: porque es **donde está el valor y donde está el riesgo**. Lo demás —una
ventana, un botón, un gráfico— es trabajo conocido y sin sorpresas. **Lo que puede salir mal de verdad
es el cálculo y la decisión**, y eso se prueba **sin dibujar nada**.

**Y por qué el instalable ANTES de la interfaz**: porque **empaquetar para Android es donde aparecen las
sorpresas de plataforma** (el NDK, los objetivos de Rust, los permisos). Es mejor descubrirlo con **un
motor que ya funciona** que con **una interfaz a medio hacer**.

## 3. Las piezas del motor

| pieza | qué resuelve | de dónde sale |
|---|---|---|
| **el modelo** | una forma **nuestra** de representar la sesión y la persona | decisión nuestra, no del cliente |
| **el lector** | traducir **su** formato a nuestro modelo | **un adaptador por formato**; el `.json` del cliente es **uno** |
| **el cálculo** | zonas, carga, tendencia | fórmulas **citadas** (Karvonen, TRIMP, ACSM, OMS) |
| **el juez de datos** | **qué falta, y cuánto afecta** | **la pieza que nadie hace y aquí es el centro** |
| **el diagnóstico** | qué se puede afirmar, **con su fundamento** | cada afirmación, **con su fuente** |

**La pieza cuarta es la importante.** Las otras las tiene cualquier aplicación de deporte. **Decidir qué
hacer cuando falta un dato** es lo que pidió el cliente sin saber que lo pedía: *«un resultado al que le
falta data puede arrojar márgenes de error no menores»*.

## 4. Las tres respuestas honestas cuando falta un dato

No hay una cuarta. **Inventarlo no es una respuesta.**

| estado | cuándo | qué se le dice al usuario |
|---|---|---|
| **Se puede** | están los datos que la afirmación necesita | la afirmación, con **su fundamento** |
| **Se puede, con reserva** | falta algo que **cambia el número**, pero se puede acotar | la afirmación, **con su margen** y **qué falta** |
| **No se puede** | falta algo **sin lo que la afirmación no tiene sentido** | **«no se puede, y por esto»** — y **qué dato haría falta** |

**Por qué esto es una regla y no una cortesía**: un plan de entrenamiento **afecta al cuerpo de una
persona**. Una afirmación sin fundamento, en este terreno, **no es un error de cálculo: es un riesgo**.

## 5. Lo que hace falta para empezar HOY (comprobado)

| qué | estado |
|---|---|
| **Rust y cargo** | **instalados** (1.98.1) |
| **las fórmulas** | **citadas** en `contexto.md` |
| **los datos de prueba** | **se construyen**, con criterio y a mano |
| **el `.json` del cliente** | **NO hace falta para empezar**: entra después, **con un adaptador** |

**O sea: el motor se puede empezar ya, y no depende de nadie.**

## 6. Lo que NO se puede hacer sin el `.json`

- **El adaptador** para su formato concreto. Pero **el motor no cambia** por eso: cambia **una pieza de
  las cinco**.
- **Saber qué datos tendrá de verdad** — y por tanto **afinar qué es obligatorio y qué es opcional**.
  Mientras, se decide **con criterio fisiológico**, y se corrige cuando llegue.

## 7. Cómo se prueba el motor (y esto es lo que lo hace fiable)

**Con `cargo test`**, que es la herramienta de pruebas de Rust. Y **no basta con que dé resultados
bonitos**. Cada cálculo se prueba **en los dos sentidos**:

| qué se prueba | cómo |
|---|---|
| la **fórmula** | con un caso **conocido a mano**: si el resultado no cuadra, la fórmula está mal |
| el **juez de datos** | **quitando un dato a propósito** y comprobando que **lo detecta y lo dice** |
| los **límites** | un pulso de 300, una duración de cero, una fecha absurda: **no debe reventar ni inventar** |
| el **fundamento** | que cada afirmación **traiga su fuente**, y que **no haya ninguna sin ella** |

**La tercera fila es la que casi nadie hace y la que evita el ridículo**: un motor que divide por cero
**no avisa, da un número**.

## 8. Lo que este motor NO va a hacer

- **No va a diagnosticar enfermedades.** Da **estado de forma y carga de entrenamiento**, no medicina.
- **No va a inventar un dato que falta.** Nunca. Es la regla que lo define.
- **No va a decir «entrena más» sin decir por qué.** Toda afirmación lleva **su fundamento**.
- **No va a mandar datos fuera sin decirlo.** Si algún día usa un modelo externo, **el usuario lo sabrá
  antes**, y sabrá **qué sale**.

## 9. Lo que hay que decidir antes de la interfaz

| decisión | cuándo hace falta |
|---|---|
| **la licencia** (las tres de Slint) | **antes de publicar**, no antes de programar |
| **qué datos son obligatorios y cuáles opcionales** | **cuando llegue el `.json`** |
| **si los datos de salud salen del dispositivo** | **antes de la primera versión que use un modelo externo** |
| **la unidad de medida** (pulsaciones, vatios, ritmo) | al construir el modelo |
