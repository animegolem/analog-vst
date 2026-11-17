//! Reduced MSK 011 one-channel model. See reference/circuit.cir and README.
use crate::control::{ControlSpec, Taper};
use crate::netlist::{BjtModel, Netlist};

pub const NAME: &str = "Class A Transistor Drive";
pub const VENDOR: &str = "Analog to VST";
pub const CLAP_ID: &str = "local.analog-to-vst.class-a-transistor-drive";
pub const VST3_ID: [u8; 16] = *b"ClassATransist01";
pub const VOLTS_PER_FULL_SCALE: f64 = 5.0;
pub const CONTROLS: &[ControlSpec] = &[ControlSpec {
    name: "Level", taper: Taper::Audio { max: 1.0 }, default: 0.5,
}];

// Fairchild/onsemi 2N5088: RE is unspecified in the published model (SPICE default 0).
const Q: BjtModel = BjtModel { is: 5.911e-15, bf: 1122.0, br: 1.271, vaf: 62.37,
    ikf: 14.92e-3, rb: 10.0, rc: 1.61, re: 0.0 };

pub fn circuit(n: &mut Netlist) {
    n.supply("vp", 12.0);
    n.supply("vn", -12.0);
    n.input("in");
    n.resistor("in", "jack2", 1e3);
    // Channels 1 and 4 are unpatched; their knobs are off (1-ohm end stop).
    n.resistor("vp", "jack1", 100e3);
    n.resistor("jack1", "b1", 99999.0);
    n.resistor("b1", "0", 1.0);
    n.pot("P2TOP", "jack2", "b2");
    n.pot("P2BOT", "b2", "0");
    n.resistor("b3", "0", 99999.0);
    n.resistor("b3", "0", 1.0);
    n.resistor("vn", "jack4", 100e3);
    n.resistor("jack4", "b4", 99999.0);
    n.resistor("b4", "0", 1.0);
    for (b,e) in [("b1","e1"),("b2","e2"),("b3","e3"),("b4","e4")] {
        n.npn("vp", b, e, Q);
        n.resistor(e, "vn", 22e3);
        n.resistor(e, "sum", 47e3);
    }
    n.resistor("vp", "trim", 35009.67484);
    n.resistor("trim", "vn", 64990.32516);
    n.resistor("trim", "sum", 330e3);
    n.resistor("sum", "vn", 2.7e3);
    n.npn("c5", "sum", "e5", Q);
    n.resistor("vp", "c5", 5.9e3);
    n.resistor("e5", "vn", 510.0);
    n.resistor("c5", "b6", 22e3);
    n.resistor("b6", "vn", 12e3);
    n.npn("dc", "b6", "e6", Q);
    n.resistor("vp", "dc", 4.7e3);
    n.resistor("e6", "vn", 510.0);
    n.capacitor("dc", "out", 4.7e-6);
    n.resistor("out", "0", 330e3);
    n.resistor("out", "0", 100e3);
    n.output("out");
}
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    set("P2TOP", 100e3 * (1.0 - plain[0]));
    set("P2BOT", 100e3 * plain[0]);
}
