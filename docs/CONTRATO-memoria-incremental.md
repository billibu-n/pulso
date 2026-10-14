# Contrato de la memoria INCREMENTAL: solo leer lo nuevo

**Escrito el 2026-10-14, ANTES del codigo**, como manda la regla 1 de
[ARQUITECTURA-piezas-y-contrato.md](ARQUITECTURA-piezas-y-contrato.md). Sustituye al contrato anterior,
que guardaba **solo el resultado** y por tanto, ante cualquier cambio, tenia que **releer los 102
ficheros enteros**.

## El problema, medido

La memoria actual guarda un unico resultado y una huella (nombre + tamaño) de lo leido. Si la huella
**coincide**, la segunda apertura es instantanea (**5,57 s -> 0,032 s**, medido). Pero si el ciclista
**añade la sesion de hoy**, la huella NO coincide, y la aplicacion **vuelve a leer los tres meses de
historico** para sumar un fichero. Eso es justo lo que el dueño pide evitar:

> *«por cada activacion de la aplicacion, no tenga que releer de nuevo toda la informacion de hace meses
> y ir sumando la informacion mas reciente»*.

## Que es esta pieza

La **memoria incremental**: recuerda **lo que aporta cada fichero** por separado, no solo el total. Asi,
al arrancar, **lee unicamente los ficheros nuevos o cambiados** y **reutiliza** el resto.

## Que recibe

- **La carpeta** con los FIT.
- **Lo recordado**: una lista de **fichas**, una por fichero leido antes, cada una con
  `nombre`, `tamano`, y **su aporte** al total (o el error, si no se pudo leer).

## Que devuelve

- El **resultado** (`Real`), **igual que si se hubiera leido todo** desde cero.
- Las **fichas nuevas** (para guardarlas).
- Un **recuento**: cuantas fichas se **recordaron** (no se leyeron) y cuantas se **leyeron**.
  Esto no es un adorno: es la prueba de que el ahorro ocurre de verdad.

## La regla de decision (una sola, y se prueba sola)

Un fichero **se reutiliza** si su **nombre Y su tamaño** coinciden con lo recordado.

- **Es nuevo** -> se lee.
- **Cambio de tamaño** -> se lee (su aporte viejo se descarta, **no se suma dos veces**).
- **Ya no esta** -> se olvida.
- **No se pudo leer** -> se recuerda el error, para no reintentarlo en cada arranque y para poder decirlo.

**El tamaño y NO la fecha**, y es a proposito: la fecha cambia al **copiar** un fichero aunque el
contenido sea el mismo, asi que con la fecha copiar sus datos invalidaria media memoria sin motivo.

## Que NO hace

- **No relee** lo que ya tiene ficha valida.
- **No borra** un aporte bueno por un fichero que no se pudo leer.
- **No inventa**: si un fichero no se puede leer, el total lo dice y no se rellena.
- **No guarda los datos crudos**: guarda aportes de nueve numeros, no las 549.842 muestras.

## La documentacion de las fechas (coherencia del dato)

Una memoria sin fechas puede sumar, pero **no puede decir que periodo cubre**, y entonces un total de
tres meses parece de hoy. Por eso:

- cada **aporte** lleva `inicio_s` (cuando empezo la sesion, en segundos epoch; `None` si el fichero no
  lo trae);
- el **resultado** lleva `desde` y `hasta` (la sesion mas antigua y la mas reciente);
- el **fichero de memoria** lleva `guardado_en` (cuando se calculo).

Y una fecha que no se sabe **se dice que no se sabe** (`None`), no se rellena con la del fichero ni con
la del equipo: la fecha de un FIT se copia, la del equipo es la de quien mira.

## Como se comprueba (las pruebas)

1. **Combinar los aportes da LO MISMO que leer todo** (la prueba que sostiene la pieza).
2. **La decision**: se reutiliza lo que no cambio, se lee lo nuevo, se relee lo que cambio de tamaño y
   se olvida lo que desaparecio.
3. **Las fechas**: `desde`/`hasta` son el minimo y el maximo; sin fechas, quedan en `None`.
4. **Ida y vuelta** por JSON, y una memoria **corrupta o de otra version** no impide arrancar: se
   recalcula.
5. **Medido**, con `cargo run -q --release --bin medir-carga`: la segunda apertura **no lee ni un FIT**.
