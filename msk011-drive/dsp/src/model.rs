//! Reduced MSK 011 one-channel model. See reference/circuit.cir and README.
use crate::control::{ControlSpec, Taper};
use crate::netlist::{BjtModel, Netlist};

pub const NAME: &str = "Class A Transistor Drive";
pub const VENDOR: &str = "Analog to VST";
pub const CLAP_ID: &str = "local.analog-to-vst.class-a-transistor-drive";
pub const VST3_ID: [u8; 16] = *b"ClassATransist01";
pub const VOLTS_PER_FULL_SCALE: f64 = 5.0;
pub const CONTROLS: &[ControlSpec] = &[ControlSpec {
    name: "Level",
    taper: Taper::Audio { max: 1.0 },
    default: 0.5,
}];

// Fairchild/onsemi 2N5088: RE is unspecified in the published model (SPICE default 0).
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
    for (b, e) in [("b1", "e1"), ("b2", "e2"), ("b3", "e3"), ("b4", "e4")] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Circuit, Element};
    use crate::oversample::MIN_CIRCUIT_RATE;

    /// The named nodes of `reference/circuit.cir` that the Model also has.
    const NODES: &[&str] = &[
        "vp", "vn", "in", "jack2", "jack1", "b1", "b2", "b3", "jack4", "b4", "e1", "sum", "e2", "e3", "e4", "trim",
        "c5", "e5", "b6", "dc", "e6", "out",
    ];

    /// ngspice 45 `.op` of `reference/circuit.cir` with the input at 0 V, at every Setting of
    /// `settings.txt`: the Setting's name, its knob rotations, and the voltage of each of `NODES`.
    /// Default temperature and gmin; `.options reltol=1e-9 abstol=1e-16 vntol=1e-12 itl1=5000`
    /// so that the printed digits are the solution's (set numdgt=15).
    #[rustfmt::skip]
    const NGSPICE_OP: &[(&str, &[f32], &[f64])] = &[
        ("default", &[0.5], &[
            1.200000000000e+01, -1.200000000000e+01, 0.0, -5.458605544078e-05,
            5.999999724186e+00, 5.944837518996e-05, -4.967331045111e-03, -5.516183704278e-07,
            -6.000000275810e+00, -6.055161744707e-05, -6.562543004684e-01, -9.803683289330e+00,
            -6.612666125363e-01, -6.563141278113e-01, -6.563739551589e-01, 2.733277339896e+00,
            -6.293048871339e+00, -1.050324960250e+01, -1.000326858151e+01, -6.818898037575e-03,
            -1.069598386399e+01, 0.0,
        ]),
        ("low", &[0.05], &[
            1.200000000000e+01, -1.200000000000e+01, 0.0, -5.461256805202e-06,
            5.999999724188e+00, 5.944837867238e-05, -5.461256804388e-04, -5.516148880450e-07,
            -6.000000275808e+00, -6.055161396468e-05, -6.562541371476e-01, -9.803479824631e+00,
            -6.568579686156e-01, -6.563139644897e-01, -6.563737918364e-01, 2.733290463565e+00,
            -6.295000290814e+00, -1.050305108909e+01, -1.000394550202e+01, -7.490190816756e-04,
            -1.069664328652e+01, 0.0,
        ]),
        ("high", &[1.0], &[
            1.200000000000e+01, -1.200000000000e+01, 0.0, -5.461202328255e-04,
            5.999999724188e+00, 5.944837867195e-05, -5.466663530583e-04, -5.516148884700e-07,
            -6.000000275808e+00, -6.055161396510e-05, -6.562541371676e-01, -9.803479849514e+00,
            -6.568585077523e-01, -6.563139645097e-01, -6.563737918564e-01, 2.733290461960e+00,
            -6.295000052173e+00, -1.050305111337e+01, -1.000394541924e+01, -7.497613759545e-04,
            -1.069664320587e+01, 0.0,
        ]),
        ("zero", &[0.0], &[
            1.200000000000e+01, -1.200000000000e+01, 0.0, -5.461529831595e-09,
            5.999999724188e+00, 5.944837910210e-05, -5.516145129911e-07, -5.516144583178e-07,
            -6.000000275808e+00, -6.055161353495e-05, -6.562541169939e-01, -9.803454717206e+00,
            -6.563139443359e-01, -6.563139443358e-01, -6.563737716825e-01, 2.733292083018e+00,
            -6.295241095144e+00, -1.050302659263e+01, -1.000402903399e+01, 9.895358768206e-11,
            -1.069672465882e+01, 0.0,
        ]),
    ];

    /// The Model's DC operating point against ngspice's, at every Setting and every named node.
    /// The Model has ngspice's thermal voltage (`netlist::VT`) and no gmin across the transistors'
    /// junctions, which ngspice has (1e-12 S). Measured: at most 0.60 uV (`dc`, Level 0); with
    /// gmin lowered to 1e-18 in ngspice every node agrees within 0.1 nV. The bound leaves 2.5
    /// times that. With the rounded thermal voltage this Solver had before (0.0258646 V) `dc` was
    /// 224 uV off and this test fails.
    #[test]
    fn operating_point_matches_ngspice() {
        const TOLERANCE_V: f64 = 1.5e-6;
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
