#!/bin/zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
COMMAND="${1:-}"

if [[ "$(uname -s)" == "Darwin" && "$COMMAND" == "dev" ]]; then
  shift
  for ARGUMENT in "$@"; do
    if [[ "$ARGUMENT" == "-r" || "$ARGUMENT" == "--runner" || "$ARGUMENT" == --runner=* ]]; then
      exec tauri dev "$@"
    fi
  done
  exec tauri dev --runner "$SCRIPT_DIR/run-macos-dev.sh" "$@"
fi

exec tauri "$@"
