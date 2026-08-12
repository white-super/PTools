#!/bin/zsh

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  print -u2 "dev-macos.sh only supports macOS"
  exit 1
fi

exec pnpm tauri dev "$@"
