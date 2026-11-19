# Feedback Pair Drive

A stereo plugin modeled after the Music Thing Modular London Drive circuit (Proto 1, Rev 1 schematic, one sheet dated 14/11/2022); trademarks belong to their owners. This is an independent project and is not endorsed by any trademark owner. Each stereo side simulates its own copy of the Circuit.

The Circuit has four transistors. A long-tailed NPN pair (Q6, Q7) and a PNP second stage (Q3) form an inverting feedback amplifier with a 100k feedback resistor. An NPN phase splitter (Q5) with equal 2.7k collector and emitter resistors follows it. There is one Control, Level.

**Status: validated, all Checks pass at all four Settings, with no held sample in the Pack** (Full Validation of the Solver as it was before the shared Solver was carried in on 2026-11-19, SPICE references from ngspice 45; none has been run since, see "Shared Solver"). Both bundles were built and installed after that run. The Plugin is **Offline-only**: it costs 172.7% of one core per stereo instance at 48 kHz on the test machine, so it can be bounced or frozen but not played live there.

**Nothing was measured on hardware.** Validation compares two simplified descriptions of one Circuit: `reference/circuit.cir` (ngspice) and `dsp/src/model.rs`. No unit was measured and no recording was consulted.

## Sources

| Source | What it gave |
|---|---|
| https://www.musicthing.co.uk/collateral/london_drive_Rev1_Schematic.pdf | Every component value and every connection. The PDF has no text layer; values were read from renders at 600 dpi, and a second reader repeated the reading independently with the same result |
| https://www.musicthing.co.uk/London-Drive/ | Description only: non-inverted and inverted outputs, three transistors for audio. No values |
| https://www.thonk.co.uk/wp-content/uploads/2022/12/mtm-london-drive-build-doc-v1.pdf | Assembly steps only. No parts list, no values |
| https://www.onsemi.com/pub/Collateral/MMBTH10.LIB | MMBTH10 SPICE model (Fairchild, 2004) |
| https://assets.nexperia.com/documents/spice-model/BC858B.txt | BC858B SPICE subcircuit (Nexperia, extraction 19/2020) |
| https://www.onsemi.com/pdf/datasheet/mpsh10-d.pdf | MMBTH10 ratings: VEBO 3.0 V, VCBO 30 V, VCEO 25 V |

No parts list is public. None was obtained.

## Reduced configuration

| Choice | Decision | Cost |
|---|---|---|
| Input | IN_HIGH only | |
| IN_LOW (second pot gang, R25 1k) | Unpatched and left out. The jack's switch contact is not connected, so the branch ends open | None: exact |
| Supply jumper SJ2 | On -12 V. The -9 V shunt regulator (Q2, R3, R4, R5, C3, C4) then feeds nothing and is left out | None in this configuration. With the tail and R2 on -9.2 V the gain at Level 1 would be 0.37 dB lower |
| Output | NON-INV (Q5 collector through C5). In ngspice `out` is at +1.0 degree to `in` at 1 kHz | |
| INVERT output | Terminated in the same assumed 100k load | An unpatched jack would leave C6 open: 0.23 dB less gain at NON-INV |
| R1 (0 ohm) | A wire. The schematic's note says resistance can be added there to reduce the maximum gain | None |
| Lamp driver | Left out: see below | Under 0.01 dB |

### The lamp driver is left out

The board has an incandescent lamp with a peak detector and driver (R10 10k, a dual MMDT3904, an MMBT3906, R11 100k, R12 330k, R13 10k, R14 47 ohm, C8 100n). The lamp is not a part the Solver supports. It is outside the audio path: its only load on the audio is the driver's base current through R10 from the amplifier output.

Measured in ngspice with the driver attached (onsemi MMBT3904 and MMBT3906 models, lamp taken as 100 ohm, which is an assumption): the gain changes by 0.003 dB at 1 kHz and 0.008 dB at 10 and 20 kHz at Level 1. Across five drive cases the output rms differs by at most 0.0001 V, and one peak by 0.008 V.

## Control

**Level**, 0 to 1. The pot is 100k, marked B100k (linear), wired as a series rheostat in the amplifier's input leg: series resistance = 100k x (1 - Level), with a 1 ohm end stop. Gain rises steeply near the top. That is the Circuit's law, not an error.

| Level | Gain at 1 kHz (ngspice AC) |
|---|---|
| 0 | -0.24 dB (0.97x) |
| 0.5 | 5.64 dB (1.91x) |
| 0.9 | 18.58 dB (8.5x) |
| 1 | 35.57 dB (60x) |

Level at 0 does not mute the output.

## Assumptions and open values

Each of these is a choice, not a sourced fact.

- **Source resistance: 1k** (a typical Eurorack output). It is not a part of the Circuit, and it sets the maximum gain: 35.57 dB with 1k, 42.44 dB with 100 ohm, 43.68 dB with 0. At Level 0.5 the difference between 0 and 1k is 0.17 dB.
- **Load: 100k on each output** (a typical Eurorack input). The board has no resistor to ground after C5 or C6. With 10k the gain at 20 Hz falls 2.1 dB.
- **Rails: ideal +12 V and -12 V.** The series 1N5819 protection diodes (D1, D2) and the 22u rail capacitors (C1, C2) are left out. With rails at +/-11.7 V (a 0.3 V drop, assumed) the clipped output is 2.6 percent lower.
- **Pot taper: linear**, from the B100k marking only.
- **Pot rotation: clockwise = more gain.** The schematic does not state it. The pot symbol carries a small arrow toward the R1 end, which supports this if it is the usual clockwise marker. That convention was not confirmed.
- **Full scale: 5 V peak = 0 dBFS**, in and out. The clipped output reaches -6.8 V, so the output can exceed full scale by up to 2.7 dB.
- **Which parts are fitted** is unknown; the schematic names no manufacturer. The onsemi (Fairchild) MMBTH10 and Nexperia BC858B models are assumed. A second onsemi MMBTH10 model (MMBTH10LT1) gives 35.23 dB at Level 1 against 35.45 dB for the first, both used in full.
- **MMBTH10 RE: 0.** The published model gives none; 0 is the SPICE default. The Model, like ngspice, then has no emitter resistor and no internal emitter node.
- **Capacitors** (C12 2.2u, C5, C6, C7 1u, C13 10n) are ideal and non-polarised, as drawn. No dielectric or tolerance is given.
- **The schematic's gain note does not agree with its values.** The note says the high input gives 2x to 400x. The drawn values give 0.97x to 60x per output with a 1k source, or 153x with a 0 ohm source. Counting both outputs differentially brings it within a factor of about 1.3; that reading is unconfirmed. The Model follows the drawn values.

## Transistor models and what they leave out

The Solver's transistor is a reduced Gummel-Poon model: IS, BF, BR, VAF, IKF, RB, RC, RE. The Netlist uses the same reduced model, with the published value of each parameter. Both leave out junction charge, leakage, non-unity emission coefficients and reverse breakdown.

| | MMBTH10 (Q5, Q6, Q7) | BC858B (Q3) |
|---|---|---|
| IS | 69.28e-18 | 1.95e-14 |
| BF | 308.6 | 297.2 |
| BR | 1.11 | 10.02 |
| VAF | 100 | 41.13 |
| IKF | 22.83 mA | 83.1 mA |
| RB, RC, RE | 10, 4, 0 | 220, 0.2657, 1.045 |
| Left out | ISE = 69.28e-18, NE = 1.197, NC, temperature terms, CJC, CJE, TF, TR and the other charge terms | NF = 1.022, NR = 1.021, ISE, NE, ISC, NC, VAR, IKR, IRB, RBM, charge terms, and the vendor's quasi-saturation helper network (a second transistor behind 561.1 ohm, and a diode) with its 0.9607 area factor |

### The MMBTH10 model is not an exact match

In the published MMBTH10 model ISE equals IS, so the leakage term carries most of the base current at the currents this Circuit uses. The full vendor model gives hFE 77 in the input pair (52 to 59 uA) and 110 in the splitter (2.2 mA). The reduced model gives 343 to 345 and 314.

The published BF of 308.6 is kept. The rule in this repository is published values, no refit. A BF of 110 with everything else as published tracks the full vendor model more closely in every row below, but it is a fit, not a published value.

Measured gap in ngspice, full vendor models (both parts) against this Netlist:

| Measure | Full vendor models | This Netlist (BF 308.6) | BF 110 refit, not used |
|---|---|---|---|
| Amplifier output at DC | 64.6 mV | 14.7 mV | 42.8 mV |
| Gain at 1 kHz, Level 0 / 0.5 / 1 (dB) | -0.309 / 5.571 / 35.446 | -0.241 / 5.641 / 35.573 | -0.309 / 5.572 / 35.496 |
| Gain at 20 Hz, Level 0 (dB) | -2.404 | -2.264 | -2.417 |
| Gain at 20 kHz, Level 1 (dB) | 35.030 | 35.197 | 35.109 |
| 1 kHz, 5 V peak, Level 1: out max / min / mean (V) | 4.955 / -6.781 / 0.544 | 5.036 / -6.732 / 0.608 | 4.974 / -6.745 / 0.543 |
| 1 kHz, 5 V peak, Level 0: out rms (V) | 3.4068 | 3.4346 | 3.4068 |

So this Netlist is within 0.17 dB small-signal and 1.6 percent on clipped peaks of the full vendor models, and 80 mV on the clipped waveform's mean. The mean figures are snapshots at 100 ms, not settled values. The ISE term is most of the gap. Charge storage and every BC858B omission together account for at most 0.07 dB and 0.6 percent in the cases measured. Q3 saturates on every clipped half-cycle, which is where the BC858B helper network acts; its separate effect was not isolated.

### Breakdown is not modeled

Neither ngspice's Gummel-Poon model nor the Solver models reverse breakdown. In overdrive the Netlist drives two junctions of the MMBTH10 past their ratings:

| Junction | Reached in the Netlist | Rating |
|---|---|---|
| Q5 base-emitter, reverse | 4.1 V (Level 0.5, 1 kHz, 5 V peak) to 8.0 V (Level 1, 100 Hz, 5 V peak) | VEBO 3.0 V |
| Q6, Q7 base-emitter, reverse | up to 4.7 V (Level 1, 100 Hz, 5 V peak) | VEBO 3.0 V |
| Q5 collector-base, reverse | 30.2 V (Level 1, 1 kHz, 5 V peak) and 31.8 V (Level 1, 100 Hz, 5 V peak) | VCBO 30 V |

It happens because the amplifier output pushes Q5's base to about -20 V through C7, below the -12 V rail. A real junction would break down and conduct there. The Netlist and the Model do not, so they can agree with each other and both differ from hardware. The ratings are guaranteed minimums; the breakdown voltage of a given part is higher and unknown. What breakdown would change in hardware was not measured. This is the main limit on fidelity to hardware.

## Validation

Full Validation with `./validate.sh`, ngspice 45, the full Pack (45 Sections), each Section started from the operating point in both the reference and the Model. No Tolerance and no Check was changed. It was run four times and passed each time (the fourth is described under "Solver change in this Project"): once in full, once after the Solver's failure handling changed, and once after `render` began to fail on a held sample and the Solver began to check the complete capacitor history (see "Solver change in this Project"). The second and third runs reused the saved ngspice references, because the Netlist, the SPICE runner and the Pack were unchanged (the reference keys were the same before and after), and rendered the Model again at all four Settings. The tables below are from the third run; its four reports are identical to the second run's, which were identical to the first run's. Because `render` now stops on any held sample, the third run's completion shows there was none in the Pack at any Setting. The first two runs did not count them.

| Setting | Level | Result |
|---|---|---|
| `default` | 0.5 | PASS |
| `zero` | 0 | PASS |
| `high` | 0.9 | PASS |
| `full` | 1 | PASS |

Worst value of each Check across the four Settings, as the harness reports it:

| Check | Worst value | Limit | Where |
|---|---|---|---|
| frequency response | 0.0534 | 0.1 | `high`, sweep at 0 dBFS |
| static gain | 0.0036 | 0.1 | `full`, 5000 Hz at 0 dBFS |
| harmonics | 0.2354 | 1 | `high`, H4 of 5000 Hz at -16 dBFS |
| attack | 0 | 0.05 | all Settings |
| release | 0 | 0.05 | all Settings |
| intermodulation | 0.1242 | 1 | `default`, -20 dBFS |
| burst waveform null | -61.58 dB | -40 dB | `high` |

Burst null per Setting: -99.53 dB (`default`), -99.37 dB (`zero`), -61.58 dB (`high`), -63.37 dB (`full`). There is no Setting that mutes the output, because Level at 0 does not mute this Circuit.

What this shows: the Model agrees with the Netlist. It does not show agreement with hardware; see the breakdown and transistor-model sections above.

**Cost:** `cargo run -q --release --example bench -p dsp 48000` printed 172.7% of one core per stereo instance at 48 kHz (one run, with the Solver as it was at the second validation run, nothing else heavy running, load average 5.8 on the test machine). The run before that Solver change printed 173.8%; one run each does not show a difference. The cost was not measured again after the complete-history check was added. Measure on yours with the same command.

**Tests:** `cargo test` passes (23 tests in `dsp`, 1 in `render`); `cd harness && uv run pytest -q` passes (7 tests).

## Checked by hand

ngspice 45, single runs, before the validation.

- **Operating point.** The Model's DC solve and ngspice agree within 20 uV on every node; a unit test holds this. Amplifier output +14.74 mV, pair tail -0.7071 V, Q3 base 11.3214 V, splitter base -5.0181 V, collector 5.8424 V, emitter -5.8228 V, both outputs 0 V. It does not depend on Level. Q3 is the first PNP in a Model in this repository.
- **Gain at 1 kHz**, Model (through the `render` binary) against ngspice: -0.2406 against -0.2405 dB at Level 0, 5.6406 against 5.6407 dB at Level 0.5, 18.5788 against 18.5789 dB at Level 0.9, 35.5733 against 35.5733 dB at Level 1.
- **Clipped sine**, 1 kHz at 5 V peak, 0.2 s: the Model nulls against ngspice to -49.6 dB at Level 1, -53.0 dB at Level 0.9, -73.9 dB at Level 0.5 and -71.2 dB at Level 0.

## Solver change in this Project

`dsp/src/circuit.rs` differs from the template in the joint Newton solve. At Level 1 with a 5 V input, the template's Solver did not converge and its output ran to 1e21 V on the first cycle. The cause was one scale applied to the whole Newton step: Q5's base, driven through C7, moves 0.7 V in one circuit sample, the extrapolated first guess put that junction at 1.58 V forward, and the step back was scaled by the 60 V the collector had to move. The Solver now limits each junction's move the way SPICE does, and lets the other voltages take the whole step. The equations being solved are unchanged. A unit test drives the Model with overdriven sines at four frequencies and requires every joint solve to converge.

Three kinds of bad sample are contained instead of asserted on, so none of the three can panic on the audio thread (one other assertion remains; see below):

- **A solve that fails.** The joint solve has 200 iterations and stops when no port's Newton correction reaches 1 nV. If it runs out, or the complete new capacitor history, the output, or any port voltage or device current is not finite, that sample is discarded. The new history is computed in scratch storage and checked whole before any of it is kept. Capacitor history, port voltages and device currents stay as the last good sample left them, the output repeats that sample's value, and a counter goes up (`Processor::failed_solves`). The next sample starts from the last good state.
- **A non-finite input sample** (NaN or infinity) is treated as silence (0) before the oversampling filters, and again at the Solver's input.
- **A NaN Control value** is rejected: the Control keeps its value (its default when the Plugin is built). Infinite values clamp to the ends of travel.

The offline renderer (`render`, which `validate.sh` uses) reads that counter after processing. If it is not zero, it exits with an error and writes no output file, so a capture with a held sample cannot reach a comparison.

What the unit tests cover, and how:

- **Running out of iterations**, two ways. One test sets the bound to 1 in the middle of a clipped sine; that hook exists only in test builds. The `render` test uses the production bound of 200: a 1 kHz sine with a normalized peak of 1e6 (5e6 V) at Level 1, 2048 samples at 48 kHz, has failed solves, and the render must fail and create no file.
- **A history that overflows from finite operands.** An RC circuit driven with +1e308 V and -1e308 V in turn: the history must stay finite on every sample, and silence afterwards must solve normally and decay to rest. This runs the Solver's linear path. The overflow was not reproduced in the transistor Model.
- **Absurd and non-finite input to the Model**: 1e6 V, 1e300 V, the largest finite value, NaN and infinities. The test requires finite output, at least one failed solve, and a return to rest. It does not record which samples failed or why.
- **NaN and infinite samples at the Processor**, and **a NaN Control**.

What the containment does not do:

- It does not show that the solve converges for every input, Control history or sample rate. No failed solve occurred in the Pack at any of the four Settings: `render` fails on one, and the latest validation run completed. A unit test of overdriven sines (8 cases, up to 5 V peak) and a reviewer's 72 probe cases (four sample rates, Level 0.5, 0.9 and 1, 4096 samples each) had none either. Extreme finite inputs do reach containment: the sine with a normalized peak of 1e6 described above had 44 held circuit-rate samples over the two channels.
- It does not make the Plugin fail visibly. Only `render` stops on a held sample; the Plugin holds the sample and carries on.
- It does not clamp the output. A held sample repeats the last good output, and at an absurd input a converged sample can itself be far outside the rails.
- After an input of the 1e6 V class the Model takes about 4 s of silence to return to rest. That was not compared with ngspice.
- A larger input can leave a state that no later sample solves from. A 1 kHz sine of 1e8 V or more (a normalized peak of 1e7 through the Plugin's input) left an accepted, finite capacitor history from which every later sample failed, at any iteration bound, with the output held at a DC value; why that state does not solve was not diagnosed. Holding cannot leave it, so after 1024 failed samples in a row the circuit is put back at its DC operating point, as at construction, and counts that (`rest_restores`). The output jumps to rest when this happens. A test drives the 1e9 V case and requires one return to rest, silence at rest afterwards, and a 5 V sine that solves with no further held sample. The Three-Transistor Drive was not probed for the same state.
- One assertion can still be reached on the audio thread in a release build: the matrix inversion that follows a Control change asserts that the matrix is not singular. No Control value or input tried has triggered it.
- The Plugin does not read or show the failure counter.

These changes came after the first Full Validation. The Full Validation was then run again with the changed Solver and passed at all four Settings, with reports identical to the first run's. The rendered samples are not bit-identical to the first run's at `full`: a render made with the changed Solver before the rerun was compared with the first run's captures and matched at `default`, `zero` and `high`; at `full`, 735 of 7,353,600 samples differed, by at most 7.3e-12 of full scale, all in the silence after the -40 dB sweep. The cause is the convergence test, which now reads the Newton correction instead of the difference of the voltages before and after the step. The rerun's captures have the same samples as that render at all four Settings.

The third run followed two further changes: `render` fails on a held sample, and the Solver computes the complete candidate history before checking it. An accepted sample's history is the same sum with its two terms in the other order. The third run's captures were compared with the second run's, sample for sample: 0 of 7,353,600 samples differ at each of the four Settings.

A fourth run followed the return to rest after 1024 failed samples in a row (see the limits below). It also reused the references and passed at all four Settings; its captures and reports are identical to the third run's (0 of 7,353,600 samples differ at each Setting), and the bundles were installed again after it.

## Shared Solver (carried in 2026-11-19; Full Validation pending)

`dsp/src/circuit.rs`, `netlist.rs`, `processor.rs` and `render/src/main.rs` are now the same files in all eight Projects; `tools/check-shared.py` at the repository root fails when a copy drifts. They started from this Project's copies. This Project gains: the DC operating point check and rest-state pin from green-overdrive, the single-diode law from precision-rectifier (no single diode here), stricter PNP tests, and a convergence status on the per-device solve (not the path this Circuit takes). The test of `render`'s failed-solve gate now runs the built binary from `render/tests/`. Compared by rendering six short inputs (silence, 1 kHz at -12 and 0 dBFS, 5 kHz at 0 dBFS, 200 Hz at +12 dBFS, a 0 dBFS sweep) at every Setting through the `render` of the commit before the carry and through the new one: with the pin removed all 24 renders are bit-identical; with it the largest difference is -138.5 dBFS (one unit in the last place of a 32-bit sample), and silence goes from -152.9 dBFS to under -285 dBFS. No sample was held. The operating point does not depend on Level. The Three-Transistor Drive now has the return to rest too. The validation figures in this README were measured with the earlier Solver. **No Full Validation has been run with the shared Solver.**

## Build and use

- `cargo test` and `cd harness && uv run pytest -q` run the tests.
- `./validate.sh` runs the Full Validation at every Setting in `settings.txt`. Delete `validation/` first after any Quick Validation.
- `./install.sh` builds and installs the bundles. Install only after an all-PASS Full Validation.

Both scripts build into `./target` whatever `CARGO_TARGET_DIR` is set to. For any other cargo command, set `CARGO_TARGET_DIR=$PWD/target`.

## Rights

Not legal advice. Site terms of use were not read, and patents were not checked.

- The schematic PDF carries no licence statement. Its values and topology are facts and are modeled here. The PDF and renders of it are not in this repository and must not be added.
- The product page carries no licence statement for this module. Its design files are offered by email request, and no public repository was found. It is treated as a public reference without an open licence.
- The onsemi MMBTH10 and Nexperia BC858B model files carry no licence text. Eight parameter values from each are used as facts and cited. The files are not copied. Nothing from the onsemi MMBTH10LT1 file, which carries a commercial-use restriction, is in the Netlist or the Model.
- The Plugin's name contains no brand or model name.
- The template is MIT-licensed. nih-plug's VST3 bindings are GPLv3, so a distributed VST3 binary is GPLv3.
