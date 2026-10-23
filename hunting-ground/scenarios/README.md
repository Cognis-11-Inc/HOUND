# Hunting scenarios

The network-disabled setup job generates `ground-truth.json` in a private named
volume. One randomly chosen source emits `probe`, `probe`, `transfer`; the other
emits three `heartbeat` messages. Each message is actually sent through both relays.
Only opaque message IDs and the observed operation travel in payloads.

Source containers read the private scenario. HOUND and sensors do not mount it.
The separate evaluator reads it alongside HOUND's evidence to check destination
coverage. No generated answer file is committed. Optional `HOUND_SCENARIO_SEED`
controls role assignment/message IDs in the setup job; keep it private from HOUND.
