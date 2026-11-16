#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOL="${ROOT}/scripts/fault_inject_scenario.py"
INPUT="${ROOT}/scenarios/smoke.ndjson"
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

python3 "${TOOL}" "${INPUT}" --drop-every 2 --output "${TMP}/drop.ndjson"
[ "$(wc -l < "${TMP}/drop.ndjson")" -eq 1 ]

python3 "${TOOL}" "${INPUT}" --delay-ms 500 --output "${TMP}/delay.ndjson"
grep -q '"timestamp_ms":1500' "${TMP}/delay.ndjson"
grep -q '"fault_injected":"delay"' "${TMP}/delay.ndjson"

python3 "${TOOL}" "${INPUT}" --contradict-every 2 --output "${TMP}/contradict.ndjson"
grep -q '"confidence":0.25' "${TMP}/contradict.ndjson"
grep -q '"fault_injected":"contradiction"' "${TMP}/contradict.ndjson"

echo "fault injection: drop, delay, and contradiction modes verified"
