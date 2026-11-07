#!/bin/bash
# SPDX-License-Identifier: MIT
# Signs, notarises and zips the RadarVU macOS bundles for the Plugin Manager, following the wiki's
# Make-Plugin-Release Step 5.4: bundle zip + opengrid-manifest.json, no pkg, no dmg, nothing on GitHub.
# Run installer/build_macos.sh first (it builds the universal bundles and stamps the build class).
# The result is staged in dist/release-<version>/bundles; it is copied to "Release Binaries" only after the
# user approves, and RELEASE-READY and the upload are the user's.
#   INCLUDE_STANDALONE=1 (default) or 0;  SKIP_SIGN=1 reuses bundles already signed in the build tree
set -euo pipefail
export COPYFILE_DISABLE=1
cd "$(dirname "$0")/.."
ROOT=$PWD
VERSION=$(sed -n 's/^project(RadarVU VERSION \([0-9.]*\)).*/\1/p' CMakeLists.txt)
ART="$ROOT/build/release-macos/RadarVU_artefacts/Release"
OUT="$ROOT/dist/release-$VERSION/bundles"
APP_ID=${RADARVU_APP_ID:-"Developer ID Application: Marcos Deida (ZQYU67JUBH)"}
PACE_ACCOUNT=${RADARVU_PACE_ACCOUNT:-mistamin}
AAX_WCGUID=${RADARVU_AAX_WCGUID:-"77661BB0-D60F-11F1-804C-005056928F3B"}
SKILL_DIR=${SKILL_DIR:-"/Volumes/Developer/DSP Engineering Wiki/skills/Make-Plugin-Release"}
INCLUDE_STANDALONE=${INCLUDE_STANDALONE:-1}
ZIP="RadarVU-$VERSION-macos.zip"

if [ "${SKIP_SIGN:-0}" = 1 ]; then
  echo "==> Reusing the bundles signed by the previous run (SKIP_SIGN=1)"
  for b in "$ART/VST3/RadarVU.vst3" "$ART/AU/RadarVU.component" "$ART/CLAP/RadarVU.clap" "$ART/AAX/RadarVU.aaxplugin"; do
    codesign --verify --deep --strict "$b"
  done
else
echo "==> Sign bundles (Developer ID, hardened runtime, timestamp)"
for b in "$ART/VST3/RadarVU.vst3" "$ART/AU/RadarVU.component" "$ART/CLAP/RadarVU.clap"; do
  codesign --force --deep --timestamp --options runtime --sign "$APP_ID" "$b"
  codesign --verify --deep --strict --verbose=2 "$b"
done
find "$ART/LV2/RadarVU.lv2" -type f \( -name '*.dylib' -o -name '*.so' \) -print0 | while IFS= read -r -d '' f; do
  codesign --force --timestamp --options runtime --sign "$APP_ID" "$f"
  codesign --verify --strict --verbose=2 "$f"
done
echo "==> AAX: Developer ID, then PACE Eden signature"
"$SKILL_DIR/scripts/sign_aax.sh" "$ART/AAX/RadarVU.aaxplugin" "$AAX_WCGUID" "$PACE_ACCOUNT" "$APP_ID"

fi

rm -rf "$OUT"; mkdir -p "$OUT/root" "$OUT/work"
if [ "$INCLUDE_STANDALONE" = 1 ]; then
  echo "==> Standalone: sign, notarise, staple"
  codesign --force --deep --timestamp --options runtime --entitlements installer/standalone.entitlements \
    --sign "$APP_ID" "$ART/Standalone/RadarVU.app"
  codesign --verify --deep --strict --verbose=2 "$ART/Standalone/RadarVU.app"
  ditto -c -k --keepParent "$ART/Standalone/RadarVU.app" "$OUT/work/RadarVU-app.zip"
  xcrun notarytool submit "$OUT/work/RadarVU-app.zip" --keychain-profile AC_NOTARY --wait
  xcrun stapler staple "$ART/Standalone/RadarVU.app"
fi

echo "==> Stage the zip payload"
R="$OUT/root"; P="$R/Library/Audio/Plug-Ins"
mkdir -p "$P/VST3" "$P/Components" "$P/CLAP" "$P/LV2" "$R/Library/Application Support/Avid/Audio/Plug-Ins"
ditto "$ART/VST3/RadarVU.vst3" "$P/VST3/RadarVU.vst3"
ditto "$ART/AU/RadarVU.component" "$P/Components/RadarVU.component"
ditto "$ART/CLAP/RadarVU.clap" "$P/CLAP/RadarVU.clap"
ditto "$ART/LV2/RadarVU.lv2" "$P/LV2/RadarVU.lv2"
ditto "$ART/AAX/RadarVU.aaxplugin" "$R/Library/Application Support/Avid/Audio/Plug-Ins/RadarVU.aaxplugin"
if [ "$INCLUDE_STANDALONE" = 1 ]; then mkdir -p "$R/Applications"; ditto "$ART/Standalone/RadarVU.app" "$R/Applications/RadarVU.app"; fi
find "$R" \( -name '._*' -o -name '.DS_Store' \) -delete

echo "==> Zip and notarise"
mkdir -p "$OUT/release"
# --norsrc --noextattr --noacl: no AppleDouble (._*) entries in the archive
ditto -c -k --norsrc --noextattr --noacl "$R" "$OUT/release/$ZIP"
if unzip -l "$OUT/release/$ZIP" | grep -qE "__MACOSX|/\._|^\s*[0-9]+\s+\S+\s+\S+\s+\._"; then echo "AppleDouble entries in the zip" >&2; exit 1; fi
xcrun notarytool submit "$OUT/release/$ZIP" --keychain-profile AC_NOTARY --wait

echo "==> Manifest"
python3 tools/write_manifest.py "$OUT/release" "$VERSION" 11.0
if [ "$INCLUDE_STANDALONE" = 0 ]; then
  python3 - "$OUT/release/opengrid-manifest.json" <<'PY'
import json, sys
p = sys.argv[1]; m = json.load(open(p))
m["archives"][0]["installs"] = [i for i in m["archives"][0]["installs"] if i["format"] != "app"]
json.dump(m, open(p, "w"), indent=2)
PY
fi
echo "done: $OUT/release"
