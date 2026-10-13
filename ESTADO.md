# ESTADO: dónde estamos y por dónde se sigue

**Escrito el 2026-10-13**, porque el contexto se estaba llenando y **una compactación a mitad de tarea
es catastrófica**. Este fichero existe para que **quien retome no tenga que reconstruir nada**: dice qué
está hecho, qué está medido, **qué falta**, y **dónde está cada cosa**.

> **Si acabas de llegar: lee esto entero antes de tocar nada.** Y después
> `docs/RECORRIDO-del-usuario.md`, que es el plan.

---

## 1. Qué es este proyecto, en tres frases

**Un motor que lee datos de entrenamiento, calcula, y dice qué se puede afirmar — y qué no.**
El cliente es **un ciclista de 60 años** que hasta ahora mandaba capturas a un modelo de lenguaje.
**La aplicación es el motor con una cara**, en **Rust + Slint**, y tiene que llegar a **su teléfono**.

## 2. Qué está HECHO y VERIFICADO

| qué | estado | dónde |
|---|---|---|
| **el motor** | **86 pruebas en verde**, cero avisos | `motor/src/` |
| **lo que calcula** | **625 resultados** con su margen y su fuente | `motor/src/bin/inventario.rs` |
| **el informe** | por plantillas, reproducible, con guarda anti-invención | `motor/src/informe.rs` |
| **la interfaz** | existe y **se MIDE**: 411x880, cabecera 56, nav 64, huecos repartidos | `motor/ui/` |
| **los dos temas** | claro cálido y **negro OLED**, con contraste medido | `motor/ui/temas.slint` + `temas/contraste.py` |
| **los iconos** | **187** de Lucide, licencia ISC respetada | `motor/ui/iconos.slint` + `herramientas/iconos-a-slint.py` |
| **Android** | **comprobado**: binario ARM aarch64 para Android | ver `docs/SLINT-…`, §10.6 |
| **la norma** | **82 reglas**, 15 patrones de fallo | `~/.dsh/skills/norma-app/SKILL.md` |

## 3. Qué FALTA, en orden, y por qué ese orden

**Los ocho pasos del recorrido del usuario, medidos: uno a medias y siete que no cumplen.**
El detalle está en `docs/RECORRIDO-del-usuario.md`. El resumen:

| # | el paso | qué falta |
|---|---|---|
| **1** | **primero: la decisión de los controles** | los widgets de serie **no aceptan nuestros colores** (§11.1 del doc de Slint). O se aceptan, o se construyen. **Es decisión del dueño y bloquea lo demás.** |
| **2** | una **puerta de entrada única** | hoy hay 13 binarios de desarrollo y ninguno se llama «la aplicación» |
| **3** | **los datos de la persona, editables** | hoy están **escritos en el código**: los resultados **no son del cliente** |
| **4** | **el primer arranque sin datos** | hoy dice `169±3` **y** «sin datos» a la vez: **se contradice** |
| **5** | **pulsar para analizar** | hoy calcula al abrir, y **tarda 5,4 s sin decir nada** |
| **6** | **vincular la cuenta** | no existe. Y necesita **credenciales de Garmin** |
| **7** | **recordar entre arranques** | hoy relee 102 ficheros cada vez |

## 4. Dónde está cada cosa (para no buscarla)

```
~/projects/pulso/                    ← ESTE ES EL REPOSITORIO. Público, sin datos de salud.
├── motor/                el motor (Rust) y la interfaz (Slint)
│   ├── src/              14 módulos: datos, rango, formulas_v2, umbral, patron, hrv,
│   │                     contexto, informe, fit, tendencia, juicio, fuentes, adaptador
│   ├── src/bin/          inventario (cuenta los 625), informe, ventana, y otros
│   ├── tests/pruebas.rs  86 pruebas · tests/medir-interfaz.rs  6 pruebas de MEDICION
│   └── ui/               temas.slint · armazon.slint · ventana.slint · iconos.slint · raiz.slint
├── docs/                10 documentos, y LAS FUENTES en docs/slint-fuentes/ (32 páginas)
├── herramientas/        traer-slint.sh · html-a-texto.py · iconos-a-slint.py
├── datos/               1003 MB DEL CLIENTE — **ignorado por git, NUNCA se sube**
├── datos-de-ejemplo/    caso inventado, esto sí se publica
└── temas/contraste.py   mide el contraste WCAG de los dos temas
```

**Y los otros dos repositorios:**

- `~/projects/mimo-nuevo` — la norma DEL PROYECTO, `PATRONES-DE-FALLO.md` (15), `TASKS.json` (86).
- `~` — la norma universal, en `.dsh/skills/norma-app/SKILL.md`.

## 5. Las trampas ya conocidas, para no volver a caer

**De Slint** (todas comprobadas — el detalle en `docs/SLINT-lo-que-hay-que-saber.md`):

1. **No se puede cambiar de estado desde Rust.** Se gobierna con **una propiedad** y condiciones `when`.
2. **`private` es el default de una propiedad**: sin `in` explícito, **el setter no existe y el
   compilador no avisa**.
3. **Dentro de un layout, `x` e `y` no se ponen**: lo coloca el layout. Ponerlo a un hijo y a otro no
   **cambia el tamaño del padre**.
4. **Los widgets de serie no aceptan nuestros colores** — y hay dos temas propios.
5. **`visible: false` no libera el hueco.**
6. **`Text {};` con punto y coma no compila.**
7. **`to-float()` usa el separador del IDIOMA**: `"1.5"` da **0** en un equipo alemán.
8. **`Timer` viene encendido** (`running: true`): consume batería si no se apaga.
9. **`ScrollView` no se desplaza con el dedo** sin `mouse-drag-pan-enabled: true`.
10. **Un solo punto de entrada** en `build.rs`, o el segundo fichero pisa al primero.
11. **Para medir hace falta información de depuración** (`with_debug_info(true)`).

**Del método** (y son las que más costaron):

12. **Un modo que solo comprueba NO puede abrir ventana.** Si la comprobación se ve, no se puede hacer
    en cualquier momento — y **aparece en la pantalla del dueño**.
13. **`pkill` con un patrón que aparece en su propia orden se mata a sí mismo.** Matar **por PID**.
14. **`/tmp` es una partición pequeña aquí**: llenarla rompe compilaciones con un error que **no es del
    código**.
15. **Los binarios de depuración ocupan cientos de MB**: «Finished in 0.2s» puede significar **que no
    recompiló**. Forzar con `touch`.

## 6. Cómo se comprueba que algo funciona (el método, no la fe)

```bash
cd ~/projects/pulso/motor
cargo test                              # 86 pruebas del motor
cargo test --test medir-interfaz        # 6 pruebas de GEOMETRIA de la interfaz
cargo run -q --bin inventario -- ../datos/garmin_files/storage   # cuenta los 625 resultados
cargo run -q --bin ventana -- ../datos/garmin_files/storage --texto   # lo que enseña, SIN ABRIR
```

**Y dos reglas del método, que están en la norma (AU-63 y AU-70):**

- **lo visual se MIDE, no se mira** — y si no se puede medir, **eso es la tarea**;
- **nunca abrir la ventana en la pantalla del dueño** para comprobar algo. Se mide.

## 7. Lo que NO está resuelto, dicho claro

- **La decisión de los controles** (paso 1): **bloquea el resto de la interfaz.**
- **Los datos de la persona están en el código**: **los resultados no son del cliente todavía.**
- **Cuatro páginas de Slint sin traer**: tipos de propiedad, **propiedades y bindings** (el hueco más
  grave: no se sabe qué pasa al asignar a mano a un binding), operadores y pureza.
- **13 iconos de Simple Icons** sin convertir: son de relleno, no de trazo.
- **Nada probado en un segundo proyecto**: las 82 reglas **solo se han usado en uno**.
- **`~` es un repositorio local** con 32 ficheros de caché sin decidir.
