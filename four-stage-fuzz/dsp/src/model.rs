//! ElectroSmash V3 signal path; see reference/circuit.cir for sources and assumptions.
use crate::control::{ControlSpec, Taper};
use crate::netlist::{BjtModel, Netlist, D1N4148};
pub const NAME: &str = "Four Stage Sustainer";
pub const VENDOR: &str = "Analog to VST";
pub const CLAP_ID: &str = "local.analog-to-vst.four-stage-sustainer";
pub const VST3_ID: [u8; 16] = *b"FourStageSusta01";
pub const VOLTS_PER_FULL_SCALE: f64 = 0.2;
pub const CONTROLS: &[ControlSpec] = &[
    ControlSpec { name: "Sustain", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "Tone", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "Volume", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
];
const Q: BjtModel = BjtModel { is: 5.911e-15, bf: 1122.0, br: 1.271, vaf: 62.37,
    ikf: 14.92e-3, rb: 10.0, rc: 1.61, re: 0.0 };
pub fn circuit(n: &mut Netlist) {
    n.supply("vcc", 9.0); n.input("in");
    n.resistor("in", "jack", 1e3);
    n.resistor("jack", "i1", 39e3); n.capacitor("i1", "b4", 1e-6);
    n.resistor("b4", "0", 47e3); n.resistor("b4", "c4", 470e3);
    n.capacitor("b4", "c4", 470e-12); n.resistor("vcc", "c4", 10e3);
    n.resistor("e4", "0", 100.0); n.npn("c4", "b4", "e4", Q);
    n.capacitor("c4", "sustop", 1e-6);
    n.pot("RSUSTOP", "sustop", "sus"); n.pot("RSUSBOT", "sus", "susbot");
    n.resistor("susbot", "0", 1e3); n.capacitor("sus", "s3", 100e-9);
    n.resistor("s3", "b3", 10e3); n.resistor("b3", "0", 100e3);
    n.resistor("b3", "c3", 470e3); n.capacitor("b3", "c3", 470e-12);
    n.capacitor("b3", "d3", 1e-6); n.diode_pair("d3", "c3", D1N4148);
    n.resistor("vcc", "c3", 10e3); n.resistor("e3", "0", 150.0); n.npn("c3", "b3", "e3", Q);
    n.capacitor("c3", "s2", 100e-9); n.resistor("s2", "b2", 10e3);
    n.resistor("b2", "0", 100e3); n.resistor("b2", "c2", 470e3);
    n.capacitor("b2", "c2", 470e-12); n.capacitor("b2", "d2", 1e-6);
    n.diode_pair("d2", "c2", D1N4148); n.resistor("vcc", "c2", 10e3);
    n.resistor("e2", "0", 150.0); n.npn("c2", "b2", "e2", Q);
    n.capacitor("c2", "hi", 4e-9); n.resistor("hi", "0", 22e3);
    n.resistor("c2", "lo", 39e3); n.capacitor("lo", "0", 10e-9);
    n.pot("RTONEHI", "hi", "tone"); n.pot("RTONELO", "tone", "lo");
    n.capacitor("tone", "b1", 100e-9); n.resistor("vcc", "b1", 430e3);
    n.resistor("b1", "0", 100e3); n.resistor("vcc", "c1", 15e3);
    n.resistor("e1", "0", 3.3e3); n.npn("c1", "b1", "e1", Q);
    n.capacitor("c1", "voltop", 100e-9);
    n.pot("RVOLTOP", "voltop", "out"); n.pot("RVOLBOT", "out", "0");
    n.resistor("out", "0", 1e6); n.output("out");
}
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    let (s,t,v) = (plain[0],plain[1],plain[2]);
    set("RSUSTOP", 100e3*(1.0-s)); set("RSUSBOT", 100e3*s);
    set("RTONEHI", 100e3*(1.0-t)); set("RTONELO", 100e3*t);
    set("RVOLTOP", 100e3*(1.0-v)); set("RVOLBOT", 100e3*v);
}
