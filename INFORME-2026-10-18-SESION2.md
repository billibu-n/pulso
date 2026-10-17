# INFORME — Sesion 2 (2026-10-17)

> El encargo pedia `INFORME-2026-10-18-SESION2.md`; la sesion se ejecuto el **2026-10-17**
> (America/Santiago). Manda la fecha real; el nombre se respeta.

**Ruta**: `/home/billibu/projects/pulso` · rama `main` · base `6c825dc`

---

## 1. T004 — el sello vuelve a hablar

### 1.1 Diagnostico (3 causas, no una)

| # | causa | evidencia |
|---|---|---|
| 1 | **Precedencia invertida**: `sello()` leia primero `datos/trabajo-ui/TASKS.json` (el encargo de un chat viejo) y solo despues la raiz | `resumen.py:66` (antes): `... trabajo-ui/TASKS.json") or ... "TASKS.json")` |
| 2 | **Claves distintas**: esperaba `meta_del_proyecto` y `descripcion`/`title`; el TASKS.json del andamiaje usa `encargo` y `que` | `resumen.py:73` y `:51` |
| 3 | **Sin tareas**: el TASKS.json nuevo lleva `tareas: []` (las tareas viven en `PLAN.json`), asi que el sello decia «0 hechas · 0 en curso · 7 pendientes» del fichero viejo | salida antes del arreglo |

### 1.2 Arreglo (todo en `sesion/resumen.py`, nada del workspace)

1. Manda el **`TASKS.json` de la raiz**; el de `datos/trabajo-ui/` queda como respaldo.
2. Acepta **`meta_del_proyecto` o `encargo`**.
3. Acepta **`descripcion` o `title` o `que`**, y estados `in-progress`/`in_`.
4. Si el TASKS.json no trae tareas, **lee las del `PLAN.json`** (id, title, status).
5. Si no hay objetivo, lo dice con una frase clara: no deja el hueco en blanco.

### 1.3 Evidencia

| que | resultado |
|---|---|
| sello | imprime el objetivo con «Farellones», `rama: main`, **1 hecha · 9 pendientes**, cierre del chat y comandos |
| gate T004 | **VERDE**, 0,21 s |
| limite | 32 lineas (tope duro: 70) |

## 2. T005 — README y ESTADO con las cifras reales

| documento | antes | despues |
|---|---|---|
| `README.md` | «189 pruebas», «42 pruebas» (x2), «no hay interfaz ni aplicación instalable» | **414 pruebas**, desglose real (161+24+36+13+1+179), **4 pantallas**, **2 temas**, **187 iconos**, **21 instrumentos** |
| `README.md` | sin alcance | seccion nueva: dentro (actividades, potencia, zonas, tendencia, informe) / fuera (bienestar, Android, WebView, con ADR-001/002/003) |
| `ESTADO.md` | Ronda 1 | **§9 Ronda 2**: T004 y T005 cerradas, 414 pruebas, siguiente T002/T003 |
| `PLAN.json` | gate de T005 miraba 409 | mira **414** |

| que | resultado |
|---|---|
| gate T005 | **VERDE**, 0,04 s |
| `grep 414` | README: 2 · ESTADO: 2 |

## 3. Verificacion de la sesion

| que | comando | resultado |
|---|---|---|
| suite | `cd motor && cargo test` | **414 pasan, 0 fallan** |
| build | `cargo build` | **0 avisos** (el aviso conocido sigue solo en el target de pruebas) |
| gates | `gates.log` | T004 VERDE + T005 VERDE |

## 4. Commits

| hash | mensaje |
|---|---|
| `6c825dc` | `pulso: mapeo y code-map (cierre de sesion 0)` |
| *(el de esta sesion)* | `pulso: T004 sello + T005 docs (Ronda 2)` |

Sin push: `main` queda por delante de `origin/main` (no se empuja sin autorizacion).

## 5. Siguiente

**T002** (4 defectos graves de UI: grid 20 px, 7 botones sin hover, combo 32 px, switch 20 px) y
**T003** (63 px fantasma + aviso de compilacion), en paralelo. T006/T007 dependen de T001 y van
despues.

## 6. Limpieza

`cargo clean` **no se ejecuto**: el encargo lo condicionaba a autorizacion y no la hubo.
`motor/target/` sigue en **9,7 GB** (ignorado por git). Espacio liberado: **0**.
