# Three-Band Stack Boost

A stereo plugin modeled after the Tonemender circuit by runoffgroove.com (v1.0, 2005): a unity-gain buffer, a passive bass/mid/treble tone stack, and a non-inverting gain stage, on a 9 V supply. Trademarks belong to their owners. This project is not affiliated with or endorsed by them.

**Nothing was measured on hardware.** Validation compares the Rust model with an ngspice simulation of the same simplified circuit. The source page publishes no frequency-response curves, so there is no measured response to compare the transcription against either.

**Status: every setting passes a Full Validation** (2026-11-17, ngspice 45, the third of three runs; the first failed, see "Validation"). Cost: 9.5% of one core per stereo instance at 48 kHz. The model has no rail clipping: some inputs below full scale would clip in the hardware and pass cleanly here, see "Op-amp output check".

## Validation

Full Validation, 45 sections at 96 kHz, six settings, all PASS. These numbers are from the third run, made from an empty `validation/` directory with SPICE simulated again at all six settings, after a comment-only edit to the netlist changed its checksum. Every reported value is the same as in the second run. Worst value of each check over the six settings:

| Check | Worst value | Limit | Where |
|---|---|---|---|
| Frequency response | 0.04883 dB | 0.1 dB | `treble_level_min`, sweep at 0 dBFS |
| Static gain | 0.003281 dB | 0.1 dB | `all_min`, 5 kHz at -20 dBFS |
| Harmonics | 0.003281 dB | 1 dB | `all_min`, H2 of 5 kHz at -20 dBFS |
| Attack | 0 | 0.05 | no gain change in either capture |
| Release | 0 | 0.05 | no gain change in either capture |
| Intermodulation | 0.006078 dB | 1 dB | `all_min`, -20 dBFS |
| Burst waveform null | -73.34 dB | -40 dB | `all_min` |

Per setting:

| Setting | Controls | Frequency response | Static gain | Intermodulation | Burst null | Result |
|---|---|---|---|---|---|---|
| `default` | all 0.5 | 0.004607 dB | 0.0007034 dB | 0 | -98.79 dB | PASS |
| `all_max` | all 1 | 0.02757 | 6.05e-05 | 0 | -103.0 | PASS |
| `all_min` | all 0 | 0.04804 | 0.003281 | 0.006078 dB | -73.34 | PASS |
| `max_gain` | bass 0, mid 1, treble 1, level 1 | 0.02758 | 5.289e-05 | 0 | -104.2 | PASS |
| `bass_mid_min` | bass 0, mid 0, treble 1, level 0.5 | 0.003792 | 0.0009808 | 0 | -100.2 | PASS |
| `treble_level_min` | bass 1, mid 1, treble 0, level 0 | 0.04883 | 0.003263 | 0 | -94.94 | PASS |

How to read it:

- The circuit is linear. The attack and release checks have nothing to measure, and the harmonics and intermodulation checks compare two captures that are both at the noise floor.
- At `all_min` the output is about 97 dB down. The harness clamps amplitudes at -200 dBFS, so at that setting the harmonics and intermodulation values compare clamped numbers in both captures. The frequency response, static gain and burst null checks do measure the signal there.
- A value of 0 for harmonics or intermodulation means every product was below -90 dBc in both captures.

**The first Full Validation failed** the intermodulation check at two settings: `all_min` (1.398 dB at -6 dBFS) and `treble_level_min` (6.101 dB at -20 dBFS). The other four settings passed. The sidebands were in the SPICE reference, not in the model: see "Two fixes to the SPICE runner". No tolerance and no check was changed between the runs, and no setting was dropped.

Cost: `cargo run --release --example bench -p dsp 48000` reports 9.5% of one core per stereo instance at 48 kHz. Measured once, after the third run, with no other validation running; the machine was not idle (1-minute load average 15.58, just after the validation and the tests ended). The same command reported 9.4% after the second run.

## Sources and rights

Not legal advice.

| Tag | Source | Used for |
|---|---|---|
| S1 | [Project page](https://runoffgroove.com/tonemender.html) and its schematic image, marked "Tonemender v1.0 ©2005, runoffgroove.com" | Every value, the topology, pot values and tapers, the switch positions |
| S2 | [PCB layout PDF](https://runoffgroove.com/tonemender-pcb.pdf), page 1, layout by Pablo De Luca | Designators R1-R14, C1-C11, U1, SW1; a second reading of every value |
| S3 | [Perfboard layout image](https://runoffgroove.com/tonemender-perf.png) | Pot lug wiring; the 100 kΩ from the bass/mid junction to Vref |
| S4 | Texas Instruments TLC227x datasheet SLOS190H, [tlc2272.pdf](https://www.ti.com/lit/ds/symlink/tlc2272.pdf), pages 8-9 | Op-amp open-loop gain and gain-bandwidth product |

- The source page states: "Tonemender by runoffgroove.com is licensed under a Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License." Credit: runoffgroove.com.
- **Only component values and topology were taken from the sources, as facts.** The netlist and the model were written from those values. No drawing, image, layout, PDF, text or datasheet is copied into this repository, and none may be added.
- The PCB layout PDF states no licence of its own. It was read only.
- The datasheet is © Texas Instruments. Typical parameter values are used as facts.
- Open point: this repository is MIT and the source page is NonCommercial-ShareAlike. The working position is that values and topology are facts, so a netlist written from them is not an adaptation of the drawing. If it were held to be one, the two licences would conflict. That is not settled here. Commercial use of material the page licence covers would need permission from runoffgroove.com.
- The plugin scaffold is MIT. VST3 binaries built with nih-plug's VST3 bindings are GPLv3.

S1 and S2 agree on all 25 passive values. An independent second transcription from the same sources found no mismatch.

## Circuit as modeled

`reference/circuit.cir` is the netlist; `dsp/src/model.rs` is the same circuit part for part. Designators are those of S2.

| Control | Part | Wiring | Taper |
|---|---|---|---|
| Bass | R8, 500 kΩ audio | variable resistor | template audio law |
| Mid | R9, 100 kΩ audio | variable resistor to Vref, in parallel with R10 (100 kΩ) | template audio law |
| Treble | R7, 500 kΩ linear | divider | linear |
| Level | R13, 500 kΩ audio | variable resistor in the feedback of the gain stage | template audio law |

The gain stage's gain is 1 + Level / 4.7 kΩ above about 72 Hz: 1 at minimum, 11.6 at mid rotation, 107.4 at maximum. No single control mutes the output. With bass, mid and treble all at 0 the output is about -97 dB at 1 kHz.

## Assumptions and open values

Each of these is a choice, not a fact from the sources.

1. **Audio taper.** S1 says only "A". The model uses the template's law: two linear segments, 10% of the resistance at mid rotation. Real audio pots range from roughly 10% to 20% there.
2. **Rotation direction.** The sources number the pot lugs but do not state the direction. Lug 1 is taken as the counter-clockwise end, so every control increases clockwise.
3. **Op-amp type.** S1 allows TLC2272, NE5532 or TL072; S2 lists TL072. The model uses the TLC2272, which the designer suggests first: one pole, open-loop gain 5e4 and gain-bandwidth product 2.25 MHz, typical values at 25 °C from the ±5 V table of S4, used at 9 V.
4. **Source impedance.** Not in the sources. Ideal voltage source. R4 (10 MΩ) then has no effect; it is kept so the netlist matches the parts list.
5. **Load.** Not in the sources. 1 MΩ, in addition to R14 (100 kΩ).
6. **Supply.** Drawn as "+9v". Ideal 9.000 V. Vref is kept as a real node (R2, R3, C2), because signal current returns into it.
7. **Capacitors and resistors.** Ideal: no ESR, leakage, tolerance or electrolytic polarity; exact resistor and pot values.
8. **Pot ends** are floored at 1 Ω.
9. **Designator positions.** Where S2 gives two parts the same value (R2/R3, R5/R11, R10/R14, C3/C8, C9/C11), which one sits at which schematic position is inferred from the board placement; the copper was not traced. The values within each pair are identical, so the netlist is unaffected.
10. **Full scale.** 0 dBFS is 0.07 V peak at the input (`VOLTS_PER_FULL_SCALE`). See "Op-amp output check".

## Omitted, and what it costs

| Omitted or changed | Cost |
|---|---|
| Mid Shift switch (SW1), fixed at HI (open). The LO position is not modeled | With the controls at mid rotation the mid dip is at 646 Hz as modeled and would be at 380 Hz in LO (ngspice AC). The page says "around 700Hz" and "400Hz" |
| C4 (220 pF) in series with C5 (150 pF), and C7 (33 nF) in series with C8 (22 nF), each written as one capacitor: 89.189189 pF and 13.2 nF | At most 5.3e-06 dB from 20 Hz to 20 kHz over 11 control positions, against the two-capacitor form with 1e12 Ω across each open switch pole (ngspice AC) |
| Op-amp rail clipping | Not measured as an error: the model is honest only while both op-amp outputs stay inside the rails. Any input that would take an output past a rail passes cleanly where the hardware would clip. That includes some inputs below full scale: see "Op-amp output check" |
| Op-amp slew limit | Not measured as an error. An estimate for a single full-scale sine at the largest-gain position found: about 0.23 V/µs (1.81 V peak at 20 kHz), against 3.6 V/µs typical in S4. It is not a bound for other waveforms |
| Op-amp input bias current, input capacitance, common-mode limit, output impedance, noise | Not measured. S4 gives 1 pA typical bias current |
| Op-amp supply current through R1 | Not measured. S4 gives 3 mA maximum, which would lower +V by up to 0.3 V and Vref by 0.15 V. That changes headroom, not the linear response |
| The other op-amps the designer allows | At maximum level the response at 20 kHz moves by up to 1.7 dB among the three types (21.8 dB with the values used, 23.5 dB with gain 1e5 and 10 MHz). At mid level it moves by 0.2 dB (ngspice AC) |
| Bypass switching, jacks, power connector | None is drawn in the sources |
| Component tolerances and the real taper of each pot | Not measured |

**Why the series-equivalent capacitors.** With the switch open, the junction between C4 and C5 and the junction between C7 and C8 each touch two capacitors and nothing else, so they have no DC path. With both capacitors of each pair in the netlist, ngspice 45 reports a singular matrix, fails its operating-point search, and falls back to a transient operating point that is wrong (Vref near 0 V instead of 4.4978 V) while still exiting with code 0. The series equivalent is exact when the junction holds no trapped charge.

## Op-amp output check

Rail clipping is not modeled, and the gain stage reaches 107 V/V. The largest gain found at the second op-amp's output is +34.71 dB (54.4 V/V) at 5.0 kHz, with bass 0, mid 1, treble 1, level 1. A survey of 81 control corners and a 120-position search at maximum level found nothing higher. That is a search, not a proof of the maximum.

With 0 dBFS = 0.07 V peak:

- A 0.07 V, 5012 Hz sine into that position keeps the second op-amp's output within 0.555 V to 8.306 V from the operating point (start-up overshoot included); steady state is 0.690 V to 8.306 V. The first op-amp's output stays within 4.428 V to 4.568 V (ngspice transient).
- A 0 dBFS, 5 kHz burst through the harness's SPICE runner peaks at 3.91 V at the output, around the 4.498 V bias.
- S4 gives a typical output swing to 0.07 V below the positive rail and 0.09 V above the negative rail at light load.

These are spot checks with single sines and one tone burst. They do not bound the output for other input, and full scale is not a level below which the circuit cannot clip. An input that never exceeds 0.07 V peak but is not a single sine can take the second op-amp past its rails:

- At the same position, a sum of 20 harmonics of 1 kHz (1 kHz to 20 kHz), with phases chosen to add at that op-amp's output, faded in from zero over 2 ms and peaking at 0.06999 V, takes the second op-amp's output from 0.605 V to 9.470 V in the simplified circuit: above the 9 V supply (ngspice transient, 20 ms, 50 ns maximum step). The first op-amp's output stays within 4.427 V to 4.568 V. A sine's peak output is set by the gain at one frequency; this waveform's is set by the gains at all 20 frequencies together.

The hardware would clip on such an input; the model does not, and the validation does not test it. The circuit is linear, so the full-scale voltage does not change the sound. It only sets which tested signals stay inside the rails: single sines and the burst above, at the control positions searched.

## Checks made by hand

Operating point, input at 0 V. The same at every control position, because no direct current flows through any pot. `cargo test` asserts the model column against the ngspice column to 1e-6 V at three control positions (default, largest gain, every control at 0).

| Node | ngspice | Model |
|---|---|---|
| Vref | 4.497751 V | within 1e-6 V |
| Both op-amp outputs | 4.497661 V | within 1e-6 V |
| +V | 8.995502 V | within 1e-6 V |
| Output | 0.000000 V | within 1e-6 V |

Gain, ngspice `.ac` against a steady sine through the model's `render` binary at 96 kHz:

| Setting | 100 Hz | 1 kHz | 5 kHz | 20 kHz | Largest difference, 20 Hz to 20 kHz (7 frequencies) |
|---|---|---|---|---|---|
| `default`, ngspice | +2.8765 dB | +1.3185 | +12.5373 | +13.9741 | |
| `default`, model | +2.8765 | +1.3186 | +12.5382 | +13.9709 | 0.0032 dB at 20 kHz |
| `max_gain`, ngspice | +21.2330 | +27.7639 | +34.7113 | +28.2553 | |
| `max_gain`, model | +21.2330 | +27.7639 | +34.7113 | +28.2250 | 0.0303 dB at 20 kHz |
| `all_min`, ngspice | -80.4769 | -97.6637 | -97.5903 | -96.6218 | |
| `all_min`, model | -80.4769 | -97.6637 | -97.5899 | -96.6211 | 0.0007 dB at 20 kHz |

The largest difference over all six settings is 0.0303 dB, at 20 kHz in `all_max` and `max_gain`. That is frequency warping where the response falls above 5 kHz; the tolerance is 0.1 dB. The circuit rate is left at the template's 576 kHz.

One 0.1 s tone burst at 0 dBFS from the operating point, harness SPICE runner against the model: null -72.9 dB (`max_gain`, 5 kHz), -75.3 dB (`default`, 1 kHz), -70.7 dB (`all_min`, 1 kHz).

## Settings

`settings.txt` has six. Every control is at 0 and at 1 in a setting other than `all_min`. `all_min` is the near-mute corner (bass, mid and treble at 0). `max_gain` is the position of the op-amp output check.

## Two fixes to the SPICE runner

`harness/src/analog_harness/spice.py` differs from the other projects' copies in two ways.

**Section start.** The runner cut each section at its first sample. The upsampling and interpolation-compensation filters ring ahead of that sample, so the SPICE source started at a nonzero value (-0.2% of the amplitude for a 1 kHz tone), and ngspice solved the operating point as if that offset had always been at the input. The input capacitor then released it as a slow step. At `all_min`, where the circuit passes 20 Hz about 29 dB better than 1 kHz, that step put the burst null at -33.4 dB; a plain ngspice sine source agreed with the model to -70.7 dB. The runner now starts each section a few samples early, inside the silence before it, and asserts that the source starts from silence. `tests/test_harness.py` has a test that fails without the change (2.5e-3 against a limit of 2e-5). No tolerance and no check was changed.

**Maximum step.** The runner let ngspice step by at most a quarter of the oversampled period (1/1,536,000 s). With a strong 60 Hz tone and a weak 7 kHz tone, that reference showed sidebands around 7 kHz that a linear circuit cannot produce. The runner now uses a sixteenth. Measured on the intermodulation section alone:

| Setting, level | Reference option | Reference | Model |
|---|---|---|---|
| `treble_level_min`, -20 dBFS | quarter, reltol 1e-6 (as shipped) | -83.90 dBc | -122.13 dBc |
| | quarter, reltol 1e-7 | -83.63 | |
| | quarter, reltol 1e-8 | -88.59 | |
| | quarter, Gear integration | -88.02 | |
| | eighth, reltol 1e-6 | -91.01 | |
| | sixteenth, reltol 1e-6 (now) | -122.14 | -122.13 |
| `all_min`, -6 dBFS | quarter, reltol 1e-6 (as shipped) | -81.47 | -82.87 |
| | sixteenth, reltol 1e-6 (now) | -82.87 | -82.87 |

At a sixteenth the six fitted sidebands agree with the model's to within 1% each (1.5e-9 full scale at `treble_level_min`). The likely mechanism, not proven: at a quarter, ngspice's step control sometimes shortens the step, so the step size follows the signal; at a sixteenth every step is the maximum. `tests/test_harness.py` has a test on an RC low-pass that fails at a quarter (-84.7 dBc) and passes at a sixteenth (-110.1 dBc). The change also moved the `all_min` burst null from -56.92 dB to -73.34 dB. SPICE takes about 3.5 times as long per section (6 s to 21 s on the section above); the Full Validation took roughly 5 minutes per setting on this machine. No tolerance and no check was changed.

## Shared Solver (carried in 2026-11-19; Full Validation pending)

`dsp/src/circuit.rs`, `netlist.rs`, `processor.rs` and `render/src/main.rs` are now the same files in all eight Projects; `tools/check-shared.py` at the repository root fails when a copy drifts. This Circuit has no nonlinear device, so the junction limiting, the diode law and the convergence status do nothing here. What applies: the DC operating point check and rest-state pin, rejection of non-finite input samples and NaN Control values, and the hold of a non-finite result, which `render` refuses. The pin as first carried from green-overdrive left this Circuit drifting by 1.1e-9 of full scale in silence at its defaults, over the 1e-10 that `silence_stays_at_rest` allows: one unit in the last place of the 100 uF bias capacitors' history is 2e-13 A. The pin now corrects each row in the order the per-sample code sums it, and the test passes. Compared by rendering six short inputs (silence, 1 kHz at -12 and 0 dBFS, 5 kHz at 0 dBFS, 200 Hz at +12 dBFS, a 0 dBFS sweep) at every Setting through the `render` of the commit before the carry and through the new one: the largest difference is -102.4 dBFS, one unit in the last place of a 32-bit sample at a +39 dBFS peak (`all_max`), all of it from the pin (with the pin removed the 36 renders are bit-identical). The test `operating_point_matches_spice` still passes with the longer operating-point slice. The validation figures in this README were measured with the earlier Solver. **No Full Validation has been run with the shared Solver.**

## Build and test

```
env CARGO_TARGET_DIR=$PWD/target cargo test
(cd harness && uv run pytest -q)
rm -rf validation && ./validate.sh
./install.sh
```

Install only after every setting passes a Full Validation. Run one validation at a time.
