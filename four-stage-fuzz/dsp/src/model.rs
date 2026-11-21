//! ElectroSmash V3 signal path; see reference/circuit.cir for sources and assumptions.
use crate::control::{ControlSpec, Taper};
use crate::netlist::{BjtModel, Netlist, D1N4148};
pub const NAME: &str = "Four Stage Sustainer";
pub const VENDOR: &str = "Analog to VST";
pub const CLAP_ID: &str = "local.analog-to-vst.four-stage-sustainer";
pub const VST3_ID: [u8; 16] = *b"FourStageSusta01";
pub const VOLTS_PER_FULL_SCALE: f64 = 0.2;
pub const CONTROLS: &[ControlSpec] = &[
    ControlSpec {
        name: "Sustain",
        taper: Taper::Linear { min: 0.0, max: 1.0 },
        default: 0.5,
    },
    ControlSpec {
        name: "Tone",
        taper: Taper::Linear { min: 0.0, max: 1.0 },
        default: 0.5,
    },
    ControlSpec {
        name: "Volume",
        taper: Taper::Linear { min: 0.0, max: 1.0 },
        default: 0.5,
    },
];
const Q: BjtModel = BjtModel {
    is: 5.911e-15,
    bf: 1122.0,
    br: 1.271,
    vaf: 62.37,
    ikf: 14.92e-3,
    rb: 10.0,
    rc: 1.61,
    re: 0.0,
};
pub fn circuit(n: &mut Netlist) {
    n.supply("vcc", 9.0);
    n.input("in");
    n.resistor("in", "jack", 1e3);
    n.resistor("jack", "i1", 39e3);
    n.capacitor("i1", "b4", 1e-6);
    n.resistor("b4", "0", 47e3);
    n.resistor("b4", "c4", 470e3);
    n.capacitor("b4", "c4", 470e-12);
    n.resistor("vcc", "c4", 10e3);
    n.resistor("e4", "0", 100.0);
    n.npn("c4", "b4", "e4", Q);
    n.capacitor("c4", "sustop", 1e-6);
    n.pot("RSUSTOP", "sustop", "sus");
    n.pot("RSUSBOT", "sus", "susbot");
    n.resistor("susbot", "0", 1e3);
    n.capacitor("sus", "s3", 100e-9);
    n.resistor("s3", "b3", 10e3);
    n.resistor("b3", "0", 100e3);
    n.resistor("b3", "c3", 470e3);
    n.capacitor("b3", "c3", 470e-12);
    n.capacitor("b3", "d3", 1e-6);
    n.diode_pair("d3", "c3", D1N4148);
    n.resistor("vcc", "c3", 10e3);
    n.resistor("e3", "0", 150.0);
    n.npn("c3", "b3", "e3", Q);
    n.capacitor("c3", "s2", 100e-9);
    n.resistor("s2", "b2", 10e3);
    n.resistor("b2", "0", 100e3);
    n.resistor("b2", "c2", 470e3);
    n.capacitor("b2", "c2", 470e-12);
    n.capacitor("b2", "d2", 1e-6);
    n.diode_pair("d2", "c2", D1N4148);
    n.resistor("vcc", "c2", 10e3);
    n.resistor("e2", "0", 150.0);
    n.npn("c2", "b2", "e2", Q);
    n.capacitor("c2", "hi", 4e-9);
    n.resistor("hi", "0", 22e3);
    n.resistor("c2", "lo", 39e3);
    n.capacitor("lo", "0", 10e-9);
    n.pot("RTONEHI", "hi", "tone");
    n.pot("RTONELO", "tone", "lo");
    n.capacitor("tone", "b1", 100e-9);
    n.resistor("vcc", "b1", 430e3);
    n.resistor("b1", "0", 100e3);
    n.resistor("vcc", "c1", 15e3);
    n.resistor("e1", "0", 3.3e3);
    n.npn("c1", "b1", "e1", Q);
    n.capacitor("c1", "voltop", 100e-9);
    n.pot("RVOLTOP", "voltop", "out");
    n.pot("RVOLBOT", "out", "0");
    n.resistor("out", "0", 1e6);
    n.output("out");
}
pub fn pots(plain: &[f64], set: &mut dyn FnMut(&'static str, f64)) {
    let (s, t, v) = (plain[0], plain[1], plain[2]);
    set("RSUSTOP", 100e3 * (1.0 - s));
    set("RSUSBOT", 100e3 * s);
    set("RTONEHI", 100e3 * (1.0 - t));
    set("RTONELO", 100e3 * t);
    set("RVOLTOP", 100e3 * (1.0 - v));
    set("RVOLBOT", 100e3 * v);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Circuit, Element};
    use crate::oversample::MIN_CIRCUIT_RATE;

    /// The named nodes of `reference/circuit.cir` that the Model also has.
    const NODES: &[&str] = &[
        "vcc", "in", "jack", "i1", "b4", "c4", "e4", "sustop", "sus", "susbot", "s3", "b3", "c3", "d3", "e3", "s2",
        "b2", "c2", "d2", "e2", "hi", "lo", "tone", "b1", "c1", "e1", "voltop", "out",
    ];

    /// ngspice 45 `.op` of `reference/circuit.cir` with the input at 0 V, at every Setting of
    /// `settings.txt`: the Setting's name, its knob rotations, and the voltage of each of `NODES`.
    /// Default temperature and gmin; `.options reltol=1e-9 abstol=1e-16 vntol=1e-12 itl1=5000`
    /// so that the printed digits are the solution's (set numdgt=15).
    #[rustfmt::skip]
    const NGSPICE_OP: &[(&str, &[f32], &[f64])] = &[
        ("default", &[0.5, 0.5, 0.5], &[
            9.000000000000e+00, 0.0, 0.0, 0.0,
            6.392128502115e-01, 7.098997262201e+00, 1.764999932728e-02, 0.0,
            0.0, 0.0, 7.178917930948e-01, 7.178917930948e-01,
            4.281373110529e+00, 4.281373108666e+00, 6.970256501031e-02, 7.131704069173e-01,
            7.131704069173e-01, 4.245051661332e+00, 4.245051657076e+00, 6.629945175726e-02,
            5.800691711137e-01, 3.216747221631e+00, 1.898408196372e+00, 1.675868427914e+00,
            4.287568657197e+00, 1.037639674749e+00, 0.0, 0.0,
        ]),
        ("sustain_min", &[0.0, 0.5, 0.5], &[
            9.000000000000e+00, 0.0, 0.0, 0.0,
            6.392128502115e-01, 7.098997262201e+00, 1.764999932728e-02, 0.0,
            0.0, 0.0, 7.178917930948e-01, 7.178917930948e-01,
            4.281373110529e+00, 4.281373108666e+00, 6.970256501031e-02, 7.131704069173e-01,
            7.131704069173e-01, 4.245051661332e+00, 4.245051657076e+00, 6.629945175726e-02,
            5.800691711137e-01, 3.216747221631e+00, 1.898408196372e+00, 1.675868427914e+00,
            4.287568657197e+00, 1.037639674749e+00, 0.0, 0.0,
        ]),
        ("sustain_max", &[1.0, 0.5, 0.5], &[
            9.000000000000e+00, 0.0, 0.0, 0.0,
            6.392128502115e-01, 7.098997262201e+00, 1.764999932728e-02, 0.0,
            0.0, 0.0, 7.178917930948e-01, 7.178917930948e-01,
            4.281373110529e+00, 4.281373108666e+00, 6.970256501031e-02, 7.131704069173e-01,
            7.131704069173e-01, 4.245051661332e+00, 4.245051657076e+00, 6.629945175726e-02,
            5.800691711137e-01, 3.216747221631e+00, 1.898408196372e+00, 1.675868427914e+00,
            4.287568657197e+00, 1.037639674749e+00, 0.0, 0.0,
        ]),
        ("tone_min", &[0.5, 0.0, 0.5], &[
            9.000000000000e+00, 0.0, 0.0, 0.0,
            6.392128502115e-01, 7.098997262201e+00, 1.764999932728e-02, 0.0,
            0.0, 0.0, 7.178917930948e-01, 7.178917930948e-01,
            4.281373110529e+00, 4.281373108666e+00, 6.970256501031e-02, 7.131704361750e-01,
            7.131704361750e-01, 4.245051885953e+00, 4.245051881698e+00, 6.629947269976e-02,
            5.800655989156e-01, 3.216753778787e+00, 3.216727412168e+00, 1.675868427914e+00,
            4.287568657197e+00, 1.037639674749e+00, 0.0, 0.0,
        ]),
        ("tone_max", &[0.5, 1.0, 0.5], &[
            9.000000000000e+00, 0.0, 0.0, 0.0,
            6.392128502115e-01, 7.098997262201e+00, 1.764999932728e-02, 0.0,
            0.0, 0.0, 7.178917930948e-01, 7.178917930948e-01,
            4.281373110529e+00, 4.281373108666e+00, 6.970256501031e-02, 7.131704361750e-01,
            7.131704361750e-01, 4.245051885953e+00, 4.245051881698e+00, 6.629947269976e-02,
            5.800655989157e-01, 3.216753778785e+00, 5.800919655338e-01, 1.675868427914e+00,
            4.287568657197e+00, 1.037639674749e+00, 0.0, 0.0,
        ]),
        ("volume_min", &[0.5, 0.5, 0.0], &[
            9.000000000000e+00, 0.0, 0.0, 0.0,
            6.392128502115e-01, 7.098997262201e+00, 1.764999932728e-02, 0.0,
            0.0, 0.0, 7.178917930948e-01, 7.178917930948e-01,
            4.281373110529e+00, 4.281373108666e+00, 6.970256501031e-02, 7.131704069173e-01,
            7.131704069173e-01, 4.245051661332e+00, 4.245051657076e+00, 6.629945175726e-02,
            5.800691711137e-01, 3.216747221631e+00, 1.898408196372e+00, 1.675868427914e+00,
            4.287568657197e+00, 1.037639674749e+00, 0.0, 0.0,
        ]),
        ("volume_max", &[0.5, 0.5, 1.0], &[
            9.000000000000e+00, 0.0, 0.0, 0.0,
            6.392128502115e-01, 7.098997262201e+00, 1.764999932728e-02, 0.0,
            0.0, 0.0, 7.178917930948e-01, 7.178917930948e-01,
            4.281373110529e+00, 4.281373108666e+00, 6.970256501031e-02, 7.131704069173e-01,
            7.131704069173e-01, 4.245051661332e+00, 4.245051657076e+00, 6.629945175726e-02,
            5.800691711137e-01, 3.216747221631e+00, 1.898408196372e+00, 1.675868427914e+00,
            4.287568657197e+00, 1.037639674749e+00, 0.0, 0.0,
        ]),
    ];

    /// The Model's DC operating point against ngspice's, at every Setting and every named node.
    /// The Model has ngspice's thermal voltage (`netlist::VT`) and no gmin across the transistors'
    /// junctions, which ngspice has (1e-12 S). Measured: at most 2.10 uV (`c4`, the same at every
    /// Setting); with gmin lowered to 1e-18 in ngspice every node agrees within 9.1 nV. The bound
    /// leaves 1.9 times that. With the rounded thermal voltage this Solver had before
    /// (0.0258646 V) `c4` was 61 uV off and this test fails.
    #[test]
    fn operating_point_matches_ngspice() {
        const TOLERANCE_V: f64 = 4e-6;
        for &(setting, rotations, expected) in NGSPICE_OP {
            assert_eq!((rotations.len(), expected.len()), (CONTROLS.len(), NODES.len()));
            let plain: Vec<f64> = CONTROLS.iter().zip(rotations).map(|(c, &r)| c.plain(r)).collect();
            let mut n = Netlist::default();
            circuit(&mut n);
            let pot_elements = n.pots.clone();
            pots(&plain, &mut |name, ohms| {
                let index = pot_elements.iter().find(|p| p.0 == name).expect("pot in the netlist").1;
                let Element::Resistor(_, _, r) = &mut n.elements[index] else {
                    panic!("pot {name} is not a resistor");
                };
                *r = ohms.max(1.0);
            });
            let nodes: Vec<usize> = NODES.iter().map(|name| n.node(name)).collect();
            let c = Circuit::new(n.elements, (n.output, 0), MIN_CIRCUIT_RATE);
            for ((name, &volts), node) in NODES.iter().zip(expected).zip(nodes) {
                let v = c.operating_point()[node - 1];
                assert!(
                    (v - volts).abs() < TOLERANCE_V,
                    "{setting}, node {name}: model {v:.12} V, ngspice {volts:.12} V"
                );
            }
            // The comparison is not vacuous: the transistors are biased, far from 0 V.
            assert!(expected.iter().any(|v| v.abs() > 0.5), "{setting}");
        }
    }
}
