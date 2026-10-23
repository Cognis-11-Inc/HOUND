# The Hunting Ground

This directory will hold HOUND-001's reproducible, isolated network laboratory.
No containers, topology, traffic generators, or intruder are implemented yet.

- `containers/`: host, relay, traffic, and simulated-intruder images/services.
- `topologies/`: actual communicating networks with legitimate and misleading routes.
- `scenarios/`: varied origins, traffic patterns, decoys, and reproducible seeds.

The lab must use real protocol communication between controlled machines or
containers. Log fixtures support tests but cannot substitute for the lab.
Network isolation must be implemented and verified before enabling investigation.

Scenario answers belong to the evaluator. Do not expose answer files, scenario
seeds that reveal the origin, shared runtime mounts, or evaluator configuration
to the hunter. A future runner must enforce this separation.
