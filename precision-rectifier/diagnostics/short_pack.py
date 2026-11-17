"""A hand-made short signal set (not the harness Pack) through the harness's own section runner and
compare, each section from a fresh DC state. It shows how the Checks behave on a Circuit that
removes the fundamental. It proves nothing about the Project: only a Full Validation does.

Usage, from harness/ (about 3 minutes, one ngspice at a time):
  uv run python ../diagnostics/short_pack.py .. <work dir>
"""

import json
import subprocess
import sys
import time
from pathlib import Path
from string import Template

import numpy as np
import soundfile as sf

from analog_harness import analysis, compare, signals, spice

proj, work = Path(sys.argv[1]), Path(sys.argv[2])
work.mkdir(parents=True, exist_ok=True)
fs, V = 96_000, 5.0
tmpl = Template((proj / "reference/circuit.cir").read_text())
gap = int(0.5 * fs)
parts, sections, pos = [np.zeros(gap)], [], gap


def add(kind, sig, **p):
    global pos
    parts.extend([sig, np.zeros(gap)])
    sections.append({"kind": kind, "start": pos, "length": len(sig), **p})
    pos += len(sig) + gap


db = signals.db
add("sync", db(-12) * signals.sweep(fs, 100, 10_000, 0.1))
f1, f2 = 10.0, 0.45 * fs
for lv in (-40, 0):
    add("sweep", db(lv) * signals.sweep(fs, f1, f2, 1.0), level_db=lv, f1=f1, f2=f2)
for f, lv in ((100, -40), (1000, -40), (1000, 0), (5000, -40), (5000, 0)):
    add("step", signals.tone(fs, f, 0.35, lv), freq=f, level_db=lv)
segs = (0.3, 0.3, 0.4)
add(
    "burst",
    np.concatenate([signals.tone(fs, 1000, s, l) for s, l in zip(segs, (-30, -6, -30))]),
    freq=1000,
    levels_db=[-30, -6, -30],
    segments_s=list(segs),
)
x = 0.8 * signals.tone(fs, 60, 0.5, 0) + 0.2 * signals.tone(fs, 7000, 0.5, 0)
add("imd", db(-6) * x / np.max(np.abs(x)), freqs=[60, 7000], level_db=-6)
stim = np.concatenate(parts).astype(np.float32).astype(np.float64)
man = {"version": 1, "sample_rate": fs, "length": len(stim), "sections": sections}
r, m = np.zeros_like(stim), np.zeros_like(stim)
for i, sec in enumerate(sections):
    a, b = sec["start"], min(sec["start"] + sec["length"] + gap, len(stim))
    t = time.time()
    up = spice.upsample(stim[a:b] * V, fs)
    y = spice.run_section(tmpl, {}, up, fs * spice.OVERSAMPLE, work, f"m{i}")
    r[a:b] = spice.downsample(y, fs) / V
    src, dst = work / "mi.wav", work / "mo.wav"
    sf.write(src, stim[a:b], fs, subtype="FLOAT")
    subprocess.run([str(proj / "target/release/render"), str(src), str(dst)], check=True)
    m[a:b] = sf.read(dst, dtype="float64")[0]
    print(f"section {i} {sec['kind']}: ngspice+model {time.time() - t:.1f} s", flush=True)
r, m = r.astype(np.float32).astype(np.float64), m.astype(np.float32).astype(np.float64)
np.save(work / "mini_stim.npy", stim)
np.save(work / "mini_ref.npy", r)
np.save(work / "mini_model.npy", m)
(work / "mini.json").write_text(json.dumps(man))
ok = compare.run(stim, man, r, m, work / "mini_report", analysis.KNOWN)
rep = json.loads((work / "mini_report/report.json").read_text())
print(
    "passed",
    ok,
    "delays",
    rep["reference"]["delay_samples"],
    rep["model"]["delay_samples"],
    "polarity",
    rep["reference"]["polarity"],
    rep["model"]["polarity"],
)
for s in rep["reference"]["steps"]:
    print(
        "ref step",
        s["freq"],
        s["level_db"],
        round(s["gain_db"], 3),
        {k: round(v, 2) for k, v in s["harmonics_dbc"].items()},
    )
for sw_r, sw_d in zip(rep["reference"]["sweeps"], rep["model"]["sweeps"]):
    d = np.subtract(sw_d["db"], sw_r["db"])
    fr = np.array(sw_r["freqs"])
    i = int(np.argmax(np.abs(d)))
    print(
        "sweep",
        sw_r["level_db"],
        "ref dB at 20/100/1k/10k/20k:",
        [round(float(np.interp(f, fr, sw_r["db"])), 2) for f in (20, 100, 1000, 10000, 20000)],
        "worst diff",
        round(float(d[i]), 4),
        "at",
        round(float(fr[i]), 1),
        "Hz",
    )
print(
    "bursts ref",
    rep["reference"]["bursts"][0]["attack_s"],
    rep["reference"]["bursts"][0]["release_s"],
    "imd ref",
    rep["reference"]["imd"],
    "model",
    rep["model"]["imd"],
)
