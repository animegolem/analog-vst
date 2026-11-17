//! Three-Band Stack Boost: unity buffer, passive bass/mid/treble stack, non-inverting gain stage.
//! Part for part with `reference/circuit.cir`; its header and README.md list sources and
//! assumptions. Linear circuit: two single-pole op-amps, no diodes or transistors.

use crate::control::{ControlSpec, Taper};
use crate::netlist::{Netlist, OpAmp};

pub const NAME: &str = "Three-Band Stack Boost";
pub const VENDOR: &str = "Analog to VST";
/// Reverse-domain id, unique per plugin.
pub const CLAP_ID: &str = "local.analog-to-vst.three-band-stack-boost";
/// Exactly 16 bytes, unique per plugin.
pub const VST3_ID: [u8; 16] = *b"ThreeBandStkBst1";

/// Volts at the jacks for a full-scale (0 dBFS) sample. Op-amp rail clipping isn't modeled, and
/// the gain stage reaches 107 V/V, so full scale is set where a single full-scale sine keeps the
/// second op-amp's output inside its 0 to 9 V rails at the largest-gain control position found
/// (54.4 V/V at 5 kHz: 0.07 V peak in gives 3.81 V peak around the 4.5 V bias). That is a result
/// for the sines and the burst tested, not a bound: an input below full scale that is not a
/// single sine can exceed the rails (README.md, "Op-amp output check"). The circuit is linear,
/// so this number does not change the sound; any input that would take an op-amp output past a
/// rail passes cleanly where the hardware would clip.
pub const VOLTS_PER_FULL_SCALE: f64 = 0.07;

/// Typical 25 C values for the op-amp the designer suggests first; see the netlist header [S4].
const U1: OpAmp = OpAmp { a0: 5e4, gbw: 2.25e6 };

/// Front-panel controls, in order. Their names (upper-cased) are the `$NAME` placeholders in the
/// SPICE netlist, filled with `plain` values.
pub const CONTROLS: &[ControlSpec] = &[
    ControlSpec {
        name: "Bass",
        taper: Taper::Audio { max: 500e3 },
        default: 0.5,
    },
    ControlSpec {
        name: "Mid",
        taper: Taper::Audio { max: 100e3 },
        default: 0.5,
    },
    ControlSpec {
        name: "Treble",
        taper: Taper::Linear { min: 0.0, max: 1.0 },
        default: 0.5,
    },
    ControlSpec {
        name: "Level",
        taper: Taper::Audio { max: 500e3 },
        default: 0.5,
    },
];

pub fn circuit(n: &mut Netlist) {
    // Supply and bias: V9, R1, C1, R2, R3, C2.
    n.supply("v9", 9.0);
    n.resistor("v9", "vp", 100.0);
    n.capacitor("vp", "0", 100e-6);
    n.resistor("vp", "vref", 100e3);
    n.resistor("vref", "0", 100e3);
    n.capacitor("vref", "0", 100e-6);

    // Input and buffer: R4, C3, R5, U1a.
    n.input("in");
    n.resistor("in", "0", 10e6);
    n.capacitor("in", "ib", 22e-9);
    n.resistor("ib", "vref", 1e6);
    n.opamp("ib", "a", "a", U1);

    // Tone stack: R6, CTREB (C4 in series with C5), treble pot, R11, C6, bass pot,
    // CMID (C7 in series with C8), mid pot, R10.
    n.resistor("a", "s", 200e3);
    n.capacitor("a", "tt", 220e-12 * 150e-12 / 370e-12);
    n.pot("RTREBLE1", "tt", "tw");
    n.pot("RTREBLE2", "tw", "t1");
    n.resistor("tw", "vref", 1e6);
    n.capacitor("s", "t1", 47e-9);
    n.pot("RBASS", "t1", "m");
    n.capacitor("s", "m", 13.2e-9);
    n.pot("RMID", "m", "vref");
    n.resistor("m", "vref", 100e3);

    // Gain stage: U1b, C10, level pot, R12, C9.
    n.opamp("tw", "fb", "bo", U1);
    n.capacitor("fb", "bo", 47e-12);
    n.pot("RLEVEL", "fb", "bo");
    n.resistor("fb", "g", 4.7e3);
    n.capacitor("g", "vref", 470e-9);

    // Output: C11, R14, assumed 1 Meg load.
    n.capacitor("bo", "out", 470e-9);
    n.resistor("out", "0", 100e3);
    n.resistor("out", "0", 1e6);
    n.output("out");
}

/// Pot resistances from control values in physical units. Same formulas as the netlist's `{...}`
/// expressions. Called with `set(pot name, ohms)`; values are clamped to at least 1 ohm.
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    let (bass, mid, treble, level) = (plain[0], plain[1], plain[2], plain[3]);
    set("RBASS", bass);
    set("RMID", mid);
    set("RTREBLE1", 500e3 * (1.0 - treble));
    set("RTREBLE2", 500e3 * treble);
    set("RLEVEL", level);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Circuit, Element};

    /// Node voltages printed by ngspice for `reference/circuit.cir` with the input at 0 V
    /// (`uv run harness op ../reference/circuit.cir`), to its seven printed digits. The same at
    /// every control position, because no direct current flows through any pot.
    const SPICE_OPERATING_POINT: &[(&str, f64)] = &[
        ("v9", 9.0),
        ("vp", 8.995502),
        ("vref", 4.497751),
        ("ib", 4.497751),
        ("a", 4.497661),
        ("s", 4.497661),
        ("tt", 4.497751),
        ("tw", 4.497751),
        ("t1", 4.497751),
        ("m", 4.497751),
        ("fb", 4.497661),
        ("g", 4.497661),
        ("bo", 4.497661),
        ("out", 0.0),
        ("in", 0.0),
    ];

    #[test]
    fn operating_point_matches_spice() {
        // Default, largest gain, and every control at 0.
        for rotations in [[0.5f32; 4], [0.0, 1.0, 1.0, 1.0], [0.0; 4]] {
            let plain: Vec<f64> = CONTROLS.iter().zip(rotations).map(|(c, r)| c.plain(r)).collect();
            let mut n = Netlist::default();
            circuit(&mut n);
            let pot_elements = n.pots.clone();
            let mut pots_set = 0;
            pots(&plain, &mut |name, ohms| {
                let index = pot_elements.iter().find(|p| p.0 == name).expect("pot in the netlist").1;
                let Element::Resistor(_, _, r) = &mut n.elements[index] else {
                    panic!("pot {name} is not a resistor");
                };
                *r = ohms.max(1.0);
                pots_set += 1;
            });
            assert_eq!(pots_set, pot_elements.len(), "every pot in the netlist gets a value");
            let nodes: Vec<usize> = SPICE_OPERATING_POINT.iter().map(|(name, _)| n.node(name)).collect();
            let c = Circuit::new(n.elements, (n.output, 0), 672_000.0);
            for (&(name, volts), node) in SPICE_OPERATING_POINT.iter().zip(nodes) {
                let v = c.operating_point()[node - 1];
                assert!(
                    (v - volts).abs() < 1e-6,
                    "{name}: model {v} V, SPICE {volts} V at {rotations:?}"
                );
            }
        }
    }
}
