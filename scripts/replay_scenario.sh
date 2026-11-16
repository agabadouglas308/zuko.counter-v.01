#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INPUT="${1:?usage: replay_scenario.sh SCENARIO.ndjson [AUDIT.ndjson]}"
AUDIT="${2:-}"
BINARY="${ROOT}/rust-c2/target/release/rust-c2"

if [[ ! -x "${BINARY}" ]]; then
  cargo build --release --manifest-path "${ROOT}/rust-c2/Cargo.toml"
fi

if [[ -n "${AUDIT}" ]]; then
  "${BINARY}" --audit-log "${AUDIT}" < "${INPUT}"
else
  "${BINARY}" < "${INPUT}"
fi
