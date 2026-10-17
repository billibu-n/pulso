# Criterios de aceptación por tarea

> `gate.py` corre el `Comando` con el `Tiempo máximo` de aquí y devuelve VERDE/ROJO.
> Si una tarea no tiene bloque aquí, `gate.py` usa el campo `acceptance` de `PLAN.json`.

## Reglas del criterio

- Un criterio tiene que **poder fallar**: `true` siempre da verde y no demuestra nada.
- El comando se escribe al planificar la tarea, no al terminarla.
- Si la tarea no se puede automatizar, el comando empieza por `manual:` y la cierra una persona.

## T001: Conectar Sincronizar (cuantas + frase pura + handler)
- Comando: `cd motor && cargo test --quiet >/dev/null 2>&1 && grep -q on_sincronizar_garmin src/bin/pulso.rs`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T002: Arreglar 4 defectos graves de interfaz (grid, botones, combo, switch)
- Comando: `cd motor && cargo test --test medir-estados --quiet && cargo test --test medir-interfaz --quiet`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T003: Arreglar 63 px fantasma y 1 aviso de compilacion
- Comando: `bash -c 'cd motor && ! cargo build 2>&1 | grep -q "warning:"'`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T004: Arreglar el sello sesion/resumen.py
- Comando: `python3 sesion/resumen.py | grep -q Farellones && ! python3 sesion/resumen.py | grep -q '(sin meta declarada)'`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T005: Actualizar README/ESTADO con las cifras reales
- Comando: `grep -q 409 README.md && grep -q 409 ESTADO.md`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T006: Exportacion HTML del informe
- Comando: `test -f informe/informe.html`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T007: Exportacion CSV de actividades/potencia/zonas
- Comando: `test -f datos/export.csv`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T008: Smoke test del flujo completo
- Comando: `bash smoke-test.sh`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T009: Pruebas con el cliente o autorevision
- Comando: `manual: feedback del cliente o checklist de autorevision firmado`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300

## T010: Informe final y commit de cierre
- Comando: `git tag --list v0.9-farellones | grep -q .`
- Resultado esperado: comando con exit 0
- Tiempo máximo: 300
