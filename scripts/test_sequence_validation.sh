#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="${ROOT}/rust-c2/target/debug/rust-c2"
cargo build --manifest-path "${ROOT}/rust-c2/Cargo.toml"
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

cat > "${TMP}/sequence.ndjson" <<'EOF'
{"kind":"observation","schema_version":1,"event_id":"seq-1","sensor":"radar","target_id":"seq","timestamp_ms":2000,"position":{"x_m":60,"y_m":0,"z_m":50},"velocity":{"x_mps":-1,"y_mps":0,"z_mps":0},"confidence":0.8}
{"kind":"observation","schema_version":1,"event_id":"seq-1","sensor":"eo","target_id":"seq","timestamp_ms":2000,"position":{"x_m":20,"y_m":0,"z_m":50},"velocity":{"x_mps":-1,"y_mps":0,"z_mps":0},"confidence":0.9}
{"kind":"observation","schema_version":1,"event_id":"seq-2","sensor":"rf","target_id":"seq","timestamp_ms":1000,"position":{"x_m":20,"y_m":0,"z_m":50},"velocity":{"x_mps":-1,"y_mps":0,"z_mps":0},"confidence":0.9}
EOF

"${BINARY}" < "${TMP}/sequence.ndjson" > "${TMP}/out.ndjson" 2> "${TMP}/err.log"
[ "$(grep -c '"kind":"track_update"' "${TMP}/out.ndjson")" -eq 1 ]
grep -q 'duplicate event_id' "${TMP}/err.log"
grep -q 'out-of-order timestamp' "${TMP}/err.log"
echo "sequence validation: duplicate and stale observations rejected"
