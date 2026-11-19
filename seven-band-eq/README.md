# Seven Band Graphic — experimental

An independent component-level model of the Music Thing Modular Graphic EQ, designed by Tom Whitwell / Music Thing Modular. Trademarks belong to their owners; this project is not endorsed by the maker. This is not a verified hardware replica: nothing was measured on hardware. The seven faders are 10 kΩ linear with a centre detent (Alpha RA2045F-20R-15LC-B10K-C). The designer's schematic and Rev 1 BOM give them no value; the part is named in the kit seller's parts list.

## Sources and licensing

- [Published schematic](https://musicthing.co.uk/collateral/graphic_schematic.pdf), compared visually to the Rev 1 BOM.
- [Graphic repository and Rev 1 BOM](https://github.com/TomWhitwell/Graphic), commit `81ee3c67aaca1dc377d09ab53c3737f50a519d74`. The repository contains no LICENSE file and its README does not grant a license. The [module page](https://www.musicthing.co.uk/Graphic-EQ/) does not state a module-specific license. The board and panel files carry the silkscreen text "CCBYSA", with no version, and there is no license file; what that marking covers has not been assessed. Only independently transcribed circuit facts are used; neither the drawing nor BOM is redistributed.
- [Kit seller's parts list](https://docs.google.com/spreadsheets/d/1X_Aj_VlKoC7dwV-nU4a9jN4AvfiYIoKrot278mRdBLQ) ("Music Thing - Graphic EQ BOM", Thonk, read 2026-11-17): the only source for the faders. It lists `Alpha RA2045F-20R-15LC-B10K-C`, quantity 7, "Alpha 20mm Slider - B10K Centre Detent", against the seven band designators. Thonk's slider page reads "B = Linear", and the Alpha datasheet lists taper B as linear. Archived copies of that page (2019-04-23, 2022-01-17) say "B10K ... for use with the Graphic EQ PCB". Neither the designer's files nor the datasheet states a tolerance.
- The plugin scaffold is MIT; VST3 binary distribution uses GPLv3 bindings. This is not legal advice.

## Circuit assumptions

Seven bands, input and output op-amps and seven filter followers; LED circuits, supply-protection diodes and decoupling are omitted. The seven sliders are 10 kΩ linear (B taper), per the kit seller's parts list; each half is floored at 1 Ω. R/C values follow the Rev 1 BOM; the eight capacitor positions it marks as not populated (C31 to C38) are left out of the netlist and the model. Op-amps approximate NE5532 with a single pole, gain 100,000, GBW 10 MHz (the typical figure in revision J of the [TI datasheet](https://www.ti.com/lit/gpn/NE5532); the current revision gives 12 MHz, and the model keeps 10 MHz); no clipping, slew, noise or output-current limits are modeled. Ideal input source, 1 MΩ load. The DSP uses 0.5 V peak per full scale, not 5 V. A swept single-sine ngspice AC check with all faders at maximum yielded the highest op-amp output of 1.552 V peak at 0.5 V peak input, versus 15.513 V if mapped to 5 V peak. This proves only the swept-sine condition, not a bound for arbitrary broadband signals or actual output swing; real NE5532 stages clip before ideal rails. The netlist header still reads "pending rail check": the single-sine check above is done, a general bound is not, and the header is left as it is because editing it would change the netlist checksum that the validation below is keyed to.

## AC response and comparison

With one slider varied and all others centered, ngspice AC analysis relative to flat gives: 58.0 Hz ±6.83 dB, 155.4 Hz ±6.33 dB, 401.8 Hz ±6.18 dB, 989.7 Hz ±6.10 dB, 2.53 kHz ±6.12 dB, 6.45 kHz ±6.11 dB, and 17.4 to 17.5 kHz ±6.48 dB. The peaks are broad and the frequencies are read off a grid of 2000 points per decade (steps of 0.12%); boost and cut peak within 0.4% of each other. An earlier 200-point grid gave 154.6 Hz and 2.539 kHz for the second and fifth bands; the dB figures did not change. The physical labels are 63, 160, 410, 1k, 2.5k, 7.7k, 16k Hz. The range is set by fixed resistors, not by the sliders: R1 = R2 = 2.7 kΩ against the 1.8 kΩ gyrator resistors give a ceiling of 20·log10(1 + 2700/1800) = 7.96 dB. One band alone, with the other six removed, reaches ±7.7 to 7.9 dB; the other six centred bands load it down to the figures above. Slider values of 1 kΩ, 5 kΩ, 20 kΩ, 50 kΩ, 100 kΩ and 1 MΩ in the same netlist keep the per-band range between ±6.1 and ±7.7 dB (at 1 kΩ the all-centred response is no longer flat, by up to 0.62 dB). All seven sliders at maximum give +9.83 dB at 988 Hz, and all at minimum −9.84 dB. One web article says each band adjusts by 15 dB and cites nothing. ±15 dB is the published figure for the Boss GE-7 pedal, which the maker's page names as the module's inspiration; whether the article took it from there is not known. That range is not reachable with the published parts. The band labelled 7.7 kHz peaks near 6.3 kHz alone and 6.45 kHz in circuit with the published values (10 nF + 2.2 nF, 470 pF, 62 kΩ, 1.8 kΩ); the schematic, the Rev 1 BOM and the kit parts list agree on them. None of this was measured on hardware. The maker's page lists the Douglas Self reference frequencies but does not specify a gain range. `measure_ac.py` reproduces the per-band figures and the all-at-maximum op-amp outputs; the slider-value and single-band figures came from one-off ngspice runs on edited copies of the netlist.

## Tests

Full validation, 2026-11-17, after the netlist header edit (all five SPICE references simulated again with ngspice 45, full 45-section pack): PASS at all 5 settings. Worst value of each check across the settings:

| Check | Worst value | Limit | Setting |
|---|---|---|---|
| frequency response | 0.090 dB | 0.1 dB | `inverse`, sweep at -12 dBFS |
| static gain | 0.0078 dB | 0.1 dB | `alternating`, 5000 Hz at 0 dBFS |
| harmonics | 0 | 1 dB | all (no harmonics in either) |
| attack | 0 | 0.05 | all (no gain change in either) |
| release | 0 | 0.05 | all (no gain change in either) |
| intermodulation | 0 | 1 dB | all |
| burst waveform null | -89.06 dB | -40 dB | `allmin` |

Frequency response per setting: `default` 0.056, `allmax` 0.045, `allmin` 0.073, `alternating` 0.037, `inverse` 0.090 dB. The `inverse` margin is thin (0.090 against 0.1); the cause was not isolated in this run. Rerun: `rm -rf validation && ./validate.sh`.

`cargo test` (9 passed) and `uv run pytest -q` in `harness/` (7 passed) pass.

Cost: 25.7% of one core per stereo instance at 48 kHz (`cargo run -q --release --example bench -p dsp 48000`, one run, no other validation or benchmark running, load average about 16 on 16 cores). An earlier run alone measured 25.9%; a run taken while other validations were going measured 81.7% and is not a fair figure.

The circuit is linear: there are no modeled harmonics or compression, so those checks pass trivially, and the frequency response, static gain and burst null carry the comparison. Use DAW plugin rescan after installation.

## Shared Solver (carried in 2026-11-19; Full Validation pending)

`dsp/src/circuit.rs`, `netlist.rs`, `processor.rs` and `render/src/main.rs` are now the same files in all eight Projects; `tools/check-shared.py` at the repository root fails when a copy drifts. This Circuit has no nonlinear device, so the junction limiting, the diode law and the convergence status do nothing here. What applies: non-finite input samples and NaN Control values are rejected, a non-finite result is held and counted, and `render` refuses a capture with a held sample. Compared by rendering six short inputs (silence, 1 kHz at -12 and 0 dBFS, 5 kHz at 0 dBFS, 200 Hz at +12 dBFS, a 0 dBFS sweep) at every Setting through the `render` of the commit before the carry and through the new one: all 30 renders are bit-identical. The operating point is 0 V at every Control position tried, so the pin changes nothing and a return to rest restores the present operating point. `cargo test` now runs 24 tests and ignores one (the DC bounds measurement, which takes a release build). The validation figures in this README were measured with the earlier Solver. **No Full Validation has been run with the shared Solver.**
