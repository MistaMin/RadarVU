# RadarVU 1.0.0: release preparation (2026-11-01)

Follows Make-Plugin-Release, with the OpenGrid release policy (bundles zip plus manifest in `Release Binaries`, no pkg, no dmg, no GitHub binaries). Nothing was signed, uploaded or pushed. The bytes tested below are the **unsigned** universal build; signing changes them.

## Done
| Step | Result |
|---|---|
| Signing environment (`verify_signing_env.sh`) | PASS: both Developer ID certificates, notary profile `AC_NOTARY`, wraptool 6.0.1, signing iLok present |
| Formats (`verify_formats.sh`) | PASS: VST3, AU, CLAP, LV2, AAX, Standalone. CLAP and LV2 were added in this pass |
| Standalone icon (`check_standalone_icon.sh`) | PASS after generating the icon from the concept art (460 px crop, scaled to 1024; a sharper hand-made icon would be better) |
| User manual | Written (`docs/USER_MANUAL.md`), PDF built by `tools/build_user_manual.py`, 4 pages, screenshots refreshed from this build, parameter table generated from the code |
| Version | 1.0.0 |
| Universal build (arm64 + x86_64), minimum macOS 11.0 | PASS. The deployment target was being set after `project()` and so was ignored (binaries said 15.0); moved before `project()` |
| Build class stamp and manifest | PASS: `opengrid_manifest.py build/verify` reports modern, universal, min 11.0 |
| Payload checks | Version, bundle id `com.opengrid.radarvu`, VST3 vendor OpenGrid, AU name `OpenGrid: RadarVU`, class ID `ABCDEF019182FAEB4F70477252645655` all as expected |
| `auval -v aufx RdVU OpGr` on the zip's AU | PASS |
| AAX Validator on the zip's AAX | PASS all six tests. It first FAILED `test.page_table.load` (no page table in the bundle); fixed with `Resources/RadarVU.xml` and the AAX client extension. Open warning: the seven automatable parameters are not on a page of the default table (same as ArcVu) |
| REAPER VST3 and AU suites on the zip's bytes | 9 of 9 PASS each |
| Cargo tests, offscreen UI audit, state round trip | PASS (37 tests) |
| `dsp_lint --payload` on the unpacked zip | 1 finding, reviewed: `RadarVU-User-Manual.pdf` is our own manual, flagged only because of the `.pdf` file type (ArcVu's release gives the same finding). Not suppressed in the tool |

## For you to do
1. **Sign and stage** in your own Terminal with the Signing iLok plugged in:
   `cd /Volumes/Developer/OpenGrid/RadarVU && SIGN=1 installer/build_macos.sh`
   It rebuilds, retests, signs the bundles and the Standalone (Developer ID), PACE-signs the AAX (wrap GUID `77661BB0-D60F-11F1-804C-005056928F3B`, account `mistamin`), zips, and stages to `Release Binaries/MistaMin/RadarVU/1.0.0/` with the manifest. It does not create `RELEASE-READY`.
2. Then check `wraptool verify` output (expect "signed, but not wrapped" and "signature verified") and run the host suites once on the signed bytes.
3. Create the empty `RELEASE-READY` file and run the upload yourself.

## Open
- `LICENSE` names "Marcos Deida" as copyright holder; ArcVu's names "OpenGrid", and the copied `BINARY_LICENSE.txt` says OpenGrid. Which holder is your decision.
- Not tested: Pro Tools (AAX) in a host, Logic, Studio One, CLAP and LV2 in a host (no `clap-validator` installed), `pluginval` (not installed).
- Windows and Linux builds are not made: listed as not published.
- Screenshots were not reviewed by a person in a native window.
- `plugin-qa/projects/radarvu.md` still lists the old `Dev Projects/` path and old formats.
