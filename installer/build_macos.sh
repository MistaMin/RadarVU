#!/bin/bash
# SPDX-License-Identifier: MIT
# Builds and tests RadarVU for macOS (universal arm64 + x86_64): VST3, AU, CLAP, LV2, AAX and the Standalone
# app, and stamps the build class (modern, macOS 11.0) into each bundle. Nothing is signed here.
# installer/build_bundles_zip.sh then signs, zips and notarises the bundles for the Plugin Manager.
set -euo pipefail
export COPYFILE_DISABLE=1
cd "$(dirname "$0")/.."
ROOT=$PWD
VERSION=$(sed -n 's/^project(RadarVU VERSION \([0-9.]*\)).*/\1/p' CMakeLists.txt)
BUILD=${BUILD:-build/release-macos}
ART="$ROOT/$BUILD/RadarVU_artefacts/Release"
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
# breaks the signature).
STAMP=${STAMP_TOOL:-"$ROOT/tools/stamp_build_class.sh"}
for b in "$ART/VST3/RadarVU.vst3" "$ART/AU/RadarVU.component" "$ART/CLAP/RadarVU.clap" "$ART/Standalone/RadarVU.app"; do
  "$STAMP" "$b" modern 11.0
done
if [ "$AAX" = 1 ]; then "$STAMP" "$ART/AAX/RadarVU.aaxplugin" modern 11.0; fi
