# Green Diode Overdrive

Modeled after the TS-808 circuit; trademarks belong to their owners. This is an independent project and is not endorsed by any trademark owner.

An effect-on model with input and output transistor followers, a feedback-diode clipping amplifier and active tone control. See `reference/circuit.cir` for sources and explicit modeling assumptions. The electronic bypass network is not modeled. SPICE validation compares two simplified implementations, not measured hardware. Do not treat the result as validated until the full validation summary passes.

## Validation with Level at 0

`settings.txt` validates Level at 0 (`level-zero`), not at a small value. There the signal reaches the output follower about 81 dB down at 1 kHz, and the harmonics Check compares lines near 1e-10 of full scale (-200 dBFS, the harness's floor). Three things had to hold before the Model and the Netlist could be compared that far down. No Tolerance was changed.

1. **The Model rests at its operating point.** Rounding in the Solver's matrices left the DC operating point 1.7e-12 V away from a rest state of the per-sample system on the output capacitor's row. With no input the output drifted to -0.97e-6 of full scale (-120 dBFS) over about half a second, at every Setting. `Circuit::new` now folds that miss into the DC column (`pin_operating_point`), and the test `silence_stays_at_rest` holds the drift under 1e-10 of full scale with each Control at both ends. The pin lasts until a Control moves: a pot change rebuilds the matrices without it.
2. **Each Section's ngspice run starts at rest.** The harness cut each Section at its first sample, where the band-limited onset is already ringing, so ngspice took its operating point with the input held at a nonzero value (0.2 mV for a 5 kHz tone at -16 dBFS). Each run now starts 256 upsampled samples early, in the silence before the Section, and the harness asserts that the input is at rest there.
3. **ngspice's relative tolerance is 1e-7, not 1e-6.** At 1e-6 the 2nd harmonic of a 5 kHz tone at -16 dBFS sits about 30e-12 of full scale from where tighter runs put it.

What is left is the simulator's own resolution. Runs from 1e-7 to 1e-9, with the maximum step from a quarter to a sixteenth of a sample, differ from one another by up to 26e-12 of full scale on that line, without converging as the setting tightens. A line that sits at the floor can therefore read up to about 1 dB differently from one simulator setting to the next. Single Sections with Level at 0, harmonics Check value against each reference (Tolerance 1 dB):

| reltol | maximum step | 1 kHz at -28 dBFS | 5 kHz at -16 dBFS |
|---|---|---|---|
| 1e-7 | 1/4 sample (used) | 0.24 dB | 0.11 dB |
| 1e-7 | 1/8 sample | 0.02 dB | 0.82 dB |
| 1e-8 | 1/4 sample | 1.13 dB | 0.38 dB |
| 1e-9 | 1/4 sample | 0.24 dB | 0.31 dB |

These are single-Section runs through the harness's own code, not a Full Validation.

## Validation

Full Validation of 2026-11-17 (`rm -rf validation && ./validate.sh`, ngspice 45, 96 kHz pack, all four references simulated again with the pre-roll and reltol=1e-7): PASS at every Setting in `settings.txt`.

| Setting | Controls | Result |
|---|---|---|
| default | defaults | PASS |
| min-drive-tone | drive=0 tone=0 level=1 | PASS |
| max-drive-tone | drive=1 tone=1 level=1 | PASS |
| level-zero | level=0 | PASS |

Worst value of each Check over the four Settings:

| Check | Worst value | Tolerance | Where |
|---|---|---|---|
| frequency response | 0.061 dB | 0.1 dB | max-drive-tone, sweep at -40 dBFS |
| static gain | 0.0026 dB | 0.1 dB | default, 5 kHz at -4 dBFS |
| harmonics | 0.485 dB | 1 dB | level-zero, H3 of 5 kHz at -36 dBFS |
| attack | 0 ms | 0.05 ms | every Setting |
| release | 0 ms | 0.05 ms | every Setting |
| intermodulation | 0.0096 dB | 1 dB | level-zero, -20 dBFS |
| burst waveform null | -80.02 dB | -40 dB | max-drive-tone |

At level-zero alone: frequency response 0.046 dB, static gain 0.0026 dB, harmonics 0.485 dB, intermodulation 0.0096 dB, burst waveform null -93.06 dB. The level-zero harmonics value is one run against one reference; the table in the section above shows how far that reading can move with the simulator's settings.

Cost: 30.8% of one core per stereo instance at 48 kHz (`cargo run -q --release --example bench -p dsp 48000`, one trial on an Apple M3 Max with nothing else running). The Circuit runs at 576 kHz or above.

## Assumptions

`reference/circuit.cir` lists them with the sources. In short:

- Effect-on signal path only; the electronic bypass and its switching parts are not modeled.
- Transistors use a reduced DC Gummel-Poon model (no junction charge, transit time or leakage), the same in the Model and the Netlist.
- The clipping diodes use the template's small-signal diode model, with no reverse breakdown.
- Each op-amp half is a single pole with 100 dB gain and 3 MHz gain-bandwidth, with no rail clipping or slew limit. A spot check with ngspice at maximum drive kept the op-amp outputs between 3.83 and 5.14 V on a 9 V supply.
- The Tone pot is taken as linear and the Level pot as audio taper.
- Ideal 9 V supply, ideal voltage input, 1 M load; 0 dBFS is 0.2 V peak.
- Capacitor series resistance, noise and thermal drift are omitted.
- The rest-state pin in the Solver lasts until a Control moves; after that the output can drift to about -120 dBFS with no input. Validation does not exercise a Control change.
