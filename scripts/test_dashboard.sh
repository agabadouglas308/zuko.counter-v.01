#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${DASHBOARD_TEST_PORT:-18081}"
TMP="$(mktemp -d)"
PID=""
trap 'if [[ -n "${PID}" ]]; then kill "${PID}" 2>/dev/null || true; fi; rm -rf "${TMP}"' EXIT

cargo build --manifest-path "${ROOT}/rust-c2/Cargo.toml" --bin dashboard >/dev/null
"${ROOT}/scripts/replay_scenario.sh" "${ROOT}/scenarios/smoke.ndjson" "${TMP}/audit.ndjson" >/dev/null
"${ROOT}/rust-c2/target/debug/dashboard" --audit-log "${TMP}/audit.ndjson" --port "${PORT}" >"${TMP}/out" 2>"${TMP}/err" &
PID=$!
for _ in $(seq 1 20); do curl -fsS "http://127.0.0.1:${PORT}/api/events" >/dev/null 2>&1 && break; sleep 0.1; done

python3 - "${PORT}" <<'PY'
import json
import sys
import urllib.error
import urllib.request

port = sys.argv[1]
base = f"http://127.0.0.1:{port}"
events = json.load(urllib.request.urlopen(base + "/api/events"))
assert events, "audit.ndjson must contain at least one event"
id_ = events[0]["event_id"]
assert events[0]["review_state"] == "pending"
req = urllib.request.Request(base + "/api/ack", data=json.dumps({"event_id": id_}).encode(), headers={"content-type": "application/json"}, method="POST")
ack = json.load(urllib.request.urlopen(req))
assert ack["state"] == "acknowledged"
unknown = urllib.request.Request(base + "/api/ack", data=b'{"event_id":"missing"}', headers={"content-type": "application/json"}, method="POST")
try:
    urllib.request.urlopen(unknown)
    raise AssertionError("unknown event was accepted")
except urllib.error.HTTPError as error:
    assert error.code == 404
print("dashboard state machine: pending, acknowledged, and unknown states verified")
PY
