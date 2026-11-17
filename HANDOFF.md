# Handoff: analog-to-vst work (as of 2026-11-17 ~02:15 CST)

Everything lives in this repo. None of it depends on Hermes. The skill is a markdown procedure
plus a Rust/Python template, and any coding agent can follow it.

## Read first
- Skill: `~/.hermes/skills/analog-to-vst/SKILL.md` + `references/`. This is the installed copy and has
  two local edits vs upstream: the BJT limits note and two diagnosing.md rows. Upstream clone:
  `~/.hermes/cache/scratch/avst/repo` (commit f5a9d8d). For Claude Code:
  `cp -R ~/.hermes/skills/analog-to-vst ~/.claude/skills/`.
- `WORKER-BRIEF.md`: environment facts and lessons learned. Give it to any sub-agent.
- `upstream-patches/NOTES.md`: running list of upstream issues/PR candidates.
- `triage/triage.md`, `triage/triage.csv`: the 104-record research catalog sorted into T1-T4 tiers,
  with build briefs for the T1 targets. Research source: `~/Downloads/analog_vst_research.zip`
  (unzipped in `~/.hermes/cache/scratch/avst/research/`), `~/Downloads/analog_vst_catalog.xlsx`.

## Environment gotchas (these caused most of the lost time)
1. `~/.zshrc` exports `CARGO_TARGET_DIR=~/.cargo-target`. The template scripts assume `./target`.
   Every project's `validate.sh`/`install.sh` now forces `CARGO_TARGET_DIR=$PWD/target`. Use
   `env -u CARGO_TARGET_DIR` for any other cargo command, or you run another project's binaries.
2. ngspice 45 (nix; no Homebrew). It works; ignore the "use 47" warning.
3. A full validation is ~5-10 min per setting when the machine is quiet. Running several at once
   overloads the machine (load avg hit ~195) and makes every CPU benchmark meaningless. Run one
   validation at a time.
4. `validation/pack` is cached regardless of `QUICK`. Delete `validation/` before a final full run.

## Plugin status

| Project | Plugin name | Validation | Installed | CPU (stereo, 48k) | Notes |
|---|---|---|---|---|---|
| `msk011-drive` | Class A Transistor Drive | full, all PASS (default/low/high), clean rebuild | yes, VST3+CLAP | ~240% of one core | Offline/freeze only. Reduced 2N5088 model. Has a local sweep tolerance of 1e-12 in circuit.rs. |
| `green-overdrive` | Green Diode Overdrive | full, all 4 PASS with section-reset harness | yes, VST3+CLAP | ~36% | TS-808 values; bypass JFETs omitted. |
| `seven-band-eq` | Seven Band Graphic | **aborted**: SPICE stopped early on s007 (see below) | no | ~80% (measured under load) | Slider value unknown: model assumes 10k linear and gives about ±6.5 dB per band; sources suggest ±12-15 dB. Check the transcription against Douglas Self's "Small Signal Audio Design" graphic EQ. No LICENSE in TomWhitwell/Graphic. |
| `four-stage-fuzz` | Four Stage Sustainer | 6/7 PASS; `tone_min` FAIL (see below) | no | ~1000% (measured under load) | Big Muff, ElectroSmash American V3 values. Before restarting with section-reset, default and sustain_min passed. Offline only unless the solver patch helps. |
| `template-check` | (template example) | quick PASS | no | 10% | Baseline. |

**Last background job (finished):** validated `seven-band-eq`, then `four-stage-fuzz`, installing only on
all-PASS. Logs: `seven-band-eq.validate.log`, `four-stage-fuzz.validate.log`. Results:
`<project>/validation/summary.md`.

**Results of the last run (02:2x CST):**
- `four-stage-fuzz`: 6/7 PASS, with section-reset. `tone_min` FAILs only on frequency response,
  0.124 dB vs 0.1. The error grows smoothly toward 20 kHz at every level (~0.10 dB at 19.3 kHz even at
  -40 dBFS) where tone_min rolls off to about -41 dB. That matches diagnosing.md row 1, frequency
  warping: raise `MIN_CIRCUIT_RATE` in `dsp/src/oversample.rs`. The CPU cost is quadratic-ish, and this
  model is already ~1000%. Not installed.
- `seven-band-eq`: validation aborted. ngspice section `s007` (default setting) exited 0 but wrote
  a short raw file (8364 rows; the transient stopped early), so the harness raised "ngspice failed on
  s007". Likely a timestep/convergence stop in the 7-band op-amp netlist. Run
  `uv run harness spice ...` for that section with ngspice stdout visible, or check for "timestep too
  small". Not installed.

If a job dies, rerun it:
`cd <p> && rm -rf validation && ./validate.sh && ./install.sh`.

## Upstream work (local only, nothing pushed)
Clone: `upstream-work/analog-to-vst`. Branches:
- `section-reset-render` (1753cb3, based on origin/main): the model renders each test section from
  a fresh DC state, matching SPICE. It fixed real false FAILs: green level-low harmonics 16.2 to
  0.26 dB, fuzz default 13.9 to 0.12 dB. Template tests and QUICK validation pass. **Ready.**
  Patch: `upstream-patches/section-reset/`; write-up: `upstream-patches/section-reset.md`.
- `solver-coupling-groups`: commit 1 forces `CARGO_TARGET_DIR` (**ready**,
  `upstream-patches/0001-*`); commit 2 does joint Newton per coupled device group plus a relative
  sweep criterion (**experimental**, `upstream-patches/0002-*`, `solver-coupling-groups.md`).
  Outputs are bit-identical on the template and green-overdrive and within -139 dB elsewhere, and
  it removes the MSK011 low-level artifact. Its CPU numbers were measured on an overloaded machine,
  so rerun the benchmarks on an idle machine before claiming a speedup. Known gaps: greedy ordering
  (not a true topological sort); groups are fixed at construction.

Other PR candidates (details in NOTES.md): document/lint `diode_pair` (two separate `diode` calls
make the whole circuit joint-solve, ~4x CPU); guidance for near-mute settings (validate at 0.05);
cache keys; ngspice "Transient op started" silent fallback; ngspice 45/Nix docs; CPU expectations
in SKILL.md (it says 15-25% for TS808-class; real range is 36% to 1000%+).

## Correction: the "near-mute = SPICE precision" claim is unproven
Level=0 failures were attributed to SPICE reltol (µV error vs a ~-100 dB signal). That was never
confirmed: a tighter SPICE run failed to converge. An isolated section started from rest matched SPICE
within 0.00006 dB at level 0, which points to the section-state mismatch that section-reset later
fixed (green level 0.05: 16.2 dB FAIL to 0.26 dB PASS). The 0.05 substitution in green-overdrive,
msk011-drive and four-stage-fuzz is therefore probably unnecessary. Re-test level/volume = 0 with
section-reset before keeping NOTES.md item 3.

## Next steps, in order
1. Check the EQ and fuzz results (above). Then re-validate level=0 (green) / volume=0 (fuzz) /
   level 0 (msk011, which needs the section-reset harness ported first).
2. On an idle machine, benchmark all four plugins with the old solver vs the `solver-coupling-groups`
   circuit.rs. If the speedup holds, update the 0002 write-up.
3. Resolve the EQ slider value/range question, then fix the model if needed and re-validate.
4. Next T1 builds from `triage/triage.md`: Music Thing Mini Drive, London Drive, Tonemender (its page
   is CC BY-NC-SA, so non-commercial), Kinks SIGN.
5. Decide which upstream patches to send. Section-reset and CARGO_TARGET_DIR are clean
   single-purpose PRs.

## Rules we've held to
- Never loosen validation tolerances; fix the cause.
- Install only after all-PASS. Workers twice installed early; check for this.
- No brand names in plugin names; README credits "modeled after X; trademarks belong to owners".
- Take values from sources; never copy schematic files into projects.
