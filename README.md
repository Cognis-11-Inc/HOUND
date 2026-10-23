# HOUND

**Cognis 11 Inc. — Autonomous Cyber-Tracking Intelligence**

A Rust-based, protocol-native cyber-tracking intelligence with a custom PyTorch
learning model and truth maintenance, designed to pursue intrusion trails through
authorized simulated network environments.

> Give me a trail, give me a terrain, and let me hunt.

## Status

Research prototype: **HOUND-001A — The First Scent** implements HTTP observation
collection in a controlled network. Two source hosts send synthetic messages
through two relays to a destination; the Rust `hound sniff` command retrieves
sensor observations and appends validated evidence with provenance to a JSONL journal.

HOUND does not attribute an intruder yet. Neural learning, intrinsic hunting
motivation, and handler reward are future work. There is no LLM dependency.

The first milestone is [HOUND-001 — The Hunting Ground](docs/milestones/HOUND-001.md).
It establishes a reproducible environment and a measurable deterministic hunter
before introducing a learned policy.

## Project structure

```text
HOUND/
├── .github/workflows/         # Rust, HTTP, and Compose integration checks
├── crates/
│   ├── hound-core/src/
│   │   ├── sensors/          # Validated, read-only HTTP observations
│   │   ├── policy/           # Future baseline investigation policy
│   │   ├── memory/           # Append-only JSONL evidence journal
│   │   └── truth/            # Separate hypothesis types and reference validation
│   ├── hound-cli/src/        # Hunter command-line entry point
│   └── hound-eval/src/       # Reserved for later Rust attribution evaluation
├── learning/
│   ├── src/hound_learning/
│   │   ├── features/         # Protocol-state and relationship encodings
│   │   ├── models/           # Custom PyTorch models
│   │   └── training/         # Offline training and policy comparison
│   └── tests/                # Future learner tests
├── hunting-ground/
│   ├── containers/           # HTTP terrain, traffic generator, private evaluator
│   ├── topologies/           # Current topology documentation
│   └── scenarios/            # Private scenario generation documentation
├── configs/                  # Hunter and laboratory configuration
├── schemas/                  # Current observation/evidence contract documentation
├── tests/
│   ├── integration/          # Real HTTP process tests
│   └── fixtures/             # Non-secret test inputs
├── scripts/                  # Demo lifecycle and container boundary checks
├── docs/                     # Architecture and milestone acceptance criteria
├── compose.yaml              # Internal network; no published ports or Docker socket
└── artifacts/                # Ignored local captures, runs, data, and weights
```

The Python learning package and Rust policy/evaluation modules remain placeholders.

## Run the demonstration

Requires a running Docker Engine and Docker Compose v2.20+ (or a compatible
Compose client). Image builds fetch base images and Rust dependencies; the
running laboratory uses an internal network.

```sh
bash scripts/lab.sh demo      # Build, launch, generate traffic, sniff, evaluate
bash scripts/lab.sh check     # Probe isolation from HOUND's container context
bash scripts/lab.sh sniff     # Retrieve again; existing identities are deduplicated
bash scripts/lab.sh down      # Remove lab containers, network, and both data volumes
```

Expected first-run summary: **18 observations**, from six real messages received
at three sensors. The separate evaluator should report six observed destination
messages, `delivery_complete: true`, and `attribution_evaluated: false`.

Run `down` before a fresh scenario. Existing runs retain the private scenario and
evidence volumes; `sniff` does not generate new traffic. Sensor observations are
in memory and are lost on sensor restart. The journal persists until `down`.

The underlying hunter command is:

```sh
docker compose run --rm --no-deps hound sniff --output /evidence/observations.jsonl
```

The evaluator has no network and reads private scenario and evidence volumes.
HOUND receives only the evidence volume. A network-disabled setup job assigns
the synthetic intrusion pattern randomly to `node-a` or `node-b`; no role label
appears in sensor output. See [the architecture](docs/ARCHITECTURE.md) for exact
boundaries and limitations and [verification](docs/verification/HOUND-001A.md)
for the engines and checks actually exercised.

## Development

Rust 1.85 or newer and Python 3.11 or newer are required for native development.

```sh
cargo check --workspace --locked --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo build --locked --offline -p hound-cli
python3 -m unittest discover -s tests/integration -p 'test_*.py' -v
bash scripts/test-compose.sh
```

On a fresh machine run `cargo fetch --locked` first; subsequent Rust commands
can use `--offline`. The Python terrain and integration tests use only the
standard library. PyTorch is not needed for this milestone.

For explicitly authorized local sensors, the native CLI supports repeated
`--sensor ID=http://HOST:PORT/observations` options. Default Compose hostnames do
not resolve from the host, and the lab publishes no ports. The native integration
tests start separate HTTP processes on loopback; they do not prove container isolation.

## Engineering boundaries

- Rust owns sensing, investigative actions, evidence memory, and truth maintenance.
- The learner may propose where HOUND hunts next; it may not rewrite observations.
- Observations, inferences, and hypotheses remain distinct, with provenance and
  explicit uncertainty. Confidence does not turn a hypothesis into a fact.
- The evaluator owns hidden ground truth in a volume HOUND does not mount. Its
  service has `network_mode: none` and no HTTP interface. The current delivery
  evaluator is Python; the Rust `hound-eval` crate is reserved for future work.
- HOUND must be able to report **unresolved** when telemetry cannot identify an origin.
- All investigation targets belong to an explicitly authorized, isolated laboratory.

See [the doctrine](docs/HOUND_DOCTRINE.md) for the foundational design laws.

Copyright © 2026 Cognis 11 Inc. No open-source license has been selected.
