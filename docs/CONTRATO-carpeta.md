# Contrato: elegir la carpeta de sus ficheros, con un boton

**Escrito el 2026-10-14, ANTES del codigo**, como manda la regla 1 de
[ARQUITECTURA-piezas-y-contrato.md](ARQUITECTURA-piezas-y-contrato.md).

## El problema

Hoy la carpeta entra por **linea de ordenes**: el cliente tendria que saber donde estan sus ficheros y
escribir la ruta. Es **EL ATAJO QUE MAS VALE**, porque no necesita cuenta, ni permiso, ni internet: sus
datos ya estan en su equipo.

## La decision (2026-10-14)

**NO se usa un dialogo del sistema.** Medido: **Slint 1.18.1 no trae FileDialog ni FolderDialog**, y una
dependencia nueva (del tipo rfd) **hay que autorizarla**, como se autorizo la de zip. Se escribe un
**selector propio**: la aplicacion lista las subcarpetas, se sube y se baja por el arbol, y el cliente
elige. **Sin dependencia nueva.**

## Que es, y en que capa vive

La **logica del selector** es un adaptador (capa 2): mira el disco y devuelve nombres. **No dibuja**:
quien dibuja es la capa 5 (el .slint), y quien decide es el borde.

## Que recibe y que devuelve

- `subcarpetas(dir) -> Vec<String>`: los nombres de las **subcarpetas inmediatas**, ordenados, **sin
  las ocultas** (las que empiezan por punto) y sin los ficheros. Una carpeta que no se puede leer
  devuelve **vacio, no un error**: para el que elige, "aqui no puedo entrar" y "aqui no hay nada" se
  enseñan igual, y el detalle de permisos no es cosa suya.
- `ruta_padre(dir) -> Option<PathBuf>`: la carpeta de arriba. **None en la raiz**: no se puede subir
  mas, y eso se dice en vez de dar una ruta rara.
- `es_carpeta(ruta) -> bool`.

## Que NO hace (el limite)

- **No decide** cual es la carpeta buena; solo la enseña.
- **No guarda** nada: guardar la eleccion es del perfil o de quien la use.
- **No crea ni borra** carpetas.
- **No filtra por contenido**: una carpeta con FIT y otra sin ellos se listan igual. Quien sabe contar
  ficheros es `fit::listar`, que es instantaneo.

## La INTUICION de la aplicacion

El selector **enseña, junto a cada carpeta, cuantos FIT tiene** (con `fit::listar`). Asi el cliente
reconoce su carpeta de datos de un vistazo, sin recordar la ruta. **Eso es la intuicion**: la aplicacion
hace el trabajo de reconocer, y el solo confirma. Cuando una carpeta no tiene ficheros, tambien se dice.

## Como se comprueba (las pruebas de esta parte)

1. Una carpeta con subcarpetas devuelve **sus nombres, ordenados**, y **ni ficheros ni ocultas**.
2. Subir desde la raiz da **None**; desde /home da /.
3. Una carpeta que **no se puede leer** devuelve vacio y **no revienta**.
4. `es_carpeta` distingue una carpeta de un fichero que no existe.

## Donde va el panel, y por que (decision 2026-10-14)

**El selector se abre como CAPA SUPERPUESTA sobre la ventana**, con un boton dentro del panel «Datos»
(«Buscar mis ficheros»), y se cierra al usar una carpeta.

- **POR QUE NO EN LA NAVEGACION**: la navegacion tiene **4 secciones y esta medida** (la prueba
  medir-interfaz exige los 4 huecos, repartidos a 411/4). Es el **diseño aprobado**, y añadir una quinta
  seccion por mi cuenta seria cambiar lo aprobado sin el dueño.
- **POR QUE NO DENTRO DE DATOS**: ese panel ya lleva diez campos en dos columnas **para que quepan sin
  desplazar**; meter la lista de carpetas ahi lo desbordaria.
- **POR QUE SUPERPUESTA**: no reflota nada, no toca la navegacion, y es **una accion, una forma**.

## Lo que queda para despues (la capa 5)

El **panel** que dibuja la lista, el boton de subir y el de "usar esta carpeta", y **conectarlo** a
`ventana/pulso.rs` para que al elegir se vuelva a analizar. Es la parte de interfaz, y va con su
medicion (geometria, no a ojo).
