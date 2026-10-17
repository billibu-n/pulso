# Criterios de aceptación por tarea

> `gate.py` corre el `Comando` con el `Tiempo máximo` de aquí y devuelve VERDE/ROJO.
> Si una tarea no tiene bloque aquí, `gate.py` usa el campo `acceptance` de `PLAN.json`.

## Reglas del criterio

- Un criterio tiene que **poder fallar**: `true` siempre da verde y no demuestra nada.
- El comando se escribe al planificar la tarea, no al terminarla.
- Si la tarea no se puede automatizar, el comando empieza por `manual:` y la cierra una persona.
