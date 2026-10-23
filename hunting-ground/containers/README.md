# Laboratory containers

`lab.py` implements the receiving/forwarding HTTP nodes, source traffic generation,
private scenario initialization, and a delivery-only evaluator. It uses the Python
standard library. `Dockerfile.lab` packages it; `Dockerfile.hound` builds the Rust
CLI and packages the boundary probe. Root `compose.yaml` applies runtime restrictions.

No exploit or malware is executed. `probe`, `transfer`, and `heartbeat` are inert
JSON operations. Evaluation measures receipt coverage, not source attribution.
