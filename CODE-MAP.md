# Mapa del código

> Base: `git` en `ecfac21` (2026-10-15 17:37), árbol limpio. Generado con
> `harness/andamiaje/code_map.py` el **2026-10-17** y **anotado a mano** (la columna de
> responsabilidad no la inventa el generador). 1.742 ficheros contados, de los cuales 1.521 son
> datos del cliente en `datos/` (ignorados por git).
>
> **Límite del generador, declarado**: `code_map.py` no lee `Cargo.toml`, así que no detectó
> puntos de entrada ni dependencias. Las dos secciones están completadas a mano. Su detector de
> deuda también confunde el «TODO» español con una marca: las 13 «marcas» que listaba son prosa.

## Estructura

- `/motor`: el programa. Rust (55 `.rs`) + Slint (8 `.slint`). 31.924 líneas contadas a mano
  (11.556 librería · 9.677 binarios · 8.154 pruebas · 2.537 interfaz).
- `/motor/src`: 26 módulos. El centro (datos, fórmulas, juicio) no conoce ficheros ni pantalla;
  los adaptadores (`fit`, `huawei`, `adaptador`) traducen; `cara` decide qué se enseña.
- `/motor/src/bin`: 22 binarios. Uno es la aplicación (`pulso`); los otros 21 son instrumentos de
  medida. `compartido/pintar.rs` no es binario: lo incluyen la app, la captura y la prueba de geometría.
- `/motor/ui`: la cara. `ventana.slint` (~1.113 líneas) + `armazon.slint` (~589) + iconos, temas,
  selector de carpeta y dos piezas de experimento (`sonda-borde`, `prueba-widgets`).
- `/docs`: 30 documentos y las fuentes (32 HTML + 32 TXT en `slint-fuentes/`, 4 en `fuentes/`).
  Contratos, recorrido, encargo, investigación y estado del arte. 4,4 MB.
- `/herramientas`: 11 instrumentos de `garmin-lab/` (sonda, portal, sesión, datos…), el verificador
  de afirmaciones, el conversor de iconos y el script de LaTeX. 172 KB.
- `/temas`: dos medidores de contraste WCAG (`contraste.py`, `contraste-widgets.py`). Salen con
  error si bajan del mínimo.
- `/sesion`: `resumen.py`, el sello de mano del proyecto (tope duro de 70 líneas).
- `/datos`: **1,1 GB del cliente**, 1.521 ficheros (1.271 JSON, 102 FIT, 68 PNG), **ignorado por
  git** (`.gitignore:11-24`). Incluye `garmin_data.db` (986 MB), el volcado de `/gc-api/` y las capturas.
- `/datos-de-ejemplo`: caso sintético (1 JSON + 1 generador). Esto sí se publica.
- `/analisis`: 3 piezas de exploración temprana (Python) y sus patrones medidos. 20 KB.
- `/bocetos`: 14 PNG y 2 generadores (Python). Material de diseño, no producción.
- `/informe`: un informe de ejemplo en Markdown y PDF + 3 gráficos. 456 KB.
- `/ver-app.sh`: captura las cuatro secciones sin abrir ventana (usa `capturar-ui --seccion`).

## Puntos de entrada

| entrada | qué es |
|---|---|
| `cargo run` (o `cargo run --bin pulso`) | **la aplicación**. `default-run = "pulso"` en `Cargo.toml`. Abre la ventana Slint. |
| `pulso <carpeta> --texto` | la misma decisión de qué enseñar, impresa sin abrir ventana. |
| `pulso --elegir <carpeta>` | imprime la vista del selector de carpeta, sin ventana. |
| `pulso <carpeta> --huawei <zip>` | añade la composición corporal de la exportación de Huawei. |
| `pulso --ayuda` | uso y salida 0, **antes** de crear la interfaz. |
| `python3 sesion/resumen.py` | sello de mano para un chat nuevo (objetivo, tareas, cierre). |
| `python3 herramientas/verificar-afirmacion.py {api,volcado,base,pruebas,todo}` | mide una afirmación sobre datos en su capa. |
| `./ver-app.sh` | captura las 4 secciones en PNG (fuera de pantalla). |

Instrumentos (21, `motor/src/bin/`): `inventario` (cuenta los 625 resultados), `biomecanica`,
`potencia`, `avisos`, `que-traen`, `pulso-reposo`, `volcar-campos`, `medir-carga` (memoria),
`medir-huawei`, `capturar-ui`, `auditar-estados`, `sincronizacion` (transporte Garmin),
`leer_fit`, `probar_fit`, `sin_datos`, `caso`, `demo`, `hrv_real`, `patrones`, `tendencia`,
`informe`.

## Módulos y dependencias

### Capas (de dentro hacia fuera)

| capa | módulos | regla |
|---|---|---|
| **centro** | `datos`, `rango`, `juicio`, `fuentes`, `formulas`, `formulas_v2`, `umbral`, `esfuerzo`, `patron`, `hrv`, `tendencia`, `potencia`, `avisos` | calculan y deciden; **no** saben qué es un FIT, un JSON o una pantalla |
| **adaptadores** | `fit`, `huawei`, `adaptador` | traducen y no deciden; un dato ausente es `Dato::Falta`, **nunca 0** |
| **almacén** | `memoria`, `analisis`, `profile`, `carpeta` | recuerdan y nombran el análisis; perfil junto al programa |
| **decisión** | `cara` | función pura `lo_que_ensena(&Persona, Option<&Real>) -> Ensenado` |
| **borde** | `motor/ui/*.slint`, `bin/pulso.rs`, `bin/compartido/pintar.rs` | pinta y une; **no calcula ni decide** |
| **sincronización** | `sincronizacion` (transporte, 51 KB), `sincronizar` (tipos y funciones puras, 75 KB) | aislada tras contrato; la red es del instrumento, no del motor |

### Dependencias de `Cargo.toml`

| crate | para qué | nota |
|---|---|---|
| `serde` + `serde_json` | leer el JSON del perfil y los cuerpos de la API | versión fijada a propósito |
| `fitparser 0.11` | leer FIT de Garmin | la librería más viva de las tres comparadas |
| `slint 1.x` (`renderer-software`) | la interfaz | **en Android Slint exige Skia**: tal como está, no compila para Android |
| `zip =8.6.0` (solo DEFLATE) | abrir la exportación de Huawei sin descomprimir | versión exacta, features mínimas |
| `ureq 3.4.2` (+ cookies) | transporte del instrumento Garmin | HTTP/1.1 puro; contra `/gc-api/` da 403 (medido) |
| `i-slint-backend-testing` (dev) | medir geometría sin abrir ventana | solo pruebas |

No hay base de datos en el programa. Persistencia: `perfil.json` (datos de la persona) y
`analisis.json` (memoria incremental, ~40 KB), ambos junto al ejecutable o, si no se puede
escribir, en la carpeta de configuración del usuario **diciendo dónde**.

## Flujo

```
carpeta de FIT ──fit.rs──┐
perfil.json ──profile───┼──> analisis::el_analisis ──> memoria (solo fichas nuevas)
export Huawei ─huawei───┘             │
                                      v
                         cara::lo_que_ensena (pura)
                                      │
                                      v
                     Slint (4 secciones) / --texto / captura PNG
```

## Tests

`cargo test` el 2026-10-17: **409 pasan, 0 fallan, 0 ignorados, 1 aviso de compilación**.

| dónde | pruebas | qué cubren |
|---|---:|---|
| `tests/pruebas.rs` | 161 | el cálculo y el juicio, sin tocar ficheros de fuera |
| `tests/recorrido.rs` | 24 | el recorrido del usuario sin abrir ventana |
| `tests/medir-interfaz.rs` | 36 | geometría, 48 dp, márgenes, controles |
| `tests/huawei.rs` | 13 | la exportación de Huawei |
| `tests/medir-estados.rs` | 1 | hover/pulsado por píxeles (el aviso sale de aquí) |
| inline en `src/` y `bin/` | 174 | 62 de librería, 104 de `sincronizacion`, 4 de `pulso`, 3 de `medir-huawei`, 1 de `capturar-ui` |

No hay medición de cobertura. El número que manda es el de `cargo test`, no el de un `grep`:
`verificar-afirmacion.py pruebas` cuenta 235 porque solo mira `tests/` y no ve las 174 inline.

## Deuda conocida

- **Marcas `TODO/FIXME/HACK/XXX` reales en el código: 0.** Las 19 coincidencias de `grep` son la
  palabra española «todo» en comentarios. La deuda vive en `docs/` y `ESTADO.md`, no en el código.
- **Dos módulos de fórmulas vivos**: `formulas.rs` lo usan `juicio`/`esfuerzo`; `formulas_v2.rs` lo
  usan `inventario` y `sin_datos`. Dos implementaciones del mismo cálculo es deuda (AU-73).
- **Un aviso de compilación** en `tests/medir-estados.rs` (`unused Result`). El estado dice «0 avisos».
- **Documentación atrasada**: `README.md` dice «42 pruebas», «189 pruebas» y «todavía no hay
  interfaz»; el árbol tiene 409 pruebas y una interfaz medida.
- **El generador de este mapa no entiende Cargo**: puntos de entrada y dependencias son manuales.
