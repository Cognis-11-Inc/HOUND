# HOUND-001 — The Hunting Ground

**Status: in progress.** HOUND-001A implements first HTTP observation collection;
the full HOUND-001 milestone remains incomplete. See [architecture](../ARCHITECTURE.md)
and [verification](../verification/HOUND-001A.md).

## Objective

Build a reproducible, isolated network where HOUND actively gathers observations,
maintains competing hypotheses, and attempts to locate a simulated intrusion's
origin without receiving the answer.

## Definition of done

- [x] Reproducible, isolated containers exchange real protocol traffic (Compose on
      rootless Podman verified; Docker Engine validation remains pending).
- [ ] The environment includes several hosts/relays, a legitimate route, a
      misleading route, and a simulated intrusion trail.
- [x] A Rust hunter gathers observations through authorized sensor queries.
- [ ] A deterministic baseline attempts source attribution from visible evidence.
- [ ] Observations, inferences, and hypotheses remain distinct and traceable.
- [x] A separate delivery evaluator holds private ground truth outside HOUND's
      mounts/network interfaces; tested within the documented trust boundary.
- [ ] Test scenarios vary the origin and network topology.
- [ ] Results measure attribution accuracy, false accusations, investigation cost,
      calibration, and justified uncertainty.
- [ ] Ambiguous telemetry can produce an explicit unresolved result.
- [ ] A documented command reproduces the lab, baseline run, and evaluation.

## Implementation sequence

1. Define versioned observation/action/result contracts and evidence semantics.
2. Build and verify the isolated communicating laboratory and evaluator boundary.
3. Implement sensors, evidence memory, and the deterministic investigation policy.
4. Run varied scenarios and characterize the baseline with recorded metrics.
5. Design a custom PyTorch policy experiment for comparison after baseline acceptance.

## Acceptance evidence

Record setup/version information, scenario identifiers, visible observations,
investigation actions and costs, final hypotheses or unresolved outcome, and
evaluator metrics. Keep hidden answers out of the hunter's accessible runtime.

Compilation, placeholder modules, static logs, and an untrained model do not
demonstrate completion of a hunt. No hunting performance is claimed yet.
