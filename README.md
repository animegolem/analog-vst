# analog-vst

Audio plugins (VST3 and CLAP) modeled from analog circuit schematics, each checked against an
ngspice simulation of the same circuit. The projects are built with the
[analog-to-vst](https://github.com/somewheresystems/analog-to-vst) skill and template.

SPICE validation compares two simplified implementations of a circuit. It does not compare a
plugin with measured hardware.

## Layout

| Path | Contents |
|---|---|
| `green-overdrive/` | Green Diode Overdrive: op-amp, feedback diode pair, two transistor followers |
| `msk011-drive/` | Class A Transistor Drive: six-transistor discrete drive |
| `seven-band-eq/` | Seven Band Graphic: seven-band graphic equalizer |
| `four-stage-fuzz/` | Four Stage Sustainer: four-stage transistor fuzz |
| `stack-boost/` | Three-Band Stack Boost: two op-amp stages around a passive bass, mid and treble tone stack |
| `precision-rectifier/` | Precision Rectifier: two op-amp sections and two diodes, a DC-coupled full-wave rectifier with no controls |
| `three-transistor-drive/` | Three-Transistor Drive: NPN differential pair and a PNP stage with feedback, one drive control (offline-only) |
| `feedback-pair-drive/` | Feedback Pair Drive: long-tailed pair, PNP stage and phase splitter in a feedback amplifier, one level control (offline-only) |
| `template-check/` | The unmodified template example, kept as a baseline |
| `triage/` | A catalog of 104 candidate circuits sorted into tiers, with build briefs |
| `upstream-patches/` | Patches and notes for the upstream template |
| `SPEC.org` | The spec: objective, glossary, rules, live work, and what has been proven |
| `WORKER-BRIEF.md` | Environment facts and lessons for an agent that builds a plugin |

Each plugin directory is a separate Cargo workspace with its own `README.md`, which lists the
sources, the modeling assumptions, and the license notes for that circuit.

## Build and validate

You need Rust, Python with `uv`, and ngspice.

```sh
cd green-overdrive
./validate.sh   # simulate in SPICE, render the model, compare; writes validation/summary.md
./install.sh    # build and install the VST3 and CLAP bundles
```

A full validation takes 5 to 10 minutes per setting. Run one validation at a time: parallel runs
overload the machine and make the CPU benchmarks meaningless. Use `QUICK=1 ./validate.sh` while you
iterate, and delete `validation/` before the final full run.

## Rules

- Never loosen a validation tolerance. Fix the cause.
- Install a plugin only after every setting passes.
- Plugin names contain no brand or model names. Each README credits the circuit it is modeled after.
- Take component values from sources. Do not copy schematic files into a project.

## License

MIT. See `LICENSE`. The template code is copyright Somewhere Systems.
