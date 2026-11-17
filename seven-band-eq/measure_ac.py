import subprocess, tempfile
from pathlib import Path
import numpy as np
p=Path(__file__).parent
base=(p/'reference/circuit.cir').read_text()
names=['B63','B160','B410','B1K','B2K5','B7K7','B16K']
# Points per decade of the AC sweep. Peak frequencies are read off this grid, so they are good to one
# grid step: 10**(1/PPD)-1 = 0.12% at 2000. At 200 the 160 Hz and 2.5 kHz peaks read 154.6 and 2539 Hz.
PPD=2000
nodes=['io','eo']+[f'o{i}' for i in range(7)]+['out']
def sim(vals):
 text=base.replace('$SOURCE','VIN in 0 AC 0.5')
 for n,v in zip(names,vals):text=text.replace('$'+n,str(v))
 with tempfile.TemporaryDirectory(dir=p) as t:
  f=Path(t)/'ac.dat'; cir=Path(t)/'measure.cir'
  block=f'.control\nset noaskquit\nset wr_singlescale\nset wr_vecnames\nset numdgt=12\nac dec {PPD} 5 100000\nwrdata '+str(f)+' '+' '.join('vm('+n+')' for n in nodes)+'\nquit\n.endc'
  text=text.replace('$ANALYSIS',block);cir.write_text(text)
  run=subprocess.run(['ngspice','-b',str(cir)],capture_output=True,text=True)
  if not f.exists(): raise RuntimeError(run.stdout+run.stderr)
  return np.loadtxt(f,skiprows=1)
ref=sim([.5]*7); print('allmax max opamp amplitude V:')
a=sim([1]*7)
for j,n in enumerate(nodes[:-1],1):
 k=np.argmax(a[:,j]); print(n,round(a[k,j],5),'at',round(a[k,0],2),'Hz')
print('individual bands vs flat:')
for i,name in enumerate(names):
 vals=[.5]*7;vals[i]=1;b=sim(vals);vals[i]=0;c=sim(vals)
 dbb=20*np.log10(b[:,-1]/ref[:,-1]);dbc=20*np.log10(c[:,-1]/ref[:,-1]); k=np.argmax(dbb);m=np.argmin(dbc)
 print(name,'boost',round(dbb[k],2),'dB @',round(b[k,0],1),'Hz; cut',round(dbc[m],2),'dB @',round(c[m,0],1),'Hz')
