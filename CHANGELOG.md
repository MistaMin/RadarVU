# Changelog

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
