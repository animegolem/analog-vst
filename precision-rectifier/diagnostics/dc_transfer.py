"""DC transfer: the Netlist by ngspice .op against the Model with the input held (sample taken
30 ms in).

Usage, from harness/:  uv run python ../diagnostics/dc_transfer.py .. <work dir>
"""

import re
import subprocess
import sys
from pathlib import Path
from string import Template

import numpy as np
import soundfile as sf

proj, work = Path(sys.argv[1]), Path(sys.argv[2])
work.mkdir(parents=True, exist_ok=True)
fs, V = 96000, 5.0
vals = [0.0, 0.002, -0.002, 0.05, -0.05, 1.0, -1.0, 5.0, -5.0]
t = Template((proj / "reference/circuit.cir").read_text())
ref = {}
for v in vals:
    cir = work / "dcop.cir"
    cir.write_text(
        t.substitute(
            SOURCE=f"VIN in 0 DC {v}",
            ANALYSIS=".control\nset noaskquit\nset numdgt=10\nop\nprint v(out) v(x) v(a1o) v(c1o)\nquit\n.endc",
        )
    )
    out = subprocess.run(
        ["ngspice", "-b", str(cir)], check=False, capture_output=True, text=True, stdin=subprocess.DEVNULL
    ).stdout
    ref[v] = {m.group(1): float(m.group(2)) for m in re.finditer(r"v\((\w+)\) = (\S+)", out)}
for v in vals:
    sf.write(work / "d_in.wav", np.full(int(0.05 * fs), v / V, dtype=np.float32), fs, subtype="FLOAT")
    subprocess.run([str(proj / "target/release/render"), str(work / "d_in.wav"), str(work / "d_out.wav")], check=True)
    m = sf.read(work / "d_out.wav", dtype="float64")[0][int(0.03 * fs)] * V
    r = ref[v]["out"]
    print(
        f"in {v:+.3f} V: ngspice out {r:.7f} V (x {ref[v]['x']:+.6f}, a1o {ref[v]['a1o']:+.6f}, c1o {ref[v]['c1o']:+.6f}) | model out {m:.7f} V | diff {(m - r) * 1e6:+.2f} uV"
    )
