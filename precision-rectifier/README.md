# Precision Rectifier

A full-wave precision rectifier plugin: `out = |in|` at unity gain, with no Controls. It is
modeled after the full-wave output of the rectifier section of the Mutable Instruments Kinks v4.1
circuit, designed by Emilie Gillet. The hardware design is published under
[CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/). Trademarks belong to their
owners. This plugin is not made or endorsed by them. No schematic, bill of materials or graphic is
redistributed here.

Nothing was measured on hardware. Validation compares two simplified descriptions of one Circuit:
the Netlist in `reference/circuit.cir`, simulated by ngspice, and the Model in `dsp/src/model.rs`.

**Status: all-PASS in a Full Validation** (2026-11-17, one Setting, first run, ngspice 45). That
run compared the captures through a shifted window. The same captures, measured again where the
Pack puts each Section, also pass every Check; no Full Validation has been run since. See
"Validation". Cost is about one core per stereo instance at 48 kHz, so treat the Plugin as
Offline-only: bounce or freeze it. See "Cost".

## What it does to a signal

- **It doubles the pitch and adds even harmonics.** A rectified sine has no fundamental. For a
  5 V, 1 kHz sine the Netlist gives the second harmonic at -7.45 dB and the fourth at -21.40 dB,
  relative to the input amplitude.
- **The output carries DC, by design.** The Circuit is DC-coupled from input to output and has no
  output capacitor. A full-scale sine comes out with 3.18 V of DC, which is 0.64 of full scale.
  The plugin keeps this, because a DC blocker would be a part that is not in the Circuit. Put a
  DC or high-pass filter after the plugin if the next device needs one.
- **There is a short negative spike at every rising zero crossing of the input.** The first
  op-amp has to cross the dead zone between its two diodes. The spike lasts 5.95 us at 1 kHz and
  5 V, and 42 us at 50 mV. It leaves a small residue of the fundamental: -48.2 dB at 1 kHz and
  5 V, -29.5 dB at 1 kHz and 50 mV, -16.2 dB at 5 kHz and 50 mV.
- **Below about 10 mV (-54 dBFS) the rectifier goes soft.** At +2 mV in, the output is 1.06 mV;
  at -2 mV it is 1.80 mV.
- **Do not drive it above 0 dBFS.** Op-amp rail clipping is not modeled. See "Omitted".

## Sources

| Id | Source | Used for |
|---|---|---|
| S1 | Schematic `kinks_v41`, sheet 1 of 4 "HW/FW rectifiers", dated 02/04/2019: <https://pichenettes.github.io/mutable-instruments-documentation/modules/kinks/downloads/kinks_v41.pdf> | Topology and every value |
| S2 | Bill of materials "Kinks, v4.1": <https://github.com/pichenettes/eurorack/blob/master/kinks/hardware_design/Kinks.xlsx> | Second source for every value; tolerances; part numbers |
| S3 | Module documentation: <https://pichenettes.github.io/mutable-instruments-documentation/modules/kinks/> | Input impedance, supply, described behaviour |
| S4 | Texas Instruments TL07xx datasheet SLOS080: <https://www.ti.com/lit/ds/symlink/tl074.pdf> (revision W, 2025); revision N of July 2017 read from web.archive.org | Op-amp gain, bandwidth, slew rate, output swing |
| S5 | Fairchild 1N4148WT datasheet, rev. A2, read from an archive copy | Diode capacitance |
| S6 | analog-to-vst template, `D1N4148` in `dsp/src/netlist.rs` | Diode model parameters |
| S7 | README of the designer's repository: <https://raw.githubusercontent.com/pichenettes/eurorack/master/README.md> | Licence and naming guidelines |

The schematic PDF has no text layer. Values were read from rasters of sheet 1, and every one is
confirmed by the bill of materials. An independent check parsed the Eagle schematic file in the
same repository and got the same net membership, including both diode orientations. S3 states
"Input impedance: 66k for RECTIFY", which is the three 200k input resistors in parallel.

## Parts

| Ref | Value | Between |
|---|---|---|
| R2 | 200k, 0.1 % | input, IC1A inverting input |
| R3 | 100k, 0.1 % | IC1A inverting input, node `x` |
| D3 | 1N4148 | anode IC1A output, cathode IC1A inverting input |
| D4 | 1N4148 | anode node `x`, cathode IC1A output |
| IC1A | TL074 section | non-inverting input to ground |
| R4 | 100k, 0.1 % | node `x`, ground (see "Reduced configuration") |
| R5, R8 | 100k, 0.1 % each | node `x`, IC1C inverting input (in parallel) |
| R9 | 200k, 0.1 % | input, IC1C inverting input |
| R7 | 200k, 0.1 % | IC1C inverting input, IC1C output |
| IC1C | TL074 section | non-inverting input to ground |
| R13 | 180, 1 % | IC1C output, output jack (outside the feedback loop) |

IC1A is an inverting half-wave stage: `x = -in/2` for a positive input and 0 V for a negative
one. IC1C sums: `-(in + 4x)`, which is `|in|`.

## Reduced configuration

The plugin is the full-wave output only. It is not the whole module.

| Point | Choice | Basis |
|---|---|---|
| Source impedance | 0 ohm | Assumption |
| Inverter (R6, R11, IC1B, R12) and indicator (IC2, R10, LED1) | Omitted | They connect to the input and to IC1B's output only. With a 0 ohm source they have no path to the output. With a real source impedance R6 would load the input |
| Half-wave output stage (IC1D, R1, R14) | Omitted, but its input resistor R4 is kept, returned to ground in place of IC1D's virtual ground | Assumption. Measured in ngspice against the full stage: the output differs by -59.8 dB or less. Removing R4 altogether differs by up to -28.5 dB |
| Load | 100k to ground after R13 | Assumption; not in any source. It sets the gain to 0.9982 |
| Full scale | 0 dBFS = 5 V peak | Assumption |
| Resistors | Nominal values | The 100k and 200k parts are 0.1 % |
| Logic, noise and sample-and-hold sections (sheets 2 and 3), power entry (sheet 4) | Omitted | Only the supply rails and ground are shared with sheet 1 |

## Device models and open values

**Op-amp (TL074CPW).** Single pole, open-loop gain 2e5, gain-bandwidth 3 MHz.

- The gain is the datasheet's typical 200 V/mV, not a measured part.
- **The gain-bandwidth is an open value.** The current datasheet gives 3 MHz for some packages
  and 5.25 MHz for the rest, which includes the PW package in the bill of materials. The July 2017
  revision, from the module's production years (2016 to 2021), gives 3 MHz typical for every
  package; 5.25 MHz belongs to the newer die. 3 MHz is used. The choice sets the size of the
  zero-crossing spike. Measured in ngspice, 5.25 MHz changes the residual fundamental by about
  1 dB and differs from 3 MHz by -24.6 dB as a waveform null at 10 kHz and 50 mV.
- The Netlist uses the rescaled subcircuit (1e-3 A/V into 2e8 ohm and 53.05 pF). The template's
  form (100 A/V into 1k) made ngspice stop early in the seven-band-eq Project.

**Diode (1N4148WT).** `Is=2.52n Rs=0.568 N=1.752 Cjo=4p M=0 tt=0`.

- `IS`, `RS`, `N` and `CJO` are the template's set. Who published it was not confirmed. `CJO` of
  4 pF equals the datasheet maximum at 0 V, not a typical value. A vendor model could not be
  fetched.
- **`M=0` and `tt=0` are a reduction.** The template's line has `M=0.4 tt=20n`. The Solver's diode
  has a fixed capacitance only, so with `M=0.4` the Netlist and the Model would describe two
  different diodes. With `M=0` ngspice's capacitance is a constant 4.000 pF from -3 V to +0.4 V
  (measured), where the template's line gives 2.3 pF to 12.5 pF.
- Cost of the reduction, measured in ngspice alone (reduced diode against the template's): about
  2 dB in the residual fundamental at 1 kHz and 5 V; waveform null -58.9 dB at 1 kHz and 5 V,
  -33.0 dB at 20 kHz and 5 V, -33.3 dB at 10 kHz and 50 mV. The transit time alone changes
  nothing that could be seen.
- No reverse breakdown. The largest reverse voltage is about 3 V.

## Omitted in both the Netlist and the Model

- **Op-amp rail clipping and slew limiting.** At 0 dBFS the op-amp outputs stay between -2.98 V
  and +5.00 V on 12 V rails, for sines up to 43.2 kHz (measured in ngspice). The steepest output
  slope is 1.97 V/us at 43.2 kHz, against a datasheet slew rate of 8 V/us minimum. Not tested:
  inputs above 0 dBFS, and arbitrary transients.
- Op-amp input offset, bias current, noise, output impedance and any second pole.
- The supply: +12 V and -12 V through Schottky diodes and ferrite beads, with decoupling. The
  op-amp model has no supply pins.
- The diode's voltage-dependent capacitance, transit time and reverse breakdown (above).
- Jack switching. Temperature other than 27 C.

## Changes to the template in this Project

Each was made for a measured cause. No Check and no Tolerance was changed.

| File | Change | Cause, measured |
|---|---|---|
| `dsp/src/oversample.rs` | Circuit rate 576 kHz to 2304 kHz | A rectifier's harmonics fall only as 1/k^2, and those above half the circuit rate fold back. On a 1 s, 0 dBFS sweep, measured as the harness measures frequency response, against the Netlist at 16: the Model at 576 kHz differs by 0.40 dB, at 1152 kHz by 0.079 dB, at 2304 kHz by 0.013 dB. See "Is 2304 kHz needed?" below |
| `harness/src/analog_harness/spice.py` | Reference oversampling 4 to 16 | The same folding, in the reference: ngspice's output is read at 384 kHz with no filter in front. On a 1 s, 0 dBFS sweep the reference at 4 differs from the reference at 24 by 0.83 dB, at 8 by 0.13 dB, at 16 by 0.012 dB |
| `dsp/src/circuit.rs`, `Device::Diode` | ngspice's reverse-bias law and GMIN | ngspice's diode current below -3 N Vt is `-Is (1 + (3 N Vt / (e v))^3)`, not the exponential law, and it adds 1e-12 S across the junction. With 2.52 nA into 100k and 200k the difference is microvolts. At 100 Hz, -40 dBFS it was 0.0275 dB of the harness's gain measure; with the change it is under 0.0001 dB and the waveform null goes from -80 dB to -131 dB |
| `harness/…/analysis.py`, `compare.py`, `cli.py`, `validate.sh` | `compare` and `analyze` take a required `--alignment`: `known` (delay 0, polarity +1) or `chirp` (recovered from the sync chirp). `validate.sh` uses `known` | Chirp recovery put both captures 1446 samples off, which moved every Section's window. See "How the harness sees this Circuit" |
| `reference-key.sh`, `validate.sh` | The reference cache key covers the Netlist, `spice.py`, both Pack files and the ngspice version | With the template's key, a reference made at 4 would have been reused after the change to 16. Carried over from green-overdrive |
| `harness/…/cli.py`, `validate.sh`, `install.sh`, `spice.py` | Section reset, forced `CARGO_TARGET_DIR`, `SETTINGS=<file>`, refusal of a transient operating point | Carried over from the other Projects |

`diagnostics/` holds the scripts that produced these numbers.

### Is 2304 kHz needed?

It is enough. It is not shown to be the least that is enough.

The rate was chosen from figures taken through the shifted window (next section). Those were, on
a 4 s, 0 dBFS sweep against the Model at 4608 kHz: 576 kHz 0.67 dB, 1152 kHz 0.12 dB, 1536 kHz
0.10 dB, 2304 kHz 0.02 dB. The 4 s captures were not kept, so these four were not measured again.

The 1 s captures were kept and were measured again at the correct alignment. By their file
times the Model captures were made before the diode change in the table above.

| 1 s, 0 dBFS sweep, frequency-response measure | Shifted window | Correct alignment |
|---|---|---|
| Netlist at 4 against Netlist at 24 | 0.55 dB | 0.83 dB |
| Netlist at 8 against Netlist at 24 | 0.13 dB | 0.13 dB |
| Netlist at 16 against Netlist at 24 | 0.012 dB | 0.012 dB |
| Model at 576 kHz against Netlist at 16 | 0.54 dB | 0.40 dB |
| Model at 1152 kHz against Netlist at 16 | 0.107 dB | 0.079 dB |
| Model at 2304 kHz against Netlist at 16 | 0.013 dB | 0.013 dB |

- The reference needs 16: at 8 it is 0.13 dB from the reference at 24, more than the Tolerance.
- 576 kHz fails at either alignment.
- **1152 kHz is inside the Tolerance at the correct alignment on this sweep** (0.079 dB of 0.1),
  and was outside it through the shifted window (0.107 dB). So the old reason for ruling it out
  does not hold, and a lower rate would cost less. Not tried: a Full Validation at 1152 kHz, or any rate
  between 1152 and 2304 kHz at the correct alignment.

## How the harness sees this Circuit

The harness was built for Circuits that pass their fundamental. This one removes it. That
changes what three Checks mean, and it defeats the template's way of lining up a capture.

- **The sync chirp cannot align this Circuit, so the harness does not use it here.** The
  template finds a capture's delay and polarity by correlating it with the sync chirp. A
  rectified chirp hardly correlates with the chirp, but its second harmonic does, 15.06 ms early
  and with the sign flipped. The Netlist and the Model both come out at -1446 samples and
  polarity -1. The true values are 0 and +1.
- **An equal wrong shift on both sides is not harmless.** The harness moves the capture and
  keeps each Section's window where the Pack puts it, so every window reads 15.06 ms early. The
  burst null then leaves out the last 15.06 ms of the burst: 0.1 of full scale added to the last
  5 ms of the Model's burst passed all seven Checks with unchanged values, and fails the burst
  null at -34.0 dB in the correct window. The frequency-response value also depends on the
  shift, because the response window is fixed. The sign flip alone changes nothing.
- **What the harness does now.** Both captures that `validate.sh` makes have their latency
  removed, for any Circuit, so their alignment is known: delay 0, polarity +1. `validate.sh`
  passes `--alignment known` and the report states the alignment it used. `--alignment chirp`
  remains for a capture of unknown delay, such as a recording, and is only right for a Circuit
  that passes the chirp. The option is required, so the choice is always stated. A test adds the
  5 ms defect to a rectified Pack and requires the burst null to fail.
- **"Static gain" and "frequency response" measure the residue of the fundamental**, which the
  zero-crossing spike leaves, not a gain. They test how well the Model reproduces the spike.
- **"Harmonics" are in dB relative to that residue**, so the second harmonic reads about
  +41 dBc at 1 kHz and full level, above the top of the report's plot.
- **Attack and release** find no gain change on either side and pass trivially.
- **The burst waveform null** tests the rectification itself.

## Settings

One: `default`. There is no Control to put at 0 or at 1, and none that mutes the output.

## Validation

Full Validation on 2026-11-17, ngspice 45, the full Pack (45 Sections, 76.6 s at 96 kHz), run once
from an empty `validation/` directory. It passed on the first run; nothing was changed between
authoring and validation. One Setting, so every value below is also the worst value.

That run aligned both captures by the sync chirp: delay -1446 samples and polarity -1 on both
sides, which is wrong (see "How the harness sees this Circuit"). The second column below is what
the run reported. The third is the same two captures measured again with `harness compare
--alignment known` (delay 0, polarity +1 on both sides), which is what `validate.sh` does now.
Nothing was simulated or rendered again, and `validate.sh` has not been run since the change.

| Check | Result | As run (shifted window) | Same captures, known alignment | Limit | Worst case at known alignment |
|---|---|---|---|---|---|
| frequency response | pass | 0.0307 dB (sweep at 0 dBFS) | 0.0054 dB | 0.1 | sweep at -20 dBFS |
| static gain | pass | 0.00049 dB | 0.00049 dB | 0.1 | 5000 Hz at 0 dBFS |
| harmonics | pass | 0.0064 dB | 0.0064 dB | 1 | H5 of 5000 Hz at 0 dBFS |
| attack | pass | 0 | 0 | 0.05 | no gain change in either |
| release | pass | 0 | 0 | 0.05 | no gain change in either |
| intermodulation | pass | 0.0001 dB | 0.0001 dB | 1 | -6 dBFS |
| burst waveform null | pass | -112.8 dB | -112.8 dB | -40 | dB relative to reference |

| Setting | Result |
|---|---|
| default | PASS |

Read the table with "How the harness sees this Circuit" in mind:

- Attack and release pass because neither capture shows a gain change. They test nothing here.
- Frequency response and static gain compare the residue of the fundamental, not a gain.
- The burst null as run covered the burst less its last 15.06 ms. At known alignment it covers
  the whole burst (-112.85 dB against -112.82 dB).
- The frequency-response value of 0.0307 dB came from the shifted window. At known alignment the
  worst sweep reads 0.0054 dB and the 0 dBFS sweep 0.0043 dB.
- Outside the Checks: the two whole captures (all 45 Sections, 76.6 s) differ by -82.2 dB as a
  plain waveform null with no alignment. The pink-noise Section is rendered but no Check reads it.

On 2026-11-17, after the alignment change: `cargo test` 10 passed, `(cd harness && uv run pytest
-q)` 10 passed.

## What was checked by hand

Before the Full Validation, on 2026-11-17, with ngspice 45, through the harness's own section
runner. One Setting exists, so the two operating conditions below stand in for "two Settings".

| Quantity | Netlist (ngspice) | Model | Difference |
|---|---|---|---|
| Operating point, input 0 V: `out` | 0 V (-1.2e-19 V) | 0 V | none |
| DC transfer, +1 V in | 0.9971398 V | 0.9971398 V | 0.02 uV |
| DC transfer, -1 V in | 0.9979342 V | 0.9979342 V | 0.01 uV |
| DC transfer, +5 V in | 4.9897699 V | 4.9897698 V | 0.12 uV |
| DC transfer, -5 V in | 4.9906673 V | 4.9906674 V | 0.12 uV |
| DC transfer, +2 mV / -2 mV in | 1.0618 mV / 1.7980 mV | the same | under 0.01 uV |
| 1 kHz, 0 dBFS: fundamental re input | -48.2405 dB | -48.2405 dB | under 0.0001 dB |
| 1 kHz, -40 dBFS: fundamental re input | -29.4570 dB | -29.4571 dB | 0.0001 dB |
| 100 Hz, -40 dBFS: fundamental re input | -39.7800 dB | -39.7800 dB | under 0.0001 dB |
| 1 kHz, 0 dBFS: second harmonic re input | -7.453 dB | -7.453 dB | under 0.001 dB |
| 1 kHz, 0 dBFS: waveform null | | | -107.1 dB |
| 1 kHz, -40 dBFS: waveform null | | | -101.7 dB |
| 5 kHz, -40 dBFS: waveform null | | | -82.2 dB |

A hand-made short signal set (`diagnostics/short_pack.py`: sync, two 1 s sweeps, five tones, a
1 s burst, one intermodulation tone) through the harness's compare gave: frequency response
0.013 dB, static gain 0.0005 dB, harmonics 0.006 dB, intermodulation 0.0001 dB, burst null
-112.2 dB. The saved captures give the same figures at the chirp's alignment (-1446 samples, as
first measured) and at known alignment. It is not the Pack.

## Cost

About one core per stereo instance at 48 kHz. Eight runs of the benchmark on 2026-11-17, one at a
time, gave 90.4 % to 110.9 % of one core (90.4, 90.6, 90.7, 91.1, 95.9, 101.2, 101.2, 110.9). The
machine was not idle: another program was using about 12 of its 16 cores and the load average was
15 to 17, so the idle figure is probably near the low end and was not measured. Either way the
Plugin is at the edge of one core. Treat it as Offline-only (bounce or freeze); it may play live
on an idle machine with a large buffer, and that was not tried.

The cost comes from the circuit rate of 2304 kHz, four times the template's. 576 kHz fails the
frequency-response Check; whether a rate between would pass is open (see "Is 2304 kHz needed?").

```sh
env CARGO_TARGET_DIR=$PWD/target cargo run -q --release --example bench -p dsp 48000
```

Untested: how the folded harmonics sound. The decimator is 120 dB down from fs - 20 kHz, so at a
48 kHz session content between 24 kHz and 28 kHz folds into 20 kHz to 24 kHz.

## Rights

Not legal advice.

- The schematic and bill of materials are by Emilie Gillet, licensed
  [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) ("Hardware: cc-by-sa-3.0" in
  the repository README; "cc-by-sa" in the title block of every sheet). The licence allows copying
  and adapting with attribution, with adaptations under the same licence.
- The Netlist here was written from values and connections. No file was copied.
- **Open question:** whether a Netlist transcribed from a CC BY-SA drawing is an adaptation that
  must itself be CC BY-SA, when this repository is MIT. Values and topology are facts, but this
  needs someone qualified if it matters.
- The designer's repository says the brand is a registered trademark, that it should not be used
  on derivative works, and that keeping the module's name is not recommended. The plugin's name,
  IDs and directory are neutral.
- The documentation pages carry no licence statement that was found, and the datasheets are their
  publishers' copyright. Only facts were taken from them.
- No patent search was done. The circuit is a textbook two-op-amp precision rectifier.
- The template is MIT. VST3 binaries built with nih-plug are GPLv3.

## Build

```sh
cargo test
(cd harness && uv run pytest -q)
./validate.sh    # one validation at a time; delete validation/ first (the Pack is not rebuilt)
./install.sh     # only after every Setting passes a Full Validation
```

Both scripts build into `./target` whatever `CARGO_TARGET_DIR` is set to.
