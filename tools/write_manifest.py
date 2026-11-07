#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Writes opengrid-manifest.json (schema 2, the format the Plugin Manager reads) for the macOS bundles zip.
Usage: write_manifest.py <release-dir> <version> <min_os>"""
import hashlib, json, os, sys, datetime
d, version, min_os = sys.argv[1:4]
name = f"RadarVU-{version}-macos.zip"
path = os.path.join(d, name)
manifest = {
    "schema": 2, "plugin": "RadarVU", "version": version,
    "released": datetime.date.today().isoformat(),
    "highlights": [
        "Radar resolution menu (180 to 1440 points per revolution) for a much finer trace",
        "Six colour skins: Stock Dark, Retro Console, Ivory Terminal, Matrix, Modern Console, Studio Amber",
        "Pick any target loudness (-40 to -6 LUFS) and true-peak ceiling (-6 to 0 dBTP), or type a value",
        "Fixed the readout cards running past the bottom of the panel",
        "VST3, Audio Unit, CLAP, LV2, AAX and Standalone for macOS"],
    "archives": [{
        "os": "macos", "arch": ["universal"], "min_os": min_os,
        "build_class": "modern" if tuple(int(x) for x in min_os.split(".")) >= (11, 0) else "legacy", "name": name,
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
