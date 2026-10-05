#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
rustc --edition=2024 --crate-type=cdylib --target wasm32-unknown-unknown -O test_game.rs -o game.wasm
python3 - <<'PY'
from zipfile import ZipFile, ZIP_DEFLATED
with ZipFile('host-test.adm', 'w', ZIP_DEFLATED) as package:
    package.write('manifest.toml')
    package.write('game.wasm')
PY
