# RadarVU — project profile

Build layout expected next to this repository:

| Path | What |
|---|---|
| `../audiogridder/JUCE` | JUCE checkout (used by CMake) |
| `../instruments/GoodLookinDri` | Drawing toolkit (Rust path dependency) |

Rust (`cargo`) and CMake 3.22+ are required.

## Repeatable commands

```sh
# DSP crate: detector + EBU calibration + radar smoke tests
cargo test --manifest-path dsp/Cargo.toml

# Configure and build (VST3 + AU + Standalone on macOS)
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --target RadarVU_VST3 -j 8
cmake --build build --target RadarVU_AU -j 8

# Standalone
cmake --build build --target RadarVU_Standalone -j 8
```

## Formats

macOS: VST3, AU, CLAP, LV2, AAX (when `../aax-sdk-2-8-1` is present) and Standalone. Windows/Linux: VST3, LV2, Standalone. JUCE comes from `../audiogridder/JUCE`.

## Scope documents

- Request: this conversation ("a LUFS meter with a radar display mode", JUCE + Rust + GoodLookinDri, concept sketch, new `RadarVU/` folder).
- `dsp/src/loudness.rs` is the ArcVu-verified BS.1770-5 / EBU 3341 / 3342 engine; do not weaken its gating or windows without rerunning the calibration tests.
- The C ABI (`plugin/radarvu.h`) mirrors ArcVu's drawing-command replay contract (kinds 1–12).

## Known constraints

- `RadarVuPaint.text` holds 47 bytes; longer labels are cut silently.
- A paint polyline holds at most 32 points (64 floats); trails are chunked in the Rust code.
- The GUI has no host-parameter panel yet; target/ceiling/window/mode are set from the header chips and are host-automatable. unit (LU/LUFS) and colour themes are not exposed (follow-up).
