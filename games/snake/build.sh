#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown --target-dir target
python3 - <<'PY'
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED

output = Path('../../bundled-games/snake.adm')
output.parent.mkdir(parents=True, exist_ok=True)
with ZipFile(output, 'w') as package:
    for name, source in [
        ('manifest.toml', Path('manifest.toml')),
        ('game.wasm', Path('target/wasm32-unknown-unknown/release/snake.wasm')),
    ]:
        info = ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = ZIP_DEFLATED
        info.external_attr = 0o100644 << 16
        package.writestr(info, source.read_bytes())
print(output)
PY
