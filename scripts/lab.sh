#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
compose() {
    if [[ -n "${HOUND_COMPOSE_BIN:-}" ]]; then
        "$HOUND_COMPOSE_BIN" -f "$repo_root/compose.yaml" "$@"
    else
        docker compose -f "$repo_root/compose.yaml" "$@"
    fi
}
case "${1:-}" in
    demo)
        compose --profile tools build setup hound
        compose up -d --wait node-c node-d node-e
        compose run --rm --no-deps setup
        compose up -d --wait node-a node-b
        compose run --rm --no-deps hound
        compose run --rm --no-deps evaluator
        ;;
    sniff) compose run --rm --no-deps hound ;;
    check) compose run --rm --no-deps --entrypoint python hound /app/check_boundary.py ;;
    evaluate) compose run --rm --no-deps evaluator ;;
    down) compose --profile tools down --volumes --remove-orphans ;;
    *) echo "Usage: bash scripts/lab.sh {demo|sniff|check|evaluate|down}" >&2; exit 2 ;;
esac
