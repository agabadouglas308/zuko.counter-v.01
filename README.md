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

Run the Rust unit tests with:

```bash
cargo test --manifest-path rust-c2/Cargo.toml
```

The Rust side uses typed `serde`/`serde_json` models for observation validation and parsing.

## Safety boundary

The simulator is for software architecture, testing, and operator workflow research. Any real deployment would require qualified safety, legal, privacy, cybersecurity, and human-in-the-loop review. The policy engine only recommends benign actions such as `continue_observation`, `request_operator_review`, and `notify_site_security`.
