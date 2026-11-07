# RadarVU scope ledger: radar resolution and colour skins (2026-11-07)

Source of scope: the earlier ChatGPT session's request, "the radar looks like a blob, it needs to be much finer", and the uncommitted work it left. Claude took the work over, checked it end to end and finished the documentation. Nothing here is committed or released yet.

Summary: 17 rows verified, 4 unverified (reason given), 0 failed. Exceptions: 1 NEW (below).

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
| S17 | AU in a real host: auval and the REAPER suite, including free target/ceiling | AU build | all | Re-run on the final float-parameter build (`qa2/`): `auval` AU VALIDATION SUCCEEDED, Target level -40..-6 LUFS default -14 (`qa2/auval.txt`); hidden REAPER 7.80 suite on `AU: RadarVU (OpenGrid)`, 8 plug-in parameters: loads, finite, not silent, deterministic, editor opens, every parameter shown in the GUI (Target 6.3%, Ceiling 5.1%), state survives save/reopen, 5 editor reopen cycles. Editor screenshots viewed: ceiling -6.0 dBTP moves the red tick; target -40 LUFS turns the readouts red; cards inside the panel. Dev build installed temporarily with the user's consent, removed, folders verified restored | verified |
| S18 | VST3 in a real host (REAPER) | VST3 build | | A first REAPER run on `VST3: RadarVU (OpenGrid)` loaded only 6 RadarVU parameters: the released 1.0.0 in /Library shadows the dev copy, so that log is not evidence for this build (`suite-vst3-SYSTEM-1.0.0-not-this-build.log`). VST3 coverage for this build is the JUCE VST3 host run (S13) | unverified: needs the system 1.0.0 removed or a different install location; clap-validator and pluginval not installed |
| S19 | Readout cards no longer clip: equal margins inside the panel in every skin (reported by the user from a Retro Console screenshot, then confirmed to affect all skins) | dsp/src/lib.rs `readout_cells`, `READOUT_*` | readout panel, all skins, 440/1100/4000 px | Cause: five 78 px cards on an 86 px step from y=64 ended at y=486, past the panel bottom (484). New unit test `readout_cards_sit_inside_the_panel_with_equal_margins`; UI audit contact sheets and Retro Console / Ivory Terminal close-ups viewed; live Standalone window viewed | verified |
| S20 | Target (-40..-6 LUFS) and true-peak ceiling (-6..0 dBTP) are free values: popup with presets and typed Custom value | plugin/PluginProcessor.cpp (float parameters, state migration), plugin/PluginEditor.cpp `showValueMenu` / `askForValue`; dsp `sane` clamp | TARGET and CEILING chips | `cargo test` 54/54 (clamp and range test); UI audit 54 PASS: fresh defaults -14/-1, custom -17.3/-2.7 round trip, previous-release state migrates to -23 LUFS / -3 dBTP, old `<P>` layout, out-of-range clamp; hosted VST3 loads, bit-exact audio, state round trip and editor recreation | verified (value handling) |
| S21 | The target/ceiling popup and Custom value entry work by mouse and keyboard | plugin/PluginEditor.cpp | both chips | Not verified. A passive screenshot of the live Standalone showed TARGET -8.0 LUFS (not a preset or default) and CEILING -0.1 dBTP (a preset), consistent with the menus having been used. My own click-through was blocked by a macOS "RadarVU would like to access the microphone" dialog, which I did not answer (privacy decision for the user) | unverified: user answers the dialog, or clicks through once |
| S15 | Docs: README, manual (menus, skins, parameter table), changelog, skin images | README.md, docs/USER_MANUAL.md, CHANGELOG.md, tools/build_user_manual.py | none | manual PDF rebuilt and its parameter table checked for both new parameters | verified |

## Parameters in the host list

Meter type, Reference level, Target level, True-peak ceiling, Sweep window, Display mode, Radar resolution, Colour skin. All eight have rows (S7, S8 and the earlier ledger).

## Exceptions

- NEW: the `plugin-qa` knob-display rules ask for whole steps on level controls like a ceiling and no `0.x` values outside an EQ Q field. The true-peak ceiling presets (-0.1, -0.5 dBTP), the 0.1 steps of Custom value, and the target's 0.1 steps break that. Decided by the user's explicit request ("I should be able to pick the true-peak ceiling and the target loudness myself"); -0.1 dBTP is also a common mastering ceiling. Recorded here so it is a decision, not an oversight.
- Performance check: paint p95 8.3 ms at 1100 px (8.5 ms before), measured with the machine load average near 61, so treat 4000 px (69 ms) as noisy. Offscreen only.

## Not done, by decision

- No version bump and no release: needs the owner to decide whether this is 1.1.0.
- No commit.
