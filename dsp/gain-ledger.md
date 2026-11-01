# Gain ledger: <product>

Convention: dBFS is **peak** / **RMS** (circle one). 0 dBFS = <x> dBu. Alignment: <0 VU = +4 dBu at -18 dBFS>.
Interface pad: <name, dB, why>. Applied once, in <file:line>, not in both stage and wrapper.

```text
scale = 0.775 * sqrt(2) * 10^((4 - alignmentDbfs - padDb) / 20)      # replace with the product's formula
internal = host * scale * 10^(inputTrimDb / 20)
host out = internal / scale * 10^(outputTrimDb / 20)
```

| Point | Units | Stimulus | Expected | Measured | Overload policy | Evidence |
|---|---|---|---|---|---|---|
| Host input | float | calibration sine | | | allow / clamp | |
| Preamp input | V | same | | | | |
| Compressor input | V | preamp at nominal | | | detector threshold reference | |
| EQ output | V | max boost cases | | | | |
| Host output | float | default channel | | | clip / meter policy | |

Test with noise, tolerances and supply variation off first; add them after the deterministic path agrees.
Intermediate taps captured so the first divergence can be located.

## Meters

| Tap | Quantity | Ballistics | Test signals (DC, sine, burst, silence, clip; editor closed) |
|---|---|---|---|
