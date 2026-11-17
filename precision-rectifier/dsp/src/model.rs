//! The circuit being modeled: a precision full-wave rectifier (two op-amp sections, two diodes),
//! `out = |in|`, DC-coupled. Edit this file and `reference/circuit.cir` together: same parts,
//! same values. Sources and assumptions are in the netlist header and the README.

use crate::control::ControlSpec;
use crate::netlist::{DiodeModel, Netlist, OpAmp};

pub const NAME: &str = "Precision Rectifier";
pub const VENDOR: &str = "Analog to VST";
/// Reverse-domain id, unique per plugin.
pub const CLAP_ID: &str = "local.analog-to-vst.precision-rectifier";
/// Exactly 16 bytes, unique per plugin.
pub const VST3_ID: [u8; 16] = *b"PrecisionRecti01";

/// Volts at the jacks for a full-scale (0 dBFS) sample. Assumption: 5 V peak. At full scale the
/// op-amp outputs stay between -2.98 V and +5.00 V, inside the hardware's 12 V rails, which matters
/// because op-amp rail clipping isn't modeled.
pub const VOLTS_PER_FULL_SCALE: f64 = 5.0;

/// The circuit has no pot, trimmer or switch.
pub const CONTROLS: &[ControlSpec] = &[];

/// TL074 section: 200 V/mV typical open-loop gain, 3 MHz gain-bandwidth (open value, see README).
const TL074: OpAmp = OpAmp { a0: 2e5, gbw: 3e6 };

/// The netlist's `D4148`: the template's 1N4148 set with M = 0 and TT = 0, so SPICE's junction
/// capacitance is the same fixed `cjo` the solver uses.
const D4148: DiodeModel = DiodeModel {
    is: 2.52e-9,
    n: 1.752,
    rs: 0.568,
    cjo: 4e-12,
};

pub fn circuit(n: &mut Netlist) {
    n.input("in");
    // Inverting half-wave stage: x = -in/2 for in > 0, x = 0 for in < 0.
    n.resistor("in", "a1n", 200e3); // R2
    n.opamp("0", "a1n", "a1o", TL074); // XU1A
    n.diode("a1o", "a1n", D4148); // D3
    n.diode("x", "a1o", D4148); // D4
    n.resistor("a1n", "x", 100e3); // R3

    // Load of the omitted half-wave output stage.
    n.resistor("x", "0", 100e3); // R4

    // Summing stage: c1o = -(in + 4x) = |in|.
    n.resistor("x", "c1n", 100e3); // R8
    n.resistor("x", "c1n", 100e3); // R5
    n.resistor("in", "c1n", 200e3); // R9
    n.resistor("c1n", "c1o", 200e3); // R7
    n.opamp("0", "c1n", "c1o", TL074); // XU1C
    n.resistor("c1o", "out", 180.0); // R13
    n.resistor("out", "0", 100e3); // RLOAD
    n.output("out");
}

/// No controls, so no pots to set.
pub fn pots(plain: &[f64], _set: &mut dyn FnMut(&'static str, f64)) {
    assert!(plain.is_empty(), "this circuit has no controls");
}
