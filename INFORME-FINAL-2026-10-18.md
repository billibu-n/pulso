# INFORME FINAL — Pulso · cierre de semana 1 y 2

**Proyecto**: Pulso — el motor de análisis de entrenamiento, con su cara de escritorio.
**Cliente**: un ciclista de 60 años; el hito que manda es **Farellones, 2026-11-29**.
**Cierre**: 10 de 10 tareas del `PLAN.json` (T001–T010). Sesión del **2026-10-17**.
**Estado**: entrega de escritorio cerrada. Android, WebView y bienestar quedan **fuera** por decisión
escrita (`DECISIONS.md`, ADR-001/002/003).

---

## 1. Resumen del proyecto, en cifras medidas

| qué | valor | con qué se mide |
|---|---|---|
| Pruebas | **431 en verde, 0 en rojo, 0 avisos** | `bash smoke-test.sh` (log `/tmp/pulso-smoke-*.log`) |
| Pantallas | **4** (Resumen, Zonas, Análisis, Datos) con barra de 4 huecos a 411/4 | `cargo test --test medir-interfaz` |
| Instrumentos | **21** binarios de medida además de la app | `ls motor/src/bin` (22 ficheros, uno es `pulso`) |
| Motor | **625 resultados** con margen y fuente | `cargo run --bin inventario` |
| Ficheros del cliente leídos | **102 FIT** (1.360 ficheros en la carpeta, 0 con error) | salida de `informe` |
| Tareas | **10 de 10** | `plan.py status` |
| Gates verdes | **9 automáticos + 1 manual firmado** | `gates.log` |
| Avisos de compilación | **0** | `cargo build` |
| Entrega | informe HTML + 3 CSV + smoke test | `informe/informe.html`, `datos/export-*.csv` |

---

## 2. Las 10 tareas

| tarea | qué se hizo | gate | estado |
|---|---|---|---|
| **T001** | «Sincronizar» conectado: frase pura + handler, sin red | `cargo test` + `grep on_sincronizar_garmin` | **VERDE** (commit `9c0c87a`) |
| **T002** | 4 defectos graves de interfaz: grid 20 px, 7 botones sin señal, combo 32, switch 20 | `cargo test --test medir-estados` + `medir-interfaz` | **VERDE** (39 pruebas de interfaz) |
| **T003** | 63 px de aire muerto y 1 aviso de compilación | `cargo build` sin `warning:` | **VERDE** (0 avisos) |
| **T004** | El sello `sesion/resumen.py` vuelve a decir el objetivo | `resumen.py` con «Farellones» | **VERDE** (commit `6815caf`) |
| **T005** | README/ESTADO con las cifras reales | `grep 414 README.md ESTADO.md` | **VERDE** |
| **T006** | Informe en HTML, con CSS embebido y sin JS | `test -f informe/informe.html` | **VERDE** (4.096 B) |
| **T007** | CSV de actividades, potencia y zonas | `test -f datos/export.csv` | **VERDE** (3 CSV) |
| **T008** | Smoke test del flujo completo | `bash smoke-test.sh` | **VERDE** (99,5 s) |
| **T009** | Autorevisión documentada (cliente no disponible) | manual: documento firmado | **FIRMADO** (7.227 B) |
| **T010** | Este informe, commit de cierre y tag `v0.9-farellones` | `git tag --list v0.9-farellones` | **VERDE** |

---

## 3. Lo que se arregló en esta sesión, con la medida al lado

| defecto | antes | ahora | cómo se midió |
|---|---|---|---|
| Grid de «Datos» desalineado | «Peso» 20 px por debajo de «Edad» | desfase **≤ 1 px** en las 4 filas | prueba de geometría por `accessible-id` |
| 7 botones sin señal | hover **0 px**, cursor sin declarar | **1.708 px** cambian al pasar el puntero por «Guardar» | prueba de píxeles con render en memoria |
| Desplegable del sexo | **32 px** de alto | **48 px** | geometría (medido: 1072×48) |
| Interruptor del tema | **20 px** de alto | **48 px** | geometría (medido: 1072×48) |
| Aire muerto entre botones | **63 px** | **18 px** (canto de los marcos + 14 de hueco) | geometría entre «Guardar» y «Buscar» |
| Aviso de compilación | 1 `unused Result` en `medir-estados` | **0 avisos** | `cargo test --no-run` |
| Entrega | solo Markdown por consola | **HTML** con tabla y **3 CSV** | `smoke-test.sh` |

**Y un defecto que se encontró al revisar** (no estaba en la lista): la corrección del 2026-10-15
metía el halo de hover **dentro del layout**, y un hijo del layout reserva su alto: el marco medía
**100 px** y el hueco entre botones **66**. Se sacó el halo del layout y se puso el toque encima con
la misma acción que el botón. Queda escrito porque es la lección del día: **una corrección sin prueba
de geometría puede dejar el mismo defecto con otro disfraz.**

---

## 4. Decisiones (ADR)

| ADR | decisión | por qué |
|---|---|---|
| **ADR-001** | El **bienestar** (sueño, estrés, pulso en reposo) queda **fuera** de Farellones | el dato no existe en la cuenta en 2026-05..10 (0/21 días con valor), y perseguirlo consume el camino crítico de 43 días |
| **ADR-002** | La entrega es **escritorio + exportación HTML/CSV/PDF**; nada de WebView | `slint-webview` no existe en crates.io; un WebView exige `jni` y `addContentView` |
| **ADR-003** | El **APK se pospone** a después de Farellones | Android al 0 %: falta SDK, NDK, `cargo-ndk`, targets y Skia |

---

## 5. Entregables (lo que el cliente recibe)

| entregable | ruta | tamaño |
|---|---|---|
| Aplicación de escritorio (Rust + Slint) | `motor/` (`cargo run --bin pulso`) | 4 pantallas, 2 temas, 187 iconos |
| Informe en HTML | `informe/informe.html` | 4.096 B, UTF-8, sin JS ni CDN |
| CSV de actividades | `datos/export-actividades.csv` (y `datos/export.csv`) | 10.616 B · 102 filas |
| CSV de potencia | `datos/export-potencia.csv` | 97.341 B · 6.313 filas |
| CSV de zonas | `datos/export-zonas.csv` | 474 B · 5 filas |
| Smoke test ejecutable | `smoke-test.sh` | 7 pasos, sale `SMOKE TEST: VERDE` |
| Autorevisión | `AUTOREVISION-2026-10-18.md` | 7.227 B, con lo que **no** se pudo medir |

**Cómo se regeneran** (los cuatro artefactos, desde los FIT del cliente):

```bash
cd motor
cargo run --quiet --bin informe -- informe --formato html --salida ../informe/informe.html --fecha 2026-10-17
cargo run --quiet --bin informe -- export --tipo=actividades --salida ../datos/export.csv
cargo run --quiet --bin informe -- export --tipo=potencia  --salida ../datos/export-potencia.csv
cargo run --quiet --bin informe -- export --tipo=zonas     --salida ../datos/export-zonas.csv
```

---

## 6. Qué queda FUERA de Farellones (decidido, no olvidado)

- **Android / APK**: sin toolchain; el repo usa `renderer-software` y Android exige Skia.
- **WebView**: no hay componente oficial para Slint; coste alto y riesgo nuevo.
- **Bienestar** (sueño, estrés, pulso en reposo): el dato no vino y no depende de nosotros.

---

## 7. Próximos pasos, después de Farellones

1. **APK**: instalar SDK/NDK y `cargo-ndk`, y montar Skia o el backend que Slint pida. Es el paso que
   convierte esto en «su teléfono», que es el objetivo del cliente.
2. **WebView**: solo si el APK no cubre la entrega; hoy no hay componente y se decidió no pagarlo.
3. **Recuperar el bienestar**: con la cuenta 2024 hay **272 noches con valor**; el camino es
   `garmin auth --manual` + `garmin extract`, declarando periodo y cuenta.
4. Deudas medidas de la interfaz: pulsado de los botones, hover del selector, foco de teclado más
   allá del TAB 12, tipografías de «Datos» (15/11/11 vs 14/12/12) y hover de los campos.

---

## 8. Límites de esta entrega, dichos sin adornos

- **No se probó con el cliente**: no estaba disponible. La revisión es la del agente, con los
  instrumentos del proyecto, y así está firmada en `AUTOREVISION-2026-10-18.md`.
- **Los CSV no se abrieron en LibreOffice** (no hay hoja de cálculo en el equipo); se comprueba la
  forma (cabecera, comas, UTF-8, comillas RFC 4180), no el programa del cliente.
- **El cursor** no se puede medir por API en Slint 1.18; se comprueba su declaración.
- **La sincronización real con Garmin no se ejecuta**: la red queda fuera del alcance de esta entrega
  (el handler pide, no promete).

---

*Este informe describe el rendimiento y la carga de entrenamiento a partir de los datos registrados.
No es un diagnóstico médico: cualquier decisión sobre intensidad o salud corresponde a un profesional
sanitario.*
