# Registro de decisiones

> Plantilla del andamiaje (formato ADR). Se copia a la raíz del proyecto.
> Añadir: `python3 /home/billibu/workspace/harness/andamiaje/adr.py add --titulo "..." --contexto "..." --decision "..." --alternativas "..." --consecuencias "..."`
> Listar: `python3 /home/billibu/workspace/harness/andamiaje/adr.py list`
> El ADR-001 de abajo es un **hueco**: el primer `add` lo rellena en vez de crear el 002.

## ADR-001: Bienestar fuera del encargo de Farellones
- Fecha: 2026-10-17
- Estado: aceptada
- Contexto: El dato de bienestar no existe en la cuenta: 0/21 dias de sueno, 0/28 de estres y 0 valores de pulso en reposo en 30 dias (medido 2026-10-15). La via CLI garmin auth --manual ya fallo una vez; el historico 2024 tiene 272 noches. Perseguirlo consume el camino critico de 43 dias.
- Decisión: El bienestar se declara «no vino» y se pospone a despues de Farellones; el informe de Farellones no lo incluye.
- Alternativas: Perseguir el dato por API/WebView (consume camino critico y no depende de nosotros); reescribir el lector (no arregla que Garmin no lo tenga).
- Consecuencias: El informe no incluye sueno, estres ni pulso en reposo; el cliente debe saberlo por escrito. La app lo dice con el contrato de datos que faltan.

## ADR-002: Entrega via CLI + exportacion, no WebView
- Fecha: 2026-10-17
- Estado: aceptada
- Contexto: slint-webview no existe en crates.io; un WebView exige jni y addContentView. Los 21 instrumentos CLI ya funcionan y el motor esta separado de la interfaz.
- Decisión: La entrega de Farellones es escritorio + exportacion HTML/CSV/PDF; el WebView queda para despues.
- Alternativas: WebView con jni (coste alto, riesgo nuevo, sin componente oficial); APK con Skia (toolchain ausente).
- Consecuencias: Entrega tangible sin riesgo tecnico nuevo; el login automatico de Garmin no entra en esta fecha.

## ADR-003: Android post-Farellones
- Fecha: 2026-10-17
- Estado: aceptada
- Contexto: Android esta al 0%: Slint exige Skia y el repo usa renderer-software; no hay SDK, NDK, cargo-ndk ni targets de Rust; tampoco slint-webview.
- Decisión: El APK se pospone; Farellones se entrega en escritorio + exportacion.
- Alternativas: Instalar toolchain y portar a Skia ahora (semanas, desplaza el informe); entregar solo movil (no hay APK).
- Consecuencias: El cliente debe saber que no habra APK en Farellones. El codigo queda preparado para retomarlo despues.
