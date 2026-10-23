# Development scripts

- `bash scripts/lab.sh demo`: build both images, launch receivers and sources,
  collect Rust evidence, and run the private delivery evaluator.
- `bash scripts/lab.sh check`: test read-only sensors and hidden-answer absence
  from HOUND's image/network/mount context.
- `bash scripts/lab.sh sniff`: collect again without generating traffic.
- `bash scripts/lab.sh evaluate`: evaluate current evidence separately.
- `bash scripts/lab.sh down`: remove this project's containers/network/data volumes.
- `bash scripts/test-compose.sh`: use a unique test project and clean it up on exit.

Docker Compose is the default. `HOUND_COMPOSE_BIN` may point to a standalone client;
`DOCKER_HOST` can select a compatible engine. This does not install a system plugin.
