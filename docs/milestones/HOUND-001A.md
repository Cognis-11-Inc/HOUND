# HOUND-001A — The First Scent

Implemented: a focused executable observation slice. See
[verification](../verification/HOUND-001A.md) for exact runtime evidence.

- Two candidate source containers send real HTTP messages through two relays to
  one destination. Neutral node IDs do not encode the private source assignment.
- The Rust `hound sniff` CLI queries read-only observation endpoints, validates
  records, preserves sensor provenance, and journals collection provenance.
- Observations and hypotheses have separate types. Hypothesis references are
  validated; no attribution or promotion to facts is performed.
- A network-disabled evaluator reads private ground truth and collected evidence
  through separate volume mounts and checks destination delivery.
- Native HTTP integration, Rust validation tests, and the Compose boundary probe
  exercise behavior and failure cases.

The Compose application was run with rootless Podman. Docker Engine and hosted
CI results are not claimed. This completes the implemented observation slice;
the larger HOUND-001 attribution milestone remains open.

Recommended next task: HOUND-001B, correlate message IDs and measured socket hops
into an evidence-linked trail, then produce provisional or unresolved hypotheses.
Define ambiguity and contradiction fixtures before adding source-attribution
decisions. Keep ground truth evaluator-only and defer learning until the
deterministic baseline has measured results.
