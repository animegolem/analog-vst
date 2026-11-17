"""Compare the full vendor model against this project's deliberately reduced reference.

Runs ngspice operating-point and 20 kHz small-signal analyses at the calibrated
reduced-model trim. Does not change either reference/circuit.cir or the plugin.
"""
from pathlib import Path
import re
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
net = (root / 'reference/circuit.cir').read_text()
full = ('.model Q5088 NPN(IS=5.911f XTI=3 EG=1.11 VAF=62.37 BF=1122 NE=1.394 '
        'ISE=5.911f IKF=14.92m XTB=1.5 BR=1.271 NC=2 ISC=0 IKR=0 RC=1.61 '
        'CJC=4.017p MJC=.3174 VJC=.75 FC=.5 CJE=4.973p MJE=.4146 VJE=.75 '
        'TR=4.673n TF=821.7p ITF=.35 VTF=4 XTF=7 RB=10)')
level = sys.argv[1] if len(sys.argv) > 1 else '0'
for name in ['reduced', 'leakage_only', 'charge_only', 'full']:
    if name == 'reduced':
        model = None
    elif name == 'full':
        model = full
    elif name == 'leakage_only':
        model = re.sub(r'^\.model Q5088.*$', lambda m: m.group(0)[:-1] + ' ISE=5.911f NE=1.394 ISC=0 NC=2)', net, flags=re.M).splitlines()
        model = next(s for s in model if s.startswith('.model Q5088'))
    else:
        model = re.sub(r'^\.model Q5088.*$', lambda m: m.group(0)[:-1] + ' CJC=4.017p MJC=.3174 VJC=.75 FC=.5 CJE=4.973p MJE=.4146 VJE=.75 TR=4.673n TF=821.7p ITF=.35 VTF=4 XTF=7)', net, flags=re.M).splitlines()
        model = next(s for s in model if s.startswith('.model Q5088'))
    text = net if model is None else re.sub(r'^\.model Q5088.*$', model, net, flags=re.M)
    text = text.replace('$LEVEL', level).replace('$SOURCE', 'VIN in 0 DC 0 AC 1')
    text = text.replace('$ANALYSIS', '.control\nset noaskquit\nop\nprint v(b1) v(e1) v(sum) v(e5) v(c5) v(b6) v(e6) v(dc)\nac lin 1 20k 20k\nprint vdb(out) vp(out)\nquit\n.endc')
    with tempfile.NamedTemporaryFile(mode='w', suffix='.cir', dir=root / 'reference') as f:
        f.write(text); f.flush()
        p = subprocess.run(['ngspice','-b',f.name], text=True, capture_output=True, check=True)
    print('---', name, '---')
    print('\n'.join(line for line in p.stdout.splitlines() if ' = ' in line))
    print('\n'.join(line for line in p.stdout.splitlines() if re.match(r'^\s*0\s+2\.000000e\+04',line)))
