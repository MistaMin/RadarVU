# Stage contracts: <product>

One row per boundary. Do not infer a property from a class name.

| Boundary | In units | Out units | RMS/peak range | Channels | DC behaviour | History / reset | Loading or feedback crosses it | Bypass behaviour (tails, hidden state, fade, latency) |
|---|---|---|---|---|---|---|---|---|

## Routing truth table

| Layout | Controls | Detector | Audio histories | Notes |
|---|---|---|---|---|
| Mono | | | | |
| Stereo linked | shared | shared GR | separate | |
| Stereo unlinked | shared knobs | independent | separate | |
| Dual mono | independent | independent | separate | tolerance seed per channel |
| External sidechain absent / mono / stereo | | | | |

## Signal diagram

```text
(audio path)
(control paths: sidechain -> detector -> gain state; load estimates -> supply -> stages)
(meter taps)
```
