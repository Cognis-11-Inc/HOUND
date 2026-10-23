# Test fixtures

`observation.json` is a static, non-secret parsing-test example. It is never used
as live sensor input. Real HTTP integration tests generate their own messages and
retrieve observations produced by socket communication. Hidden evaluator answers
do not belong in hunter-visible fixtures.
