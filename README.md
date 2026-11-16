# cuas-sim

A defensive counter-UAS command-and-control simulator written as a Rust/C++ hybrid.

## Scope

This first milestone is intentionally limited to:

- synthetic target and sensor generation;
- multi-sensor observation fusion;
- track confidence and uncertainty handling;
- configurable protected-zone geofencing;
- explainable threat assessment;
- operator-facing recommendations and append-only audit events.

It does **not** implement RF jamming, GPS spoofing, cyber takeover, kinetic effects, directed energy, weapon aiming, fire control, or autonomous engagement. All inputs are synthetic and the output is advisory.

## Architecture

```text
C++ sensor simulator  -- NDJSON over stdout -->  Rust C2/fusion engine
        synthetic observations                         tracks + policy + audit
```

- `cpp-sensors`: deterministic synthetic radar/RF/EO/acoustic observations.
- `rust-c2`: validates observations, fuses them into tracks, evaluates a protected zone, and emits operator-readable recommendations.
- `schema`: shared NDJSON event examples and field definitions.

## Build (after installing toolchains)

Rust: stable toolchain with Cargo.

C++: C++17 compiler.

```bash
cmake -S cpp-sensors -B cpp-sensors/build
cmake --build cpp-sensors/build
cargo run --manifest-path rust-c2/Cargo.toml -- --input-cmd ./cpp-sensors/build/cuassensors
```

For a smoke test without process piping:

```bash
./cpp-sensors/build/cuassensors | cargo run --manifest-path rust-c2/Cargo.toml
```

To retain an append-only audit stream while running the Rust C2 process:

```bash
./cpp-sensors/build/cuassensors \
  | cargo run --manifest-path rust-c2/Cargo.toml -- --audit-log ./audit.ndjson
```

Events are schema-versioned and carry event IDs. Source files are changed only through an explicit developer or CI action; the running process does not rewrite its own code.

Run the Rust unit tests with:

```bash
cargo test --manifest-path rust-c2/Cargo.toml
```

The Rust side uses typed `serde`/`serde_json` models for observation validation and parsing.

## Safety boundary

The simulator is for software architecture, testing, and operator workflow research. Any real deployment would require qualified safety, legal, privacy, cybersecurity, and human-in-the-loop review. The policy engine only recommends benign actions such as `continue_observation`, `request_operator_review`, and `notify_site_security`.

## Reproducible scenarios

The C++ simulator can be recorded as an NDJSON scenario and replayed through the Rust engine:

```bash
./scripts/record_scenario.sh ./scenarios/baseline.ndjson
./scripts/replay_scenario.sh ./scenarios/baseline.ndjson ./audit.ndjson
```

A small checked-in fixture is available for quick checks:

```bash
./scripts/replay_scenario.sh ./scenarios/smoke.ndjson
```

Run the C++ deterministic test through CTest:

```bash
cmake -S cpp-sensors -B cpp-sensors/build
cmake --build cpp-sensors/build
ctest --test-dir cpp-sensors/build --output-on-failure
```

These scripts are explicit developer tools. They can overwrite the requested scenario or audit output paths, but they never overwrite source code or modify themselves at runtime.

## Milestone 3: local operator review

The read-only dashboard serves audit records from loopback only. It has no sensor controls, effector controls, aiming controls, or engagement APIs. The only state change it permits is a local in-memory acknowledgement of an advisory review item.

Generate an audit file, then start the dashboard in another terminal:

```bash
./scripts/replay_scenario.sh ./scenarios/smoke.ndjson ./audit.ndjson
cargo run --manifest-path rust-c2/Cargo.toml --bin dashboard -- --audit-log ./audit.ndjson --port 8080
```

Open `http://127.0.0.1:8080`. Acknowledgements reset when the dashboard process exits; the audit file remains append-only and unchanged.

## Milestone 4: resilience scenarios

Synthetic scenarios can be degraded deterministically before replay. The fault injector supports dropped observations, fixed timestamp delays, and contradictory low-confidence positions:

```bash
python3 scripts/fault_inject_scenario.py scenarios/smoke.ndjson \
  --drop-every 3 --delay-ms 500 --contradict-every 5 \
  --output /tmp/degraded.ndjson
./scripts/replay_scenario.sh /tmp/degraded.ndjson
./scripts/test_fault_injection.sh
```

These transformations are test fixtures only. They do not access real sensors or create control paths to any external system.
