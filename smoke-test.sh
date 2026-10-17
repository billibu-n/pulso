#!/usr/bin/env bash
# SMOKE TEST DEL FLUJO COMPLETO (T008, 2026-10-17)
#
# POR QUE EXISTE: los gates por tarea comprueban UNA cosa cada uno (que el HTML se escriba, que un
# CSV exista). Ninguno recorre el flujo de punta a punta, asi que una rotura entre piezas -- el motor
# pinta bien pero la exportacion no lee los FIT, por ejemplo -- pasaba desapercibida. Esto es el
# portero del CONJUNTO: si algo del recorrido se rompe, sale ROJO con el paso que fallo.
#
# QUE COMPRUEBA, en orden:
#   1. Las pruebas del motor (todas).
#   2. El build, SIN avisos.
#   3. Que la app se PINTA: se renderiza en memoria con el mismo camino de pintado que la aplicacion.
#   4. Que «Sincronizar» responde: la frase del resumen de Garmin, que es la funcion pura del handler.
#   5. Que el informe HTML se genera con datos reales (> 1 KB y con tabla).
#   6. Que los 3 CSV se generan con datos reales (> 100 bytes cada uno).
#   7. Que el informe no inventa numeros (la guarda del motor).
#
# Uso:  bash smoke-test.sh
# Sale 0 si TODO esta verde, y 1 si algo falla.

set -u
cd "$(dirname "$0")" || exit 1
export CARGO_TERM_COLOR=never

FALLOS=0
paso_ok()  { printf '  [ OK ] %s\n' "$1"; }
paso_mal() { printf '  [MAL ] %s\n' "$1"; FALLOS=$((FALLOS + 1)); }

echo "SMOKE TEST — $(date '+%Y-%m-%d %H:%M')"
echo "cwd: $(pwd)"
echo

# ---------------------------------------------------------------------------------------------
# 1. LAS PRUEBAS DEL MOTOR
# ---------------------------------------------------------------------------------------------
echo "1. Pruebas del motor"
if cargo test --quiet --manifest-path motor/Cargo.toml > /tmp/pulso-smoke-tests.log 2>&1; then
  PASAAS=$(grep -E "test result: ok\." /tmp/pulso-smoke-tests.log | awk '{s+=$4} END {print s+0}')
  paso_ok "cargo test — $PASAAS pruebas en verde"
else
  paso_mal "cargo test — mira /tmp/pulso-smoke-tests.log"
  grep -E "FAILED|panicked|error\[" /tmp/pulso-smoke-tests.log | head -5
fi

# ---------------------------------------------------------------------------------------------
# 2. EL BUILD, SIN AVISOS
# ---------------------------------------------------------------------------------------------
echo "2. Build sin avisos"
BUILD=$(cargo build --manifest-path motor/Cargo.toml 2>&1)
RC_BUILD=$?
AVISOS=$(printf '%s' "$BUILD" | grep -c "warning:")
if [ $RC_BUILD -eq 0 ] && [ "$AVISOS" -eq 0 ]; then
  paso_ok "cargo build — 0 avisos"
else
  paso_mal "cargo build — rc=$RC_BUILD, $AVISOS avisos"
  printf '%s\n' "$BUILD" | grep "warning:" | head -5
fi

# ---------------------------------------------------------------------------------------------
# 3. LA APP SE PINTA (renderizador de memoria, mismo camino que la aplicacion)
# ---------------------------------------------------------------------------------------------
echo "3. La app se pinta y responde al puntero"
if cargo test --quiet --manifest-path motor/Cargo.toml --test medir-estados > /tmp/pulso-smoke-pintado.log 2>&1; then
  paso_ok "render en memoria + hover (medir-estados)"
else
  paso_mal "render en memoria + hover (medir-estados)"
  tail -10 /tmp/pulso-smoke-pintado.log
fi

# ---------------------------------------------------------------------------------------------
# 4. «SINCRONIZAR» RESPONDE: la frase del resumen, que es la funcion pura del handler
# ---------------------------------------------------------------------------------------------
echo "4. Sincronizar responde con SU frase"
if cargo test --quiet --manifest-path motor/Cargo.toml --test pruebas \
     el_resumen_de_garmin_son_los_numeros_reales_en_una_linea_y_sin_fechas \
     > /tmp/pulso-smoke-sincronizar.log 2>&1; then
  paso_ok "frase del resumen de Garmin"
else
  paso_mal "frase del resumen de Garmin"
  tail -10 /tmp/pulso-smoke-sincronizar.log
fi

# ---------------------------------------------------------------------------------------------
# 5 y 6. LOS ARTEFACTOS DE LA ENTREGA, generados AHORA desde los FIT del cliente
# ---------------------------------------------------------------------------------------------
SALIDA=$(mktemp -d /tmp/pulso-smoke-XXXXXX)
INFORME_BIN=(cargo run --quiet --manifest-path motor/Cargo.toml --bin informe --)
DATOS="datos/garmin_files/storage"

echo "5. Informe HTML"
"${INFORME_BIN[@]}" informe --carpeta "$DATOS" --formato html \
  --fecha "$(date '+%Y-%m-%d')" --salida "$SALIDA/informe.html" > /tmp/pulso-smoke-html.log 2>&1
HTML="$SALIDA/informe.html"
if [ -f "$HTML" ] && [ "$(stat -c%s "$HTML")" -gt 1024 ] && grep -q "<table>" "$HTML"; then
  paso_ok "informe.html — $(stat -c%s "$HTML") bytes, con tabla"
else
  paso_mal "informe.html — falta, pesa poco o no tiene tabla"
  cat /tmp/pulso-smoke-html.log | tail -5
fi

echo "6. Los 3 CSV"
for TIPO in actividades potencia zonas; do
  CSV="$SALIDA/export-$TIPO.csv"
  "${INFORME_BIN[@]}" export --carpeta "$DATOS" --tipo="$TIPO" --salida "$CSV" \
    > /tmp/pulso-smoke-csv-$TIPO.log 2>&1
  if [ -f "$CSV" ] && [ "$(stat -c%s "$CSV")" -gt 100 ]; then
    paso_ok "export-$TIPO.csv — $(stat -c%s "$CSV") bytes, $(($(wc -l < "$CSV") - 1)) filas"
  else
    paso_mal "export-$TIPO.csv — falta o pesa menos de 100 bytes"
    tail -5 /tmp/pulso-smoke-csv-$TIPO.log
  fi
done

# ---------------------------------------------------------------------------------------------
# 7. LA GUARDA DEL MOTOR: ¿el informe afirma algun numero que no salio de un calculo?
# ---------------------------------------------------------------------------------------------
echo "7. El informe no inventa numeros"
if grep -q "No: todos los numeros del texto salieron de un calculo del motor" /tmp/pulso-smoke-html.log; then
  paso_ok "guarda anti-invencion del informe"
else
  paso_mal "guarda anti-invencion del informe — no consta que se haya comprobado, o hay avisos"
  grep -i "aviso\|inventa" /tmp/pulso-smoke-html.log | head -5
fi

# ---------------------------------------------------------------------------------------------
# RESULTADO
# ---------------------------------------------------------------------------------------------
echo
if [ "$FALLOS" -eq 0 ]; then
  echo "SMOKE TEST: VERDE"
  echo "artefactos de la pasada: $SALIDA"
  exit 0
fi
echo "SMOKE TEST: ROJO ($FALLOS paso(s) fallido(s))"
exit 1
