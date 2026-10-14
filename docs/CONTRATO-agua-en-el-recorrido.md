# Contrato: el agua corporal de Huawei entra al recorrido CON SU FECHA

**Escrito el 2026-10-14, ANTES del codigo**, como manda la regla 1 de
[ARQUITECTURA-piezas-y-contrato.md](ARQUITECTURA-piezas-y-contrato.md). Es la continuacion de
[CONTRATO-huawei.md](CONTRATO-huawei.md): alli se **lee** el ZIP; aqui, lo que se lee **entra al
recorrido** de la aplicacion.

## El problema, con el dato delante

`huawei.rs` ya sabe leer la exportacion de Huawei, pero **nadie la usaba**. Hasta hoy el agua entraba
por `Hoy::con_agua(pct)`, que guardaba el porcentaje **sin decir de que dia era**. Y la regla del agua
es una regla del dia —«el parte de hoy»—, asi que un 55,3 % de junio y uno de esta manana se
enseñaban exactamente igual. Es el mismo defecto que el proyecto ya corrigio con la edad escrita a
mano: **un dato que caduca y nadie lo nota es peor que un dato que falta**, porque el que falta se ve.

Y habia una segunda mentira, mas pequeña: el modulo decia que el agua **la metia el a mano**. El
cliente no tiene por que teclear lo que su bascula ya midio y su exportacion ya trae en un ZIP.

## Que recibe

- **La lista de `Composicion`** que devuelve `huawei::leer`: unidades del SI, un campo ausente en
  `Dato::Falta`, **ordenada de la mas antigua a la mas reciente** (lo garantiza el contrato de
  `huawei.rs`).
- **Nada mas.** En particular **no recibe la fecha del equipo**: la fecha de la bascula es del dia en
  que se peso, no del dia en que se mira la pantalla.

## Que devuelve

- **Un `Hoy`** cuyo `agua_corporal_pct` es **la medicion con agua MAS RECIENTE**, y cuyo
  `fecha_agua` dice **de que dia** es esa medicion.
- **Sin agua en ninguna composicion**: `Hoy::sin_datos()`, y `lo_que_falta` lo dice.

## La regla de decision (una sola, y se prueba sola)

Entre las composiciones **que traen agua**, se toma **la de fecha mayor**. Las que no traen agua
**no tapan** a las que si: si el ultimo pesaje solo trae peso, se usa el agua del pesaje anterior. Si
ninguna trae fecha pero alguna trae agua, se usa esa (la lista viene en orden) y la fecha queda
`Falta`. El campo `fecha` del dia **se guarda tal cual**; no se deduce ni se rellena.

## Que NO hace (el limite, en cinco lineas)

- **No abre el ZIP ni toca la red**: eso es `huawei.rs`. Aqui solo se decide entre lo ya leido.
- **No estima** el agua desde el peso ni la grasa: si falta, falta.
- **No inventa la fecha**: si la medicion usada no la trae, `fecha_agua` queda en `Dato::Falta` y el
  texto dice que no se sabe; **no se pone la del equipo**.
- **No decide un umbral de caducidad**: no hay fuente que diga cuanto dura un dato de agua corporal,
  asi que **la fecha se ENSEÑA** y quien la lea decide. Inventar un corte seria decidir por el.
- **No conoce la pantalla** ni a `Persona`: solo el modelo (`Composicion` -> `Hoy`).

## El campo nuevo, y por que cambia la firma

`Hoy` gana `fecha_agua: Dato<Fecha>`. Y `Hoy::con_agua` pasa de un argumento a dos:
`con_agua(pct, fecha)`. **No se conserva la version de un argumento**: era justo la que permitia
olvidar el dia. Obligar a pasar la fecha es lo que hace imposible volver a cometer el defecto.

## Como se comprueba (las pruebas)

1. **Dos composiciones, la mas reciente con agua manda**, y su fecha viaja con el valor.
2. **La mas reciente sin agua NO tapa** a la anterior con agua.
3. **Sin agua en ninguna** -> `Hoy::sin_datos()` y `lo_que_falta` dice que falta.
4. **Agua sin fecha** -> el valor se usa, `fecha_agua` queda `Falta` y el texto lo dice.
5. **El aviso del agua lleva la fecha** cuando se sabe (y no la inventa cuando no).
6. **El puente completo**: un JSON minimo inventado -> `huawei::leer_texto` -> `Hoy::de_composiciones`
   -> el aviso sale con la fecha. Sin un solo dato del cliente.

## La conexion (el borde)

- `pulso --huawei <ruta>` (ZIP o carpeta) lee con `huawei::leer` y construye el `Hoy`. Sin ese
  argumento no hay agua: se dice que falta y **de donde sale**, en vez de teclearla.
- `--texto` imprime el estado del agua **con su fecha**, sin abrir ventana.
- `bin/avisos` (el instrumento de medida) acepta tambien `--huawei <ruta>`, para ver el aviso del
  agua con la fecha sobre datos reales.

## Lo que este contrato NO toca (dicho para no darlo por hecho)

- **No** conecta los avisos al panel de la ventana: hoy la ventana enseña el resumen, las zonas y el
  aviso de deriva. Llevar los avisos del modulo `avisos` a la pantalla es **otro paso**.
- **No** decide cada cuanto pide el cliente su exportacion a Huawei: lo decide el.
