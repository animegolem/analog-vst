//! Three-transistor discrete amplifier: an NPN differential pair driving a PNP common-emitter
//! stage, with a passive input network and one drive pot in front. Part for part with
//! `reference/circuit.cir`; see its header and the README for sources and assumptions.

use crate::control::{ControlSpec, Taper};
use crate::netlist::{BjtModel, Netlist};

pub const NAME: &str = "Three-Transistor Drive";
pub const VENDOR: &str = "Analog to VST";
/// Reverse-domain id, unique per plugin.
pub const CLAP_ID: &str = "local.analog-to-vst.three-transistor-drive";
/// Exactly 16 bytes, unique per plugin.
pub const VST3_ID: [u8; 16] = *b"ThreeTransisto01";

/// Volts at the jacks for a full-scale (0 dBFS) sample: 5 V peak, a modular-synthesizer audio
/// level. The amplifier clips near +11.7 V and -10.8 V at `out`, so the plugin's output exceeds
/// full scale (by up to about 7 dB) at high drive.
pub const VOLTS_PER_FULL_SCALE: f64 = 5.0;

/// The one control. Its plain value is the wiper position of the 1M pot as a fraction of the
/// track, the `$DRIVE` placeholder in the netlist.
pub const CONTROLS: &[ControlSpec] = &[ControlSpec {
    name: "Drive",
    taper: Taper::Audio { max: 1.0 },
    default: 0.5,
}];

/// Input pair: Fairchild 2N5088 model, standing in for the MBT6429DW1T1G. Reduced Gummel-Poon,
/// the same eight parameters as `.model QN6429`. RE is absent from the published model (0).
const QN6429: BjtModel = BjtModel {
    is: 5.911e-15,
    bf: 1122.0,
    br: 1.271,
    vaf: 62.37,
    ikf: 14.92e-3,
    rb: 10.0,
    rc: 1.61,
    re: 0.0,
};

/// Output transistor: Fairchild 2N3906/MMBT3906 model, reduced, the same as `.model QP3906`.
const QP3906: BjtModel = BjtModel {
    is: 1.41e-15,
    bf: 180.7,
    br: 4.977,
    vaf: 18.7,
    ikf: 80e-3,
    rb: 10.0,
    rc: 2.5,
    re: 0.0,
};

pub fn circuit(n: &mut Netlist) {
    n.supply("vp", 12.0);
    n.supply("vn", -12.0);

    n.input("in");
    n.resistor("in", "jh", 1e3); // RS
    n.resistor("jh", "0", 1e6); // R17
    n.resistor("jh", "pt", 68e3); // R15
    n.resistor("jh", "pt", 68e3); // R16
    n.pot("RDRIVE1", "pt", "w");
    n.pot("RDRIVE2", "w", "0");
    n.resistor("w", "ci", 1e3); // R1
    n.capacitor("ci", "b1", 100e-9); // C3
    n.resistor("b1", "0", 100e3); // R2

    n.npn("c1", "b1", "e", QN6429); // Q5Q1
    n.npn("vp", "b2", "e", QN6429); // Q5Q2
    n.resistor("e", "vn", 100e3); // R3
    n.resistor("vp", "c1", 27e3); // R4
    n.capacitor("vp", "cz", 100e-12); // C4
    n.resistor("cz", "c1", 100.0); // R5
    n.pnp("a", "c1", "vp", QP3906); // Q2
    n.resistor("a", "vn", 10e3); // R9

    n.capacitor("b2", "a", 10e-12); // C5
    n.resistor("b2", "fb", 1e3); // R6
    n.resistor("fb", "a", 200e3); // R7
    n.resistor("fb", "cf", 1e3); // R8
    n.capacitor("cf", "0", 220e-6); // C6

    n.capacitor("a", "out", 1e-6); // C7
    n.resistor("out", "0", 100e3); // RLOAD
    n.output("out");
}

/// Pot resistances from the control's plain value. Same formulas as the netlist's `{...}`
/// expressions. Called with `set(pot name, ohms)`; values are clamped to at least 1 ohm.
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    let drive = plain[0];
    set("RDRIVE1", 1e6 * (1.0 - drive));
    set("RDRIVE2", 1e6 * drive);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Circuit, Element};
    use crate::oversample::MIN_CIRCUIT_RATE;

    /// The circuit at a drive value (wiper fraction) with the names of the nodes asked for.
    fn build(drive: f64, names: &[&str]) -> (Circuit, Vec<usize>) {
        let mut n = Netlist::default();
        circuit(&mut n);
        let pot_elements = n.pots.clone();
        pots(&[drive], &mut |name, ohms| {
            let index = pot_elements
                .iter()
                .find(|p| p.0 == name)
                .expect("pot is in the netlist")
                .1;
            let Element::Resistor(_, _, r) = &mut n.elements[index] else {
                panic!("pot {name} is not a resistor");
            };
            *r = ohms.max(1.0);
        });
        let nodes = names.iter().map(|name| n.node(name)).collect();
        (Circuit::new(n.elements, (n.output, 0), MIN_CIRCUIT_RATE), nodes)
    }

    /// The model's DC operating point against ngspice 45 on `reference/circuit.cir` (input 0 V;
    /// the bias is the same at every drive value because C3 blocks DC). This is the first circuit
    /// here with a PNP. The model differs from the netlist in two known ways, both worth a few
    /// microvolts: its thermal voltage constant is rounded, and RE = 0 is entered as 0.001 ohm.
    /// Measured: every node within 8 uV.
    #[test]
    fn operating_point_matches_ngspice() {
        const SPICE: &[(&str, f64)] = &[
            ("a", 3.685_722_177_4e-2),
            ("b1", -2.271_912_626e-3),
            ("b2", 2.416_364_874_9e-2),
            ("cf", 2.422_680_085_4e-2),
            ("e", -5.760_916_121e-1),
            ("c1", 1.130_132_516_0e1),
            ("out", 0.0),
        ];
        let names: Vec<&str> = SPICE.iter().map(|s| s.0).collect();
        for drive in [0.0, 0.1, 1.0] {
            let (c, nodes) = build(drive, &names);
            let op = c.operating_point();
            for (&(name, volts), &node) in SPICE.iter().zip(&nodes) {
                let got = op[node - 1];
                assert!(
                    (got - volts).abs() < 20e-6,
                    "drive {drive}, node {name}: model {got:.9} V, ngspice {volts:.9} V"
                );
            }
        }
    }

    /// A 5 V peak (0 dBFS) 1 kHz sine at full drive overdrives the amplifier about 120 times: the
    /// PNP goes from saturation to cutoff within a few samples. The solve must converge on every
    /// sample (the solver's failure count stays 0) and land on ngspice's clipped levels at
    /// `out` over 40 to 50 ms: +11.684 V, -10.755 V, 11.129 V RMS (`tran 1u 50m 0 0.4u`, reltol
    /// 1e-6). Levels are read from the mean of two neighbouring samples: after a clipping corner
    /// trapezoidal integration rings at half the circuit rate, far above what the decimation
    /// filter passes.
    #[test]
    fn hard_clipping_converges_to_the_ngspice_levels() {
        let (mut c, _) = build(1.0, &[]);
        let rate = MIN_CIRCUIT_RATE;
        let (from, to) = ((0.04 * rate) as usize, (0.05 * rate) as usize);
        let (mut max, mut min, mut sum_sq, mut last) = (f64::MIN, f64::MAX, 0.0, 0.0);
        for k in 0..to {
            let y = c.tick(5.0 * (std::f64::consts::TAU * 1000.0 * k as f64 / rate).sin());
            assert!(y.is_finite() && y.abs() < 13.0, "sample {k}: {y}");
            let mean = 0.5 * (y + last);
            last = y;
            if k >= from {
                max = max.max(mean);
                min = min.min(mean);
                sum_sq += y * y;
            }
        }
        assert_eq!(c.failures(), 0, "every sample converged");
        let rms = (sum_sq / (to - from) as f64).sqrt();
        assert!((max - 11.683_84).abs() < 2e-3, "positive clip {max}");
        assert!((min + 10.754_58).abs() < 2e-3, "negative clip {min}");
        assert!((rms - 11.129_0).abs() < 2e-3, "rms {rms}");
        println!("clip levels: max {max:.5} V, min {min:.5} V, rms {rms:.5} V");
    }

    /// The solver's own iteration bound, as `set_joint_limit` accepts it.
    const JOINT_LIMIT: usize = 200;

    /// The 0 dBFS, 1 kHz test tone at the circuit's rate.
    fn sine(k: usize) -> f64 {
        5.0 * (std::f64::consts::TAU * 1000.0 * k as f64 / MIN_CIRCUIT_RATE).sin()
    }

    /// The joint solve is made to fail on every sample for 2 ms of the full-drive 5 V sine, by
    /// lowering its iteration bound to 1 (with the real bound of 200 no input found so far makes
    /// it fail). Each failed sample must repeat the previous output and leave the circuit's state
    /// alone. The state is checked through its consequences: with the bound restored, the circuit
    /// must carry on exactly like a twin that was simply never given those 2 ms. The two share
    /// their capacitor states and differ only in the solver's starting point, so their outputs
    /// agree to the solver's precision. Measured: 9216 of 9216 samples failed, and the outputs
    /// over the following 5 ms were equal (difference 0 V).
    #[test]
    fn failed_solves_hold_the_state_and_the_next_samples_recover() {
        let (mut c, _) = build(1.0, &[]);
        let (mut skipped, _) = build(1.0, &[]);
        let at = |ms: f64| (ms * 1e-3 * MIN_CIRCUIT_RATE) as usize;
        let mut last = 0.0;
        for k in 0..at(5.0) {
            last = c.tick(sine(k));
            assert_eq!(last, skipped.tick(sine(k)));
        }
        assert_eq!(c.failures(), 0);

        c.set_joint_limit(1);
        for k in at(5.0)..at(7.0) {
            assert_eq!(
                c.tick(sine(k)),
                last,
                "sample {k}: a failed sample repeats the last output"
            );
        }
        assert_eq!(
            c.failures(),
            (at(7.0) - at(5.0)) as u64,
            "every sample in the window failed"
        );
        assert!(last.is_finite() && last.abs() < 13.0);

        // 2 ms is two whole periods, so the twin's input continues without a jump.
        c.set_joint_limit(JOINT_LIMIT);
        let mut worst = 0.0f64;
        for k in at(7.0)..at(12.0) {
            let (y, want) = (c.tick(sine(k)), skipped.tick(sine(k)));
            assert!(y.is_finite() && y.abs() < 13.0, "sample {k}: {y}");
            worst = worst.max((y - want).abs());
        }
        assert_eq!(
            c.failures(),
            (at(7.0) - at(5.0)) as u64,
            "no failure after the bound is restored"
        );
        assert_eq!(skipped.failures(), 0);
        assert!(
            worst < 1e-6,
            "difference to the twin that skipped the window: {worst:e} V"
        );
        println!("largest difference to the twin: {worst:e} V");
    }

    /// NaN and both infinities at the circuit's input count as 0 V: from rest, the output during
    /// and after them is bit-identical to a twin fed silence, and it stays at rest. Rest is not
    /// exactly 0 V: the output wanders by a few nanovolts of rounding (measured below).
    #[test]
    fn non_finite_input_then_silence_stays_at_rest() {
        let (mut c, _) = build(1.0, &[]);
        let (mut silent, _) = build(1.0, &[]);
        let rest = silent.tick(0.0);
        assert_eq!(c.tick(0.0).to_bits(), rest.to_bits());
        for u in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            assert_eq!(c.tick(u).to_bits(), silent.tick(0.0).to_bits(), "input {u}");
        }
        let mut worst = 0.0f64;
        for k in 0..10_000 {
            let y = c.tick(0.0);
            assert_eq!(y.to_bits(), silent.tick(0.0).to_bits(), "sample {k}");
            worst = worst.max((y - rest).abs());
        }
        assert!(worst < 1e-6, "largest distance from rest: {worst:e} V");
        assert_eq!(c.failures(), 0);
        println!("largest distance from rest: {worst:e} V");
    }
}
