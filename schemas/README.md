# Data contracts

The sensor response is JSON: `{"schema_version": 1, "observations": [...]}`.
The executable contract is defined by Rust types in `hound-core/src/sensors/`
and checked against real Python sensor responses in the integration tests.
See `tests/fixtures/observation.json` for a labeled test fixture, never live evidence.

An observation contains an ID, sensor ID, Unix timestamp in milliseconds, HTTP
event, source/destination socket endpoints, and capture provenance. The event
contains protocol/method/path, opaque message ID, operation, and payload size.
Provenance records `http_server_receive`, a sensor instance ID, and sequence.
Identity is `SENSOR:INSTANCE:SEQUENCE`. No role label or evaluator answer is allowed.

The JSONL journal wraps each observation in `EvidenceRecord`, adding `collection`
with `sensor_url` and `collected_at_unix_ms` without changing the observation.
Unknown fields and unsupported schema versions are rejected.

Hypotheses are a separate Rust type with support/contradiction references and
provisional/supported/rejected/unresolved status. Inference, action, and learned
policy contracts remain future work. No JSON Schema document is implemented yet.
