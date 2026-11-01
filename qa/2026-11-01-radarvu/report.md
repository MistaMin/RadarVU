# RadarVU: DSP engineering + QA report (2026-11-01)

Source: uncommitted changes on 4f4c806. Release build in `build-qa/` (the old `build/` cache points at the former `Dev Projects/` path and was left alone). Nothing committed, signed or pushed.

## Findings and fixes
- No ballistics evidence existed: the only PPM test checked "finite". Added `dsp/src/ballistics.rs` (16 tests). The ported ArcVu detector passes all of them (VU rise/overswing at 44.1-192 kHz, calibration at 100 Hz/1 kHz/10 kHz, PPM fall times at 48/96 kHz, EBU 10 ms integration, SR and block-size invariance, silence, over-full-scale, holds, stereo policy, mono vs dual-mono).
- Fixed: phase correlation smoothing was 0.9995 per sample, so the window was 42 ms at 48 kHz and 10 ms at 192 kHz. Now 41.7 ms at every rate (`CORRELATION_TAU_S`). The same line exists in ArcVu; not changed there.

## Checks
| Check | Result |
|---|---|
| `cargo test --release` | PASS, 37 passed (21 old, 16 new) |
| State save/restore without a host (`RadarVU_QA`) | PASS, 6 of 6 parameters |
| dsp_lint (names/provenance) | PASS, 0 findings |
| Release build VST3/AU/Standalone/QA | PASS |
| `auval -v aufx RdVU OpGr` on the new install | PASS |
| Offscreen visual sweep (`RadarVU_QA`) | PASS, all 6 parameters + FILL/FFT change the GUI |
| REAPER VST3 suite, final build | **9 PASS** (`final-vst3/`, `suite-final-vst3.txt`) |
| REAPER AU suite, final build | **9 PASS** (`final-au/`, `suite-final-au.txt`); `auval` SUCCEEDED |
| (superseded) first VST3 run before the state fix | **FAIL, 8 of 9**: loads (10 params), finite, not silent, deterministic, editor opens, every parameter shown in the GUI, editor reopens all PASS. `state survives save and reopen` FAILS (Meter type RMS->VU, Sweep window 15 s->10 s, Display mode Spectrum->Radar); reproduced in two fresh QA dirs (`reaper-vst3/`, `reaper-state/`). Cause not yet found, see below |
| Numeric readout checks | SKIPPED: the user must choose the readout behaviour first |
| pluginval / AAX / CLAP | SKIPPED: not installed / not built |
| Human-eye screenshot review | NOT DONE |

## State round trip: fixed
The REAPER suite's state check could not read RadarVU's own state layout (`<P id index>`), fell back to comparing displayed text, and failed in the suite's order (pre-renders, then the editor opens, then the parameter burst). Whether the host or the plugin was at fault was not proven. The state is now written in the standard layout (`<PARAM id value>`), so the harness reads the plugin's real state, and the earlier `<P>` layout still loads (tested). Change: `plugin/PluginProcessor.cpp` `getStateInformation` / `setStateInformation`. Verified: host-free test in `RadarVU_QA` (12 of 12), and both REAPER suites below.

## Needs the user
1. (Resolved by default) Numeric readout stays the live detector value with the 5 s hold, as in ArcVu.
2. Input/output section: RadarVU has none (levels, trim, routing); the DSP rules require one. Add it, or sign off a waiver as a pass-through meter?
3. Input/output section: left out by your decision (pass-through meter).
4. `plugin-qa/projects/radarvu.md` still lists the old `Dev Projects/` path.
