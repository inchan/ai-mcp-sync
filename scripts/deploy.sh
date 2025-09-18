#!/usr/bin/env bash
set -euo pipefail

echo "배포 파이프라인은 아직 구성되지 않았습니다."
echo "1. backend/에서 'cargo build --release' 실행"
echo "2. frontend/에서 'npm install && npm run build' 실행"
echo "3. 산출물을 패키징하여 배포합니다."
