# Feedback Pair Drive

A stereo plugin modeled after the Music Thing Modular London Drive circuit (Proto 1, Rev 1 schematic, one sheet dated 14/11/2022); trademarks belong to their owners. This is an independent project and is not endorsed by any trademark owner. Each stereo side simulates its own copy of the Circuit.

The Circuit has four transistors. A long-tailed NPN pair (Q6, Q7) and a PNP second stage (Q3) form an inverting feedback amplifier with a 100k feedback resistor. An NPN phase splitter (Q5) with equal 2.7k collector and emitter resistors follows it. There is one Control, Level.

**Status: not validated.** The Netlist and the Model are written and agree by hand at the points listed under "Checked by hand". `./validate.sh` has not been run, so no Check has been evaluated, and the Plugin has not been built or installed. Cost is not measured.

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
- **MMBTH10 RE: 0.** The published model gives none; 0 is the SPICE default. The Model uses 1 milliohm where the Netlist has 0.
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

## Checked by hand

ngspice 45, single runs, before any validation.

- **Operating point.** The Model's DC solve and ngspice agree within 20 uV on every node; a unit test holds this. Amplifier output +14.74 mV, pair tail -0.7071 V, Q3 base 11.3214 V, splitter base -5.0181 V, collector 5.8424 V, emitter -5.8228 V, both outputs 0 V. It does not depend on Level. Q3 is the first PNP in a Model in this repository.
- **Gain at 1 kHz**, Model (through the `render` binary) against ngspice: -0.2406 against -0.2405 dB at Level 0, 5.6406 against 5.6407 dB at Level 0.5, 18.5788 against 18.5789 dB at Level 0.9, 35.5733 against 35.5733 dB at Level 1.
- **Clipped sine**, 1 kHz at 5 V peak, 0.2 s: the Model nulls against ngspice to -49.6 dB at Level 1, -53.0 dB at Level 0.9, -73.9 dB at Level 0.5 and -71.2 dB at Level 0.

## Solver change in this Project

`dsp/src/circuit.rs` differs from the template in the joint Newton solve. At Level 1 with a 5 V input, the template's Solver did not converge and its output ran to 1e21 V on the first cycle. The cause was one scale applied to the whole Newton step: Q5's base, driven through C7, moves 0.7 V in one circuit sample, the extrapolated first guess put that junction at 1.58 V forward, and the step back was scaled by the 60 V the collector had to move. The Solver now limits each junction's move the way SPICE does, and lets the other voltages take the whole step. The converged answer is unchanged. In a debug build the Solver asserts that every joint solve converged, and a unit test drives the Model with overdriven sines at four frequencies.

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
