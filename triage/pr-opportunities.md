# Upstream PR opportunities, ranked

Target: `somewheresystems/analog-to-vst` at `f5a9d8dafdb2691dc8a07fc685e1991b796f3c19`. The GitHub authenticated `gh issue list --state all --limit 100` and `gh pr list --state all --limit 100`, independently checked with REST list endpoints, returned empty arrays. As of this review there are no visible issues or PRs to duplicate; this does not rule out private discussions or work not represented there.[13][14] Research only: nothing was opened, posted or pushed.

**How to read counts:** an affected count is the number of distinct **T2 CSV rows** with this feature among their explicit blockers. These are optimistic engineering opportunity counts, not verified complete unlocks. They overlap: 1176 needs JFET *and* transformer; G9/G-Pultec need triode *and* transformer; 4-Pole Mission needs two device families; Dual SVF needs V2164 *and* zener. Dynamic CV, switching, missing parameters and full-device calibration may remain. No new device/model has been implemented or validated here, so the **verified new-circuit unlock count for every proposed PR is zero**. S/M/L are rough implementation scopes, not promises.

## 1. LM13700 and voltage-controlled gain family

**Affected T2 records: 15 distinct candidates, conditional** — the union of OTA-family (`shruthi-smr4-mkii`, `shruthi-four-pole-mission`, `shruthi-polivoks`, `befaco-chopping-kinky`, `northcoast-msk007`, `northcoast-msk009`, `kassutronics-wavefolder`, `aion-aurora-ota-compressor`) and VCA-family (`winterbloom-neptune`, `mutable-ripples`, `mutable-blades`, `mutable-veils`, `mutable-shelves`, `shruthi-four-pole-mission`, `shruthi-dual-svf`, `gyraf-gssl`); 4-Pole Mission appears in both, so union is 15. This is a **family of PRs**, not a claim that one LM13700 model substitutes for a V2164, a programmable op-amp, THAT VCA or every other gain cell. Exact chips and schematic revisions need auditing. **Size L** overall; an initial constrained model could be M.

- Implement and independently characterize LM13700 transconductance vs bias current, differential-input saturation, linearizing diodes, buffers and compliance; then separate V2164 exponential VCA and other applicable gain cells. Compare datasheet curves before claiming IC fidelity; establish rights/provenance of any vendor macromodel.
- `template/dsp/src/circuit.rs`: add `Element`/`Device` variants and port mappings, `ports`, `eval` (current/Jacobian) and `converged`, or decompose into already supported equivalent devices. `Device::eval` currently hard-codes `[f64; 2]` currents and `[f64; 4]` Jacobian: generalize workspaces and solver loops if the chosen nonlinear representation needs >2 ports. Fixed-gm `Element::Vccs` is not an OTA/VCA.
- `template/dsp/src/netlist.rs`: typed parameters and helper. Add a mathematically matching `.subckt`/behavioral reference in `template/reference/circuit.cir` or a dedicated fixture, plus examples and model documentation.
- Tests: finite-difference Jacobians including cutoff/saturation/bias and polarity; matched SPICE DC sweeps, AC gain vs control, large-signal compression, transient CV sweeps, block-size/automation continuity, and regression of `npn_jacobian_matches_finite_differences`. Test non-allocating realtime and convergence across temperature/operating ranges.
- Risks: parameter availability, device-specific compliance and distortion, Newton convergence, CPU, extra ports, power limits, legal reuse of macromodels. `shruthi-polivoks` needs its own programmable op-amp behavior rather than an arbitrary LM13700.

## 2. Independent audio/CV inputs and coupled stereo/sidechain

**Affected: 1 explicitly tagged T2 row (`gyraf-gssl`) for shared sidechain; 15 further T2 voltage-controlled candidates have potential dynamic-CV benefit, not independently confirmed as completed by port support.** These are `winterbloom-neptune`, `mutable-ripples`, `mutable-blades`, `mutable-veils`, `mutable-blinds`, `mutable-shelves`, `shruthi-smr4-mkii`, `shruthi-four-pole-mission`, `shruthi-dual-svf`, `shruthi-polivoks`, `befaco-chopping-kinky`, `northcoast-msk007`, `northcoast-msk009`, `kassutronics-wavefolder`, and `aion-aurora-ota-compressor`. It changes zero T2 rows to verified buildable by itself. Additional inputs also restore functionality to three reduced T1 mixer/drive cases. **Size L.**

- `circuit.rs`: replace `assert_eq!(inputs, 1)` and singular `input_row`, one-input state-space column and `tick(u: f64)` with indexed inputs and `tick(&[f64], &mut [f64])`; maintain preallocated matrices and an audio/control scaling policy. `netlist.rs`: named input and CV-port declarations. Preserve legacy one-port compatibility.
- `processor.rs`: support per-port resampling and a deliberately shared circuit or linked sidechain; the current stereo wrapper clones two independent mono circuits, which cannot represent the GSSL's shared control. `plugin/src/lib.rs`: expose declared audio/sidechain buses and sample-accurate CV/automation where host formats permit. Update `render/src/main.rs` and `harness/src/analog_harness/{spice,signals,cli}.py` for synchronized multi-source stimuli.
- Tests: DC source indexing; two independent signals; cross-coupled known linear circuit; bus routing and absent-sidechain behavior; stepped CV automation at different block sizes; stereo linkage; SPICE comparison with synchronized filesources and unchanged single-port regression. Risks: host CV support, sidechain normalization, oversampler latency/alignment, allocation, model bounds and meaningful voltage scaling.

## 3. JFET device

**Affected T2: 3** — `urei-1176ln`, `aion-helios-rat`, `runoffgroove-condor`. Condor is the strongest near-standalone reduced candidate; Helios also needs rail/slew for high-signal fidelity, 1176 also needs transformer. Solaris bypasses its optional JFET so is not counted. **Size M**, assuming a clearly bounded SPICE level-1 model.

- `circuit.rs`: `Element::Jfet` / `Device::Jfet`, two port pairs for drain-source and gate-source, `ports`/`eval`/`converged`, matched SPICE Shichman-Hodges channel current, gate diode behavior, polarity and analytic derivatives; represent model's series resistances and explicitly state capacitance limits.
- `netlist.rs`: N/P JFET model with `VTO`, `BETA`, `LAMBDA`, gate current etc. Add equivalent ngspice `.model ... NJF/PJF` test fixtures; use **exactly the same** chosen equations/parameters, not vendor model names with different physics.
- Tests: finite-difference Jacobian across cutoff, linear/triode, saturation, forward gate junction and region boundaries; ngspice IV/bias/transients; source-follower and resistance/gain-control fixtures; stress high drive and control sweeps. Risks: parameter spread, discontinuous derivatives, capacitance omissions, gate overdrive, convergence.

## 4. Coupled inductors/transformer

**Affected T2: 3** — `urei-1176ln`, `gyraf-g9`, `gyraf-gpultec`. Each also requires JFET or triode, so none is independently completed. **Size M–L.**

- `circuit.rs`: coupled inductor elements and trapezoidal companion history matrix; existing independent `Element::Inductor` is not a transformer. Implement winding orientation/dot convention, signed mutual inductance, leakage resistance and safe DC treatment. `netlist.rs`: coupling/helper with explicit L1/L2/k; SPICE fixture `L1`, `L2`, `K1 L1 L2 k` and winding resistors.
- Tests: analytic ideal turns ratio/polarity, open/short load, magnetizing current, SPICE AC and transient, passivity/energy, weak coupling and parameter extremes. Linear coupling is not saturation/hysteresis: nonlinear cores, source parameters and magnetic memory are additional work. Risks: singular matrices at perfect coupling, history stability, CPU, missing winding/core specs.

## 5. Triode Koren model

**Affected T2: 2** — `gyraf-g9`, `gyraf-gpultec`; both still require coupled transformers. **Size M–L.**

- `circuit.rs`: nonlinear two-port plate-grid/cathode `Element` + `Device` with a documented Koren-type law, plate and grid current, analytic Jacobian and convergence rules. `netlist.rs`: helper and independently licensed parameter sets. Implement matching ngspice behavioral-source/subcircuit fixture; include grid conduction if saturation drives the grid positive.
- Tests: device derivatives, plate curves over grid/plate ranges, grid conduction and DC operating point, transient SPICE null, signal-level sweeps. Risks: Koren variants are not interchangeable; beware exponent overflow, current continuity, cathode bias and parameter fitting; a linear transformer does not establish analog hardware accuracy.

## 6. Rail-clipped/slew-limited op-amp

**Affected T2: 2** — `mutable-blades`, `aion-helios-rat`; these still need their VCA or JFET. Also improves T1 operating-range fidelity for Stratus, Graphic EQ, Tonemender, Kinks and Ears, but does not automatically validate any of them. **Size M–L.**

- `circuit.rs` and `netlist.rs`: replace or augment the existing `opamp` VCCS–RC–VCVS helper with explicitly rail-aware finite output swing, slew-limited internal node, output loading and recovery, as an optional bounded macro. Add equivalent `reference/circuit.cir` model rather than silently changing the existing one-pole behavior.
- Tests: finite differences for each nonlinear device, gain/bandwidth, both rails, slew ramps, overload recovery, closed-loop clipping and per-model SPICE agreement; preserve legacy tests. Risks: hard clamps break differentiability; unrealistic recovery and stability can be worse than a transparent linear approximation.

## 7. Zener and germanium BJT leakage

**Affected: 1 T2 each** — `shruthi-dual-svf` needs zener **and** V2164; `aion-solaris-fuzz` needs BJT leakage with pickup simulator bypassed. **Size S–M** per model.

- Add reverse-breakdown `BV`/`IBV` terms and suitable conductance/Jacobian to `Device::Diode`, plus fields in `DiodeModel`; use ngspice D model with matching breakdown. Extend `Bjt`/`BjtModel` and BJT `eval` for documented junction leakage currents/emission factors; calibrate germanium parameter set, avoid inventing a measured device.
- Tests: both branch derivatives and knees; npn/pnp symmetry, reverse and forward leakage, DC bias and temperature sensitivity against matching SPICE, plus overload/recovery. Risks: germanium leakage and temperature vary widely; current code fixes thermal voltage at 27 C. This is not a full thermal model.

## 8. Multiple observed outputs and >8 controls

**Affected: 0 T2 rows become fully buildable without their device extensions;** benefits full outputs in Ripples, Kinks, Blades, Veils, 4-Pole Mission and MSK-009 (6 catalog records) as well as >8 controls in complex modules. **Size M–L.**

- Generalize `Netlist.output`, `Circuit::new` output tuple, output rows in the state-space transform and `tick` results; expose plugin multi-output buses in `plugin/src/lib.rs`, render files and SPICE `.save`/raw handling in `harness/src/analog_harness/spice.py`. Separately replace fixed `[ControlParam; 8]` safely with a declared, stable parameter scheme.
- Tests: output-node ordering, polarity, shared state, multiple SPICE vectors, backwards compatibility, parameter ID and preset migration across versions, no allocations. Risks: host bus mapping, state/preset breakage and simultaneous-output versus independent duplicate circuits.

## 9. Harness correctness: cache keys and state continuity

**Affected: 0 additional T2 rows directly;** protects every future validation, including T1/T2. **Size M.**

- In `template/validate.sh`, `validation/pack/pack.json` is reused merely if present, so a QUICK run can reuse a previous full pack and vice versa; the reference key contains only netlist hash and `render --plain` values, not pack content, ngspice version or harness implementation. Switching stimuli or simulator can silently reuse a stale reference. Hash the pack manifest/audio, solver binary/version, harness protocol version, netlist and control settings; invalidate or namespace caches on any change. Do not add the real-time model binary to the SPICE reference cache key unless it affects its control mapping, but always rerender the model.
- `harness/src/analog_harness/spice.py` independently resets each section to DC; add a continuous-state mode for compressor recovery/long memory and explicit fixtures that prove whether the reset assumption is safe. Add tests in `harness/tests/test_harness.py` and `tests/` for QUICK→full→QUICK, mutated input pack, version/key change and long settling; fail if a stale reference is accepted.
- Risks: more simulation time/storage, cache migration and flaky long transients. SPICE-vs-model agreement remains relative to the selected common model, not hardware equivalence.

## 10. ngspice version, non-Homebrew and Linux/Nix docs

**Affected: 0 T2 unlocks;** broad onboarding/reproducibility. **Size S** for documented evidence, M for a CI matrix.

- Local `ngspice --version` reports **45**, installed under the Nix profile. The task input reports the template passed **QUICK** validation on Nix ngspice 45; that prior run's artifact was not supplied here, so treat it as **user-provided, not independently reproduced**. It does not establish full-pack performance, XSPICE availability on every build, or support for older versions.
- Update `skills/analog-to-vst/SKILL.md`, `GUIDE.md`, `README.md`, `template/validate.sh` version warning and `.github/workflows/ci.yml`: distinguish tested version/build/features from a hard 47 prerequisite; recommend measuring `tests/spice_smoke.py` and a QUICK validation with XSPICE `filesource`, then full settings before claiming support. 45 is a candidate supported version after CI reproduces it, not an unconditional minimum.
- Add Nix-based and distro/source-build routes alongside Homebrew, with tested actual dependency names, XSPICE/KLU options and OS-specific commands; do not assert all Ubuntu ngspice 42 builds are identical. Document Rust/uv, Linux libraries already listed in CI, and Linux installation separately: `template/install.sh` explicitly uses macOS `~/Library/...` and `/usr/bin/trash`.
- Tests: clean macOS/Linux job, Nix build or pinned shell, ngspice version + `filesource` smoke, QUICK and full pack separate caches, bundle load check; measure elapsed time rather than weakening thresholds. Risks: distro XSPICE differences, Nix channel changes, architecture-dependent performance.

## 11. Wrapper license and honest validation-claim docs

**Affected: 0 T2 unlocks;** potentially prevents incorrect distribution claims. **Size S–M.**

- Audit pinned `nih_plug`/`vst3-sys` features and dependency graph for both artifacts; configure a genuine CLAP-only build only if it removes GPL-bound dependencies, test `cargo tree -e features`, link graph and distribution terms. The newer MIT VST3 SDK does not automatically relicense a separately pinned binding. `GUIDE.md` currently asserts CLAP may stay MIT without establishing this for the default feature configuration.
- Rewrite GUIDE's promise of proof of behavior of the real circuit: describe accuracy vs a shared SPICE model, disclose generic op-amp/diode/BJT assumptions and require independent source transcription/hardware measurement for physical-fidelity claims. Audit scripts, dependency licenses, generated bundles and notice files. Risks: legal interpretation requires qualified review and artifact-specific evidence.

## Sources

[13] https://github.com/somewheresystems/analog-to-vst/issues?q=is%3Aissue
[14] https://github.com/somewheresystems/analog-to-vst/pulls?q=is%3Apr
