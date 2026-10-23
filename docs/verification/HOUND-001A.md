# HOUND-001A verification — 2026-10-23

Verified locally in the project worktree. No hosted CI result is claimed for
these changes. The existing repository remote remains `Cognis-11-Inc/HOUND`.

## Native checks

Host tools: Rust/Cargo 1.99.1; Python 3.14.8. The container build additionally
compiled the release binary using Rust 1.85 and ran the lab with Python 3.11.

| Exact command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo check --workspace --locked --offline` | Passed |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Passed |
| `cargo test --workspace --locked --offline` | 7 tests passed; no failures |
| `cargo build --locked --offline -p hound-cli` | Passed |
| `python3 -m unittest discover -s tests/integration -p 'test_*.py' -v` | 5 tests passed; no failures |
| `python3 -m compileall -q learning/src hunting-ground/containers scripts tests/integration` | Passed |
| `bash -n scripts/lab.sh scripts/test-compose.sh` | Passed |
| `git diff --check` | Passed |

Rust tests cover parsing, validation, provenance round trips, hidden/unknown field
rejection, sensor identity/version checks, hypothesis reference separation,
deduplication, conflicting evidence refusal, and real HTTP framing/redirect refusal.
Python tests use separate HTTP processes and the built Rust CLI. They exercise
actual forwarding, collection, provenance preservation, repeated sniffing, absent
answer routes, read-only sensors, empty environments, failed sensors, and failed
forwarding. A retained receipt is not mistaken for destination delivery.

## Container run

The Docker client is installed, but the default Docker daemon socket is absent
and the `docker compose` plugin is unavailable. Docker Engine integration was
therefore **not run**. Rootless Podman 6.1.4 was usable; a temporary standalone Docker
Compose v5.6.1 client connected to its compatibility API instead. No system Docker
configuration was changed.

The actual commands used for this compatible-engine run were:

```sh
podman system service --time=0 unix:///tmp/hound001a-tools/podman.sock
```

In a separate process:

```sh
DOCKER_HOST=unix:///tmp/hound001a-tools/podman.sock \
  /tmp/hound001a-tools/docker-compose --profile tools config --quiet

DOCKER_HOST=unix:///tmp/hound001a-tools/podman.sock \
  DOCKER_BUILDKIT=0 COMPOSE_BAKE=false \
  HOUND_COMPOSE_BIN=/tmp/hound001a-tools/docker-compose \
  bash scripts/test-compose.sh
```

Both commands passed. The temporary Compose client path is session-specific;
normal Docker users run `bash scripts/test-compose.sh` directly. Podman/Compose
success is evidence of the application running in containers, not proof that a
Docker Engine run has passed.

The test built both images, created the internal terrain, ran both source hosts
with distinct network addresses, and collected these results:

- Six synthetic messages crossed two relays and reached the destination.
- Each of the three receiving sensors reported six real socket-receipt events.
- First sniff retrieved and recorded 18 observations.
- A second sniff retrieved 18 and appended zero new records.
- The private evaluator reported six expected and six observed destination
  messages, `delivery_complete: true`, and `attribution_evaluated: false`.
- The probe ran using HOUND's own image, network, and volume configuration.
  Private scenario data and the Docker socket were absent; evaluator access and
  ground-truth/traversal/query routes were unavailable; observation POST was denied.
  UID 10001, zero effective capabilities, `no-new-privileges`, a read-only root
  filesystem, and absence of the private scenario seed were also confirmed.
- The script removed its test containers, internal network, and data volumes.

The runtime sensor IDs, message IDs, timestamps, and socket endpoints were generated
by the run. `tests/fixtures/observation.json` was not used as live sensor evidence.
The ignored local transcript is `artifacts/compose-test.log`.

## Limits

The network is fixed, the HTTP services are trusted and unauthenticated, and the
single-writer journal is not tamper-proof or transactionally crash-safe. Sensors
store observations in memory and lose them at restart. The evaluator measures
delivery coverage, not attribution, calibration, or model performance. There is no
neural integration, GUI, reward system, or complete truth-maintenance engine.
See [architecture](../ARCHITECTURE.md) for the full trust boundary.

The CI workflow now includes Rust tests, real HTTP process tests, and a Docker
Compose integration job. Those jobs need a future push/run before their hosted
results can be reported.
