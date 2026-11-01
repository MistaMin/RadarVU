#!/bin/bash
# SPDX-License-Identifier: MIT
# Builds, tests and packages RadarVU for macOS (universal arm64 + x86_64): VST3, AU, CLAP, LV2, AAX and the
# Standalone app, as one bundles archive plus opengrid-manifest.json (no installer package, no disk image).
#
#   installer/build_macos.sh                       unsigned (local checks)
#   SIGN=1 installer/build_macos.sh                sign the bundles and the Standalone, PACE-sign the AAX
#
# Run SIGN=1 from your own Terminal: codesign and wraptool can need the keychain or iLok prompt, which an
# unattended shell cannot answer. The Signing iLok must be plugged in for the AAX.
# The result is staged in "Release Binaries/<owner>/RadarVU/<version>/". The empty RELEASE-READY file is
# yours to create after you have checked the archive.
set -euo pipefail
export COPYFILE_DISABLE=1
cd "$(dirname "$0")/.."
ROOT=$PWD
VERSION=$(sed -n 's/^project(RadarVU VERSION \([0-9.]*\)).*/\1/p' CMakeLists.txt)
BUILD=${BUILD:-build/release-macos}
OUT=${OUT:-dist/release-$VERSION/macos}
ART="$ROOT/$BUILD/RadarVU_artefacts/Release"
APP_ID=${RADARVU_APP_ID:-"Developer ID Application: Marcos Deida (ZQYU67JUBH)"}
PACE_ACCOUNT=${RADARVU_PACE_ACCOUNT:-mistamin}
AAX_WCGUID=${RADARVU_AAX_WCGUID:-"77661BB0-D60F-11F1-804C-005056928F3B"}   # RadarVU, Signing Only (PACE Central)
WRAPTOOL=${RADARVU_WRAPTOOL:-/Applications/PACEAntiPiracy/Eden/Fusion/Current/bin/wraptool}
RELEASE_DIR=${RELEASE_DIR:-"/Volumes/Developer/OpenGrid/Release Binaries/MistaMin/RadarVU/$VERSION"}
MANIFEST_TOOL=${MANIFEST_TOOL:-"$ROOT/../OpenGridder-Rust/tools/opengrid_manifest.py"}
export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"

echo "==> RadarVU $VERSION: configure and build (universal)"
cmake -S . -B "$BUILD" -G Ninja -DCMAKE_BUILD_TYPE=Release -DRADARVU_UNIVERSAL=ON -DRADARVU_BUILD_QA=ON
cmake --build "$BUILD" --config Release

echo "==> Tests"
CARGO_TARGET_DIR="$ROOT/$BUILD/dsp-test" cargo test --release --manifest-path dsp/Cargo.toml --offline --lib
QA="$BUILD/RadarVU_QA_artefacts/Release/RadarVU_QA"
SCRATCH=$(mktemp -d)
"$QA" "$SCRATCH/ui" > "$SCRATCH/ui.log" 2>&1 || { tail -20 "$SCRATCH/ui.log"; echo "UI audit failed" >&2; exit 1; }
grep -E "state round trip:|visual sweep:|toggles:" "$SCRATCH/ui.log"

for f in VST3/RadarVU.vst3 AU/RadarVU.component CLAP/RadarVU.clap LV2/RadarVU.lv2 Standalone/RadarVU.app; do
  [ -e "$ART/$f" ] || { echo "missing $f" >&2; exit 1; }
done
AAX=0; [ -d "$ART/AAX/RadarVU.aaxplugin" ] && AAX=1

# Mark the build class and minimum system in each bundle's Info.plist (before signing: editing it later
# breaks the signature). The manifest tool reads these and checks them against the Mach-O load commands.
STAMP=${STAMP_TOOL:-"$ROOT/../OpenGridder-Rust/scripts/stamp-build-class.sh"}
for b in "$ART/VST3/RadarVU.vst3" "$ART/AU/RadarVU.component" "$ART/CLAP/RadarVU.clap" "$ART/Standalone/RadarVU.app"; do
  "$STAMP" "$b" modern 11.0
done
[ "$AAX" = 1 ] && "$STAMP" "$ART/AAX/RadarVU.aaxplugin" modern 11.0

if [ "${SIGN:-0}" = "1" ]; then
  echo "==> Sign bundles (Developer ID, hardened runtime, timestamp)"
  for b in "$ART/VST3/RadarVU.vst3" "$ART/AU/RadarVU.component" "$ART/CLAP/RadarVU.clap"; do
    codesign --force --deep --timestamp --options runtime --sign "$APP_ID" "$b"
    codesign --verify --deep --strict --verbose=2 "$b"
  done
  find "$ART/LV2/RadarVU.lv2" -type f \( -name '*.dylib' -o -name '*.so' \) -print0 | while IFS= read -r -d '' f; do
    codesign --force --timestamp --options runtime --sign "$APP_ID" "$f"
    codesign --verify --strict --verbose=2 "$f"
  done
  codesign --force --deep --timestamp --options runtime --entitlements installer/standalone.entitlements \
    --sign "$APP_ID" "$ART/Standalone/RadarVU.app"
  codesign --verify --deep --strict --verbose=2 "$ART/Standalone/RadarVU.app"
  if [ "$AAX" = 1 ]; then
    echo "==> AAX: Developer ID, then PACE Eden signature"
    codesign --force --deep --timestamp --options runtime --sign "$APP_ID" "$ART/AAX/RadarVU.aaxplugin"
    "$WRAPTOOL" sign --verbose --account "$PACE_ACCOUNT" --wcguid "$AAX_WCGUID" --signid "$APP_ID" \
      --in "$ART/AAX/RadarVU.aaxplugin" --out "$ART/AAX/RadarVU.aaxplugin" --autoinstall on
    "$WRAPTOOL" verify --verbose --in "$ART/AAX/RadarVU.aaxplugin"
  fi
else
  echo "==> Unsigned build: the AAX is NOT PACE-signed and must not be released"
fi

echo "==> Bundles archive"
rm -rf "$OUT"; mkdir -p "$OUT"
BUNDLES="$OUT/bundles"; mkdir -p "$BUNDLES/Licenses"
for f in VST3/RadarVU.vst3 AU/RadarVU.component CLAP/RadarVU.clap LV2/RadarVU.lv2 Standalone/RadarVU.app; do
  ditto "$ART/$f" "$BUNDLES/$(basename "$f")"
done
[ "$AAX" = 1 ] && ditto "$ART/AAX/RadarVU.aaxplugin" "$BUNDLES/RadarVU.aaxplugin"
python3 tools/collect_licenses.py "$BUILD" "$BUNDLES/Licenses/third-party" mac
cp LICENSE BINARY_LICENSE.txt THIRD_PARTY_LICENSES.md CHANGELOG.md "$BUNDLES/Licenses/"
python3 tools/build_user_manual.py
cp output/pdf/RadarVU-User-Manual.pdf "$BUNDLES/"
ZIP="$OUT/RadarVU-$VERSION-macos-bundles.zip"
(cd "$BUNDLES" && ditto -c -k --sequesterRsrc . "$ROOT/$ZIP")
rm -rf "$BUNDLES"

echo "==> Stage in the release folder and write the manifest"
# Only a signed run may touch the real release folder; an unsigned run stays inside dist/.
if [ "${SIGN:-0}" != "1" ]; then RELEASE_DIR="$ROOT/$OUT/unsigned-stage"; fi
mkdir -p "$RELEASE_DIR"
rm -f "$RELEASE_DIR/RELEASE-READY"
cp "$ZIP" "$RELEASE_DIR/"
python3 "$MANIFEST_TOOL" build "$RELEASE_DIR" RadarVU "$VERSION"
python3 "$MANIFEST_TOOL" verify "$RELEASE_DIR"
echo "done: $RELEASE_DIR"
echo "Check the archive, then create the empty RELEASE-READY file yourself and run the upload."
