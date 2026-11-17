# Plugin QA: AU: RadarVU (OpenGrid)

| Check | Status | Detail |
|---|---|---|
| loads | PASS | AU: RadarVU (OpenGrid), 11 parameters |
| finite output | PASS | peak -3.5 dBFS |
| not silent | PASS | rms -14.9 dBFS |
| deterministic render | PASS | repeat differs by -240.0 dB |
| editor opens | PASS | 1080x613, 5.0% animates by itself |
| every parameter changes the sound | PASS |  |
| every parameter is shown in the GUI | PASS |  |
| state survives save and reopen | PASS |  |
| editor reopens | PASS | 5 close/open cycles |

| # | Parameter | Context | Audio | Visual |
|---|---|---|---|---|
| 0 | Display mode |  | excused (-240.0 dB) | ok (21.867%) |
| 1 | Colour skin |  | excused (-240.0 dB) | ok (24.022%) |
| 2 | Meter type |  | excused (-240.0 dB) | ok (17.981%) |
| 3 | Radar resolution |  | excused (-240.0 dB) | ok (19.550%) |
| 4 | True-peak ceiling |  | excused (-240.0 dB) | ok (18.978%) |
| 5 | Reference level |  | excused (-240.0 dB) | ok (8.897%) |
| 6 | Target level |  | excused (-240.0 dB) | ok (21.810%) |
| 7 | Sweep window |  | excused (-240.0 dB) | ok (20.410%) |
| 8 | Bypass |  | excused (-240.0 dB) | excused (20.333%) |
| 9 | Wet |  | excused (-240.0 dB) | excused (6.192%) |
| 10 | Delta |  | ok (-3.5 dB) | excused (19.155%) |
