//! Independent component transcription; see reference/circuit.cir and README.
use crate::control::{ControlSpec,Taper};
use crate::netlist::{Netlist,OpAmp};
pub const NAME: &str = "Seven Band Graphic";
pub const VENDOR: &str = "Analog to VST";
pub const CLAP_ID: &str = "local.analog-to-vst.seven-band-graphic";
pub const VST3_ID: [u8; 16] = *b"SevenBandGraph01";
pub const VOLTS_PER_FULL_SCALE: f64 = 0.5;
const AMP: OpAmp = OpAmp { a0: 100000.0, gbw: 10000000.0 };
pub const CONTROLS: &[ControlSpec] = &[
    ControlSpec { name: "B63", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "B160", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "B410", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "B1K", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "B2K5", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "B7K7", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "B16K", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
];
pub fn circuit(n: &mut Netlist) {
    n.input("in");
    n.resistor("in", "im", 10000.0);
    n.resistor("im", "io", 10000.0);
    n.opamp("0", "im", "io", AMP);
    n.resistor("io", "plus", 2700.0);
    n.opamp("plus", "minus", "eo", AMP);
    n.resistor("minus", "eo", 2700.0);
    n.resistor("eo", "out", 470.0);
    n.resistor("out", "0", 1000000.0);
    n.pot("P0T", "minus", "w0");
    n.pot("P0B", "w0", "plus");
    n.capacitor("w0", "a0", 4.7e-07);
    n.capacitor("w0", "a0", 4.7e-07);
    n.capacitor("w0", "a0", 3.3e-07);
    n.capacitor("a0", "b0", 2.2e-08);
    n.resistor("b0", "0", 130000.0);
    n.opamp("b0", "o0", "o0", AMP);
    n.resistor("a0", "o0", 1800.0);
    n.pot("P1T", "minus", "w1");
    n.pot("P1B", "w1", "plus");
    n.capacitor("w1", "a1", 4.7e-07);
    n.capacitor("w1", "a1", 3.3e-08);
    n.capacitor("a1", "b1", 1e-08);
    n.resistor("b1", "0", 110000.0);
    n.opamp("b1", "o1", "o1", AMP);
    n.resistor("a1", "o1", 1800.0);
    n.pot("P2T", "minus", "w2");
    n.pot("P2B", "w2", "plus");
    n.capacitor("w2", "a2", 1e-07);
    n.capacitor("w2", "a2", 1e-07);
    n.capacitor("a2", "b2", 4.7e-09);
    n.resistor("b2", "0", 91000.0);
    n.opamp("b2", "o2", "o2", AMP);
    n.resistor("a2", "o2", 1800.0);
    n.pot("P3T", "minus", "w3");
    n.pot("P3B", "w3", "plus");
    n.capacitor("w3", "a3", 4.7e-08);
    n.capacitor("w3", "a3", 3.3e-08);
    n.capacitor("a3", "b3", 2.2e-09);
    n.resistor("b3", "0", 82000.0);
    n.opamp("b3", "o3", "o3", AMP);
    n.resistor("a3", "o3", 1800.0);
    n.pot("P4T", "minus", "w4");
    n.pot("P4B", "w4", "plus");
    n.capacitor("w4", "a4", 3.3e-08);
    n.capacitor("a4", "b4", 1e-09);
    n.resistor("b4", "0", 68000.0);
    n.opamp("b4", "o4", "o4", AMP);
    n.resistor("a4", "o4", 1800.0);
    n.pot("P5T", "minus", "w5");
    n.pot("P5B", "w5", "plus");
    n.capacitor("w5", "a5", 1e-08);
    n.capacitor("w5", "a5", 2.2e-09);
    n.capacitor("a5", "b5", 4.7e-10);
    n.resistor("b5", "0", 62000.0);
    n.opamp("b5", "o5", "o5", AMP);
    n.resistor("a5", "o5", 1800.0);
    n.pot("P6T", "minus", "w6");
    n.pot("P6B", "w6", "plus");
    n.capacitor("w6", "a6", 4.7e-09);
    n.capacitor("a6", "b6", 2.2e-10);
    n.resistor("b6", "0", 51000.0);
    n.opamp("b6", "o6", "o6", AMP);
    n.resistor("a6", "o6", 1800.0);
    n.output("out");
}
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    set("P0T", 10000.0*(1.0-plain[0]));
    set("P0B", 10000.0*plain[0]);
    set("P1T", 10000.0*(1.0-plain[1]));
    set("P1B", 10000.0*plain[1]);
    set("P2T", 10000.0*(1.0-plain[2]));
    set("P2B", 10000.0*plain[2]);
    set("P3T", 10000.0*(1.0-plain[3]));
    set("P3B", 10000.0*plain[3]);
    set("P4T", 10000.0*(1.0-plain[4]));
    set("P4B", 10000.0*plain[4]);
    set("P5T", 10000.0*(1.0-plain[5]));
    set("P5B", 10000.0*plain[5]);
    set("P6T", 10000.0*(1.0-plain[6]));
    set("P6B", 10000.0*plain[6]);
}
