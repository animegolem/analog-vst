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
