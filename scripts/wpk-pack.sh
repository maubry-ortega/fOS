#!/usr/bin/env bash
set -euo pipefail

APP_NAME=${1:-app}
OUT_DIR=${2:-wpk}
ZIG_BIN_DIR=./sdk/zig-out/bin

# Aceptar binario con o sin extensión .wasm y renombrar a .wasm
WASM_SRC=""
if [ -f "${ZIG_BIN_DIR}/${APP_NAME}.wasm" ]; then
  WASM_SRC="${ZIG_BIN_DIR}/${APP_NAME}.wasm"
elif [ -f "${ZIG_BIN_DIR}/${APP_NAME}" ]; then
  WASM_SRC="${ZIG_BIN_DIR}/${APP_NAME}"
else
  echo "Falta ${ZIG_BIN_DIR}/${APP_NAME}[.wasm]. Construye con: zig build wasm" >&2
  exit 1
fi

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR/assets"

cat > "$OUT_DIR/manifest.toml" <<EOF
name = "${APP_NAME}"
id = "com.fos.${APP_NAME}"
version = "0.1.0"
entry = "${APP_NAME}.wasm"
permissions = ["log"]
min_platform = "fOS:0.1"
EOF

cp "$WASM_SRC" "$OUT_DIR/${APP_NAME}.wasm"
if [ -f ./sdk/src/assets/app.lua ]; then
  cp ./sdk/src/assets/app.lua "$OUT_DIR/assets/app.lua"
fi
echo "{}" > "$OUT_DIR/hashes.json"
echo "UNSIGNED" > "$OUT_DIR/signature.sig"

cd "$OUT_DIR"
zip -q -r "../${APP_NAME}.wpk" .
cd - >/dev/null
echo "[OK] Empaquetado: ${APP_NAME}.wpk"

# Exportar también el wasm crudo a la raíz para pruebas del microkernel
cp "$OUT_DIR/${APP_NAME}.wasm" "./${APP_NAME}.wasm"
echo "[OK] Exportado: ${APP_NAME}.wasm"
if [ -f "$OUT_DIR/assets/app.lua" ]; then
  cp "$OUT_DIR/assets/app.lua" "./app.lua"
  echo "[OK] Exportado: app.lua"
fi


