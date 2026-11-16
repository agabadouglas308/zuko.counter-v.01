#!/usr/bin/env bash
set -euo pipefail

binary="$1"
output="$(${binary})"

count="$(printf '%s\n' "${output}" | grep -c '"kind":"observation"')"
[ "${count}" -eq 30 ] || { echo "expected 30 observations, got ${count}" >&2; exit 1; }

first="$(printf '%s\n' "${output}" | head -n 1)"
printf '%s\n' "${first}" | grep -q '"schema_version":1'
printf '%s\n' "${first}" | grep -q '"event_id":"obs-0-radar"'
printf '%s\n' "${first}" | grep -q '"sensor":"radar"'

last="$(printf '%s\n' "${output}" | tail -n 1)"
printf '%s\n' "${last}" | grep -q '"sensor":"rf"'
printf '%s\n' "${last}" | grep -q '"timestamp_ms":11000'

echo "deterministic sensor stream: ${count} observations verified"
