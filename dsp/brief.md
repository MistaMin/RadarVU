# DSP brief: <product>

Status: draft / approved by <user> on <date>. Defaults are marked (default); everything else was answered.

## Product and owners
Product / manufacturer / project dir:
Owners: product, DSP, host integration, measurement, sound acceptance, release (one person may hold several):
Framework / UI toolkit / build:
Formats (built, shipped) / platforms / architectures:
Channel layouts / sidechain / insert loops:
Names: product, flavour names, forbidden terms file (outside the repo):
Licence: code, binaries, third-party notices:
Version policy (user decides):

## Reference
Authority files (path, revision, sha256):
Known revision gaps between schematic, deck and manual:
Private material location (git-ignored) and what must never ship:
Conflict table (item | sources | recommendation | user decision):

## Behaviour contract
Stage order and signal diagram:
Always-on analogue behaviours (saturation, transformer, tolerance, noise, hum, channel variation):
UI-only features (skins):
Level convention (0 dBFS = ? dBu, peak or RMS):
Controls and ranges follow the circuit; labelled extensions:
Allowed sound differences:
Original engine retention / fallback policy:

## Rates, oversampling, latency
Supported rates / buffers:
Modes and factors at 44.1/48 kHz, rule for higher rates, +L variants:
Reported latency policy:
Offline render policy:

## Budgets
CPU target (mono/stereo, default mode, % of a core on <machine>):
Worst-callback budget at 48 kHz/64 and 96 kHz/32:
Instance count expected in a session:
Declared workload matrix:

## Models
Per stage: engine, nonlinear parts, fitted data source:
Detector: circuit / behavioural (decision):
Supply model: on/off, internal rate (deviation signed off by):

## Acceptance
Thresholds: stage vs ngspice (dB/deg), harmonics, IMD, alias, null floor, attack peak, release, recovery:
Holdout plan (identification / development / final):
Listening: material, owner, normalisation, blind or not:
Hosts and machines: DAWs, device, buffer, rate:
Gaps allowed to ship open (owner, next experiment, threshold):

## Rules in force
No threads. No sound-changing approximations without approval. No clean DSP path. No push/sign/publish
without the user. Signing in the user's terminal. Dev VST3 in ~/Library for testing, removed after.
