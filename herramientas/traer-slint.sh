#!/bin/bash
# Se trae la documentacion OFICIAL de Slint, para estudiarla y destilarla.
#
# POR QUE SE GUARDA: una pagina web cambia y desaparece. Lo que se estudia se guarda,
# con su fecha y su direccion, para poder CITARLO y para poder volver a mirarlo.
B=https://docs.slint.dev/latest/docs/slint
D=~/projects/pulso/docs/slint-fuentes
mkdir -p "$D"; cd "$D" || exit 1

paginas=(
  "guide/language/concepts/slint-language:conceptos-lenguaje"
  "guide/language/coding/file:fichero"
  "guide/language/coding/properties:propiedades"
  "guide/language/coding/expressions-and-statements:expresiones"
  "guide/language/coding/positioning-and-layouts:disposicion"
  "guide/language/coding/globals:globales"
  "guide/language/coding/repetition-and-data-models:repeticion"
  "guide/language/coding/states:estados"
  "guide/language/coding/functions-and-callbacks:funciones"
  "guide/language/coding/name-resolution:ambito"
  "guide/language/coding/structs-and-enums:estructuras"
  "guide/language/coding/animation:animacion"
  "guide/development/debugging_techniques:depuracion"
  "guide/development/custom-controls:controles"
  "guide/development/best-practices:buenas-practicas"
  "guide/development/fonts:fuentes"
  "guide/development/translations:traducciones"
  "guide/platforms/mobile/android:android"
  "guide/platforms/desktop/general:escritorio"
  "reference/language/file-structure:ref-estructura"
  "reference/language/type-conversions:ref-conversiones"
  "reference/language/repetition-and-conditional-elements:ref-condicionales"
  "reference/language/container-components:ref-contenedores"
  "reference/property-types/colors-and-brushes:ref-colores"
  "reference/property-types/arrays-and-models:ref-modelos"
  "reference/common:ref-comunes"
  "reference/std-widgets/style:ref-estilos"
  "reference/std-widgets/views/scrollview:widget-scrollview"
  "reference/std-widgets/views/lineedit:widget-lineedit"
  "reference/std-widgets/basic-widgets/spinner:widget-spinner"
  "reference/timer:temporizador"
  "reference/gestures/flickable:flickable"
)
for p in "${paginas[@]}"; do
  url="${p%%:*}"; nom="${p##*:}"
  curl -sL --max-time 40 "$B/$url/" -o "$nom.html" 2>/dev/null
  t=$(stat -c%s "$nom.html" 2>/dev/null || echo 0)
  printf "  %-24s %6s bytes\n" "$nom" "$t"
done
