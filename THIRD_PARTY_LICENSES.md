# Third-party licenses

RadarVU's own source code is MIT licensed (see `LICENSE`). Official complete binaries are proprietary freeware under the publisher's commercial JUCE licence (see `BINARY_LICENSE.txt`), not a relicensing of JUCE or other dependencies under MIT. End users need no JUCE developer licence to use the official downloads. Independent builders must satisfy the JUCE licence applicable to their builds. Installers and archives include the licence and notice texts below, copied from the exact dependency revisions of each build by `tools/collect_licenses.py` (with an `INDEX.txt` listing every file).

| Component | Used for | License |
|---|---|---|
| JUCE | plugin framework, user interface, plugin formats | AGPLv3 or the commercial JUCE licence. OpenGrid builds its released binaries under its own JUCE licence. Anyone building from source needs a JUCE licence that covers their use and distribution. |
| GoodLookinDri | procedural drawing (Rust, linked statically) | MIT |
| Rust standard library | detector and drawing code | MIT or Apache License 2.0 |
| CLAP, clap-helpers, clap-juce-extensions | CLAP format | MIT |
| VST3 SDK (bundled with JUCE) | VST3 format | MIT |
| LV2, serd, sord, sratom, lilv, zix (bundled with JUCE) | LV2 format | ISC |
| Audio Unit SDK (bundled with JUCE, macOS only) | Audio Unit format | Apache License 2.0 |
| AAX SDK (Avid) | AAX format | Avid AAX Developer Agreement / SDK License |
| zlib, libpng, HarfBuzz, libwebp, lunasvg, SheenBidi and audio codecs (bundled with JUCE) | JUCE internals | their own licenses, included |

The AAX SDK is an external proprietary dependency under the Avid AAX Developer Agreement and is not committed to this repository. Official AAX binaries are digitally signed with PACE Eden for Pro Tools compatibility.

VST is a trademark of Steinberg Media Technologies GmbH. AAX and Pro Tools are trademarks of Avid Technology, Inc. Other format names are used only to describe compatibility.

The open-source licence for JUCE is **AGPLv3**, not plain GPL. A commercial JUCE licence is the alternative.
