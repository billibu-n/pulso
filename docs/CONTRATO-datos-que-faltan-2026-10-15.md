# CONTRATO — Qué dice la aplicación cuando el dato NO está (Garmin y todo lo demás)

**Escrito el 2026-10-15.** Nace de una instrucción del dueño, textual:

> «Debemos evitar invadir al cliente con preguntas específicas; la idea es que **la aplicación sepa
> cómo enfrentar casos donde falten datos o la disponibilidad o información del usuario**.»

Y de una medida de ese mismo día: la cuenta de la prueba **no tiene sueño en 21 de 21 días, ni estrés
en 28 de 28, y el pulso en reposo lleva ~3 semanas sin valor** — mientras las actividades llegan
perfectas. Es decir: **el caso "falta el dato" no es un caso raro; es el estado normal de esta
cuenta.** Este contrato es para ESE caso.

Complementa a `CONTRATO-garmin-sync.md` (la vía) y no lo cambia: aquélla dice **de dónde** viene el
dato; éste dice **qué se enseña cuando no viene**.

---

## 1. Los cuatro principios, que son la norma

1. **La app dice lo que puede PROBAR con el cuerpo que recibió. Nada más.**
   Si el dato vino `null`, la app dice «Garmin no lo trajo», **no** «no te pusiste el reloj». La
   diferencia no es de estilo: es la diferencia entre un dato y una acusación.
2. **La app NO pregunta NADA al cliente por el dato que falta.** Ni un formulario, ni un diálogo, ni
   «¿te pusiste el reloj?». Muestra el estado y **cómo se corrige en la propia app**, si se corrige
   desde la app. Lo que hay que hacer en el reloj no se pide: se dice como hecho del dato.
3. **Se distingue lo que no se midió de lo que se midió vacío.** Ya rige en el resto del motor
   (`formulas.rs:289` `no_se_puede`, `umbral.rs:42` / `potencia.rs:74` `que_falta`, y las pruebas
   `una_cifra_que_no_existe_es_None_y_NO_una_cadena_vacia`, `vacio_y_no_medido_NO_se_confunden`).
   **En el diario de Garmin todavía NO rige: aquí se paga esa deuda.**
4. **Nada de datos de salud sale del equipo del cliente, y nada se pide por canales que el cliente no
   controle.** El reporte de fallos no manda cuerpos de dato; las credenciales no pasan por el chat ni
   por el repo (protocolo de `docs/SINCRONIZACION-garmin-2026-10-15.md` §7, sin excepciones). Y un
   fallo se cuenta **sin** el cuerpo: con la clave que faltó y el estado HTTP, nunca con el valor.

## 2. Lo que la app SABE y lo que NO PUEDE saber (medido, para no inventar)

Esto es lo que se midió el 2026-10-15 sobre los cuerpos reales de `/gc-api/`, y **limita lo que la app
puede afirmar**. Es la parte que impide que alguien escriba en la interfaz una causa que no consta:

| lo que la app quiere decir | ¿se puede? | por qué (medido) |
|---|---|---|
| «este día no tiene sueño/estrés» | **SÍ** | el cuerpo trae la clave con `null` (`avgStressLevel: null`, fases `null`) |
| «lleva N días sin traerlo» | **SÍ** | se cuenta sobre los días pedidos |
| «Garmin dice que ese día no incluye bienestar» | **SÍ** | `resumen-dia.json` trae **`includesWellnessData: false`** e `includesActivityData: true` |
| «qué dispositivos tiene la cuenta» | **SÍ** | `dispositivos.json`: 3, con `productDisplayName`, `deviceStatus`, y 267 `capabilities` cada uno |
| «**cuál** de sus dispositivos produjo el dato» | **A VECES** | `training.json` trae `deviceId` (y `recordedDevices`); los cuerpos de sueño y estrés **NO lo traen** |
| «**cuándo** sincronizó por última vez» | **NO** | **`lastPrimarySyncDate` viene `null`** y en los 9 cuerpos **no hay ningún** `lastSyncTime`/`lastSyncDate`. La API **no lo publica** |
| «por qué falta» (¿no se lo puso? ¿se descargó? ¿no sincroniza?) | **NO** | no está en ningún cuerpo. Cualquier causa escrita en la interfaz sería **inventada** |
| «¿tiene el reloj puesto?» | **NO se pregunta** | principio 2 |

**La consecuencia de diseño, en una línea: la app informa de la PRESENCIA, con su cuenta y su fecha,
y del hecho de que Garmin no publica la causa. No especula, y no pregunta.**

## 3. Las formas: tres estados, no dos

| estado | qué es | qué se enseña |
|---|---|---|
| **CON DATO** | la clave vino con valor | la cifra, con su fecha y su fuente |
| **SIN MEDIDA** | la clave vino, y con `null` | «Garmin no trajo este dato» + desde cuándo + **qué hizo la app** (lo pidió y no venía) |
| **NO SE PUDO COMPROBAR** | no se llegó a preguntar (sin sesión, límite de tasa, error de red) | «no se pudo comprobar» + **el motivo concreto** + **se reintenta** |

**SIN MEDIDA y NO SE PUDO COMPROBAR NO se escriben igual y NO se suman.** Son dos frutas distintas:
una dice «Garmin no lo tiene»; la otra dice «no llegamos a mirar». Confundirlas es exactamente el
error que se cometió el 2026-10-15 al leer los 9 cuerpos (se confundió «la clave existe» con «el dato
existe») y que costó un chat entero de diagnóstico.

## 4. Los cuatro huecos concretos que quedan abiertos hoy

Estos NO son ideas: están medidos en el árbol de trabajo del 2026-10-15.

1. **El tipo del diario no distingue la clave ausente del `null`.** Decisión registrada en
   `motor/src/sincronizar.rs` §DUDAS 4 (75 referencias a los campos de `Diario`). Con el dato real, el
   `null` es **el caso frecuente**, así que esta deuda **se usa todos los días**. Qué hay que
   decidir: si el aviso «esta clave ya no viene» (que hoy vive en la comprobación de forma del
   instrumento hermano, `objeto`/`cambiado`) basta, o si el tipo tiene que cargarlo. **Se decide con
   el número delante, no por gusto.**
2. **El lector de actividades no lee el cuerpo real.** `grafo-x.json` trae la lista **anidada** en
   `data.activitiesScalar.activityList` y `parsear_actividades` espera una **lista pelada**: con el
   cuerpo real **falla** («invalid type: map, expected a sequence»). Lo impide una regla explícita del
   módulo y su prueba (`el_envoltorio_no_vale_como_lista_de_actividades`). **Ojo: ésta es la única
   vía que trae las actividades**, así que la regla —o su alcance— hay que revisarla.
3. **`entrenamiento` no está enchufado al instrumento hermano** (dos sitios: `training_tiene_dato` y
   el bucle de peticiones). Se lee y está probado, pero nadie lo llama: el flujo real todavía no pide
   `training_status`/`vo2max`.
4. **`resumen-dia`, `dispositivos` y `perfil` no tienen lector.** Y son justo los que traen
   `includesWellnessData` (el aviso más útil que da Garmin sobre el bienestar) y la lista de
   dispositivos con sus capacidades. Sin lector, la app **no puede** decir ni siquiera lo que sí sabe.

Además, dos cosas de la vía que la app tiene que tratar y que hoy no están en el código:

5. **Hay límite de tasa** (medido: ~40 peticiones seguidas → **403** de Cloudflare; las mismas
   espaciadas 18-20 s → **200**). El 403 **NO es «la sesión se cayó»**: es «espera y vuelve».
   Va al estado NO SE PUDO COMPROBAR, con reintento, nunca a SIN MEDIDA.
6. **El renderer**: en Android Slint **solo admite Skia**; el repo usa `renderer-software`. Tal como
   está, **no compila para Android** (medido: `i-slint-backend-selector-1.18.1/api.rs:438`).

## 5. Las frases (el tono es parte del contrato)

| situación | cómo se dice | cómo NO se dice |
|---|---|---|
| sin medida, con historia | «Sueño: Garmin no trajo el dato en los últimos 21 días.» | «No te pusiste el reloj.» |
| sin medida, sin historia | «Sueño: Garmin no trajo este dato.» | «Falta tu sueño.» |
| no se pudo comprobar | «No se pudo comprobar: Garmin no respondió (se reintenta).» | «Sin datos.» |
| límite de tasa | «Garmin pidió esperar. Se reintenta solo.» | «Error de conexión.» |
| dato correcto | la cifra, **con su fecha y su fuente** (`api-interna`) | una cifra sin fecha ni fuente |

Reglas de forma, que ya rigen en la interfaz y aquí se heredan: **UNA línea**, con icono, sin números
escritos a mano, y una cifra que no existe **no es un 0 ni un guion** (`cara.rs`,
`medir-interfaz.rs`).

## 6. Lo que este contrato PROHÍBE (y conviene que esté escrito)

1. **Pedirle al cliente que averigüe por qué falta.** Ni por la app, ni por el dueño. Se midió que la
   causa **no está publicada** (§2): preguntarla es trasladarle al cliente un trabajo que la app no
   puede hacer, y encima invade.
2. **Escribir una causa que no consta.** «Se descargó», «no sincroniza», «no se lo puso»: nada de eso
   está en el cuerpo.
3. **Contar SIN MEDIDA como NO SE PUDO COMPROBAR** (o al revés) para «que no salga tan feo».
4. **Mandar el cuerpo del dato** en cualquier reporte de fallo: sólo la clave y el estado.
5. **Guardar datos de salud en el repositorio.** `datos/` está ignorado (`**/datos/`); los 9 cuerpos
   son de una persona real y no entran en git **nunca**, ni como fixture de prueba.
6. **Decir «error de la app»** cuando Garmin simplemente no trajo el dato: es una afirmación falsa
   sobre el propio programa, y manda al cliente a buscar el problema donde no está.

## 7. Cómo se comprueba que esto se cumple

- Una prueba por frase de la tabla §5, con el cuerpo **real** que la produce (o su forma exacta, con
  valores inventados: los de salud **no** entran al repo).
- Una prueba que exija que **SIN MEDIDA y NO SE PUDO COMPROBAR den textos distintos** (la prueba no
  mira el código: mira las dos cadenas y comprueba que **no son iguales**).
- Una prueba de que **ninguna frase de la interfaz promete una causa**: se buscan en los textos las
  palabras de causa («descarg», «sincroniz», «no se lo puso», «se le olvidó») y **no deben aparecer**
  en el camino del dato, sólo en el de la vía.
- La comprobación de forma del instrumento hermano sigue avisando cuando **una clave que antes venía
  deja de venir** — eso es NO SE PUDO COMPROBAR, no SIN MEDIDA.

---

**Estado**: propuesta escrita el 2026-10-15, **pendiente de aprobación del dueño**. No se implementa
nada de §4 (los cuatro huecos) sin su visto bueno, y el orden lo decide él.
