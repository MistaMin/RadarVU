# RadarVU scope ledger: radar resolution and colour skins (2026-11-07)

Source of scope: the earlier ChatGPT session's request, "the radar looks like a blob, it needs to be much finer", and the uncommitted work it left. Claude took the work over, checked it end to end and finished the documentation. Nothing here is committed or released yet.

Summary: 15 rows verified, 3 unverified (reason given), 0 failed.

| ID | Item | Code | GUI | Test / evidence (2026-11-07, this build) | Status |
|---|---|---|---|---|---|
| S1 | Radar points per revolution 180/360/720/1440, default 720 | dsp/src/lib.rs, plugin/PluginProcessor.cpp (`HISTORY_POINTS`) | RADAR RESOLUTION menu | `cargo test`: 52/52 incl. capture cadence for all windows and resolutions, history independent of buffer size | verified |
| S2 | Resolution change starts a fresh trail but keeps loudness statistics | dsp/src/lib.rs | menu | `cargo test` (resolution change tests) | verified |
| S3 | Loudness trace follows 400 ms momentary; contour visible over the fill (fill brightness: see S16) | dsp/src/lib.rs | radar, FILL on and off | `radar_contour_is_visible_with_fill_and_fits_the_paint_abi`; contact sheets viewed | verified |
| S4 | Six skins: Stock Dark, Retro Console, Ivory Terminal, Matrix, Modern Console, Studio Amber | dsp/src/skin.rs | COLOUR SKIN menu | `skins_preserve_history_measurements_and_fit_the_drawing_budget`; UI audit sweep: skin changes 12.9% of pixels | verified |
| S5 | Every skin covers Radar and Spectrum, controls and menus, at 440/1100/4000 px | dsp/src/skin.rs, plugin/PluginEditor.cpp | all | UI audit wrote 36 frames and 6 contact sheets; viewed `skins-radar-1100`, `skins-spectrum-1100`, `skins-radar-440` | verified |
| S6 | Skin text contrast and distinct warning colours | dsp/src/skin.rs | all skins | text-contrast unit test; contact sheets viewed | verified |
| S7 | Host parameter `resolution` (4 choices) automates | plugin/PluginProcessor.cpp | menu | UI audit sweep: resolution changes the GUI (178 px) | verified |
| S8 | Host parameter `skin` (6 choices) automates | plugin/PluginProcessor.cpp | menu | UI audit sweep: skin changes the GUI | verified |
| S9 | State saves and restores all 4x6 combinations | plugin/PluginProcessor.cpp | none: state | UI audit state round trip: all 24 PASS | verified |
| S10 | Old projects reopen at 720 points / Stock Dark | `setStateInformation` | none: state | UI audit "old project defaults" PASS | verified |
| S11 | Menus themed to the active skin, anchored when the editor is resized | plugin/PluginEditor.cpp `showChoiceMenu` | both menus | code reading and screenshots of the closed menus only | unverified: an open popup cannot be captured offscreen; needs a manual click in a host |
| S12 | AAX page tables list resolution and skin | Resources/RadarVU.xml | none: Pro Tools | `xmllint` well-formed | unverified: not loaded in Pro Tools (needs PACE-signed build) |
| S13 | Audio is untouched; editor recreates; drawing limits hold | plugin/PluginProcessor.cpp | none | Music audit on real stereo music (direct, 3 combinations, and hosted VST3): `max_audio_difference` 0.0, state and editor recreation true, 361 to 610 draw commands, 32 points max per command | verified |
| S14 | Paint cost acceptable | | | Direct: p95 3.4 ms at 440 px, 8.5 ms at 1100 px, 64 ms at 4000 px offscreen | verified (4000 px is the maximum size) |
| S16 | Fill keeps a visible age gradient and colour (blob complaint) | dsp/src/lib.rs `FILL_OLDEST_BG` / `FILL_NEWEST_BG` | radar, all skins | Newest fill was 30% heat colour (dark brown); now about 70% at the newest end fading to 15%, contour unchanged. `cargo test` 52/52 after the change; UI audit re-run, 50 PASS 0 FAIL; contact sheet viewed; README skin images regenerated from it | verified |
| S17 | AU in a real host: auval and the REAPER suite | AU build | all | `auval -v aufx RdVU OpGr`: AU VALIDATION SUCCEEDED, lists 8 parameters incl. Radar resolution and Colour skin (`auval.txt`). Hidden REAPER 7.80 suite on `AU: RadarVU (OpenGrid)`: loads with 8 plug-in parameters, finite, not silent, deterministic, editor opens, every parameter shown in the GUI (Colour skin 13.6%, Radar resolution 4.4% pixels), state survives save/reopen, 5 editor reopen cycles. Editor screenshots viewed (Stock Dark, Studio Amber). Dev build installed temporarily with the user's consent and removed afterwards; folders verified restored | verified |
| S18 | VST3 in a real host (REAPER) | VST3 build | | A first REAPER run on `VST3: RadarVU (OpenGrid)` loaded only 6 RadarVU parameters: the released 1.0.0 in /Library shadows the dev copy, so that log is not evidence for this build (`suite-vst3-SYSTEM-1.0.0-not-this-build.log`). VST3 coverage for this build is the JUCE VST3 host run (S13) | unverified: needs the system 1.0.0 removed or a different install location; clap-validator and pluginval not installed |
| S15 | Docs: README, manual (menus, skins, parameter table), changelog, skin images | README.md, docs/USER_MANUAL.md, CHANGELOG.md, tools/build_user_manual.py | none | manual PDF rebuilt and its parameter table checked for both new parameters | verified |

## Parameters in the host list

Meter type, Reference level, Target level, True-peak ceiling, Sweep window, Display mode, Radar resolution, Colour skin. All eight have rows (S7, S8 and the earlier ledger).

## Not done, by decision

- No version bump and no release: needs the owner to decide whether this is 1.1.0.
- No commit.
