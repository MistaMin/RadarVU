# Plugin QA: VST3: RadarVU

| Check | Status | Detail |
|---|---|---|
| loads | PASS | VST3: RadarVU (OpenGrid), 10 parameters |
| finite output | PASS | peak -3.2 dBFS |
| not silent | PASS | rms -14.9 dBFS |
| deterministic render | PASS | repeat differs by -240.0 dB |
| editor opens | PASS | 1115x623, 25.2% animates by itself |
| every parameter changes the sound | PASS |  |
| every parameter is shown in the GUI | PASS |  |
| state survives save and reopen | PASS |  |
| editor reopens | PASS | 5 close/open cycles |

| # | Parameter | Context | Audio | Visual |
|---|---|---|---|---|
| 0 | Meter type |  | excused (-240.0 dB) | ok (5.064%) |
| 1 | Reference level |  | excused (-240.0 dB) | ok (8.296%) |
| 2 | Target level |  | excused (-240.0 dB) | ok (6.920%) |
| 3 | True-peak ceiling |  | excused (-240.0 dB) | ok (5.447%) |
| 4 | Sweep window |  | excused (-240.0 dB) | ok (9.340%) |
| 5 | Display mode |  | excused (-240.0 dB) | ok (1.420%) |
| 6 | Bypass |  | excused (-240.0 dB) | excused (1.702%) |
| 7 | Bypass |  | excused (-138.5 dB) | excused (1.994%) |
| 8 | Wet |  | excused (-138.5 dB) | excused (3.563%) |
| 9 | Delta |  | ok (-3.2 dB) | excused (4.451%) |
