# Plugin QA: VST3: RadarVU (OpenGrid)

| Check | Status | Detail |
|---|---|---|
| loads | PASS | VST3: RadarVU (OpenGrid), 12 parameters |
| finite output | PASS | peak -3.5 dBFS |
| not silent | PASS | rms -14.9 dBFS |
| deterministic render | PASS | repeat differs by -240.0 dB |
| editor opens | PASS | 1100x613, 0.0% animates by itself |
| every parameter changes the sound | PASS |  |
| every parameter is shown in the GUI | PASS |  |
| state survives save and reopen | PASS |  |
| editor reopens | PASS | 5 close/open cycles |

| # | Parameter | Context | Audio | Visual |
|---|---|---|---|---|
| 0 | Meter type |  | excused (-240.0 dB) | ok (20.296%) |
| 1 | Reference level |  | excused (-240.0 dB) | ok (20.516%) |
| 2 | Target level |  | excused (-240.0 dB) | ok (7.826%) |
| 3 | True-peak ceiling |  | excused (-240.0 dB) | ok (21.855%) |
| 4 | Sweep window |  | excused (-240.0 dB) | ok (20.525%) |
| 5 | Display mode |  | excused (-240.0 dB) | ok (24.085%) |
| 6 | Radar resolution |  | excused (-240.0 dB) | ok (20.552%) |
| 7 | Colour skin |  | excused (-240.0 dB) | ok (25.285%) |
| 8 | Bypass |  | excused (-240.0 dB) | excused (22.122%) |
| 9 | Bypass |  | excused (-240.0 dB) | excused (19.725%) |
| 10 | Wet |  | excused (-3.5 dB) | excused (22.172%) |
| 11 | Delta |  | ok (-3.5 dB) | excused (4.764%) |
