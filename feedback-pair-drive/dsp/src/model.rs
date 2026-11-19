//! The Circuit being modeled: a four-transistor feedback drive. A long-tailed NPN pair (Q6, Q7)
//! and a PNP second stage (Q3) form an inverting feedback amplifier; an NPN phase splitter (Q5)
//! follows it. Part for part with `reference/circuit.cir`: same parts, same values, same pot
//! formula, same node names. Sources and assumptions are in that file's header and the README.

use crate::control::{ControlSpec, Taper};
use crate::netlist::{BjtModel, Netlist};

pub const NAME: &str = "Feedback Pair Drive";
pub const VENDOR: &str = "Analog to VST";
/// Reverse-domain id, unique per plugin.
pub const CLAP_ID: &str = "local.analog-to-vst.feedback-pair-drive";
/// Exactly 16 bytes, unique per plugin.
pub const VST3_ID: [u8; 16] = *b"FeedbackPairDr01";

/// Volts at the jacks for a full-scale (0 dBFS) sample: 5 V peak, a Eurorack audio level. The
/// clipped output reaches about -6.8 V, so the output can exceed full scale by up to 2.7 dB.
pub const VOLTS_PER_FULL_SCALE: f64 = 5.0;

/// One Control. The pot is linear (marked B100k) and wired as a series rheostat, so `plain` is the
/// fraction of travel and `pots` turns it into ohms. It is the `LEVEL` placeholder in the Netlist.
pub const CONTROLS: &[ControlSpec] = &[ControlSpec {
    name: "Level",
    taper: Taper::Linear { min: 0.0, max: 1.0 },
    default: 0.5,
}];

/// Full resistance of the level pot, in ohms.
const LEVEL_POT_OHMS: f64 = 100e3;

/// onsemi (Fairchild) MMBTH10, published values. RE is not in the published model (SPICE default
/// 0). ISE/NE, NC, temperature and charge-storage terms are left out, as in the Netlist.
const QH10: BjtModel = BjtModel {
    is: 69.28e-18,
    bf: 308.6,
    br: 1.11,
    vaf: 100.0,
    ikf: 22.83e-3,
    rb: 10.0,
    rc: 4.0,
    re: 0.0,
};

/// Nexperia BC858B, published values of the main device. The vendor's quasi-saturation helper
/// network, leakage, non-unity emission coefficients and charge storage are left out.
const QBC858B: BjtModel = BjtModel {
    is: 1.95e-14,
    bf: 297.2,
    br: 10.02,
    vaf: 41.13,
    ikf: 83.1e-3,
    rb: 220.0,
    rc: 0.2657,
    re: 1.045,
};

pub fn circuit(n: &mut Netlist) {
    n.supply("vp", 12.0);
    n.supply("vn", -12.0);

    // Source, level rheostat and input coupling into the summing junction.
    n.input("in");
    n.resistor("in", "jack", 1e3);
    n.pot("RLEVEL", "jack", "pw");
    n.capacitor("pw", "sum", 2.2e-6);
    n.resistor("sum", "b6", 470.0);

    // Feedback amplifier: long-tailed pair, PNP second stage, feedback from `vo` to `b6`.
    n.resistor("b6", "vo", 100e3);
    n.npn("vp", "b6", "tail", QH10);
    n.npn("c7", "0", "tail", QH10);
    n.resistor("tail", "vn", 100e3);
    n.resistor("vp", "c7", 15e3);
    n.capacitor("vp", "c7", 10e-9);
    n.pnp("vo", "c7", "vp", QBC858B);
    n.resistor("vo", "vn", 2.7e3);

    // Phase splitter. The output is the collector side; the emitter side keeps its load.
    n.capacitor("vo", "b5", 1e-6);
    n.resistor("vp", "b5", 36e3);
    n.resistor("b5", "vn", 15e3);
    n.npn("c5", "b5", "e5", QH10);
    n.resistor("vp", "c5", 2.7e3);
    n.resistor("e5", "vn", 2.7e3);
    n.capacitor("c5", "out", 1e-6);
    n.resistor("out", "0", 100e3);
    n.capacitor("e5", "outinv", 1e-6);
    n.resistor("outinv", "0", 100e3);
    n.output("out");
}

/// Pot resistance from the Control value. Same formula as the Netlist's `{...}` expression. Called
/// with `set(pot name, ohms)`; values are clamped to at least 1 ohm.
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    assert_eq!(plain.len(), CONTROLS.len());
    let level = plain[0];
    assert!((0.0..=1.0).contains(&level), "level is a fraction of travel");
    set("RLEVEL", LEVEL_POT_OHMS * (1.0 - level));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::Circuit;

    /// The Model's circuit at the lowest rate the Processor uses, with the pot at `level`.
    fn build(level: f64) -> (Netlist, Circuit) {
        let mut n = Netlist::default();
        circuit(&mut n);
        let mut c = Circuit::new(n.elements.clone(), (n.output, 0), 576_000.0);
        let pot_index = n.pots[0].1;
        pots(&[level], &mut |name, ohms| {
            assert_eq!(name, "RLEVEL");
            c.set_resistor(pot_index, ohms.max(1.0));
        });
        (n, c)
    }

    #[test]
    fn netlist_has_the_parts_of_the_reference() {
        let (n, _) = build(0.5);
        let count = |f: fn(&crate::circuit::Element) -> bool| n.elements.iter().filter(|e| f(e)).count();
        use crate::circuit::Element as E;
        // 13 circuit resistors (9 fixed, pot, source, two loads), RB and RC of 4 transistors, and
        // the RE of the one transistor that has it (the three MMBTH10 have RE = 0: no resistor).
        assert_eq!(count(|e| matches!(e, E::Resistor(..))), 13 + 8 + 1);
        assert_eq!(count(|e| matches!(e, E::Capacitor(..))), 5);
        assert_eq!(count(|e| matches!(e, E::Npn { .. })), 3);
        assert_eq!(count(|e| matches!(e, E::Pnp { .. })), 1);
        assert_eq!(count(|e| matches!(e, E::Dc(..))), 2);
        assert_eq!(n.pots.len(), 1);
    }

    /// ngspice 45 operating point of `reference/circuit.cir` (`uv run harness op`, input at 0 V),
    /// as printed, in volts. It does not depend on LEVEL: C12 blocks DC.
    const NGSPICE_OP: &[(&str, f64)] = &[
        ("b6", -1.24163e-03),
        ("sum", -1.24163e-03),
        ("tail", -7.07129e-01),
        ("c7", 1.132140e+01),
        ("vo", 1.474020e-02),
        ("b5", -5.01813e+00),
        ("c5", 5.842434e+00),
        ("e5", -5.82281e+00),
        ("out", 0.0),
        ("outinv", 0.0),
        ("jack", 0.0),
        ("pw", 0.0),
    ];

    #[test]
    fn operating_point_matches_ngspice() {
        // First use of the Solver's PNP path in a Model: Q3's operating point sets `c7` and `vo`.
        // The Solver has no gmin on the transistors' junctions and a rounded thermal voltage;
        // ngspice prints six or seven digits. 20 uV covers those on every node. Measured: at
        // most 12.4 uV (`c7`). RE = 0 adds no resistor, as in ngspice: going from the earlier
        // 0.001 ohm stand-in to none moved `c5` by -2.236 uV and `e5` by +2.245 uV, and ngspice
        // moves them by -2.237 and +2.245 uV between RE=1m and RE=0.
        const TOLERANCE_V: f64 = 20e-6;
        let (mut n, c) = build(0.5);
        let op = c.operating_point().to_vec();
        for &(name, expected) in NGSPICE_OP {
            let v = op[n.node(name) - 1];
            assert!(
                (v - expected).abs() < TOLERANCE_V,
                "{name}: model {v:.9} V, ngspice {expected:.9} V"
            );
        }
    }

    #[test]
    fn silence_stays_at_the_operating_point() {
        // The output is AC coupled, so it rests at 0 V. Measured: it starts at 0.7 nV and settles
        // to 113 nV (-153 dB re full scale) within a second, the same at every level. That is the
        // rounding floor of the state-space matrices: the 1 uF capacitors hold about 6 V, so their
        // companion currents at this rate are near 7 A and must cancel to nothing at `out`.
        const REST_LIMIT_V: f64 = 1e-6;
        const TICKS: usize = 576_000;
        for level in [0.0, 0.5, 1.0] {
            let (_, mut c) = build(level);
            for _ in 0..TICKS {
                let y = c.tick(0.0);
                assert!(y.abs() < REST_LIMIT_V, "level {level}: output {y} V at rest");
            }
        }
    }

    #[test]
    fn overdriven_sines_stay_bounded_and_converge() {
        // Q3 and Q5 saturate and cut off on every clipped cycle, and Q5's base is driven through
        // C7 at up to 0.7 V per circuit sample. Every joint Newton solve must converge (the
        // Solver counts the ones that do not), and the output must stay inside what the rails
        // allow: `out` is `c5` less C5's 5.8 V, and `c5` lies between the rails.
        const RAIL_V: f64 = 12.0;
        const RATE: f64 = 576_000.0;
        const TICKS: usize = 57_600;
        let cases = [
            (1.0, 1000.0, 5.0),
            (1.0, 100.0, 5.0),
            (1.0, 5000.0, 5.0),
            (1.0, 20_000.0, 5.0),
            (1.0, 1000.0, 0.5),
            (0.9, 1000.0, 5.0),
            (0.5, 1000.0, 5.0),
            (0.0, 1000.0, 5.0),
        ];
        for (level, freq, volts) in cases {
            let (_, mut c) = build(level);
            for k in 0..TICKS {
                let y = c.tick(volts * (std::f64::consts::TAU * freq * k as f64 / RATE).sin());
                assert!(
                    y.abs() < RAIL_V,
                    "level {level}, {freq} Hz, {volts} V: output {y} V at sample {k}"
                );
            }
            assert_eq!(c.failed_solves(), 0, "level {level}, {freq} Hz, {volts} V");
        }
    }

    #[test]
    fn a_failed_solve_is_held_and_the_circuit_recovers() {
        // Force the joint Newton solve to run out of iterations (a bound of 1) in the middle of
        // a clipped sine. Each failed sample must repeat the last valid output and leave the
        // circuit where it was; once the bound is restored the solves converge again and silence
        // brings the output back to rest.
        const RATE: f64 = 576_000.0;
        const RAIL_V: f64 = 12.0;
        let drive = |k: usize| 5.0 * (std::f64::consts::TAU * 1000.0 * k as f64 / RATE).sin();
        let (_, mut c) = build(1.0);
        let mut last = 0.0;
        for k in 0..2000 {
            last = c.tick(drive(k));
        }
        assert_eq!(c.failed_solves(), 0);

        c.limit_joint_iterations(1);
        let mut held = 0;
        // Fewer samples than the bound on consecutive held samples, so this tests the hold alone.
        for k in 2000..2800 {
            let failed_before = c.failed_solves();
            let y = c.tick(drive(k));
            assert!(y.is_finite() && y.abs() < RAIL_V, "output {y} V at sample {k}");
            if c.failed_solves() > failed_before {
                assert_eq!(y, last, "a failed sample repeats the last valid output");
                held += 1;
            }
            last = y;
        }
        assert!(held > 400, "the bound of 1 forced only {held} failures in 800 samples");
        assert_eq!(c.failed_solves(), held);
        assert_eq!(c.rest_restores(), 0);

        c.limit_joint_iterations(200);
        for k in 2800..6800 {
            let y = c.tick(drive(k));
            assert!(y.is_finite() && y.abs() < RAIL_V, "output {y} V at sample {k}");
        }
        assert_eq!(c.failed_solves(), held, "solves converge again under the normal bound");
        // Two seconds of silence: the last tenth of a second is at rest.
        for k in 0..1_152_000 {
            let y = c.tick(0.0);
            assert!(y.is_finite() && y.abs() < RAIL_V);
            assert!(k < 1_094_400 || y.abs() < 1e-6, "output {y} V after {k} silent samples");
        }
        assert_eq!(c.failed_solves(), held);
    }

    #[test]
    fn absurd_and_non_finite_input_cannot_poison_the_circuit() {
        // Inputs far outside any audio level, then NaN and infinities, then silence. Whatever the
        // solve does with the absurd samples, the output stays finite, and five seconds of
        // silence bring it back to rest.
        let (_, mut c) = build(1.0);
        let bad = [
            1e6,
            -1e6,
            1e300,
            -1e300,
            f64::MAX,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ];
        for round in 0..64 {
            for u in bad {
                let y = c.tick(u);
                assert!(y.is_finite(), "output {y} for input {u} in round {round}");
            }
        }
        // The samples that did converge charged the capacitors far beyond the rails, so the
        // way back is the Circuit's own slow discharge: measured at rest again after about 4 s.
        let failed = c.failed_solves();
        assert!(
            failed > 0,
            "none of the absurd samples failed, so this did not test containment"
        );
        for k in 0..2_880_000 {
            let y = c.tick(0.0);
            assert!(y.is_finite());
            assert!(k < 2_592_000 || y.abs() < 1e-6, "output {y} V after {k} silent samples");
        }
        assert_eq!(c.failed_solves(), failed, "silence solves normally afterwards");

        // A NaN sample is a silent sample: from rest, the circuit does not move.
        let (_, mut c) = build(0.5);
        for u in [f64::NAN, 0.0, f64::INFINITY, 0.0, 0.0] {
            assert!(c.tick(u).abs() < 1e-6);
        }
        assert_eq!(c.failed_solves(), 0);
    }

    #[test]
    fn a_state_no_sample_solves_from_is_left_by_going_back_to_rest() {
        // A 1e9 V, 1 kHz sine for 6000 samples leaves an accepted, finite capacitor history from
        // which no later sample solves. Without the return to rest every silent sample after it is
        // held for ever.
        let (_, mut c) = build(1.0);
        for k in 0..6000 {
            let u = 1e9 * (std::f64::consts::TAU * 1000.0 * k as f64 / 576_000.0).sin();
            assert!(c.tick(u).is_finite());
        }
        assert_eq!(c.rest_restores(), 0, "the sine itself does not need a return to rest");
        for _ in 0..4096 {
            assert!(c.tick(0.0).is_finite());
        }
        assert_eq!(c.rest_restores(), 1, "the stuck state is left once");
        let failed = c.failed_solves();
        for _ in 0..57_600 {
            assert!(c.tick(0.0).abs() < 1e-6, "silence after the return is at rest");
        }
        // Ordinary input solves again.
        for k in 0..57_600 {
            let u = 5.0 * (std::f64::consts::TAU * 1000.0 * k as f64 / 576_000.0).sin();
            assert!(c.tick(u).is_finite());
        }
        assert_eq!(c.failed_solves(), failed, "nothing is held after the return to rest");
        assert_eq!(c.rest_restores(), 1);
    }
}
