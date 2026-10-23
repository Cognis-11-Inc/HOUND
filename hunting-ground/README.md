# The Hunting Ground

HOUND-001A's communicating HTTP laboratory is defined by the root `compose.yaml`.
Run `bash scripts/lab.sh demo` from the repository root; clean up with
`bash scripts/lab.sh down` (this also deletes scenario/evidence volumes).

- `containers/`: Python HTTP terrain, source traffic, private delivery evaluator,
  and Dockerfiles for the lab and Rust HOUND image.
- `topologies/`: the current fixed relay chain and future topology work.
- `scenarios/`: randomly assigned synthetic intrusion-like versus benign traffic.

The lab must use real protocol communication between controlled machines or
containers. Log fixtures support tests but cannot substitute for the lab.
See [implemented boundaries and limitations](../docs/ARCHITECTURE.md) and
[verification](../docs/verification/HOUND-001A.md) for the runtime evidence.

Scenario answers belong to the evaluator. Do not expose answer files, scenario
seeds that reveal the origin, shared runtime mounts, or evaluator configuration
to the hunter. HOUND mounts no private scenario volume; the evaluator has no network.
