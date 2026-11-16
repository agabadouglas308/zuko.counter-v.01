#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT="${1:-${ROOT}/scenarios/baseline.ndjson}"
BINARY="${ROOT}/cpp-sensors/build/cuassensors"

if [[ ! -x "${BINARY}" ]]; then
  cmake -S "${ROOT}/cpp-sensors" -B "${ROOT}/cpp-sensors/build"
  cmake --build "${ROOT}/cpp-sensors/build"
fi

mkdir -p "$(dirname "${OUTPUT}")"
"${BINARY}" > "${OUTPUT}"
printf 'recorded %s observations to %s\n' "$(grep -c '"kind":"observation"' "${OUTPUT}")" "${OUTPUT}"
