# HOUND architecture

Design intent for Cognis 11 Inc.'s research prototype. The repository currently
contains module boundaries and documentation, with no implemented hunt.

## Components and flow

```text
Isolated communicating hunting ground
          | observations / authorized actions
          v
Rust hunter: sensors -> evidence memory -> truth maintenance -> policy
                                                              ^
                                                              |
                                     Custom PyTorch action proposals

Hunter results -> Separate evaluator <- Hidden scenario ground truth
```

`hound-cli` depends on `hound-core`. `hound-eval` is a separate workspace member
and is not a dependency of the hunter. This dependency layout establishes a code
boundary; the laboratory runner must additionally enforce process, filesystem,
and network separation. No such enforcement is implemented in the scaffold.

## Evidence and authority

1. **Observation:** host B sent a packet to relay C at a measured time.
2. **Inference:** host B may be part of an intrusion path.
3. **Hypothesis:** host A may be the origin, with explicit confidence and support.

These are different kinds of records. Observations retain provenance; hypotheses
retain support, contradictions, and uncertainty. A confident model prediction
cannot rewrite measurements or turn a hypothesis into a fact. An investigation
may conclude unresolved when the visible evidence cannot identify an origin.

Rust owns the body, protocol sensors, investigative memory, truth maintenance,
and admitted actions. PyTorch learns from protocol states, relationships, and
hunting outcomes, and may propose the next action. There is no LLM dependency.

## Measurement before learning

HOUND-001 first requires real protocol communication, a deterministic baseline,
hidden evaluator answers, varied origins and topologies, and recorded metrics.
Only after measuring that baseline should a learned policy be compared with it.

Compare attribution accuracy, false accusations, investigation cost, calibration,
and justified uncertainty under the same visibility and action budget. Preserve
scenario provenance and evaluation separation so results remain interpretable.
