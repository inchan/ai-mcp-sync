#!/usr/bin/env bash
set -euo pipefail

pushd "$(dirname "$0")/.." >/dev/null

cargo build --manifest-path backend/Cargo.toml

if command -v npm >/dev/null 2>&1; then
  pushd frontend >/dev/null
  npm install
  npm run build
  popd >/dev/null
else
  echo "npm이 설치되어 있지 않아 프런트엔드 빌드를 건너뜁니다." >&2
fi

popd >/dev/null
