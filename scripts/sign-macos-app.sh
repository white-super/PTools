#!/bin/zsh

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_PATH="$PROJECT_ROOT/src-tauri/target/release/bundle/macos/PTools.app"
REQUIREMENT_PATH="$PROJECT_ROOT/src-tauri/macos-designated-requirement.txt"

if [[ ! -d "$APP_PATH" ]]; then
  print -u2 "macOS app bundle not found: $APP_PATH"
  exit 1
fi

codesign --force --deep --sign - --requirements "$REQUIREMENT_PATH" "$APP_PATH"
codesign --verify --deep --strict "$APP_PATH"
