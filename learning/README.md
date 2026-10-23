# Custom PyTorch learner

This Python package reserves the custom learning component. There is no LLM
dependency, model implementation, training loop, trained checkpoint, or live
Rust/Python bridge yet.

- `features/`: protocol-state and mathematical relationship representations.
- `models/`: custom PyTorch architectures.
- `training/`: offline training, validation, and baseline comparison.
- `tests/`: future tests for feature integrity and learner behavior.

The learner can propose investigative actions. Rust retains authority over
evidence, truth maintenance, and action admission. Evaluator answers must never
enter features, observations, or online policy inputs.

`pyproject.toml` declares PyTorch but does not pin a reproducible training
environment yet. Select and lock the platform-specific environment when an
authorized learning experiment is implemented. Scaffold checks require only
Python; importing the placeholder package does not import PyTorch.
