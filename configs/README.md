# Configuration

Lab configuration lives in root `compose.yaml`. HOUND defaults to the three
laboratory sensor endpoints at `http://node-{c,d,e}:8080/observations`.
The native CLI accepts repeated `--sensor ID=http://HOST:PORT/observations` options
and `--output PATH`. Configuration is trusted and must name authorized targets.

The setup job alone accepts the optional private `HOUND_SCENARIO_SEED` environment
variable. HOUND receives neither this seed nor the generated answer file.
