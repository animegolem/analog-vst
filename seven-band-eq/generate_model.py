from pathlib import Path
p=Path(__file__).parent
names=['B63','B160','B410','B1K','B2K5','B7K7','B16K']
# Tuned components: (parallel caps, series cap, feedback resistor)
bands=[([470e-9,470e-9,330e-9],22e-9,130e3),([470e-9,33e-9],10e-9,110e3),([100e-9,100e-9],4.7e-9,91e3),([47e-9,33e-9],2.2e-9,82e3),([33e-9],1e-9,68e3),([10e-9,2.2e-9],470e-12,62e3),([4.7e-9],220e-12,51e3)]
r=['//! Independent component transcription; see reference/circuit.cir and README.','use crate::control::{ControlSpec,Taper};','use crate::netlist::{Netlist,OpAmp};','pub const NAME: &str = "Seven Band Graphic";','pub const VENDOR: &str = "Analog to VST";','pub const CLAP_ID: &str = "local.analog-to-vst.seven-band-graphic";','pub const VST3_ID: [u8; 16] = *b"SevenBandGraph01";','pub const VOLTS_PER_FULL_SCALE: f64 = 0.5;','const AMP: OpAmp = OpAmp { a0: 100000.0, gbw: 10000000.0 };','pub const CONTROLS: &[ControlSpec] = &[']
r += [f'    ControlSpec {{ name: "{n}", taper: Taper::Linear {{ min: 0.0, max: 1.0 }}, default: 0.5 }},' for n in names]
r += ['];','pub fn circuit(n: &mut Netlist) {','    n.input("in");','    n.resistor("in", "im", 10000.0);','    n.resistor("im", "io", 10000.0);','    n.opamp("0", "im", "io", AMP);','    n.resistor("io", "plus", 2700.0);','    n.opamp("plus", "minus", "eo", AMP);','    n.resistor("minus", "eo", 2700.0);','    n.resistor("eo", "out", 470.0);','    n.resistor("out", "0", 1000000.0);']
s=['* Seven-band graphic EQ: Rev 1 from Tom Whitwell / Music Thing Modular.', '* Source: https://musicthing.co.uk/collateral/graphic_schematic.pdf', '* BOM: https://github.com/TomWhitwell/Graphic (Rev 1 workbook).', '* Faders: Alpha RA2045F-20R-15LC-B10K-C, 10k linear (B taper), centre detent. The schematic and the', '* Rev 1 workbook give no value; the part number is from the Thonk kit parts list (Graphic EQ BOM sheet).', '* NE5532 approximated single-pole A0=100000 GBW=10MHz; no clipping/slew/noise.', '* Ideal source; 1Meg external load; LEDs and supply decoupling omitted.', '* 0dBFS = 0.5V peak pending rail check.', '.subckt opamp inp inn out','* A0 = gm*Rp = 1e5, pole = 1/(2*pi*Rp*Cp) = 100 Hz: the same op-amp as the template\'s','* gm=100, Rp=1k form. With gm=100, ngspice 45 stops with "Timestep too small" 8 ms into the','* 0 dBFS sweep of this circuit (nine op-amps); with these values the same section completes.','G1 0 p inp inn 1e-3','Rp p 0 1e8','Cp p 0 15.91549430919p','E1 out 0 p 0 1','.ends','$SOURCE','R25 in im 10k','R26 im io 10k','XIN 0 im io opamp','R1 io plus 2.7k','XOUT plus minus eo opamp','R2 minus eo 2.7k','R3 eo out 470','RL out 0 1meg']
for i,(name,(caps,cs,rg)) in enumerate(zip(names,bands)):
 w=f'w{i}'; a=f'a{i}'; b=f'b{i}'; o=f'o{i}'
 r += [f'    n.pot("P{i}T", "minus", "{w}");',f'    n.pot("P{i}B", "{w}", "plus");']
 s += [f'RP{i}T minus {w} {{max(10000*(1-${name}),1)}}',f'RP{i}B {w} plus {{max(10000*${name},1)}}']
 for j,c in enumerate(caps):
  r += [f'    n.capacitor("{w}", "{a}", {c});']
  s += [f'C{i}_{j} {w} {a} {c}']
 r += [f'    n.capacitor("{a}", "{b}", {cs});',f'    n.resistor("{b}", "0", {rg});',f'    n.opamp("{b}", "{o}", "{o}", AMP);',f'    n.resistor("{a}", "{o}", 1800.0);']
 s += [f'CS{i} {a} {b} {cs}',f'RG{i} {b} 0 {rg}',f'XF{i} {b} {o} {o} opamp',f'RF{i} {a} {o} 1800']
r += ['    n.output("out");','}','pub fn pots(plain: &[f64], set: &mut dyn FnMut(&\'static str, f64)) {']
for i in range(7): r += [f'    set("P{i}T", 10000.0*(1.0-plain[{i}]));',f'    set("P{i}B", 10000.0*plain[{i}]);']
r += ['}']
s += ['$ANALYSIS','.end']
(p/'dsp/src/model.rs').write_text('\n'.join(r)+'\n')
(p/'reference/circuit.cir').write_text('\n'.join(s)+'\n')
settings=['# Every control covered at 0 and 1.','default']
for label,arr in [('allmax',[1]*7),('allmin',[0]*7),('alternating',[1,0,1,0,1,0,1]),('inverse',[0,1,0,1,0,1,0])]:
 settings.append(label+' '+' '.join(f'{n.lower()}={v}' for n,v in zip(names,arr)))
(p/'settings.txt').write_text('\n'.join(settings)+'\n')
