#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Writes opengrid-manifest.json (schema 2, the format the Plugin Manager reads) for the macOS pkg.
Usage: write_manifest.py <release-dir> <version> <min_os>"""
import hashlib, json, os, sys, datetime
d, version, min_os = sys.argv[1:4]
name = f"RadarVU-{version}-macos.pkg"
path = os.path.join(d, name)
manifest = {
    "schema": 2, "plugin": "RadarVU", "version": version,
    "released": datetime.date.today().isoformat(),
    "highlights": [
        "Polar radar display with a sweeping history of the signal",
        "VU, RMS, K-System, DIN, Nordic, BBC and EBU meters on the reference ballistics",
        "Loudness panel to ITU-R BS.1770-5, EBU Tech 3341 and 3342, plus stereo goniometer and spectrum",
        "Passes the audio through unchanged",
        "VST3, Audio Unit, CLAP, LV2, AAX and Standalone for macOS"],
    "archives": [{
        "os": "macos", "arch": ["universal"], "min_os": min_os, "name": name,
        "size": os.path.getsize(path), "sha256": hashlib.sha256(open(path, "rb").read()).hexdigest(),
        "installs": [
            {"format": "app", "path": "Applications/RadarVU.app"},
            {"format": "vst3", "path": "Library/Audio/Plug-Ins/VST3/RadarVU.vst3"},
            {"format": "au", "path": "Library/Audio/Plug-Ins/Components/RadarVU.component"},
            {"format": "clap", "path": "Library/Audio/Plug-Ins/CLAP/RadarVU.clap"},
            {"format": "lv2", "path": "Library/Audio/Plug-Ins/LV2/RadarVU.lv2"},
            {"format": "aax", "path": "Library/Application Support/Avid/Audio/Plug-Ins/RadarVU.aaxplugin"}]}]}
json.dump(manifest, open(os.path.join(d, "opengrid-manifest.json"), "w"), indent=2)
print("manifest written:", name, manifest["archives"][0]["sha256"])
