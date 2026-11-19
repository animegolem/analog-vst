# Green Diode Overdrive

Modeled after the TS-808 circuit; trademarks belong to their owners. This is an independent project and is not endorsed by any trademark owner.

An effect-on model with input and output transistor followers, a feedback-diode clipping amplifier and active tone control. See `reference/circuit.cir` for sources and explicit modeling assumptions. The electronic bypass network is not modeled. SPICE validation compares two simplified implementations, not measured hardware. Do not treat the result as validated until the full validation summary passes.

## Validation with Level at 0

`settings.txt` validates Level at 0 (`level-zero`), not at a small value. There the signal reaches the output follower about 81 dB down at 1 kHz, and the harmonics Check compares lines near 1e-10 of full scale (-200 dBFS, the harness's floor). Three things had to hold before the Model and the Netlist could be compared that far down. No Tolerance was changed.

1. **The Model rests at its operating point.** Rounding in the Solver's matrices left the DC operating point 1.7e-12 V away from a rest state of the per-sample system on the output capacitor's row. With no input the output drifted to -0.97e-6 of full scale (-120 dBFS) over about half a second, at every Setting. `Circuit::new` now folds that miss into the DC column (`pin_operating_point`), and the test `silence_stays_at_rest` holds the drift under 1e-10 of full scale with each Control at both ends. The pin holds whatever point it is given, so `Circuit::new` first requires that the DC solve converged and that its result satisfies the DC equations (bounds and measurements are beside `MAX_DC_STEP` and `MAX_DC_RESIDUAL` in `dsp/src/circuit.rs`); a point that fails panics when the Plugin is initialized, not while it processes audio. The pin lasts until a Control moves: a pot change rebuilds the matrices without it, and without a DC solve or these checks.
2. **Each Section's ngspice run starts at rest.** The harness cut each Section at its first sample, where the band-limited onset is already ringing, so ngspice took its operating point with the input held at a nonzero value (0.2 mV for a 5 kHz tone at -16 dBFS). Each run now starts early, in the silence before the Section and ahead of the upsampling filter's ring (256 upsampled samples at 96 kHz, 396 at 44.1 kHz), and the harness asserts that the input is at rest there.
3. **ngspice's relative tolerance is 1e-7, not 1e-6.** At 1e-6 the 2nd harmonic of a 5 kHz tone at -16 dBFS sits about 30e-12 of full scale from where tighter runs put it.

The reference also has numerical uncertainty at these levels; the source of the remaining Model/reference residuals is not established. Runs from 1e-7 to 1e-9, with the maximum step from a quarter to a sixteenth of a sample, differ from one another by up to 26e-12 of full scale on that line, without converging as the setting tightens. A line that sits at the floor can therefore read up to about 1 dB differently from one simulator setting to the next. Single Sections with Level at 0, harmonics Check value against each reference (Tolerance 1 dB):

| reltol | maximum step | 1 kHz at -28 dBFS | 5 kHz at -16 dBFS |
|---|---|---|---|
| 1e-7 | 1/4 sample (used) | 0.24 dB | 0.11 dB |
| 1e-7 | 1/8 sample | 0.02 dB | 0.82 dB |
| 1e-8 | 1/4 sample | 1.13 dB | 0.38 dB |
| 1e-9 | 1/4 sample | 0.24 dB | 0.31 dB |

These are single-Section runs through the harness's own code, not a Full Validation.

## Validation

Full Validation of 2026-11-17 (`rm -rf validation && ./validate.sh`, ngspice 45, 96 kHz pack, all four references simulated again with the derived pre-roll and reltol=1e-7), run after the DC operating-point checks and the derived pre-roll went in: PASS at every Setting in `settings.txt`. Every Check value below is the same, to the digits the harness prints, as in the Full Validation before those changes.

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

What this run does not show:

- Convergence of the reference, or margin across simulator settings. The level-zero harmonics PASS holds at the simulator settings used; one tighter single-Section run in the table above reads 1.13 dB against the 1 dB Tolerance.
- Any sample rate but 96 kHz. The pre-roll at 44.1, 48, 88.2 and 192 kHz is covered by a harness test with ngspice replaced by a wire, not by a Full Validation.
- That the DC operating point is right in an absolute sense. The residual check uses the same element stamps and device equations as the DC solve, so an error in those passes it; only the comparison with ngspice covers them. The DC solve does not reach its own 1e-12 V stopping step on this Circuit (the last step measured at most 1.8e-9 V) and the cause is not established.
- Behaviour while a Control moves.

Cost: 30.7% of one core per stereo instance at 48 kHz (`cargo run -q --release --example bench -p dsp 48000`, one trial on an Apple M3 Max, no other validation or benchmark running, load averages 5.03, 16.14 and 24.41 over 1, 5 and 15 minutes). The Circuit runs at 576 kHz or above.

## Shared Solver (carried in 2026-11-19; Full Validation pending)

`dsp/src/circuit.rs`, `netlist.rs`, `processor.rs` and `render/src/main.rs` are now the same files in all eight Projects; `tools/check-shared.py` at the repository root fails when a copy drifts. This Project already had the DC operating point check and the rest-state pin. It gains: a sample whose solve does not converge or is not finite is held, counted and refused by `render`; after 1024 held samples in a row the circuit goes back to the operating point solved at construction; non-finite input samples and NaN Control values are rejected. This Circuit is solved one device at a time in one round, and that path now reports convergence too. The pin now corrects each row in the order the per-sample code sums it. The first carry was compared by rendering six short inputs (silence, 1 kHz at -12 and 0 dBFS, 5 kHz at 0 dBFS, 200 Hz at +12 dBFS, a 0 dBFS sweep) at every Setting through the `render` of the commit before the carry and through the new one: the largest difference is -126.4 dBFS (one unit in the last place of a 32-bit sample at a +14.6 dBFS peak), from the change to the pin; before that change the 24 renders were bit-identical. No sample was held in those renders or in eight harder ones per Setting (up to +20 dBFS, to 20 kHz, noise). A return to rest after Tone has moved restores a state 22 microvolts from the present operating point and costs the same 7.6 microvolt output transient as the Tone change itself (unit test, forced). The validation figures in this README were measured with the earlier Solver. **No Full Validation has been run with the shared Solver.**

**Second pass, 2026-11-19.** This Model's transistor has RE = 0.001 ohm in the Netlist, so the change that stops a terminal resistance of 0 from becoming a 1 milliohm resistor does not touch this Circuit (renders with and without it are bit-identical). `render` now also refuses a capture that contains a sample that is not finite, and when it refuses a capture it removes a file already at the output path (never the input). `render/tests/refused_capture.rs` is a fifth shared file and runs the built binary on a sine at the largest 32-bit float value with every Control at 1; this Model refuses it through held samples (2077). The ignored DC bounds test now builds the Model at 4096 seeded Control positions (not five values per Control) at the four base rates; worst over all of them: last Newton step 1.98e-9 (50 times under the bound, the least room of any Project; 58 times was reported from the five-value grid), relative DC residual 2.55e-16 (3928 times), pin distance 1.15e-8 V (87 times). The test asserts 10 times room on the step again, as this Project's own test did before the carry, and 30 times on the residual where that test asserted 1000: the Class A Transistor Drive leaves 35.8 and the test is shared. The fault-injection test `wrong_operating_point_is_not_pinned` uses a synthetic emitter follower, not this Model. Compared again with the `render` of the commit before the carry on five short inputs (silence, 1 kHz at -12 dBFS, 5 kHz at 0 dBFS, 440 Hz with 5.5 kHz at 0 dBFS peak, a 2 kHz burst at -3 dBFS between silences; 48 kHz mono) at all four Settings: the largest difference is -150.5 dBFS (`min-drive-tone`, 1 kHz), 3 of 20 renders are bit-identical, and with the pin removed from both builds all 20 are, so the difference is the change to the pin. No sample was held. Cost, bench run from the commit before the carry and from this Solver back to back, three times each in turn, one-minute load average 9: 31.5, 31.1, 31.1 before, 33.6, 33.6, 33.7 after (% of one core per stereo instance at 48 kHz). Not run with this Solver: `./validate.sh` in any mode and `./install.sh`. **No Full Validation has been run with the shared Solver.**

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
