# Section-reset validation patch

Branch `section-reset-render`, commit `1753cb3`, based on `origin/main`; patch: `section-reset/0001-Render-validation-sections-from-fresh-DC-state.patch`. Not pushed.

## Problem and change

`spice.render` simulates each section, plus its 0.5-second trailing gap, from a fresh DC operating point, then assembles the results. Previously `validate.sh` rendered the model once over the whole pack, so its capacitors and bias retained previous-section state. That is not a comparable initial condition for a slowly recovering circuit. Green Diode Overdrive at `level=0.05` failed its harmonic check by 16.23 dB (H2, 100 Hz, -40 dBFS); a separately initialized tone had previously agreed with SPICE within approximately 0.00006 dB. Four Stage Sustainer showed a 13.88 dB default harmonic failure; its isolated-tone diagnostic agreed within 0.04 dB.

The harness now renders each `[start, min(start + length + int(0.5 * fs), pack_length))` separately through the existing `render` executable. Each invocation constructs a new `Processor`, which initializes its circuit to its DC operating point at the specified controls. Outputs are assembled in manifest order with `out[a:b] = y`, just like SPICE, including overlap overwrite and unfilled zeros. It changes neither circuit, tolerance, SPICE runner behavior nor the normal continuous plugin renderer. It does not establish correctness of continuous-state behavior; that requires a separate continuous reference.

Files in patch: `template/harness/src/analog_harness/cli.py`, `spice.py` (documentation only), `harness/tests/test_harness.py`, and `validate.sh`. The tests exercise restart with a deliberately long-lived accumulator, overlap overwrite, and an actual compiled model: the second section is bit-identical to its isolated model render, while the continuous render differs. `cargo test`: 9 passed; `uv run pytest -q`: 10 passed. Ruff and `git diff --check` passed.

## Filter boundary qualification

The two paths are not mathematically sample-identical at section boundaries: SPICE upsamples the entire pack and slices, then stitches before downsampling, whereas the existing model resets both its circuit and its resampling filters for each section and stitches at base rate. In particular custom overlapping/non-silent sections can differ from SPICE even with perfectly matched circuits. Do not interpret this as proof of sample-exact boundary handling for arbitrary manifests.

For the actual Green full pack at 96 kHz (45 sections), direct comparison of SPICE's 4x 57-tap upsampler on the whole pack versus separate slices showed **zero difference in every active section** and in the first 28 base samples at each start. Differences occurred only in the last 27 oversampled samples (6.75 base samples) of a trailing gap as the whole-pack filter sees the next section; maximum input difference there was 0.02813 full scale. SPICE's stitched decimation and the model's separately reset decimation can also differ close to boundaries. The actual unchanged validation checks, including frequency response and burst null, all pass; this empirical result applies to this generated pack and these settings, not arbitrary overlapping stimuli. Harmonic and IMD measurements exclude 200 ms at section start and 50 ms at section end; sweep analysis fades out the end of its 0.5-second tail.

## Before and after

* Green full pack `level-low`: harmonics 16.23 dB FAIL -> 0.261 dB PASS (worst case now H3 at 5 kHz, -40 dBFS); burst null -58.25 -> -75.56 dB. The same four cached references were reused after verifying Green's cache-key logic matches; existing ngspice and cache fixes were retained. All four settings passed: default, min-drive-tone, max-drive-tone and level-low. Tolerances unchanged.
* Template QUICK: default, max and min all passed before and after. The reported default intermodulation changed 0.09488 -> 0.09489 dB; min static gain changed 0.0003367 -> 0.0003366 dB. Other displayed values were unchanged. Existing quick pack and SPICE references were reused.
* Four Stage Sustainer with existing references: default harmonics 13.88 dB FAIL -> 0.1159 dB PASS; sustain_min also PASS (harmonics 0.0249 dB). All checks passed for both. Only these two settings had existing SPICE references; the other five settings cannot be evaluated via comparison alone. Their legacy keys match netlist/controls but do not hash the pack or SPICE runner, so full reference provenance cannot be cryptographically verified; those inputs predate the references.

Installation after Green passed: `~/Library/Audio/Plug-Ins/VST3/Green Diode Overdrive.vst3` and `~/Library/Audio/Plug-Ins/CLAP/Green Diode Overdrive.clap`, both verified with `ls -ld`.
