# Three-Transistor Drive

A stereo plugin modeled after the audio stage of the Music Thing Modular Mini Drive (Rev 1) circuit, designed by Tom Whitwell. Trademarks belong to their owners. This project is not endorsed by the maker. No schematic, drawing or graphic is redistributed here.

The Circuit is a discrete amplifier: an NPN differential pair drives a PNP common-emitter stage, with feedback that sets a gain of 201 (R7 200k, R8 1k). A passive input network and a 1M pot sit in front of it. Each stereo side runs its own copy.

**Nothing was measured on hardware.** Every number below comes from simulation. Validation compares two simplified descriptions of one Circuit (the Netlist in `reference/circuit.cir` and the Model in `dsp/src/model.rs`); it says nothing about how close either is to a real module.

## Status

- Authored, not validated. `./validate.sh` has not been run, not even with `QUICK=1`. The plugin is not installed.
- `cargo test` passes (14 tests) and `uv run pytest -q` in `harness/` passes (7 tests).
- CPU cost is not benchmarked. A rough timing on a busy machine (5 s of stereo at 48 kHz rendered in 43 s) puts it near 860% of one core, so treat the Plugin as **Offline-only**: bounce or freeze, do not play live. Measure with `env CARGO_TARGET_DIR=$PWD/target cargo run -q --release --example bench -p dsp 48000` on an idle machine.

## Control and levels

| Control | Part | Range | Default |
|---|---|---|---|
| Drive | VR1, 1M, audio taper | wiper from the ground end (0) to the input end (1) | 0.5 |

- The template's audio taper is two straight segments with 10% of the track at mid rotation. The Netlist placeholder `$DRIVE` is the wiper position as a fraction of the track.
- At Drive 0 the wiper is 1 ohm from ground and the output is near silence (-75.6 dB at 1 kHz).
- Small-signal gain from `in` to `out` at 1 kHz: 41.89 dB at Drive 1, 18.83 dB at Drive 0.5.
- Full scale (0 dBFS) is 5 V peak at both jacks, a modular-synthesizer audio level. **The output exceeds full scale at high drive.** The amplifier clips near +11.7 V and -10.8 V at the output jack, which is about +7 dBFS. Follow the plugin with a gain stage or a float-safe signal path. The two clip levels differ, so the clipped wave is asymmetric.
- There is no output level control in the Circuit.

## Sources

| # | Source | What it gave |
|---|---|---|
| S1 | Schematic, one page, "Mini Drive Rev 1 ... March 2020", sheet 1/2: https://www.musicthing.co.uk/collateral/mini_drive_Rev1_Schematic.pdf | Every part value and the topology. The drawing has no extractable text; values were read from it by eye, twice, by two independent passes that agree |
| S2 | Module page: https://www.musicthing.co.uk/Mini-Drive/ | Three transistors for the audio and three for the lamp; the two outputs are identical |
| S3 | Kit seller's page: https://www.thonk.co.uk/shop/mini-drive/ | The knob is called "Drive" and goes from silent to a lot of gain; the outputs are passively paralleled |
| S4 | About page: https://www.musicthing.co.uk/about/ | Licence statement (see Rights) |
| S5 | onsemi (former Fairchild) 2N5088 datasheet, "Spice Model": https://www.onsemi.com/pdf/datasheet/2n5088-d.pdf | NPN model parameters, used as a stand-in |
| S6 | Fairchild 2N3906 / MMBT3906 / PZT3906 datasheet (1997), "Spice Model": https://web.ece.ucsb.edu/~ilan/Classes/ECE2A_F2010/labs/Datasheets/2n3906.pdf | PNP model parameters |
| S7 | onsemi MBT6429DW1T1 datasheet: https://www.onsemi.com/pdf/datasheet/mbt6429dw1t1-d.pdf | Ratings of the fitted dual NPN: hFE 500 to 1250 at 0.1 mA, VBE(on) 0.56 to 0.66 V at 1 mA, VEBO 6.0 V. It has no SPICE model |
| S8 | onsemi MMBT3904L datasheet: https://www.onsemi.com/pdf/datasheet/mmbt3904lt1-d.pdf | V(BR)EBO 6.0 V minimum for the lamp driver's first transistor |

Values as read from S1: R15 68k, R16 68k, R17 1M, VR1 "A1M", R1 1k, C3 100n, R2 100k, R3 100k, R4 27k, C4 100p, R5 100R, R6 1k, R7 200k, R8 1k, C6 220u, C5 10p, R9 10k, C7 1u; Q5Q1 and Q5Q2 are the two halves of an MBT6429DW1T1G; Q2 is an MMBT3906. The Netlist header lists where each one connects.

Not obtained: sheet 2 of the drawing (the PDF has one page), a parts list, a vendor SPICE model for the MBT6429DW1T1G, and any hardware measurement.

## Configuration modeled

- A cable in the high input only. The low input is unpatched, so its switch contact joins its tip to the high input's tip and R15 sits in parallel with R16 (34k). R17 loads the source.
- One of the two paralleled output jacks is loaded. The other is unpatched and adds nothing.
- Cross-check of the jack reading: the low input alone would give 0.50 of the high input's gain, which matches a note on the drawing.

## Assumptions and open values

1. **VR1 taper.** The drawing says "A1M". Audio taper is assumed from the "A" prefix; no parts list confirms it.
2. **VR1 direction.** Clockwise is assumed to raise the wiper (more gain). The pot symbol's arrow weakly supports this.
3. **Input pair model.** No vendor model exists for the MBT6429DW1T1G. The Fairchild 2N5088 model (S5) stands in: IS 5.911 fA, BF 1122, BR 1.271, VAF 62.37 V, IKF 14.92 mA, RB 10, RC 1.61. The two halves are identical. Two known mismatches with the fitted part's datasheet (S7):
   - The reduced model runs at a current gain of about 1324 (Q5Q1) and 1331 (Q5Q2) at this bias, above the datasheet's 500 to 1250, because of the Early term. With the complete 2N5088 model the gain is about 424, below the range.
   - VBE at 1 mA is about 0.67 V, just above the datasheet's 0.56 to 0.66 V.
4. **Q2 model.** The drawing names no maker. The Fairchild 3906 model (S6) is used: IS 1.41 fA, BF 180.7, BR 4.977, VAF 18.7 V, IKF 80 mA, RB 10, RC 2.5.
5. **RE** is absent from both published models, so 0 (the SPICE default). The Model enters 0.001 ohm.
6. **Source impedance** 1k and **load** 100k on one output. Neither is on the drawing. The negative clip level depends on the load.
7. **Supply.** Ideal +12 V and -12 V. The hardware's series diodes (D1, D2, 1N5819) lower each rail by a diode drop; that is not modeled.
8. **Capacitors** are ideal at their marked values. C6 and C7's type, tolerance and ESR are not on the drawing.
9. **Temperature** 27 C, with no temperature dependence.
10. **Full scale** is 5 V peak, chosen here; the Circuit does not define it.

The Netlist has four elements that are not on the drawing: RS, RLOAD, VCC and VEE.

## What is left out, and what it costs

### Lamp and lamp driver (a limit on hardware fidelity)

The hardware has an incandescent lamp and a three-transistor driver (R10, Q1Q1, C8, R11, R12, Q4, R13, Q1Q2, R14). The driver taps the amplifier's output node through R10 (10k). The Solver has no lamp, and the whole branch is omitted.

- With the driver added to the Netlist as an experiment and the transistor model as it stands, its loading is immaterial: the output differs by -75.7 dB RMS at 5 V peak input, full drive. An independent re-run with a different 3904 model found the same negative peak (-10.41 V) with and without the driver.
- **Emitter-base breakdown is not modeled, and this may matter a great deal on hardware.** In that experiment the driver's first transistor has its emitter-base junction reverse-biased by about 21 V on every negative half-cycle. Its rating is 6 V minimum (S8). Neither ngspice's transistor model nor the Solver has reverse breakdown. If the real part breaks down a few volts above its rating, the driver discharges into the output node and roughly halves the negative swing: with an assumed 7 V breakdown the negative peak of the amplifier's output node falls from -10.41 V to -5.25 V (and to -6.33 V with 10 V).
- This is a hypothesis. No measurement of the hardware was found. If it holds, this Plugin clips more symmetrically than the module does, and the difference is large.

### Transistor charge storage and leakage

Both descriptions use a reduced Gummel-Poon transistor: IS, BF, BR, VAF, IKF, RB, RC, RE only. Dropped from the NPN: leakage (ISE 5.911 fA, NE 1.394) and charge storage (CJC, CJE, TF, TR). Dropped from the PNP: charge storage only; its published ISE and ISC are 0.

Measured cost, reduced Netlist against the same Netlist with the complete published parameter sets (from the sourcing pass; the independent check did not repeat these):

| Quantity | Reduced | Complete |
|---|---|---|
| Gain at 1 kHz, Drive 1 | 41.885 dB | 41.851 dB |
| Gain at 20 kHz, Drive 1 | 41.107 dB | 40.885 dB |
| Gain at 20 kHz, wiper at 0.1 | 18.051 dB | 17.792 dB |
| Upper -3 dB frequency, Drive 1 | 45.0 kHz | 39.9 kHz |
| DC at the amplifier's output node | 36.9 mV | 52.1 mV |
| Clip levels at that node, 5 V peak, 1 kHz | +11.945 / -10.412 V | +11.945 / -10.413 V |
| RMS difference of `out`, 5 V peak input | | -36.2 dB |
| RMS difference of `out`, 0.05 V peak input | | -45.8 dB |

So this Plugin is about 0.2 dB brighter at 20 kHz than the same Circuit with complete device models. A passing validation would not see that: it compares the Model with the reduced Netlist.

### Other omissions and limits

- **Input pair reverse bias.** At 5 V peak input and full drive, the pair's base-emitter junctions reach 2.99 V and 2.87 V of reverse bias, inside the 6.0 V rating (S7). At 10 V peak they reach 6.6 V and 6.4 V, past it. Breakdown is not modeled, so inputs above full scale are outside what this Model describes.
- **Supply parts.** D1, D2 and the 22u decoupling capacitors are omitted.
- **Slow bias movement.** C6 (220u) charges through R7 (200k), about 44 s. Under sustained clipping the bias drifts over tens of seconds. Both descriptions include this; long runs were not compared.
- **The jacks, noise, capacitor leakage and ESR** are not modeled.
- **Sheet 2** of the drawing is unknown.

## Solver notes for this Circuit

- **First PNP in the repository.** The Solver's PNP path had no test. Three were added in `dsp/src/circuit.rs` (mirror of the NPN, Jacobian against finite differences, a PNP stage's operating point), and `dsp/src/model.rs` checks this Circuit's operating point against ngspice: every node within 8 microvolts.
- **Junction limiting.** The template's joint Newton solve stopped at 64 iterations without converging when the amplifier clipped hard, and the output then ran away (10^23 V, then NaN in the file). The solve now limits each junction's step as SPICE does and limits the extrapolated starting point the same way. A test drives a 5 V peak, 1 kHz sine at full drive, requires convergence on every sample, and asserts the clip levels against ngspice: +11.684 V, -10.755 V, 11.129 V RMS. In the renders counted so far (at 576 kHz) the solve took 2.3 iterations on average and 17 at most.
- **Circuit rate 4.608 MHz** (the template runs at 576 kHz). At full drive the output transistor takes about half a microsecond to come out of cutoff, and that delay sets the even harmonics. At 576 kHz the second harmonic of a 5 kHz tone was up to 10.3 dB away from SPICE; at 4.608 MHz the worst harmonic error found by hand is 0.29 dB against a Tolerance of 1 dB. This is why the Plugin costs so much.
- **Remaining edge-timing difference.** At full drive and full-scale input the Model's clipped edges still differ from SPICE by a fraction of a microsecond at high frequencies. Whole-tone waveform difference, by hand: -56 dB at 1 kHz, -42 dB at 5 kHz, -31 dB at 10 kHz, -24 dB at 20 kHz. Gain and harmonics agree at all of them.

## Validation

`settings.txt` has four Settings: `default`, `drive_min` (Drive 0, a muted output), `drive_low` (Drive 0.1) and `drive_max` (Drive 1).

```
rm -rf validation && ./validate.sh
```

Each Section is rendered from a fresh DC state, as SPICE does. Delete `validation/` when switching between `QUICK=1` and a full run: the Pack is cached without regard to the mode.

Checked by hand so far, on short signals, Model against ngspice 45:

| Quantity | Setting | Model | ngspice |
|---|---|---|---|
| DC operating point, 7 nodes | any | within 8 uV | |
| Gain at 1 kHz | default | 18.8277 dB | 18.8277 dB |
| Gain at 1 kHz | drive_max | 41.8854 dB | 41.8854 dB |
| Gain at 1 kHz | drive_low | 8.8795 dB | 8.8795 dB |
| Gain at 1 kHz | drive_min | -75.5876 dB | -75.5876 dB |
| Gain at 20 Hz / 20 kHz | default | 18.0999 / 18.0513 dB | 18.0999 / 18.0514 dB |
| Gain at 20 Hz / 20 kHz | drive_max | 40.5433 / 41.1070 dB | 40.5433 / 41.1071 dB |

## Build

```
env CARGO_TARGET_DIR=$PWD/target cargo test
cd harness && uv run pytest -q
./validate.sh
./install.sh
```

`validate.sh` and `install.sh` force `CARGO_TARGET_DIR=$PWD/target` themselves. Install only after every Setting passes a full validation.

## Rights

Not legal advice.

- **Schematic (S1).** No licence text is on the drawing and the module page states none. It is treated as a public reference: values and topology are facts and were transcribed. The file and images of it are not in this repository.
- **Music Thing Modular site (S2, S4).** The About page says "Most Music Thing designs are published under a CCBYSA Creative Commons license" and links CC BY-SA 2.0 UK. It asks that new products not use the brand or the module names. Whether this module is among the "most" is not stated anywhere found, so no module-specific open grant is verified. If CC BY-SA did apply and a netlist counted as an adaptation, attribution and share-alike would follow; this repository is MIT and relies on the facts-only position.
- **Kit seller's page (S3).** An ordinary shop page; two descriptive facts were used.
- **Datasheets (S5 to S8).** Copyright onsemi (formerly Fairchild). Model parameters and ratings are used as facts; the datasheets are not copied. No vendor `.LIB` file is in this repository.
- **Name.** The plugin name, IDs and directory contain no brand or model name.
- **Licence of this project.** MIT (see `LICENSE`). VST3 builds use nih-plug's GPLv3 bindings, so a distributed VST3 binary is GPLv3; CLAP binaries can stay MIT.
