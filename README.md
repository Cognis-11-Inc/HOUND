# HOUND

**Cognis 11 Inc. — Autonomous Cyber-Tracking Intelligence**

A Rust-based, protocol-native cyber-tracking intelligence with a custom PyTorch
learning model and truth maintenance, designed to pursue intrusion trails through
authorized simulated network environments.

> Give me a trail, give me a terrain, and let me hunt.

## Status

Research prototype: initial project structure only. The Rust workspace builds, but
protocol sensing, investigation, source attribution, the network laboratory,
evaluation, and model training are not implemented yet. There is no LLM dependency.

The first milestone is [HOUND-001 — The Hunting Ground](docs/milestones/HOUND-001.md).
It establishes a reproducible environment and a measurable deterministic hunter
before introducing a learned policy.

## Project structure

```text
HOUND/
├── .github/workflows/         # Scaffold build checks
├── crates/
│   ├── hound-core/src/
│   │   ├── sensors/          # Authorized protocol observations
│   │   ├── policy/           # Investigation decisions and baseline policy
│   │   ├── memory/           # Evidence storage and provenance
│   │   └── truth/            # Inferences, hypotheses, uncertainty
│   ├── hound-cli/src/        # Hunter command-line entry point
│   └── hound-eval/src/
│       ├── ground_truth/     # Evaluator-only answer handling
│       └── metrics/          # Accuracy, cost, false accusations, calibration
├── learning/
│   ├── src/hound_learning/
│   │   ├── features/         # Protocol-state and relationship encodings
│   │   ├── models/           # Custom PyTorch models
│   │   └── training/         # Offline training and policy comparison
│   └── tests/                # Future learner tests
├── hunting-ground/
│   ├── containers/           # Real communicating hosts and relays
│   ├── topologies/           # Isolated network layouts
│   └── scenarios/            # Legitimate traffic, decoys, simulated intrusions
├── configs/                  # Hunter and laboratory configuration
├── schemas/                  # Versioned observation/action/result contracts
├── tests/
│   ├── integration/          # Future end-to-end lab checks
│   └── fixtures/             # Non-secret test inputs
├── scripts/                  # Future setup, lab, and evaluation commands
├── docs/                     # Architecture and milestone acceptance criteria
└── artifacts/                # Ignored local captures, runs, data, and weights
```

Directory markers and module documentation reserve these responsibilities; they
do not constitute an implemented hunter or laboratory.

## Development

Rust 1.85 or newer and Python 3.11 or newer are required for the scaffold.

```sh
cargo check --workspace --locked --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
python3 -m compileall -q learning/src
cargo run --locked --offline -p hound-cli
```

The CLI currently prints the scaffold status. CI checks the Rust scaffold and
Python syntax; behavioral tests will accompany the first implemented features.
The optional Python package declares PyTorch as a dependency. Training setup,
dependency locking, and model selection remain future work.

## Engineering boundaries

- Rust owns sensing, investigative actions, evidence memory, and truth maintenance.
- The learner may propose where HOUND hunts next; it may not rewrite observations.
- Observations, inferences, and hypotheses remain distinct, with provenance and
  explicit uncertainty. Confidence does not turn a hypothesis into a fact.
- The evaluator owns hidden ground truth. Its crate is not a hunter dependency;
  runtime isolation and access controls still need to be implemented and tested.
- HOUND must be able to report **unresolved** when telemetry cannot identify an origin.
- All investigation targets belong to an explicitly authorized, isolated laboratory.

See [the architecture](docs/architecture.md) for the intended data flow.

Copyright © 2026 Cognis 11 Inc. No open-source license has been selected.
