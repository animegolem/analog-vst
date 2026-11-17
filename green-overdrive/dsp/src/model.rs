//! Green Diode Overdrive circuit; see reference/circuit.cir for sources and assumptions.
use crate::control::{ControlSpec, Taper};
use crate::netlist::{BjtModel, Netlist, D1N4148, JRC4558};
pub const NAME: &str = "Green Diode Overdrive";
pub const VENDOR: &str = "Analog to VST";
pub const CLAP_ID: &str = "local.analog-to-vst.green-diode-overdrive";
pub const VST3_ID: [u8; 16] = *b"GreenDiodeOver01";
pub const VOLTS_PER_FULL_SCALE: f64 = 0.2;
pub const CONTROLS: &[ControlSpec] = &[
    ControlSpec { name: "Drive", taper: Taper::Audio { max: 500e3 }, default: 0.5 },
    ControlSpec { name: "Tone", taper: Taper::Linear { min: 0.0, max: 1.0 }, default: 0.5 },
    ControlSpec { name: "Level", taper: Taper::Audio { max: 1.0 }, default: 0.5 },
];
const Q1815: BjtModel = BjtModel { is: 40e-15, bf: 170.0, br: 3.6, vaf: 100.0, ikf: 0.25, rb: 50.0, rc: 0.76, re: 0.001 };
pub fn circuit(n: &mut Netlist) {
    n.supply("vcc", 9.0);
    n.resistor("vcc", "vref", 10e3);
    n.resistor("vref", "0", 10e3);
    n.capacitor("vref", "0", 47e-6);
    n.capacitor("vcc", "0", 100e-6);
    n.input("in");
    n.resistor("in", "i1", 1e3);
    n.capacitor("i1", "ib", 20e-9);
    n.resistor("ib", "vref", 510e3);
    n.npn("vcc", "ib", "ie", Q1815);
    n.resistor("ie", "0", 10e3);
    n.capacitor("ie", "ap", 1e-6);
    n.resistor("ap", "vref", 10e3);
    n.opamp("ap", "am", "ao", JRC4558);
    n.resistor("am", "g", 4.7e3);
    n.capacitor("g", "0", 47e-9);
    n.resistor("am", "dr", 51e3);
    n.pot("RDRIVE", "dr", "ao");
    // Antiparallel 1N914 pair as one device (same physics as D1/D2 in the netlist); two separate
    // diodes are so tightly coupled that the solver falls back to one joint solve for every device.
    n.diode_pair("am", "ao", D1N4148);
    n.capacitor("am", "ao", 51e-12);
    n.resistor("ao", "tp", 1e3);
    n.capacitor("tp", "0", 220e-9);
    n.resistor("tp", "vref", 10e3);
    n.opamp("tp", "tm", "to", JRC4558);
    n.resistor("to", "tm", 1e3);
    n.pot("RTONE1", "tp", "tw");
    n.pot("RTONE2", "tw", "tm");
    n.resistor("tw", "tc", 220.0);
    n.capacitor("tc", "0", 220e-9);
    n.capacitor("to", "lv1", 1e-6);
    n.resistor("lv1", "lvtop", 1e3);
    n.pot("RLEVEL1", "lvtop", "lv");
    n.pot("RLEVEL2", "lv", "0");
    n.capacitor("lv", "ob", 100e-9);
    n.resistor("ob", "vref", 510e3);
    n.npn("vcc", "ob", "oe", Q1815);
    n.resistor("oe", "0", 10e3);
    n.resistor("oe", "oc", 100.0);
    n.capacitor("oc", "out", 10e-6);
    n.resistor("out", "0", 10e3);
    n.resistor("out", "0", 1e6);
    n.output("out");
}
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    let (drive, tone, level) = (plain[0], plain[1], plain[2]);
    set("RDRIVE", drive);
    set("RTONE1", 20e3 * tone);
    set("RTONE2", 20e3 * (1.0-tone));
    set("RLEVEL1", 100e3 * (1.0-level));
    set("RLEVEL2", 100e3 * level);
}
