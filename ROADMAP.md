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

- [x] read-only local dashboard for tracks, confidence, and sensor provenance;
- [x] explicit human acknowledgement state machine;
- [x] validate dashboard acknowledgement transitions and reject unknown event IDs;
- [x] replay mode for recorded synthetic scenarios;
- [ ] fault injection for dropped, delayed, and contradictory observations.

## Milestone 4 — resilience and validation (in progress)

- [x] deterministic fault-injection tool for dropped, delayed, and contradictory observations;
- [x] replay assertions for event counts, ordering, and policy transitions;
- [x] explicit stale-track and out-of-order timestamp behavior;
- [ ] integration tests covering malformed, partial, and duplicate streams;
- [ ] failure-mode documentation for operators and developers.

## Milestone 5 — configuration and delivery

- [ ] external versioned configuration for protected zones and policy thresholds;
- [ ] configuration validation and safe startup failure behavior;
- [ ] reproducible developer and CI commands;
- [ ] dependency auditing and supply-chain checks;
- [ ] release packaging for Fedora development environments.

## Milestone 6 — observability and controlled deployment

- [ ] health and readiness endpoints for local services;
- [ ] structured operational metrics and bounded log retention guidance;
- [ ] access-control design for any non-loopback deployment;
- [ ] threat model, privacy review, and cybersecurity checklist;
- [ ] deployment runbook with explicit human-in-the-loop and safety gates.

## Out of scope

- effectors, weapons, aiming, engagement, jamming, spoofing, cyber takeover, or autonomous physical intervention;
- collection or processing of real RF payloads or personally identifying operator data.
