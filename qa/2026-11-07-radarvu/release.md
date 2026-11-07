# RadarVU 1.0.1 release preparation (2026-11-07)

Followed the wiki's Make-Plugin-Release (`DSP Engineering Wiki/skills/Make-Plugin-Release`), Step 5.4: bundle zip plus manifest.
Decisions (owner): no Standalone in the zip; sign and notarise from the agent session.

| Step | Result |
|---|---|
| Signing environment (`verify_signing_env.sh`) | PASS: certificates, `AC_NOTARY`, wraptool 6.0.1, signing iLok |
| Formats (`verify_formats.sh`) | PASS: VST3, AU, CLAP, LV2, AAX, Standalone built (Standalone not shipped in this zip) |
| Standalone icon | PASS |
| Manual and screenshots | manual and README match 1.0.1; PDF rebuilt |
| Universal build, min macOS 11.0, build class `modern` | PASS; each bundle stamped 1.0.1 / modern / 11.0, arm64 + x86_64 |
| Tests on the build | 54 of 54 cargo tests; UI audit state round trip PASS, all parameters shown, FILL/FFT toggle |
| Signing | Developer ID on VST3, AU, CLAP, LV2 library; AAX Developer ID plus PACE Eden "signed, not wrapped" and `wraptool verify` OK; `codesign --verify --deep --strict` OK on the unzipped bytes |
| Notarisation | zip accepted by Apple (a zip cannot be stapled; `spctl` on a zip reports "no usable signature", which is expected) |
| Manifest | schema 2, modern, min_os 11.0, universal, installs vst3/au/clap/lv2/aax, SHA-256 `96904c82...6cfe`, 46 117 771 bytes (second build: rezipped without AppleDouble `._` entries after a zip check found 82; the first zip also differed because a Terminal run included the Standalone) |

Tests on the signed zip bytes (unpacked): AAX Validator PASS on all six tests (page-table warning about automatable parameters not on the default page, as in 1.0.0); JUCE-hosted VST3 music audit PASS (audio bit-exact, state round trip, editor recreated, 8 parameters); `dsp_lint` PASS on the payload (0 findings) and on the repository with commit messages (0 findings after removing two AI tool names from tracked QA files); company name OpenGrid in the VST3 (class IDs `ABCDEF019182FAEB4F70477252645655`, `ABCDEF011234ABCD4F70477252645655`), AU (`OpenGrid: RadarVU`) and CLAP; bundle id `com.opengrid.radarvu`.

Staged: copied to `Release Binaries/MistaMin/RadarVU/1.0.1/` (zip + manifest) on the owner's instruction, macOS only; no `RELEASE-READY` yet.

Not done: `auval` and the REAPER AU/VST3 suites on the signed bytes (they need a temporary install in the user plug-in folders, which the owner has not approved for this round); Windows archive built on the owner's Windows machine from the `v1.0.1` tag, then added to the same manifest; Pro Tools; `RELEASE-READY`, upload, git tag and push.
Note: before the questions were answered, one Standalone zip was submitted to Apple and accepted; it is not part of this release.
