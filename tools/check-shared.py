#!/usr/bin/env python3
"""Checks that every Project's copy of each shared Solver-side file is byte-identical.

Exits non-zero, naming the Projects and the file, when copies differ or one is missing.
Run from anywhere: python3 tools/check-shared.py

Not in the list, on purpose:
- dsp/src/model.rs is the Circuit, per Project.
- dsp/src/oversample.rs holds the per-Project MIN_CIRCUIT_RATE and the tests that follow it.
- render/tests/ otherwise holds tests that need an input that fails on one Circuit.
- harness/, validate.sh and install.sh are outside the Solver.
- template-check/ is the unmodified template, kept as the baseline.
"""

import hashlib
import pathlib
import sys

PROJECTS = [
    "green-overdrive",
    "msk011-drive",
    "seven-band-eq",
    "four-stage-fuzz",
    "stack-boost",
    "precision-rectifier",
    "three-transistor-drive",
    "feedback-pair-drive",
]

SHARED = [
    "dsp/Cargo.toml",
    "dsp/examples/bench.rs",
    "dsp/src/circuit.rs",
    "dsp/src/control.rs",
    "dsp/src/lib.rs",
    "dsp/src/netlist.rs",
    "dsp/src/processor.rs",
    "plugin/Cargo.toml",
    "plugin/src/lib.rs",
    "render/Cargo.toml",
    "render/src/main.rs",
    "render/tests/refused_capture.rs",
]


def main() -> int:
    root = pathlib.Path(__file__).resolve().parent.parent
    drifted = 0
    for name in SHARED:
        groups: dict[str, list[str]] = {}
        for project in PROJECTS:
            path = root / project / name
            key = hashlib.sha256(path.read_bytes()).hexdigest()[:12] if path.is_file() else "missing"
            groups.setdefault(key, []).append(project)
        if len(groups) == 1 and "missing" not in groups:
            continue
        drifted += 1
        print(f"DIFFERS {name}")
        # Largest group first: the others are the copies that drifted from it.
        for key, projects in sorted(groups.items(), key=lambda g: -len(g[1])):
            for project in projects:
                print(f"  {key}  {project}/{name}")
    if drifted:
        print(f"{drifted} of {len(SHARED)} shared files differ between Projects")
        return 1
    print(f"{len(SHARED)} shared files are identical in {len(PROJECTS)} Projects")
    return 0


if __name__ == "__main__":
    sys.exit(main())
