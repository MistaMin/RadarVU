# RadarVU — scope ledger

Created for the first RadarVU build, then extended with ArcVu's full measurement set. Sources of scope: the user request in this conversation, the ArcVu loudness work (`ArcVu/qa/2026-10-28-loudness/ledger.md`), ArcVu's ballistic detector (`ArcVu/dsp/src/lib.rs`, `extra.rs`), and the GoodLookinDri drawing contract.

| ID | Item | Source | Code | GUI | Test | Evidence | Status |
|---|---|---|---|---|---|---|---|
| S1 | Loudness engine measures M / S / I / LRA / true peak to BS.1770-5 + EBU 3341/3342 | request; ArcVu loudness engine | `dsp/src/loudness.rs` | readout panel | `cargo test` EBU cases 1–19 + LRA 1–4 | 21 passed | verified |
| S2 | Radar display: filled multicolour polar history, angle=time, radius=level | request | `dsp/src/lib.rs` `paint_radar` | radar scope | `cargo test` smoke + offscreen render | `radarvu-radar.png` | verified |
| S3 | Multicolour heat-map fill fades with age (blue→green→amber→red) | request | `dsp/src/lib.rs` `paint_trail` | radar scope | offscreen render | `radarvu-radar.png` (colour-analysed) | verified (rendered) |
| S4 | Grid rings + labels on the selected meter's scale | request | `dsp/src/lib.rs` `paint_radar` | radar scope | offscreen render | `radarvu-radar.png` | verified (rendered) |
| S5 | Stereo goniometer at 8 o'clock (lower-left, mid/side) | design decision | `dsp/src/lib.rs` `paint_gonio` | lower-left column | offscreen render | `radarvu-radar.png` | verified (rendered) |
| S6 | Readout panel: M, S, I, TP (+ ceiling scale), LRA | request | `dsp/src/lib.rs` `paint_readouts` | right panel | offscreen render | `radarvu-radar.png` | verified (rendered) |
| S7 | Spectrum display mode (full multicolour FFT analyser) | request | `dsp/src/lib.rs` `paint_spectrum` (GoodLookinDri spectrum) | scope area | offscreen render | `radarvu-spectrum.png` | verified (rendered) |
| S8 | Target parameter (−23/−18/−16/−14 LUFS) | request | `plugin/PluginProcessor.cpp` | header chip | host list + visual sweep | 7869 px | verified |
| S9 | Ceiling parameter (−1/−2/−3 dBTP) | request | `plugin/PluginProcessor.cpp` | header chip + TP tick | host list + visual sweep | 2414 px | verified |
| S10 | Window parameter (5/10/15/20/30 s) | request | `plugin/PluginProcessor.cpp` | header chip | host list + visual sweep | 3650 px | verified |
| S11 | Mode parameter (Radar/Bar) | request | `plugin/PluginProcessor.cpp` | header chips | host list + visual sweep | 22037 px | verified |
| S12 | Reset clears meters, integrated, range and sweep | request | `radarvu_reset` | header chip | unit test | `cargo test` | verified |
| S13 | Audio passes through unchanged | request (a meter) | `plugin/PluginProcessor.cpp` | n/a | REAPER render + audio-excused params | host suites | verified |
| S14 | C ABI + JUCE replay of the drawing list | ArcVu pattern | `plugin/*`, `dsp/src/lib.rs` | whole GUI | builds + host loads + editor reopens | VST3/AU/Standalone; suites | verified |
| S15 | State save/load round-trips all parameters | ArcVu pattern | `plugin/PluginProcessor.cpp` | n/a | host "state survives" | both suites PASS | verified |
| S16 | Concept sketch of the GUI | request | `docs/concept/` | n/a | visual | rendered PNG | verified |
| S17 | VU meter (IEC 60268-17 galvanometer) | request; ArcVu | `dsp/src/lib.rs` `meter_process` | radar + numeric readout | unit test `vu_reads_zero_for_a_reference_sine` | `cargo test` | verified |
| S18 | RMS absolute dBFS (+ AES-17) | request; ArcVu | `dsp/src/lib.rs` `reading` | radar + readout | unit tests `rms_reads_absolute_dbfs`, `rms_aes17_...` | `cargo test` | verified |
| S19 | K-12 / K-14 / K-20 scales | request; ArcVu | `dsp/src/lib.rs` `reading` | radar | unit test `k12_reads_zero_...` | `cargo test` | verified |
| S20 | PPM DIN/Nordic/BBC/EBU + EBU slow (BS.645-2, Tech 3205-E) | request; ArcVu | `dsp/src/lib.rs` `meter_process` | radar + readout | unit test `ppm_reads_a_finite_level` | `cargo test` | verified |
| S21 | Peak + hold, over (3-sample) and clip counters | request; ArcVu | `dsp/src/lib.rs` `meter_process` | numeric readout (PK/OVER/CLIP) | offscreen render | `radarvu-radar.png` | verified (rendered) |
| S22 | Phase correlation | request; ArcVu | `dsp/src/lib.rs` `meter_process` | phase gauge | unit test `correlation_reads_mono...` | `cargo test` | verified |
| S23 | Meter type parameter (VU/RMS/K/PPM/Loudness) | request | `plugin/PluginProcessor.cpp` | header chip | host list + visual sweep | 5210 px | verified |
| S24 | Reference parameter (−20/−18/−14 dBFS) | request; ArcVu | `plugin/PluginProcessor.cpp` | header chip | host list + visual sweep | 2517 px | verified |
| S25 | Colour-fill toggle (radar heat map on/off) | request | `dsp/src/lib.rs` `paint_trail` | header chip | offscreen toggle diff | 81379 px | verified |
| S26 | FFT toggle (spectrum analyser on/off) | request | `dsp/src/lib.rs` `paint_spectrum` | header chip | offscreen toggle diff | 6668 px | verified |

## Status summary

**26 verified, 0 unverified, 0 out of scope.** "rendered" = offscreen screenshot + colour analysis; the host suites now also confirm every parameter is shown in the GUI (a human-eye screenshot review remains a listed follow-up).

## Evidence (2026-10-30)

- `cargo test --manifest-path dsp/Cargo.toml --offline --lib` → **21 passed** (11 EBU 3341/3342 + detector VU/RMS/K/AES-17/PPM/correlation + radar smoke).
- `auval -v aufx RdVU OpGr` → **AU VALIDATION SUCCEEDED**.
- REAPER host suite (hidden, isolated config): **VST3** (10 params) and **AU** (9 params) both **9 PASS** — loads, finite, not silent, deterministic, editor opens, every-parameter-changes-sound, every-parameter-shown-in-the-GUI, state round-trip, editor reopens.
- Offscreen `RadarVU_QA` visual sweep: all 6 parameters + the fill/FFT toggles change the GUI.
- Performance: editor made opaque (`setOpaque(true)`) and the per-frame path count halved (radar 240→96 wedges, goniometer 1024→512); standalone CPU ~29 % → ~14 % — `performance.md`.
- VST3 class ID `ABCDEF019182FAEB4F70477252645655` (32 hex digits).

## Follow-ups

Human-eye screenshot review; Logic/Studio One (computer use); AAX/CLAP/LV2 formats; a native parameter panel (AES-17, EBU slow mode, peak scale, LU/LUFS unit, colour themes); persist the FILL/FFT toggles across sessions; old-vs-new standalone CPU comparison and the spectrum-mode CPU sample (SKIPPED in `performance.md`).
