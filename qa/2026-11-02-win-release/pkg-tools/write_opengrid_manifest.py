#!/usr/bin/env python3
"""Writes opengrid-manifest.json (schema 2) for one plugin release.

Usage:
  write_opengrid_manifest.py --plugin NAME --version X.Y.Z --released YYYY-MM-DD \
      --highlights FILE --out opengrid-manifest.json --archive ZIP [--archive ZIP ...]

Each ZIP must have its ZIP.installs.json sidecar, written by make_bundle_zip.sh or .ps1. The sidecar
gives the OS, CPU, minimum OS and the install paths. The SHA-256 and size come from the ZIP itself.
Highlights: one per line in FILE, at most five. Lines starting with '#' are ignored.
"""
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

MAX_HIGHLIGHTS = 5
VERSION = re.compile(r"^\d+(\.\d+){0,3}(-[0-9A-Za-z.]+)?$")


def _parse_os(v: str):
    parts = re.split(r"[.-]", v)
    return tuple(int(p) if p.isdigit() else 0 for p in parts[:2])

def read_highlights(path: Path) -> list[str]:
    lines = [l.strip() for l in path.read_text(encoding="utf-8").splitlines()]
    items = [l for l in lines if l and not l.startswith("#")]
    if len(items) > MAX_HIGHLIGHTS:
        sys.exit(f"{len(items)} highlights; at most {MAX_HIGHLIGHTS} are shown in the manager")
    return items


def archive_entry(zip_path: Path) -> dict:
    sidecar = Path(str(zip_path) + ".installs.json")
    if not sidecar.exists():
        sys.exit(f"{zip_path.name}: no installs sidecar ({sidecar.name}); build it with make_bundle_zip")
    meta = json.loads(sidecar.read_text(encoding="utf-8-sig"))
    data = zip_path.read_bytes()
    if not meta.get("installs"):
        sys.exit(f"{zip_path.name}: the sidecar lists no installs")
    return {
        "os": meta["os"],
        "arch": [meta["arch"]],
        "min_os": meta["min_os"],
        # modern: macOS 11+ or Windows 10+; legacy: older systems (plugin-qa build-class rule)
        "build_class": "legacy" if meta["os"] == "macos" and _parse_os(meta["min_os"]) < (11, 0) else "modern",
        "name": zip_path.name,
        "size": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "installs": meta["installs"],
    }


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--plugin", required=True)
    ap.add_argument("--version", required=True)
    ap.add_argument("--released", required=True)
    ap.add_argument("--highlights", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--archive", action="append", required=True, type=Path)
    args = ap.parse_args()

    if not VERSION.match(args.version):
        sys.exit(f"{args.version!r} is not a version number")
    archives = [archive_entry(p) for p in args.archive]
    names = [a["name"] for a in archives]
    if len(set(names)) != len(names):
        sys.exit("two archives have the same name")
    manifest = {
        "schema": 2,
        "plugin": args.plugin,
        "version": args.version,
        "released": args.released,
        "highlights": read_highlights(args.highlights),
        "archives": archives,
    }
    args.out.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {args.out} with {len(archives)} archive(s)")


if __name__ == "__main__":
    main()
