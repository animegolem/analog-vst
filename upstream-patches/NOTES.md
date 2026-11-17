# Upstream candidates for somewheresystems/analog-to-vst (f5a9d8d)

Nothing has been filed or pushed. Evidence comes from the green-overdrive and msk011-drive projects.

## Status of the local branches (clone in `upstream-work/analog-to-vst`, not tracked here)

- `section-reset-render` (1753cb3): the model renders each test section from a fresh DC state, as
  SPICE does. Ready. Patch: `section-reset/`; write-up: `section-reset.md`.
- `solver-coupling-groups`: commit 1 forces `CARGO_TARGET_DIR` (ready, `0001-*`). Commit 2 solves each
  coupled device group jointly and adds a relative sweep criterion (experimental, `0002-*`,
  `solver-coupling-groups.md`). Its CPU numbers were measured on an overloaded machine. Known gaps:
  greedy ordering, not a true topological sort; groups are fixed at construction.

Item 3 below states a diagnosis that was never proven: that a muted setting fails because the
SPICE reference cannot resolve it. With section reset, four-stage-fuzz (volume 0) and msk011-drive
(level 0) pass at a muted setting, so the cause there was the section-state mismatch. green-overdrive
still fails at level 0; its 0.05 setting is a workaround and the cause is unresolved. Do not send
item 3 upstream as written.

## 1. A single tightly coupled device pair makes the whole circuit use the joint solve (high impact, small)

`template/dsp/src/circuit.rs` `order_devices()` computes ONE `loop_gain` over all device pairs and sets
the global `self.joint = loop_gain >= 0.1`. Two antiparallel diodes written as separate
`n.diode()` calls have loop gain ~0.9999 with each other. That alone puts every device in the circuit,
including the unrelated input/output BJT followers, into one joint Newton solve.

Green Diode Overdrive, 48 kHz, one stereo instance:

| Model | CPU |
|---|---|
| two `n.diode()` calls (as written) | 107-135% |
| same circuit, diodes removed | 32% |
| same circuit, BJTs replaced by resistors | 34% |
| `n.diode_pair()` instead of two diodes | **31-36%** |

Fixes, smallest first:
- **Docs (S):** SKILL.md step 4: "write antiparallel diode pairs with `diode_pair`, never two
  `diode` calls; the netlist keeps two `D` lines." The template example already does this, but
  nothing says why it matters.
- **Lint (S):** `Netlist::diode()` (or a build-time pass) detects two identical diodes antiparallel
  on the same nodes and merges them into `DiodePair`, or panics with a hint.
- **Solver (M):** compute strongly connected components of the coupling graph (threshold 0.1) and
  joint-solve per component, Gauss-Seidel/sequential between components, instead of the global
  `joint` flag. This also helps circuits with one strongly coupled pair, such as a two-transistor
  fuzz, next to buffers.

## 2. Gauss-Seidel sweep tolerance causes low-level harmonic artifacts (msk011-drive)

`sweep_error() < 1e-7` left spurious H3/H5 at about -79 dBc for a -40 dBFS 5 kHz input with six BJTs;
SPICE has them below -120 dBc. Tightening it to 1e-12 fixed the error but raised CPU from 177% to 241%.
A relative criterion (scaled to signal level) or a joint solve per strongly coupled block (see #1)
is probably the real fix.

## 3. Near-muted settings exceed what the SPICE reference can resolve (harness/docs, S)

With a level pot at 0, the output is ~-100 dB. ngspice `reltol=1e-6` against ~4.5 V bias nodes gives
roughly µV absolute error, comparable to the signal, so static gain, harmonics and null "fail"
against an imprecise reference. Both projects validated at 0.05 instead. Options: the skill says
"0 and 1, except volume controls that mute: use 0.05", or the harness tightens SPICE options for
settings whose output RMS is below a threshold.

## 4. Validation cache keys (S)

`validate.sh` reuses `validation/pack` whenever it exists, regardless of `QUICK`, and keys SPICE
references on the netlist hash only. A QUICK pack can therefore be reused in a "full" run, and a
change to the harness/SPICE runner does not invalidate the references. green-overdrive's
validate.sh hashes `spice.py`, `pack.json`, `pack.wav` and the ngspice version into the key.

## 4b. A user-level CARGO_TARGET_DIR silently breaks validation and install (S, important)

`validate.sh` runs `cargo build -p render` and then `./target/release/render`. `install.sh` copies
`./target/bundled`, and harness `cli.py` uses `cargo run -p render --plain` for the SPICE control
values. With `CARGO_TARGET_DIR` exported (common: a shared target dir in ~/.zshrc), the build goes
elsewhere. The scripts then use stale local binaries, and `cargo run` can execute another
project's `render`, which feeds wrong control values to SPICE (seen here as `KeyError: 'B63'` with
the template's GAIN/LEVEL printed). It fails silently when the control names happen to match.
Fix: `export CARGO_TARGET_DIR="$PWD/target"` after the `cd` in both scripts (applied in all local
projects), or resolve the target dir with `cargo metadata`.

## 5. ngspice silently falls back to a transient operating point (S)

When gmin/source stepping fails, ngspice prints "Transient op started" and still exits 0, with
wrong initial capacitor voltages. green-overdrive's `spice.py` raises on that string and adds
`itl1=2000 itl6=1000`.

## 6. Docs: ngspice version / install (S)

nixpkgs ships only ngspice 45; the template validated on 45 without the slowdown (quick pack
~2 min, full ~7 min per setting on Apple Silicon). Suggest "47 recommended; 45 works on macOS
arm64; Ubuntu's 42 is the slow one", plus a Nix line next to Homebrew.

## 7. CPU expectations (docs)

SKILL.md says a TS808-class circuit costs 15-25% of a core. Measured: 31-36% here (with
`diode_pair`) and 241% for a six-BJT class-A mixer. Worth stating that cost scales with the number of
nonlinear devices and how strongly they are coupled, and that heavy models may only suit offline
bounce/freeze.
