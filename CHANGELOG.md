# Changelog

## Unreleased

- Radar resolution menu: 180, 360, 720 (default) or 1440 points per revolution, sampled at their capture times independent of the host buffer size. A thin contour stays visible over the dimmed fill. In Loudness mode the trace follows the 400 ms momentary measurement.
- Six colour skins (Stock Dark, Retro Console, Ivory Terminal, Matrix, Modern Console, Studio Amber) covering Radar and Spectrum modes, controls and menus.
- Two new host parameters, Radar resolution and Colour skin, which automate and save with the project. Older projects reopen with 720 points and Stock Dark. AAX page tables list both.
- QA: UI audit exports every skin in both modes at 440, 1100 and 4000 px with contact sheets; optional real-music and hosted-VST3 audit.

## 1.0.0

First release by OpenGrid.

- A polar radar display: the sweep turns once per window (5 to 30 s) and the radius is the level on the selected meter's own scale, with a multicolour heat-map fill that fades with age.
- Meters: VU (IEC 60268-17), RMS (absolute dBFS), K-12/K-14/K-20, the DIN, Nordic, BBC and EBU peak programme meters, and Loudness. Peak with hold, over and clip counters, and a phase-correlation gauge.
- Loudness panel measured to ITU-R BS.1770-5 and checked against the EBU Tech 3341 and 3342 test signals: momentary, short-term, integrated, true peak and loudness range, with a selectable target and true-peak ceiling.
- Stereo goniometer and a multicolour spectrum analyser mode. The colour fill and the analyser can be switched off.
- The phase-correlation window is the same length at every sample rate.
- Host-automatable parameters for meter type, reference, target, ceiling, sweep window and display mode, saved with the project.
- The audio passes through unchanged.
- Formats: VST3, Audio Unit, CLAP, LV2, AAX (macOS) and Standalone.
