# Shared brief for analog-to-vst workers

- Skill: ~/.hermes/skills/analog-to-vst/SKILL.md + references/ (read first; follow the procedure).
- Upstream clone (read-only reference): ~/.hermes/cache/scratch/avst/repo (commit f5a9d8d).
  Scaffold: `python3 ~/.hermes/cache/scratch/avst/repo/scripts/new_project.py <dir> --name '<Name>'`.
- Working examples (read, do not edit): green-overdrive (op-amp + diode pair + 2 BJTs, 36% CPU),
  msk011-drive (6 BJTs, all PASS, 241% CPU).
- Research catalog: ~/.hermes/cache/scratch/avst/research/analog-vst-research/data/catalog.json; triage at
  triage/triage.md (build briefs per target). Upstream issues log: upstream-patches/NOTES.md.

## Environment
- macOS arm64, nix (no Homebrew). ngspice 45 is the only version; it works, ignore the "use 47" warning.
- Full validate.sh takes ~5-10 min per setting. Run it with terminal(background=true, notify=true) and wait with
  process wait; do NOT kill a running validation to "check" something. QUICK=1 for iteration; delete validation/pack
  before the final full run (the pack cache ignores QUICK).
- Command approvals are auto-approved for workers.
- Only edit inside your own project directory.

- ~/.zshrc exports CARGO_TARGET_DIR=~/.cargo-target and BACKGROUND terminal processes inherit it.
  Project validate.sh/install.sh now force `CARGO_TARGET_DIR=$PWD/target`; for any other background cargo command
  prefix `env -u CARGO_TARGET_DIR`. Otherwise you run other projects' render/bench binaries.

## Lessons already learned (apply them)
1. Antiparallel diodes: write `n.diode_pair(a, b, model)` in model.rs (netlist keeps two D lines). Two separate
   `n.diode` calls make the solver joint-solve every device in the circuit (CPU ~4x).
2. Controls that mute the output (volume at 0) are below SPICE precision (~-100 dB): validate at 0.05 instead,
   comment why in settings.txt.
3. Template BJT model is reduced Gummel-Poon (no charge storage/leakage). Take IS,BF,BR,VAF,IKF,RB,RC,RE from a cited
   vendor model and use the SAME reduced .model in the .cir. Document the omission.
4. Op-amps: single-pole, no rail clipping/slew. Check op-amp outputs stay inside the rails at max settings with your
   chosen VOLTS_PER_FULL_SCALE; if not, choose scaling that keeps them in, and document.
5. Run `cargo run --release --example bench -p dsp 48000` as soon as model.rs compiles and report it.
6. Never loosen tolerances. On FAIL, diagnose with references/diagnosing.md.
7. Plugin names never contain brand/model names. README: "modeled after the <X> circuit; trademarks belong to their
   owners", sources, assumptions, license notes.
8. Append any new skill/template friction (concrete file + proposed fix) to <project>/NOTES-skill-friction.md.

## Final answer
Validation summary table, worst value per check, CPU, assumptions, bundle paths verified with ls (after ./install.sh),
and anything not achieved stated plainly.
