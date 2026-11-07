# Plugin QA: AU: OpenGrid: RadarVU

| Check | Status | Detail |
|---|---|---|
| loads | PASS | AU: RadarVU (OpenGrid), 11 parameters |
| finite output | PASS | peak -9.1 dBFS |
| not silent | PASS | rms -26.4 dBFS |
| deterministic render | PASS | repeat differs by -240.0 dB |
| editor opens | PASS | 1100x613, 0.0% animates by itself |
| every parameter changes the sound | PASS |  |
| every parameter is shown in the GUI | PASS |  |
| state survives save and reopen | PASS |  |
| editor reopens | PASS | 5 close/open cycles |

| # | Parameter | Context | Audio | Visual |
|---|---|---|---|---|
| 0 | Display mode |  | excused (-240.0 dB) | ok (16.610%) |
| 1 | Colour skin |  | excused (-240.0 dB) | ok (13.414%) |
| 2 | Meter type |  | excused (-240.0 dB) | ok (5.577%) |
| 3 | Radar resolution |  | excused (-240.0 dB) | ok (4.440%) |
| 4 | True-peak ceiling |  | excused (-240.0 dB) | ok (5.141%) |
| 5 | Reference level |  | excused (-240.0 dB) | ok (4.908%) |
| 6 | Target level |  | excused (-240.0 dB) | ok (6.346%) |
| 7 | Sweep window |  | excused (-240.0 dB) | ok (5.166%) |
| 8 | Bypass |  | excused (-138.5 dB) | excused (5.495%) |
| 9 | Wet |  | excused (-138.5 dB) | excused (4.236%) |
| 10 | Delta |  | ok (-9.1 dB) | excused (4.890%) |
