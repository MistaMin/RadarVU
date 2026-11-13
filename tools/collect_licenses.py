#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Marcos Quirino Deida
"""Collect the project license and the third-party notices for the code that the shipped binaries contain.

Usage: collect_licenses.py <build dir> <destination dir> <mac|win|linux>

Reads the JUCE and clap-juce-extensions locations from the build's CMakeCache.txt (falling back to the
FetchContent default paths), copies the license texts from those exact revisions, and writes INDEX.txt.
The Audio Unit SDK notices are included on macOS only. The AAX SDK is an external proprietary
dependency under the Avid AAX Developer Agreement; official AAX binaries are digitally signed with
PACE Eden and distributed as freeware (the release notes).
"""
import shutil
import sys
from pathlib import Path


def cache(build: Path):
    values = {}
    for line in (build / "CMakeCache.txt").read_text(errors="replace").splitlines():
        if line and not line.startswith(("#", "//")) and "=" in line and ":" in line.split("=", 1)[0]:
            k, v = line.split("=", 1)
            values[k.split(":", 1)[0]] = v
    return values


def main():
    root = Path(__file__).resolve().parents[1]
    build, dest, platform = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve(), sys.argv[3]
    c = cache(build)
    juce = Path(c.get("JUCE_DIR") or build / "_deps/juce-src")
    candidates = [c.get("FETCHCONTENT_SOURCE_DIR_CLAP-JUCE-EXTENSIONS"), c.get("ARCVU_CLAP_EXT_SOURCE"), build / "_deps/clap-juce-extensions-src"]
    clap = next((Path(x) for x in candidates if x and (Path(x) / "LICENSE.md").exists()), Path(candidates[-1]))
    if dest.exists():
        shutil.rmtree(dest)

    def put(src: Path, rel: str):
        out = dest / rel
        out.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, out)

    put(root / "LICENSE", "RadarVU/LICENSE")
    put(root / "THIRD_PARTY_LICENSES.md", "RadarVU/THIRD_PARTY_LICENSES.md")
    put(root / "BINARY_LICENSE.txt", "RadarVU/BINARY_LICENSE.txt")
    put(root / "../instruments/GoodLookinDri/LICENSE", "GoodLookinDri/LICENSE")
    put(juce / "LICENSE.md", "JUCE/LICENSE.md")
    put(clap / "LICENSE.md", "CLAP/clap-juce-extensions-LICENSE.md")
    put(clap / "clap-libs/clap/LICENSE", "CLAP/clap-LICENSE")
    put(clap / "clap-libs/clap-helpers/LICENSE", "CLAP/clap-helpers-LICENSE")
    aax = Path(c.get("ARCVU_AAX_SDK_PATH") or root / "../aax-sdk-2-8-1")
    if (aax / "LICENSE.txt").exists():
        put(aax / "LICENSE.txt", "AAX-SDK/LICENSE.txt")

    mods = juce / "modules"
    roots = [
        mods / "juce_audio_formats/codecs",
        mods / "juce_core/zip/zlib",
        mods / "juce_graphics/drawables/lunasvg",
        mods / "juce_graphics/fonts/harfbuzz",
        mods / "juce_graphics/image_formats/libwebp",
        mods / "juce_graphics/image_formats/pnglib",
        mods / "juce_graphics/unicode/sheenbidi",
        mods / "juce_audio_processors_headless/format_types/VST3_SDK",
        mods / "juce_audio_processors_headless/format_types/LV2_SDK",
    ]
    if platform == "mac":
        roots.append(mods / "juce_audio_plugin_client/AU/AudioUnitSDK")
    for base in roots:
        if not base.exists():
            continue
        for f in base.rglob("*"):
            name = f.name.upper()
            if not f.is_file() or not name.startswith(("LICENSE", "COPYING", "NOTICE")):
                continue
            s = f.as_posix()
            if "/VST3_SDK/" in s:
                rel = "VST3-SDK/" + s.split("/VST3_SDK/", 1)[1]
            elif "/LV2_SDK/" in s:
                rel = "LV2-SDK/" + s.split("/LV2_SDK/", 1)[1]
            elif "/AudioUnitSDK/" in s:
                rel = "AudioUnit-SDK/" + s.split("/AudioUnitSDK/", 1)[1]
            else:
                rel = "JUCE/modules/" + s.split("/modules/", 1)[1]
            put(f, rel)
    put(mods / "juce_graphics/image_formats/jpglib/README", "JUCE/modules/juce_graphics/image_formats/jpglib/README")
    # The bundled Zix sources carry their ISC notices in file headers rather
    # than a standalone COPYING file. Preserve those exact notices too.
    notices = {}
    lv2 = mods / "juce_audio_processors_headless/format_types/LV2_SDK"
    for source in sorted(lv2.rglob("*")):
        if source.is_file() and "zix" in source.parts and source.suffix in (".h", ".c"):
            text = source.read_text(errors="replace")
            if text.startswith("/*"):
                notice = text[:text.index("*/") + 2]
                if "Copyright" in notice and "Permission" in notice:
                    notices.setdefault(notice, []).append(source.relative_to(lv2).as_posix())
    if notices:
        out = dest / "LV2-SDK/zix/COPYING"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text("\n\n".join("Files: " + ", ".join(names) + "\n" + notice
                                    for notice, names in notices.items()) + "\n")
    files = sorted(p.relative_to(dest).as_posix() for p in dest.rglob("*") if p.is_file())
    (dest / "INDEX.txt").write_text("\n".join(files) + "\n")
    print(f"collected {len(files)} license files into {dest}")


if __name__ == "__main__":
    main()
