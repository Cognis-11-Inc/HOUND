# Integration tests

Build `cargo build --locked -p hound-cli`, then run:

```sh
python3 -m unittest discover -s tests/integration -p 'test_*.py' -v
```

These tests launch separate HTTP processes on loopback. They check actual
forwarding, CLI collection, provenance, deduplication, absent ground-truth routes,
partial-failure behavior, and empty environments. They do not prove container
isolation. `bash scripts/test-compose.sh` additionally exercises the Compose
containers and the boundary probe in HOUND's own runtime context.
