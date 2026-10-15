#!/usr/bin/env bash
# VER LA APLICACION SIN ABRIRLA.  Uso:  ./ver-app.sh  [carpeta-de-datos]
#
# Escrito el 2026-10-15, a peticion del dueno: «no puedo ver nada y no se que hay ni que falta».
#
#   Por que existe: el dueno trabaja en Wayland y yo apuntaba a `DISPLAY=:0`, que NO responde
#   (medido: `xset q` falla). Resultado: la ventana se abria en un sitio que nadie mira. Esto
#   renderiza la ventana EN MEMORIA y la deja en PNG, que se mira desde cualquier parte.
#
#   Que hace: pinta las CUATRO secciones (Resumen, Zonas, Sesiones, Datos) del estado REAL, y
#   ademas el caso de ejemplo con datos inventados (para ver el diseno CON datos).
#
#   No abre ventana, no toca el perfil y no escribe en la carpeta de datos: solo lee.
set -u
RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"     # portabilidad: la ruta se deriva
CARPETA="${1:-$RAIZ/datos/garmin_files/storage}"
SALIDA="$RAIZ/datos/capturas/ver-app"
SECCIONES=(Resumen Zonas Sesiones Datos)

command -v magick >/dev/null || command -v convert >/dev/null || {
  echo "falta ImageMagick (magick/convert) para pasar de PPM a PNG"; exit 1; }
CONV="$(command -v magick || command -v convert)"

echo "Compilando la app (solo si hace falta)..."
( cd "$RAIZ/motor" && cargo build --bin capturar-ui --quiet ) || { echo "no compila"; exit 1; }

mkdir -p "$SALIDA"
BIN="$RAIZ/motor/target/debug/capturar-ui"
for i in "${!SECCIONES[@]}"; do
  N="${SECCIONES[$i]}"
  if "$BIN" "/tmp/_ver_$i.ppm" --seccion "$i" --carpeta "$CARPETA" >/dev/null 2>&1; then
    "$CONV" "/tmp/_ver_$i.ppm" "$SALIDA/$i-$N.png" 2>/dev/null && echo "  $N-$N.png"
  else
    echo "  (fallo el $N)"
  fi
done
if "$BIN" /tmp/_ver_ej.ppm --ejemplo --seccion 0 >/dev/null 2>&1; then
  "$CONV" /tmp/_ver_ej.ppm "$SALIDA/EJEMPLO-Resumen-con-datos.png" 2>/dev/null && echo "  EJEMPLO-Resumen-con-datos.png"
fi
rm -f /tmp/_ver_*.ppm

echo
echo "Listo. Las imagenes estan en:"
echo "  $SALIDA"
echo
echo "Que significa cada una:"
echo "  Resumen / Zonas / Sesiones / Datos  = EL ESTADO REAL de hoy. Si salen vacias, es porque"
echo "                     faltan la edad y el umbral (se escriben en «Datos»), no porque falle."
echo "  EJEMPLO-...        = el diseno CON datos inventados, para ver como queda lleno."
