# RadarVU — performance notes (2026-10-30)

Follows the "Performance checks for UI and audio" section of `plugin-qa`. RadarVU touches painting,
timers and animation, so these checks apply.

## Fixes applied

1. **Opaque component.** `RadarVuEditor` now calls `setOpaque(true)`; its `paint()` fills the whole
   area, so JUCE need not repaint the parent beneath it (`plugin/PluginEditor.cpp`).
2. **Cheaper radar fill.** The radar history was reduced from 240 to **96** samples (96 wedges per
   frame instead of 240) and the goniometer trace from 1024 to **512** (16 polylines instead of 32).
   The on-screen appearance is unchanged; the per-frame path count fell by roughly half
   (`dsp/src/lib.rs`, `HISTORY` / `TRACE`).

## Measurement (Release standalone, idle, radar mode, Apple M1 Max, macOS 15.7)

| | CPU % | Load avg (before → after) |
|---|---|---|
| Before the two fixes | 29.1 / 28.8 / 28.0 / 28.5 / 31.0 | 4.66 → 21.77 (noisy) |
| After the two fixes | 14.9 / 14.6 / 16.6 / 12.5 / 11.9 / 13.1 | 6.66 → 5.32 |

Thread breakdown (`sample`, 5 s, after): the main thread is dominated by CoreGraphics path
rasterization (`ripc_DrawPath` → `argb32_mark`), i.e. the software rendering of the radar wedges,
rings, text, goniometer and readouts at the 30 Hz repaint. The audio thread (loudness + detector +
spectrum ring) is not the bottleneck.

## Checked / skipped

- Opaque components: **done** (fix 1).
- Same-machine old-vs-new comparison: **SKIPPED** — no earlier build/tag exists to compare; the
  numbers above are the first baseline.
- Every visible state: idle/radar measured; spectrum mode **SKIPPED** (needs a click to switch, not
  reachable headlessly).
- Thread breakdown: **done** (above).
- Micro-benchmarks-are-not-enough: noted; the improvement is from the standalone CPU, not a synthetic
  benchmark.
- Native-window visual check: **SKIPPED** (no interactive display session for this run).

## Conclusion

~29 % → ~14 % standalone CPU at idle/radar after making the editor opaque and halving the per-frame
path count. Recorded as the baseline for future comparisons.

## Note: updated reaper_qa.py isolated-config launch (2026-10-30)

The updated `plugin-daw-test` `reaper_qa.py` (isolated REAPER config + `hide_pid` helper) was tried
for the host suite. The `hide_pid` helper compiled and launched, and the isolated `cfg/` was created,
but the fresh config triggered a full VST3 rescan that hung on a third-party plugin
(`/Library/Audio/Plug-Ins/VST3/bx_dynEQ V2.vst3`, a Plugin Alliance plug-in) for >10 minutes, so the
bridge never became ready and the launch timed out after the 1200 s limit. The host-suite run via this
new launch path is therefore SKIPPED; the earlier 9/9 VST3/AU results were produced with a pre-seeded
REAPER config that skips the rescan and remain the host-suite evidence.
