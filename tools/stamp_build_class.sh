#!/bin/bash
# Marks an app or bundle as a "modern" or "legacy" build, in its Info.plist:
#   OGBuildClass  modern | legacy
#   OGMinOS       the lowest system it runs on (macOS "11.0", "10.12", "10.6")
# Usage: stamp-build-class.sh <bundle.app|.bundle|Info.plist> <modern|legacy> <min_os>
# Run it BEFORE signing; changing the Info.plist afterwards breaks the signature.
set -euo pipefail
target="${1:?bundle or Info.plist}"; class="${2:?modern|legacy}"; min_os="${3:?minimum macOS, e.g. 11.0}"
case "$class" in modern|legacy) ;; *) echo "build class must be modern or legacy" >&2; exit 2 ;; esac
plist="$target"
[ -d "$target" ] && plist="$target/Contents/Info.plist"
[ -f "$plist" ] || { echo "no Info.plist at $plist" >&2; exit 2; }
for pair in "OGBuildClass:$class" "OGMinOS:$min_os"; do
  key="${pair%%:*}"; value="${pair#*:}"
  /usr/libexec/PlistBuddy -c "Set :$key $value" "$plist" 2>/dev/null || /usr/libexec/PlistBuddy -c "Add :$key string $value" "$plist"
done
