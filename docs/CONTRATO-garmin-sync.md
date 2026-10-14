# CONTRATO — Sincronizar la cuenta Garmin (con su plan B)

**Escrito el 2026-10-14.** Decisión del dueño: el cliente **no exporta a mano**; entra con **su cuenta**
dentro de la aplicación, una vez, y desde entonces la aplicación trae los datos sola. Este contrato se
escribe **ANTES** de tocar código (`arq:150-163`). Aprobado en conversación con el dueño el 2026-10-14.

## 1. El objetivo, en una frase

El cliente conecta SU cuenta de Garmin dentro de la aplicación; la aplicación trae las actividades
nuevas (FIT) y los diarios, los suma a lo que ya tiene, y **dice de dónde sale cada dato**.

## 2. El contrato (5 líneas)

- **RECIBE**: una vía de datos — (a) la sesión iniciada con las credenciales de Garmin **en su
  equipo**, o (b) un ZIP de la **exportación oficial** de Garmin, o (c) una **carpeta con FIT**
  (reloj por USB o carpeta elegida).
- **DEVUELVE**: las actividades nuevas (FIT) y los diarios (sueño, reposo, estrés, training status)
  con su fecha, cada uno **marcado con su fuente** (`api-interna` / `exportacion-oficial` /
  `carpeta`).
- **NO HACE**: no sube nada a ningún servidor; no guarda la **contraseña** (solo la sesión, en el
  almacén local del cliente, o nada); no toca el motor de cálculo; no rellena huecos: un diario que
  viene vacío se dice que falta.

## 3. Las vías, en orden, y el plan B (el SOS)

| vía | qué es | riesgo | cuándo se usa |
|---|---|---|---|
| **1 · API interna** (principal) | login con las credenciales del cliente contra la API interna de Garmin (la que usa Connect; las herramientas comunitarias `garmin-connect-export` / `garmin-health-data` ya la usan, `INVESTIGACION-garmin.md`) | **se rompe** cuando Garmin cambia algo (vía NO oficial) | mientras funcione |
| **2 · Exportación oficial** (SOS 1) | el cliente pide «Export Your Data» en Garmin y la app **lee el ZIP** — mismo patrón que el ZIP de Huawei, ya construido y probado (`motor/src/huawei.rs`) | un paso manual del cliente, pero **uno solo** | si la 1 se rompe |
| **3 · Reloj por USB** (SOS 2) | el reloj montado como unidad: la app lee `ACTIVITY/*.fit` (el selector de carpeta ya existe) | el cliente conecta el cable | si 1 y 2 no están |
| **4 · Carpeta sincronizada** (SOS 3, el estado de hoy) | soltar los FIT nuevos en la carpeta elegida; la memoria incremental ya relee **solo lo nuevo** (medido: 0,086 s por sesión) | el paso más manual | última red |

- La pieza de sincronización queda **AISLADA** tras este contrato: cambiar de la vía 1 a la 2 **no
  toca el motor ni la pantalla**. La pantalla Datos enseña qué vía está activa y de dónde sale cada
  dato (AU-87: la fuente es dato).
- **Nada de lo sincronizado se pierde** al cambiar de vía: la memoria es de solo añadir y cada aporte
  guarda su fecha (AU-19).

## 4. «Constantemente midiendo», con honestidad

Medir lo hace **el reloj de Garmin**; la aplicación es el consumidor. Sincroniza **al abrir** + botón
**«Sincronizar ahora»**; cuando corra en Android, sincronización periódica en segundo plano. Cada
sincronización es barata: la memoria incremental ya está medida (`ESTADO.md §6-sexies`).

## 5. Probar ANTES de construir (medir, no suponer)

1. Con una **cuenta de prueba** (o la del cliente, si autoriza): los **diarios en vivo** — en la
   exportación masiva venían VACÍOS (medido, `ESTADO.md §6-bis`); ¿por la API interna vienen llenos?
2. El **acceso de dos factores (MFA)** de Garmin: ¿el login automático sobrevive? Medir el flujo real.
3. **Tiempo de una sincronización** (actividades nuevas + diarios) y qué pasa cuando Garmin **rechaza
   el login**: error tipado, no pánico.
4. **La prueba del SOS**: romper la vía 1 (a propósito) y comprobar que la 2 trae lo mismo con los
   datos ya guardados intactos (AU-52: los dos lados).

## 6. Orden de construcción (un hito = un commit)

1. Este contrato (hecho).
2. **Instrumento de medida**: `sincronizacion --probar` que imprime qué trae cada vía **sin tocar el
   motor** (el patrón de `medir-huawei`).
3. El módulo `motor/src/sincronizacion.rs` con sus pruebas (qué cuenta como «nuevo» es una **función
   pura**, como en memoria-incremental).
4. La pantalla en Datos: «Conectar cuenta Garmin», la vía activa, el botón **Sincronizar**, y el aviso
   de qué trajo.
5. Android: sincronizar al abrir + periódica en segundo plano.

## 7. Decisión pendiente del dueño

Cuenta para la primera prueba: ¿**una cuenta de prueba** o la del cliente? (Con MFA incluido en la
prueba, pase lo que pase.)
