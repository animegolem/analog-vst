//! Netlist builder with named nodes and SPICE-equivalent part models.
//!
//! Each helper adds the same elements the matching SPICE line would simulate, so the model and
//! `reference/circuit.cir` describe one circuit.

use crate::circuit::{Bjt, Element, Node};

/// Thermal voltage kT/q at 27 C, exactly as ngspice 45 computes it at its default temperature:
/// `CONSTboltz / CHARGE` (`CONSTKoverQ`, src/main.c) times `27.0 + CONSTCtoK` (`REFTEMP`), with
/// `CONSTboltz` 1.38064852e-23, `CHARGE` 1.6021766208e-19 and `CONSTCtoK` 273.15 from
/// src/include/ngspice/const.h at the ngspice-45 tag. The arithmetic is written out so that the
/// value has ngspice's bits: 0.025864917007157463. Checked against the binary: an ideal diode
/// (IS = 1e-14, N = 1, RS = 0, gmin = 1e-30) forced to 0.6 V carries 1.187196289587074e-4 A in
/// ngspice 45, which gives this value back to 1 part in 1e16. The rounded 0.0258646 this Solver
/// used before is 1.2 parts in 1e5 lower, which put the Models' operating points up to 224 uV
/// from ngspice's.
pub const VT: f64 = (1.380_648_52e-23 / 1.602_176_620_8e-19) * (27.0 + 273.15);

/// SPICE `D` model parameters: saturation current, emission coefficient, series resistance.
/// `cjo` is added as a fixed capacitance (zero-bias junction capacitance).
#[derive(Clone, Copy, Debug)]
pub struct DiodeModel {
    pub is: f64,
    pub n: f64,
    pub rs: f64,
    pub cjo: f64,
}

/// SPICE Gummel-Poon `NPN`/`PNP` parameters used by the solver (charge storage and leakage omitted).
/// `rb`, `rc` and `re` of exactly 0 add no resistor and no internal node, as in ngspice.
#[derive(Clone, Copy, Debug)]
pub struct BjtModel {
    pub is: f64,
    pub bf: f64,
    pub br: f64,
    pub vaf: f64,
    pub ikf: f64,
    pub rb: f64,
    pub rc: f64,
    pub re: f64,
}

/// Single-pole op-amp: open-loop gain `a0` (V/V) and gain-bandwidth product `gbw` (Hz).
/// Matches the `opamp` subcircuit in the SPICE template.
#[derive(Clone, Copy, Debug)]
pub struct OpAmp {
    pub a0: f64,
    pub gbw: f64,
}

pub const D1N4148: DiodeModel = DiodeModel {
    is: 2.52e-9,
    n: 1.752,
    rs: 0.568,
    cjo: 4e-12,
};
pub const Q2N3904: BjtModel = BjtModel {
    is: 1e-14,
    bf: 300.0,
    br: 4.0,
    vaf: 100.0,
    ikf: 0.4,
    rb: 20.0,
    rc: 0.1,
    re: 0.1,
};
pub const JRC4558: OpAmp = OpAmp { a0: 1e5, gbw: 3e6 };

#[derive(Default)]
pub struct Netlist {
    names: Vec<String>,
    pub(crate) elements: Vec<Element>,
    pub(crate) pots: Vec<(&'static str, usize)>,
    pub(crate) output: Node,
    internal: usize,
}

impl Netlist {
    /// Node by name; "0" and "gnd" are ground.
    pub fn node(&mut self, name: &str) -> Node {
        if name == "0" || name.eq_ignore_ascii_case("gnd") {
            return 0;
        }
        match self.names.iter().position(|n| n == name) {
            Some(i) => i + 1,
            None => {
                self.names.push(name.to_string());
                self.names.len()
            }
        }
    }

    fn fresh(&mut self) -> Node {
        self.internal += 1;
        let name = format!("_internal{}", self.internal);
        self.node(&name)
    }

    pub fn resistor(&mut self, a: &str, b: &str, ohms: f64) {
        let (a, b) = (self.node(a), self.node(b));
        self.elements.push(Element::Resistor(a, b, ohms));
    }

    pub fn capacitor(&mut self, a: &str, b: &str, farads: f64) {
        let (a, b) = (self.node(a), self.node(b));
        self.elements.push(Element::Capacitor(a, b, farads));
    }

    pub fn inductor(&mut self, a: &str, b: &str, henries: f64) {
        let (a, b) = (self.node(a), self.node(b));
        self.elements.push(Element::Inductor(a, b, henries));
    }

    /// A resistor whose value comes from `model::pots` (a pot section or variable resistor).
    pub fn pot(&mut self, name: &'static str, a: &str, b: &str) {
        self.resistor(a, b, 1e3);
        self.pots.push((name, self.elements.len() - 1));
    }

    /// The audio input, a voltage source from `node` to ground.
    pub fn input(&mut self, node: &str) {
        let n = self.node(node);
        self.elements.push(Element::Input(n, 0));
    }

    /// The audio output, measured from `node` to ground.
    pub fn output(&mut self, node: &str) {
        self.output = self.node(node);
    }

    /// A supply: constant voltage from `node` to ground.
    pub fn supply(&mut self, node: &str, volts: f64) {
        let n = self.node(node);
        self.elements.push(Element::Dc(n, 0, volts));
    }

    /// One diode, with ngspice's reverse-bias law and GMIN (see `Device::Diode`). SPICE's `M` and
    /// `TT` are not modeled: the netlist's `.model` must set both to 0.
    pub fn diode(&mut self, anode: &str, cathode: &str, m: DiodeModel) {
        let (a, b) = (self.node(anode), self.node(cathode));
        let j = self.series_rs(a, m.rs);
        self.elements.push(Element::Diode {
            a: j,
            b,
            is: m.is,
            nvt: m.n * VT,
        });
        self.elements.push(Element::Capacitor(j, b, m.cjo));
    }

    /// Two identical diodes in antiparallel between `a` and `b`. One series resistance stands in
    /// for both diodes' RS, exact while only one diode conducts at a time.
    pub fn diode_pair(&mut self, a: &str, b: &str, m: DiodeModel) {
        let (a, b) = (self.node(a), self.node(b));
        let j = self.series_rs(a, m.rs);
        self.elements.push(Element::DiodePair {
            a: j,
            b,
            is: m.is,
            nvt: m.n * VT,
        });
        self.elements.push(Element::Capacitor(j, b, 2.0 * m.cjo));
    }

    /// Node behind a diode's series resistance (the node itself when there is none).
    fn series_rs(&mut self, a: Node, rs: f64) -> Node {
        if rs <= 0.0 {
            return a;
        }
        let j = self.fresh();
        self.elements.push(Element::Resistor(a, j, rs));
        j
    }

    pub fn npn(&mut self, c: &str, b: &str, e: &str, m: BjtModel) {
        self.bjt(c, b, e, m, false);
    }

    pub fn pnp(&mut self, c: &str, b: &str, e: &str, m: BjtModel) {
        self.bjt(c, b, e, m, true);
    }

    /// A transistor with its terminal resistances. A resistance of exactly 0 adds no resistor and
    /// no internal node: the device terminal is the external node, as in ngspice (RB, RC or RE of
    /// 0 each leave out one internal node there). A 1 milliohm stand-in is not harmless: its
    /// 1000 S puts terms of thousands of amperes into a Kirchhoff row, and their rounding stops
    /// the DC solve short of convergence.
    fn bjt(&mut self, c: &str, b: &str, e: &str, m: BjtModel, pnp: bool) {
        for ohms in [m.rb, m.rc, m.re] {
            assert!(ohms >= 0.0, "a terminal resistance is not negative or NaN: {ohms}");
        }
        let (c, b, e) = (self.node(c), self.node(b), self.node(e));
        let ci = if m.rc == 0.0 { c } else { self.fresh() };
        let bi = if m.rb == 0.0 { b } else { self.fresh() };
        let ei = if m.re == 0.0 { e } else { self.fresh() };
        let model = Bjt {
            is: m.is,
            bf: m.bf,
            br: m.br,
            vaf: m.vaf,
            ikf: m.ikf,
            vt: VT,
        };
        for (external, internal, ohms) in [(b, bi, m.rb), (c, ci, m.rc), (ei, e, m.re)] {
            if ohms != 0.0 {
                self.elements
                    .push(Element::Resistor(external, internal, ohms.max(1e-3)));
            }
        }
        self.elements.push(if pnp {
            Element::Pnp {
                c: ci,
                b: bi,
                e: ei,
                model,
            }
        } else {
            Element::Npn {
                c: ci,
                b: bi,
                e: ei,
                model,
            }
        });
    }

    /// Op-amp with ideal output, inputs drawing no current, no rail clipping.
    pub fn opamp(&mut self, plus: &str, minus: &str, out: &str, m: OpAmp) {
        let (p, n, o, x) = (self.node(plus), self.node(minus), self.node(out), self.fresh());
        // Transconductance into 1k || C gives gain a0 with its pole at gbw / a0.
        let r = 1e3;
        self.elements.extend([
            Element::Vccs {
                p: 0,
                n: x,
                cp: p,
                cn: n,
                gm: m.a0 / r,
            },
            Element::Resistor(x, 0, r),
            Element::Capacitor(x, 0, m.a0 / (2.0 * std::f64::consts::PI * m.gbw * r)),
            Element::Vcvs {
                p: o,
                n: 0,
                cp: x,
                cn: 0,
                gain: 1.0,
            },
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermal_voltage_is_the_one_ngspice_computes() {
        assert_eq!(VT.to_bits(), 0.025_864_917_007_157_463_f64.to_bits(), "{VT:e}");
        // The diode current that ngspice 45 prints for 0.6 V across IS = 1e-14, N = 1.
        let i = 1e-14 * ((0.6 / VT).exp() - 1.0);
        assert!((i / 1.187_196_289_587_074e-4 - 1.0).abs() < 1e-13, "{i:e}");
    }

    #[test]
    fn a_terminal_resistance_of_zero_adds_no_resistor_and_no_node() {
        let with = |rb: f64, rc: f64, re: f64| {
            let mut n = Netlist::default();
            n.npn("c", "b", "e", BjtModel { rb, rc, re, ..Q2N3904 });
            n
        };
        // All three present: three resistors, three internal nodes, in this order.
        let mut all = with(20.0, 0.1, 0.1);
        let (c, b, e) = (all.node("c"), all.node("b"), all.node("e"));
        assert_eq!(all.names.len(), 6);
        assert!(
            matches!(all.elements[..], [
                Element::Resistor(b0, 5, 20.0),
                Element::Resistor(c0, 4, 0.1),
                Element::Resistor(6, e0, 0.1),
                Element::Npn { c: 4, b: 5, e: 6, .. },
            ] if (c0, b0, e0) == (c, b, e)),
            "{:?}",
            all.elements
        );
        // RE = 0: the device emitter is the external node.
        let mut no_re = with(20.0, 0.1, 0.0);
        let e = no_re.node("e");
        assert_eq!(no_re.names.len(), 5);
        assert!(
            matches!(no_re.elements[..], [
                Element::Resistor(_, 5, 20.0),
                Element::Resistor(_, 4, 0.1),
                Element::Npn { c: 4, b: 5, e: e0, .. },
            ] if e0 == e),
            "{:?}",
            no_re.elements
        );
        // All zero: the device alone, on the external nodes.
        let mut none = with(0.0, 0.0, 0.0);
        let (c, b, e) = (none.node("c"), none.node("b"), none.node("e"));
        assert_eq!(none.names.len(), 3);
        assert!(
            matches!(none.elements[..], [Element::Npn { c: c0, b: b0, e: e0, .. }] if (c0, b0, e0) == (c, b, e)),
            "{:?}",
            none.elements
        );
    }
}
