# Solver coupling groups — experimental candidate

Base: upstream f5a9d8d. Local branch `solver-coupling-groups`. No push or PR.

## Problem and implementation

The original solver switches every device into one joint Newton solve if any two have normalized mutual loop gain at least 0.1. The change makes connected components of those pairwise-strong edges, then orders blocks greedily by the strongest remaining incoming coupling. A block with multiple devices receives a joint Newton solve; singleton blocks use the existing per-device Newton method. Outer Gauss–Seidel sweeps carry the currents between blocks. A single block is the original all-device joint solve special case.

The old sweep check assumed port indices follow device solve order, although `order_devices` reorders devices, not ports. The new check uses the actual port indices of later blocks. It tests the conservative sum of absolute voltage changes against `1e-12 V + 1e-8 * max_port_sample_movement`. Using movement instead of DC bias scales with the signal; the floor avoids chasing floating-point cancellation at rest. This is a heuristic for sweep mismatch, not a proven bound on final output error, particularly for ill-conditioned feedback. It does not necessarily reduce cost versus a fixed 1e-12 V threshold.

A separate commit makes the template's `validate.sh` and `install.sh` explicitly set `CARGO_TARGET_DIR="$PWD/target"`. Those scripts and the harness assume `./target`, and an inherited shared Cargo target directory otherwise lets them use an unrelated executable. A stubbed integration test passed with a deliberately wrong inherited target; no real plugin install was performed.

## Measured CPU

`cargo run --release --example bench -p dsp 48000` twice per version, stereo, reported as percent of a single core. Each project used its own absolute `CARGO_TARGET_DIR` and each solver change rebuilt both render and bench in that project; logs record the `target/release/...` paths. The machine was exceptionally busy with other work, so these wall-clock measurements are noisy and **do not establish a reliable speedup**, especially for msk011. They are not comparable to idle-machine figures.

| Project / model | Before, run 1 / 2 | After, run 1 / 2 |
|---|---:|---:|
| template example | 40.3 / 51.0% | 28.6 / 33.3% |
| green-overdrive, diode_pair | 167.7 / 164.5% | 118.4 / 93.2% |
| green-overdrive, two diodes | 445.2 / 404.2% | 142.8 / 151.6% |
| msk011-drive, upstream 1e-7 | 276.2 / 486.6% | 875.7 / 902.2% |
| msk011-drive, local 1e-12 | 838.8 / 1023.5% | 875.7 / 902.2% |

The upstream 1e-7 msk011 row is an inaccurate baseline for its reported low-level harmonics. The first after run is slower than the first local-tight run; repeat on an idle machine before any performance claim.

## Output comparisons

One second of a generated 48 kHz mono two-tone (440 Hz plus 5 kHz), rendered through each old and new solver with the project renderer. Output files are float32; max absolute difference is in normalized output-sample units, and dB is 20 log10(max difference / peak absolute old output). Zero means float32 sample-exact for this signal only. The msk011 reference below is its local 1e-12 version; all others are stock upstream. Extreme settings: template gain=1 level=1; green drive=1 tone=1 level=1; msk011 level=1.

| Project / model | Setting | Max difference | dB relative to peak |
|---|---|---:|---:|
| template | default | 0 | −∞ |
| template | extreme | 0 | −∞ |
| green diode_pair | default | 0 | −∞ |
| green diode_pair | extreme | 0 | −∞ |
| green two diodes | default | 7.4506e-9 | −144.66 |
| green two diodes | extreme | 2.3842e-7 | −139.10 |
| msk011, local-tight reference | default | 7.4506e-9 | −140.56 |
| msk011, local-tight reference | extreme | 5.9605e-8 | −142.50 |

These two-tone checks are not a proof of equivalence for every signal or knob setting; they also do not include a separate comparison of new output to the inaccurate upstream msk011 solver. Full raw log and WAVs: `/Users/golem/.hermes/cache/scratch/solver-eval/` (scratch may be pruned).

## Tests and SPICE

- Template workspace `cargo test`: 10 DSP tests passed, including the synthetic independent follower and antiparallel diode grouping case. Its 2,000-sample grouped-versus-single-joint comparison checks output and each port to 1e-9 V. `cargo fmt --all -- --check` passed.
- Template `QUICK=1 ./validate.sh`: default, max, min PASS, ngspice 45.
- msk011 scratch `QUICK=1 ./validate.sh`: default, low, high PASS, ngspice 45. The harmonic check for default and low passes because **all** reference and model products are below its −90 dBc floor; the reported zero is not a measured zero difference. For the specifically problematic 5 kHz, −40 dBFS default step, model H3/H5 = −137.86/−141.32 dBc versus SPICE −136.43/−140.06 dBc. These are below the harness comparison floor and should not be interpreted as a high-precision distortion match. High's worst harmonic difference was 0.01505 dB.

## Risks / outstanding work

- This is **not yet an upstream-ready proof**: greedy ordering is not a formal topological sort, and pairwise strong edges can miss multi-device strong cycles. Components and order are fixed at construction even when pots change the coupling matrix. The 32-sweep cap can silently terminate without convergence; changes in controls or multi-way feedback need tests and probably a safe fallback.
- The stopping threshold uses the maximum movement of any port, so a large unrelated signal can mask an error in a small-signal block. It bounds only the estimated contribution of later-block current changes, not nonlinear solve error or amplified error in the final output. The threshold needs a per-block analysis and broader low-level tests.
- Multi-device components still use the full-size Jacobian storage, and no strong claim can be made about speed for circuits other than the measured cases. Busy-machine timings make even those results inconclusive.
- Test signals were short; no null across the full validation pack against both old solvers was done. No runtime knob-sweep, strongly coupled three-device cycle, convergence-cap, or realtime allocation regression test was added. Repeat benchmarks on an idle machine and extend coverage before proposing upstream.
