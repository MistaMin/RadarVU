# RadarVU — QA report (2026-10-30)

Follows `plugin-qa`. All runs unattended; hidden REAPER instance with an isolated config. No system install, signing, commit, push or upload was done. This report supersedes the earlier LUFS-only report.

## Scope
24 items in `ledger.md`: **24 verified**, 0 unverified, 0 out of scope. RadarVU now carries ArcVu's full measurement set: VU, RMS (+AES-17), K-12/K-14/K-20, DIN/Nordic/BBC/EBU PPM (+ EBU slow mode), peak+hold, over/clip, and phase correlation, in addition to the BS.1770-5 loudness engine and the radar/goniometer display.

## Builds
- Configuration: Release, `-DRADARVU_BUILD_QA=ON`.
- Formats built and installed to the user folders: **VST3**, **AU**, **Standalone**. AAX/CLAP/LV2 not wired (follow-up).
- VST3 class ID `ABCDEF019182FAEB4F70477252645655` (32 hex digits). Source: uncommitted changes.

## Checks

| Check | Result |
|---|---|
| Rust unit + EBU + detector tests | **PASS** — 21 passed (11 EBU 3341/3342 + VU/RMS/K/AES-17/PPM/correlation + radar smoke) |
| AU validator (`auval -v aufx RdVU OpGr`) | **PASS** — "AU VALIDATION SUCCEEDED" |
| REAPER VST3 suite | **9 PASS** — loads (10 params), finite, not silent, deterministic, editor opens, every-param-changes-sound, every-param-shown-in-the-GUI, state round-trip, editor reopens |
| REAPER AU suite | **9 PASS** — loads (9 params), same |
| Editor window + host visual sweep | **PASS** — editor found (1100×613) and every parameter confirmed shown in the GUI |
| Offscreen visual sweep (`RadarVU_QA`) | **PASS** — all 6 params change the GUI |
| Performance checks (standalone) | **PASS** — editor opaque + fill halved; CPU ~29 % → ~14 % (see `performance.md`; old-vs-new and native-window checks SKIPPED) |
| pluginval / AAX / CLAP validators | SKIPPED — pluginval not installed; AAX/CLAP not built |

Offscreen visual sweep + toggles (pixels changed at 1100×560): all 6 parameters and the fill/FFT toggles change the GUI (see `performance.md`).

## Latest UI changes
- **The stereo goniometer moved to 8 o'clock** (lower-left), tangent to the scope.
- **The radar heat ramp is more beautiful** — a six-stop deep-blue → cyan → green → amber → orange → red ramp.
- **FILL and FFT toggles** were added to the header: the radar colour fill and the spectrum analyser can each be switched off.

## Exceptions (qa/2026-10-29-radarvu/exceptions.txt)
All six plugin parameters are display-only (RadarVU is a meter, audio passes through unchanged): `audio:` Meter type, Reference level, Target level, True-peak ceiling, Sweep window, Display mode. REAPER host-added Bypass/Wet are `audio:`+`visual:`, Delta is `visual:` (host controls). Each has a reason on its line.

## Fixes made in this pass
- Ported ArcVu's ballistic detector (`dsp/src/lib.rs` `meter_process`, `reading`, `peak_db`) and scales (`dsp/src/extra.rs`): VU galvanometer (IEC 60268-17), RMS absolute dBFS with AES-17, K-12/K-14/K-20, DIN/Nordic/BBC/EBU PPM (ITU-R BS.645-2) with the EBU three-stage attack + slow mode (Tech 3205-E), sample peak with hold, 3-consecutive-sample over counter, clip, and phase correlation (1→0, 0→−20, −1→−39).
- Added the **Meter type** (VU…EBU, Loudness) and **Reference** (−20/−18/−14 dBFS) header chips; the radar's rings, sweep and trail now plot the selected meter, and a numeric readout under the scope shows the level, peak, over/clip and a phase gauge.
- The True-Peak unit bug and the ceiling tick (from the previous pass) remain fixed.
- **Performance:** made the editor opaque (`setOpaque(true)`) and halved the per-frame path count (radar 240 → 96 wedges, goniometer 1024 → 512). Standalone CPU at idle/radar fell from ~29 % to ~14 %. Details and the skipped checks are in `performance.md`.

## Not done / needs the user
- Human-eye screenshot review (renders are colour-analysed; this model cannot view images).
- Logic/Studio One (computer use); AAX/CLAP/LV2 formats; a native parameter panel (AES-17, EBU slow mode, peak scale, LU/LUFS unit, colour themes) — follow-ups.
- Old-vs-new standalone CPU comparison (no earlier build/tag) and the spectrum-mode CPU sample (needs an interactive click) — recorded as SKIPPED in `performance.md`.
