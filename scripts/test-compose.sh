#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
# A separate project scopes test containers, networks, and volumes.
export COMPOSE_PROJECT_NAME="hound001a-test-$$"
trap 'bash "$repo_root/scripts/lab.sh" down' EXIT
bash "$repo_root/scripts/lab.sh" demo
bash "$repo_root/scripts/lab.sh" check
bash "$repo_root/scripts/lab.sh" sniff
