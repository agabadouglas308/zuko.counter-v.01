# Roadmap

## Milestone 1 — simulator foundation

- [x] deterministic C++ synthetic sensor stream;
- [x] Rust NDJSON ingestion and validation;
- [x] confidence-weighted track fusion;
- [x] protected-zone policy with benign recommendations;
- [x] explicit safety boundary and audit-oriented output.

## Milestone 2 — engineering quality

- [x] replace the intentionally minimal parser with typed `serde` + `serde_json` models;
- [x] add Rust unit tests for malformed input, confidence clamping, and geofence edges;
- [x] add C++ deterministic stream tests through CTest;
- [x] add timestamps, schema versioning, and correlation IDs;
- [x] add structured append-only audit storage.

## Milestone 3 — operator workflow

- [ ] read-only local dashboard for tracks, confidence, and sensor provenance;
- [ ] explicit human acknowledgement state machine;
- [x] replay mode for recorded synthetic scenarios;
- [ ] fault injection for dropped, delayed, and contradictory observations.

## Out of scope

- effectors, weapons, aiming, engagement, jamming, spoofing, cyber takeover, or autonomous physical intervention;
- collection or processing of real RF payloads or personally identifying operator data.
