#!/bin/zsh

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  print -u2 "run-macos-dev.sh only supports macOS"
  exit 1
fi

if [[ $# -eq 0 ]]; then
  print -u2 "Tauri did not provide a development binary"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REQUIREMENT_PATH="$PROJECT_ROOT/src-tauri/macos-designated-requirement.txt"
INFO_PLIST_PATH="$PROJECT_ROOT/src-tauri/macos-dev-Info.plist"
ICON_PATH="$PROJECT_ROOT/src-tauri/icons/icon.icns"

cargo "$@"

TARGET_DIRECTORY="$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
PROFILE="debug"
TARGET=""
NEXT_OPTION=""

for ARGUMENT in "$@"; do
  if [[ -n "$NEXT_OPTION" ]]; then
    if [[ "$NEXT_OPTION" == "target" ]]; then
      TARGET="$ARGUMENT"
    else
      PROFILE="$ARGUMENT"
    fi
    NEXT_OPTION=""
    continue
  fi

  case "$ARGUMENT" in
    --release) PROFILE="release" ;;
    --target) NEXT_OPTION="target" ;;
    --target=*) TARGET="${ARGUMENT#--target=}" ;;
    --profile) NEXT_OPTION="profile" ;;
    --profile=*) PROFILE="${ARGUMENT#--profile=}" ;;
  esac
done

if [[ -n "$TARGET" ]]; then
  TARGET_DIRECTORY="$TARGET_DIRECTORY/$TARGET"
fi

APP_BINARY="$TARGET_DIRECTORY/$PROFILE/ptools"
APP_BUNDLE_PATH="$TARGET_DIRECTORY/PTools.app"
APP_BUNDLE_EXECUTABLE="$APP_BUNDLE_PATH/Contents/MacOS/ptools"

if [[ ! -f "$APP_BINARY" ]]; then
  print -u2 "Tauri development binary not found: $APP_BINARY"
  exit 1
fi

mkdir -p "$APP_BUNDLE_PATH/Contents/MacOS" "$APP_BUNDLE_PATH/Contents/Resources"
cp "$APP_BINARY" "$APP_BUNDLE_EXECUTABLE"
cp "$INFO_PLIST_PATH" "$APP_BUNDLE_PATH/Contents/Info.plist"
cp "$ICON_PATH" "$APP_BUNDLE_PATH/Contents/Resources/icon.icns"

codesign \
  --force \
  --deep \
  --sign - \
  --requirements "$REQUIREMENT_PATH" \
  "$APP_BUNDLE_PATH"
codesign --verify --deep --strict "$APP_BUNDLE_PATH"

exec "$APP_BUNDLE_EXECUTABLE"
