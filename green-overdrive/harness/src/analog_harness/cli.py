import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
import soundfile as sf

from . import analysis, compare, signals, spice

REPO = Path(__file__).resolve().parents[3]


def read_capture(path: Path, fs: int) -> np.ndarray:
    x, rate = sf.read(path, dtype="float64", always_2d=True)
    if rate != fs:
        sys.exit(f"{path}: sample rate {rate} does not match the pack's {fs}")
    return x[:, 0]


def plain_controls(controls: list[str]) -> dict[str, str]:
    """Control values in physical units from the Rust model, keyed by netlist placeholder."""
    plain = subprocess.run(
        ["cargo", "run", "-q", "--release", "-p", "render", "--", "--plain", *controls],
        cwd=REPO,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return dict(line.split("=", 1) for line in plain.split())


def render_sections(stimulus: np.ndarray, manifest: dict, controls: list[str], executable: Path) -> np.ndarray:
    """Render each SPICE section and its trailing gap using a fresh processor."""
    fs = manifest["sample_rate"]
    out = np.zeros_like(stimulus)
    with tempfile.TemporaryDirectory() as tmp:
        src, dst = Path(tmp) / "in.wav", Path(tmp) / "out.wav"
        for i, sec in enumerate(manifest["sections"], 1):
            a = sec["start"]
            b = min(a + sec["length"] + int(0.5 * fs), len(stimulus))
            sf.write(src, stimulus[a:b], fs, subtype="FLOAT")
            subprocess.run([str(executable), str(src), str(dst), *controls], check=True)
            y, rate = sf.read(dst, dtype="float64")
            if rate != fs or y.ndim != 1 or len(y) != b - a:
                raise ValueError("model section capture has unexpected shape or sample rate")
            out[a:b] = y
            print(f"  model section {i}/{len(manifest['sections'])} done", file=sys.stderr, flush=True)
    return out


def main():
    p = argparse.ArgumentParser(prog="harness")
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("signals", help="write pack.wav and pack.json")
    s.add_argument("out_dir", type=Path)
    s.add_argument("--rate", type=int, default=96_000)
    s.add_argument("--quick", action="store_true", help="shorter pack with fewer levels, for CI")

    a = sub.add_parser("analyze", help="measure one capture of the pack")
    a.add_argument("pack", type=Path, help="pack.json")
    a.add_argument("capture", type=Path)
    a.add_argument("-o", "--out", type=Path)

    c = sub.add_parser("compare", help="validate a model capture against a reference capture")
    c.add_argument("pack", type=Path, help="pack.json")
    c.add_argument("reference", type=Path)
    c.add_argument("model", type=Path)
    c.add_argument("-o", "--out", type=Path, required=True, help="report directory")

    r = sub.add_parser("spice", help="render a SPICE reference capture of the pack with ngspice")
    r.add_argument("netlist", type=Path)
    r.add_argument("pack", type=Path, help="pack.json")
    r.add_argument("controls", nargs="*", help="control=knob position (0..1); fills the netlist placeholders")
    r.add_argument("-o", "--out", type=Path, required=True)

    model = sub.add_parser("model-sections", help="render each section from the model's DC state")
    model.add_argument("pack", type=Path, help="pack.json")
    model.add_argument("controls", nargs="*", help="control=knob position (0..1)")
    model.add_argument("-o", "--out", type=Path, required=True)

    o = sub.add_parser("op", help="print the netlist's DC operating point (node voltages)")
    o.add_argument("netlist", type=Path)
    o.add_argument("controls", nargs="*", help="control=knob position (0..1)")

    args = p.parse_args()
    if args.cmd == "signals":
        print(signals.write(args.out_dir, args.rate, args.quick))
        return
    if args.cmd == "op":
        params = plain_controls(args.controls)
        params.pop("_VOLTS_PER_FULL_SCALE", None)
        print(spice.operating_point(args.netlist, params))
        return

    stimulus, manifest = signals.load(args.pack)
    fs = manifest["sample_rate"]
    if args.cmd == "model-sections":
        y = render_sections(stimulus, manifest, args.controls, REPO / "target/release/render")
        sf.write(args.out, y.astype(np.float32), fs, subtype="FLOAT")
    elif args.cmd == "spice":
        params = plain_controls(args.controls)
        scale = float(params.pop("_VOLTS_PER_FULL_SCALE", 1.0))
        print(" ".join(f"{k}={v}" for k, v in params.items()))
        y = spice.render(args.netlist, stimulus * scale, manifest, params) / scale
        sf.write(args.out, y.astype(np.float32), fs, subtype="FLOAT")
    elif args.cmd == "analyze":
        result = json.dumps(analysis.analyze(stimulus, read_capture(args.capture, fs), manifest), indent=1)
        args.out.write_text(result) if args.out else print(result)
    else:
        ok = compare.run(stimulus, manifest, read_capture(args.reference, fs), read_capture(args.model, fs), args.out)
        sys.exit(0 if ok else 1)
