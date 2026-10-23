# HOUND-001A: implemented architecture

This slice collects observations. It does not decide which source is an intruder.

```text
node-a ─┐
        ├─HTTP POST─> node-c ─HTTP POST─> node-d ─HTTP POST─> node-e
node-b ─┘            relay              relay              destination
                        │                 │                  │
                        └──── GET /observations (JSON) ───────┘
                                           │
                                      Rust HOUND
                                           │
                               evidence volume (JSONL)
                                           │ read-only
                                    private evaluator
                                           ↑ read-only
                         scenario volume (hidden ground truth)
```

## Terrain and scenario

`compose.yaml` defines an internal `terrain` network, five neutral nodes, a
one-shot HOUND investigator, a setup job, and a delivery evaluator.
`hunting-ground/containers/lab.py` uses Python's standard-library HTTP server.
Both source hosts remain alive after sending, keeping their network identities
distinct. Each sends three messages through `node-c`, `node-d`, and `node-e`.

The setup job creates a private scenario: a randomly chosen source emits
`probe`, `probe`, `transfer`; the other emits three `heartbeat` operations.
These are inert JSON messages, with no exploits or external targets. Payloads
contain only an opaque message ID and the operation being simulated. Relays
forward to a configured next hop; traffic cannot supply a forwarding URL.

Every receiving node records an event from a real accepted TCP socket. A receipt
is recorded before forwarding. Forwarding failure returns HTTP 502 but preserves
the receipt; receipt evidence alone does not prove downstream delivery. The
evaluator checks message-ID coverage at the destination and fails on incomplete
delivery. It does not score attribution or independently verify sensor honesty.

Scenario configuration is generated at runtime, not embedded in observations.
`HOUND_SCENARIO_SEED` may be supplied to the setup job for a reproducible role
assignment/message sequence. An unset or empty seed uses system randomness.
The seed is never passed to HOUND. A known seed can reveal the role assignment;
seed confidentiality is part of the trusted evaluation setup. Scenario IDs and
sensor instance IDs are fresh UUIDs. `down` removes the scenario and evidence.

## Rust observation pipeline

`hound-core::sensors::Sensor` abstracts observation retrieval. `HttpSensor` is the
only implementation. Its configured ID must match every returned observation.
The client permits HTTP with an explicit port and the exact `/observations`
path; it issues only GET, follows no redirects, and uses no proxy configuration.
It supports IPv4/DNS hosts, HTTP/1.0 or HTTP/1.1 responses with Content-Length,
and a one-MiB response cap with three-second socket timeouts. It does not support
TLS, IPv6 URL literals, chunked responses, or streaming. DNS resolution is handled
by the operating system. Configured hosts are trusted authorized targets;
the URL parser is not an authorization oracle.

Sensor envelopes have `schema_version: 1`. Observation deserialization rejects
unknown fields, invalid IP addresses, and out-of-range numeric values. Validation
checks identity/provenance consistency, nonzero timestamps and ports, capture
method, and the supported event vocabulary. Duplicate IDs in a batch or a sensor
ID mismatch fail the collection. This is structural validation, not independent
proof of a timestamp or event's truth.

`hound sniff` validates all configured sensor responses before opening its journal.
Each `EvidenceRecord` contains the original observation, its retrieval URL, and a
separate collection timestamp. The journal appends new identities and deduplicates
identical re-observations. A changed observation with an existing ID is an error;
the original record and collection provenance are retained. An incomplete final
line or corrupt existing record is rejected. The journal is a single-writer file,
not a transactional, signed, or tamper-proof ledger. A failed append may require
manual recovery; concurrent writers are unsupported.

`Hypothesis` is a separate Rust type with optional suspected origin, supporting
and contradicting observation references, and provisional/supported/rejected/
unresolved status. References must exist and cannot occur on both sides. There is
no hypothesis-to-observation conversion and no automatic hypothesis creation or
promotion. No inference engine or action-selection policy is implemented yet.

## Runtime boundaries

- HOUND mounts only the evidence volume. It has no scenario mount, scenario seed,
  evaluator result mount, Docker socket, or host filesystem bind mount.
- The setup job and evaluator use `network_mode: none`. Sources read the private
  scenario volume; relays and destination never mount it. The evaluator reads
  evidence and scenario read-only and prints a report to the host operator.
- Sensor routes are exactly GET `/health` and GET `/observations`. Unknown paths,
  file traversal, ground-truth paths, and query variants return 404. POST to the
  observation interface returns 405. There is no static file server or evaluation
  endpoint. POST `/ingest` is the separate traffic interface.
- All services run as UID/GID 10001 with a read-only root filesystem, all Linux
  capabilities dropped, `no-new-privileges`, PID/memory/CPU limits, and a bounded
  temporary filesystem. No service publishes ports or uses host networking.

The Compose network uses Docker's [internal network option](https://docs.docker.com/reference/compose-file/networks/#internal).
Restrictions use [Compose service controls](https://docs.docker.com/reference/compose-file/services/).
The scripts test these boundaries in HOUND's own image/network/volume context.

## Limits of isolation

The host operator, container engine, source generators, and sensor implementations
are trusted. Host administrators can inspect volumes or change Compose definitions.
Containers share the host kernel; this is a controlled simulation, not a hardened
sandbox for hostile code. Internal networking does not isolate containers from
every host-local service or prevent all host-mediated communication. Build-time
image/dependency downloads use external networks. Base image tags are versioned
but not digest-pinned.

Sensors have no authentication, TLS, or signatures. HOUND shares the terrain and
could reach `/ingest` if its program were replaced; the implemented sensor client
only issues read requests. A compromised sensor could lie. Its reported timestamps
are wall-clock values, not a globally ordered clock. Events are in-memory, capped
at 1,000 per sensor; restart loses them and assigns a new sensor instance ID.
The current topology is fixed; both sources use the same relay chain. Learning,
path selection, attribution, calibration, and reward mechanisms remain future work.

The Rust `hound-eval` crate and the PyTorch package remain scaffolds. The active
evaluator is the network-disabled Python `evaluate` mode, not the Rust crate.
