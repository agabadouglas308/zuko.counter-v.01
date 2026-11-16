# Shared event contract

The C++ process emits one JSON object per line. The Rust process accepts the same lines on stdin.

Example:

```json
{"kind":"observation","sensor":"radar","target_id":"sim-01","timestamp_ms":1000,"position":{"x_m":120.0,"y_m":35.0,"z_m":80.0},"velocity":{"x_mps":-4.0,"y_mps":0.2,"z_mps":0.0},"confidence":0.86}
```

Fields are deliberately generic and synthetic. `target_id` is a scenario identifier, not an identity claim.
