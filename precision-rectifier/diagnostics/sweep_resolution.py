"""One sweep through the Netlist at several reference oversampling factors and through several
builds of the Model, compared pairwise with the harness's own frequency-response measure.

Shows what each side's sampling rate contributes to the frequency-response Check. Alignment is
forced to the delay and polarity the harness finds on the full Pack for this Circuit.

Usage, from harness/:
  uv run python ../diagnostics/sweep_resolution.py .. <work dir> <seconds> <level dBFS> \
      <oversampling factors, e.g. 4,16 or none> <label=path to a render binary> ...
Build a Model at another circuit rate by copying the Project elsewhere, changing
MIN_CIRCUIT_RATE in dsp/src/oversample.rs and building `render` there.
"""

import itertools
import subprocess
import sys
import time
from pathlib import Path
from string import Template

import numpy as np
import soundfile as sf

from analog_harness import analysis, signals, spice

DELAY, POLARITY = -1446, -1

proj, work = Path(sys.argv[1]), Path(sys.argv[2])
secs, level = float(sys.argv[3]), float(sys.argv[4])
factors = [int(v) for v in sys.argv[5].split(",") if v != "none"]
binaries = dict(arg.split("=", 1) for arg in sys.argv[6:])
work.mkdir(parents=True, exist_ok=True)
fs = 96_000
volts = float(
    subprocess.run([str(proj / "target/release/render"), "--plain"], check=True, capture_output=True, text=True)
    .stdout.split("_VOLTS_PER_FULL_SCALE=")[1]
    .split()[0]
)
template = Template((proj / "reference/circuit.cir").read_text())
gap = int(0.5 * fs)
f1, f2 = 10.0, 0.45 * fs
sweep = signals.db(level) * signals.sweep(fs, f1, f2, secs)
stim = np.concatenate([np.zeros(gap), sweep, np.zeros(gap)]).astype(np.float32).astype(np.float64)
sec = {"kind": "sweep", "start": gap, "length": len(sweep), "level_db": level, "f1": f1, "f2": f2}
x = stim[gap:]

caps = {}
for factor in factors:
    spice.OVERSAMPLE = factor
    start = time.time()
    y = spice.run_section(template, {}, spice.upsample(x * volts, fs), fs * factor, work, f"res{factor}")
    caps[f"netlist x{factor}"] = spice.downsample(y, fs) / volts
    print(f"ngspice at {factor}x: {time.time() - start:.1f} s", flush=True)
for label, binary in binaries.items():
    sf.write(work / "res_in.wav", x, fs, subtype="FLOAT")
    subprocess.run([binary, str(work / "res_in.wav"), str(work / "res_out.wav")], check=True)
    caps[f"model {label}"] = sf.read(work / "res_out.wav", dtype="float64")[0]


def response(y):
    full = np.concatenate([np.zeros(gap), y.astype(np.float32).astype(np.float64)])
    aligned = np.concatenate([np.zeros(-DELAY), full])[: len(stim)] * POLARITY
    return analysis.frequency_response(stim, aligned, sec, fs)


measured = {k: response(v) for k, v in caps.items()}
freqs = np.array(next(iter(measured.values()))["freqs"])
for a, b in itertools.combinations(measured, 2):
    d = np.subtract(measured[a]["db"], measured[b]["db"])
    i = int(np.argmax(np.abs(d)))
    null = 20 * np.log10(np.std(caps[a] - caps[b]) / np.std(caps[b]))
    print(f"{a:>14} vs {b:<14}: worst {abs(d[i]):.4f} dB at {freqs[i]:6.0f} Hz; waveform null {null:.1f} dB")
