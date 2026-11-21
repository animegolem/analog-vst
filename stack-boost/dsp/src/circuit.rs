//! Nodal DK-method circuit solver (Holters & Zölzer, DAFx 2015).
//!
//! A netlist of linear elements and nonlinear devices is discretized with the trapezoidal rule and
//! reduced to a state-space system. Each sample costs one matrix-vector product plus a Newton solve
//! over the nonlinear devices' port voltages only. Devices that do not load each other (for example
//! stages separated by op-amps) are solved one after another rather than jointly.
//!
//! The circuit starts at its DC operating point, found with a SPICE-style Newton solve. Variable
//! resistors (pots) can change at runtime; `update` rebuilds the matrices without allocating.

// Matrix code reads more clearly with explicit row and column indices.
#![allow(clippy::needless_range_loop)]

/// Bound on the joint Newton solve's iterations per sample. A solve that uses them all has
/// failed: `tick` holds the circuit at its last valid state for that sample.
const JOINT_MAX_ITERATIONS: usize = 200;

/// The joint Newton solve stops when no port's Newton correction reaches this much (volts).
const JOINT_TOLERANCE_V: f64 = 1e-9;

/// Bound on the rounds of per-device solves per sample when devices are solved one at a time.
const SWEEP_MAX_ROUNDS: usize = 32;

/// Rounds of per-device solves stop when `sweep_error` is under this much (volts). The looser
/// 1e-7 this Solver once had left spurious third and fifth harmonics near -79 dBc in the Class A
/// Transistor Drive, the one Project that repeats rounds.
const SWEEP_TOLERANCE_V: f64 = 1e-12;

/// Bound on one device's Newton iterations in a per-device solve.
const DEVICE_MAX_ITERATIONS: usize = 64;

/// Bound on the corrections the pin makes to one row (see `pin_operating_point`). The loop
/// stops earlier only when a row gives the rest value exactly.
const PIN_MAX_ROUNDS: usize = 4;

/// Conductance standing in for an inductor at DC.
const INDUCTOR_DC_SHORT: f64 = 1e9;

/// SPICE's default GMIN: the conductance ngspice puts across every pn junction. Applied to
/// `Device::Diode` only; the diode pair and the transistors do not have it.
const GMIN: f64 = 1e-12;

/// Largest last Newton step at which the DC operating point counts as converged. The step is the
/// largest change in any unknown: node voltages in volts and voltage-source currents in amperes,
/// taken together. This bound, `MAX_DC_RESIDUAL` and `MAX_PIN_VOLTS` are the acceptance criteria;
/// none is a bound on the operating point's error. As a local heuristic only (exact arithmetic,
/// one junction, quadratic convergence), a step this size leaves about step^2 / (2 Vt) = 2e-13 V.
/// The step cannot be asked to reach zero: the DC equations are evaluated in double precision,
/// and the steps can stop shrinking first.
///
/// Measured by the test `dc_bounds_over_controls_and_rates` on each of the eight Models (4096
/// seeded Control positions, one for the Model with no Control, at the Circuit rates of 44.1, 48,
/// 96 and 192 kHz), the worst last step was: Green Diode Overdrive 2.0e-9, Three-Band Stack Boost
/// 3.4e-10, Class A Transistor Drive 4.3e-11, Four Stage Sustainer 2.2e-11, Feedback Pair Drive
/// 6.9e-13, Three-Transistor Drive 4.0e-14, Seven Band Graphic and Precision Rectifier 0. The
/// bound is 50 times the worst of them. At their worst position the first four do not reach the
/// 1e-12 at which the loop stops early.
///
/// While `Netlist` entered a transistor's RE of 0 as 1 milliohm, the Class A Transistor Drive's
/// solve stopped shrinking far earlier: the same scan measured 5.6e-8 at Level rotation
/// 0.2496256, 1.8 times under the bound.
const MAX_DC_STEP: f64 = 1e-7;

/// Largest `dc_residual` at which the operating point counts as a solution of the DC equations.
/// Measured over the same positions, the worst was: Class A Transistor Drive 2.8e-14, Four Stage
/// Sustainer 5.5e-15, Feedback Pair Drive 1.5e-15, Three-Transistor Drive 5.1e-16, Three-Band
/// Stack Boost 2.6e-16, Green Diode Overdrive 2.5e-16, the other two 0. The Class A Transistor
/// Drive's worst row is the collector node behind one transistor's RC of 1.61 ohm: two terms of
/// 2.55 mA that cancel to 1.4e-16 A. The DC equations do not contain the sample rate, and the
/// values were the same at every rate. The fault-injection test `wrong_operating_point_is_not_pinned` solves an
/// emitter follower with 1e-13 A too much flowing into its bias node, 0.5 nV off, and measures
/// 5.6e-11; with 100 nA, 0.5 mV off, it measures 5.6e-5. The bound is 35.8 times the worst
/// residual of that scan and 56 times under that of the smaller fault. The scan's worst is not
/// a ceiling: 50,000 Level positions of the Class A Transistor Drive at one rate gave 2.93e-14,
/// 34 times under the bound (and a last step of 4.6e-11); the two other Models with one Control
/// gave the scan's own worst values again.
const MAX_DC_RESIDUAL: f64 = 1e-12;

/// Largest distance, in volts, between a row of the discrete system at the operating point and
/// that operating point, before the pin removes it. A gross-mismatch check only: a row this far
/// from rest means the DC solve and the discrete system describe different circuits. It cannot
/// tell a wrong operating point from a right one, because a slow capacitor turns a wrong DC
/// voltage into a small miss per sample: the test `wrong_operating_point_is_not_pinned` builds a
/// point 0.5 mV off that passes this bound. `dc_residual` is the check for that. Measured over
/// the same positions, the largest distance was 1.2e-8 V (Green Diode Overdrive), 87 times
/// under the bound; the next is the Feedback Pair Drive's 5.6e-10 V.
const MAX_PIN_VOLTS: f64 = 1e-6;

/// Node index; 0 is ground.
pub type Node = usize;

#[derive(Clone, Copy, Debug)]
pub struct Bjt {
    pub is: f64,
    pub bf: f64,
    pub br: f64,
    /// Forward Early voltage.
    pub vaf: f64,
    /// Forward high-injection knee current.
    pub ikf: f64,
    pub vt: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum Element {
    Resistor(Node, Node, f64),
    Capacitor(Node, Node, f64),
    Inductor(Node, Node, f64),
    /// The circuit input: v(a) - v(b) equals the input sample.
    Input(Node, Node),
    /// Constant voltage source: v(a) - v(b) = volts.
    Dc(Node, Node, f64),
    /// Current `gm * (v(cp) - v(cn))` flowing from `p` to `n` through the source (SPICE G).
    Vccs {
        p: Node,
        n: Node,
        cp: Node,
        cn: Node,
        gm: f64,
    },
    /// `v(p) - v(n) = gain * (v(cp) - v(cn))` (SPICE E).
    Vcvs {
        p: Node,
        n: Node,
        cp: Node,
        cn: Node,
        gain: f64,
    },
    /// Antiparallel pair of identical diodes: `i = 2 Is sinh(v / (N Vt))` from `a` to `b`.
    DiodePair {
        a: Node,
        b: Node,
        is: f64,
        nvt: f64,
    },
    /// Single diode, anode `a`, cathode `b`: `i = Is (exp(v / (N Vt)) - 1)`, with ngspice's
    /// reverse-bias law and GMIN (see `Device::eval`).
    Diode {
        a: Node,
        b: Node,
        is: f64,
        nvt: f64,
    },
    /// NPN transistor, Gummel-Poon without charge storage or leakage. Add terminal resistances
    /// (RB, RC, RE) as resistors.
    Npn {
        c: Node,
        b: Node,
        e: Node,
        model: Bjt,
    },
    /// PNP transistor, same model with all junction polarities reversed.
    Pnp {
        c: Node,
        b: Node,
        e: Node,
        model: Bjt,
    },
}

#[derive(Clone, Copy)]
enum Device {
    DiodePair {
        is: f64,
        nvt: f64,
    },
    Diode {
        is: f64,
        nvt: f64,
    },
    /// `polarity` is 1 for NPN, -1 for PNP.
    Bjt {
        model: Bjt,
        polarity: f64,
    },
}

impl Device {
    fn ports(&self) -> usize {
        match self {
            Device::DiodePair { .. } | Device::Diode { .. } => 1,
            Device::Bjt { .. } => 2,
        }
    }

    /// Whether a Newton step this small leaves negligible error. Convergence is quadratic in the
    /// junction voltages: after a step under 10 uV the remaining error is about step^2 / (2 N Vt),
    /// under 2 nV. A reverse-biased junction is nearly linear, so its step does not matter.
    #[inline]
    fn converged(&self, v: &[f64], step: &[f64]) -> bool {
        const TOL: f64 = 1e-5;
        match self {
            Device::DiodePair { .. } => step[0].abs() < TOL,
            Device::Diode { .. } => step[0].abs() < TOL || v[0] < -0.2,
            Device::Bjt { polarity, .. } => {
                let (vbc, step_vbc) = (polarity * (v[1] - v[0]), step[1] - step[0]);
                step[1].abs() < TOL && (vbc < -0.2 || step_vbc.abs() < TOL)
            }
        }
    }

    /// Limits a proposed move of the port voltages from `old` to `new` so that no junction is
    /// thrown far up its exponential (SPICE's junction limiting). Returns whether `new` changed.
    /// A Newton step, or an extrapolated first guess, that lands a junction volts into forward
    /// bias asks for a current of gigaamps; the iteration then creeps back by about one thermal
    /// voltage per step and can run out of steps.
    #[inline]
    fn limit(&self, old: &[f64], new: &mut [f64]) -> bool {
        match *self {
            Device::Diode { is, nvt } => {
                let v = limit_junction(new[0], old[0], nvt, critical_voltage(is, nvt));
                let limited = v != new[0];
                new[0] = v;
                limited
            }
            // The pair conducts in both directions: limit the junction that is being driven.
            Device::DiodePair { is, nvt } => {
                let sign = if new[0] < 0.0 { -1.0 } else { 1.0 };
                let v = sign * limit_junction(sign * new[0], sign * old[0], nvt, critical_voltage(is, nvt));
                let limited = v != new[0];
                new[0] = v;
                limited
            }
            // Junction voltages: vbe = s v[1], vbc = s (v[1] - v[0]).
            Device::Bjt { model: m, polarity: s } => {
                let vcrit = critical_voltage(m.is, m.vt);
                let (vbe_new, vbc_new) = (s * new[1], s * (new[1] - new[0]));
                let vbe = limit_junction(vbe_new, s * old[1], m.vt, vcrit);
                let vbc = limit_junction(vbc_new, s * (old[1] - old[0]), m.vt, vcrit);
                if vbe == vbe_new && vbc == vbc_new {
                    return false;
                }
                new[1] = s * vbe;
                new[0] = s * (vbe - vbc);
                true
            }
        }
    }

    /// Port currents and their Jacobian (row-major, ports x ports) at port voltages `v`.
    #[inline]
    fn eval(&self, v: &[f64], i: &mut [f64; 2], j: &mut [f64; 4]) {
        match *self {
            Device::DiodePair { is, nvt } => {
                let ep = (v[0] / nvt).clamp(-200.0, 200.0).exp();
                let en = 1.0 / ep;
                i[0] = is * (ep - en);
                j[0] = is / nvt * (ep + en);
            }
            // ngspice's diode (dioload.c, no breakdown): the exponential law down to -3 N Vt, below
            // that -Is (1 + (3 N Vt / (e v))^3), and GMIN across the junction in both regions. The
            // two branches meet in value at -3 N Vt. Measured on the Precision Rectifier (Is =
            // 2.52 nA into 100k and 200k resistors): the difference from the plain exponential is
            // microvolts, which is 0.03 dB of what the harness measures as gain at 100 Hz, -40 dBFS.
            Device::Diode { is, nvt } => {
                if v[0] >= -3.0 * nvt {
                    let e = (v[0] / nvt).min(200.0).exp();
                    i[0] = is * (e - 1.0) + GMIN * v[0];
                    j[0] = is / nvt * e + GMIN;
                } else {
                    let arg = (3.0 * nvt / (v[0] * std::f64::consts::E)).powi(3);
                    i[0] = -is * (1.0 + arg) + GMIN * v[0];
                    j[0] = is * 3.0 * arg / v[0] + GMIN;
                }
            }
            // Ports: (c, e) carries Ic, (b, e) carries Ib. vbc = vbe - vce. A PNP is an NPN with
            // voltages and currents negated; the Jacobian is unchanged by the double negation.
            Device::Bjt { model: m, polarity: s } => {
                let (vce, vbe) = (s * v[0], s * v[1]);
                let vbc = vbe - vce;
                let ef = (vbe / m.vt).min(80.0).exp();
                let er = (vbc / m.vt).min(80.0).exp();
                let (if_, ir) = (m.is * (ef - 1.0), m.is * (er - 1.0));
                let (gf, gr) = (m.is * ef / m.vt, m.is * er / m.vt);
                let q1 = 1.0 / (1.0 - vbc / m.vaf);
                let dq1_dvbc = q1 * q1 / m.vaf;
                let sq = (1.0 + 4.0 * if_ / m.ikf).max(0.0).sqrt();
                let qb = q1 * (1.0 + sq) / 2.0;
                let dqb_dvbe = q1 * gf / m.ikf / sq;
                let dqb_dvbc = dq1_dvbc * (1.0 + sq) / 2.0;
                let ic = (if_ - ir) / qb - ir / m.br;
                let dic_dvbe = gf / qb - (if_ - ir) / (qb * qb) * dqb_dvbe;
                let dic_dvbc = -gr / qb - (if_ - ir) / (qb * qb) * dqb_dvbc - gr / m.br;
                let ib = if_ / m.bf + ir / m.br;
                let (dib_dvbe, dib_dvbc) = (gf / m.bf, gr / m.br);
                i[0] = s * ic;
                i[1] = s * ib;
                // d/dvce = -d/dvbc; d/dvbe (port) = d/dvbe + d/dvbc.
                j[0] = -dic_dvbc;
                j[1] = dic_dvbe + dic_dvbc;
                j[2] = -dib_dvbc;
                j[3] = dib_dvbe + dib_dvbc;
            }
        }
    }
}

/// Junction voltage above which limiting applies: where the exponential's curvature is greatest
/// (SPICE's `vcrit`). `nvt` is the emission coefficient times the thermal voltage.
#[inline]
fn critical_voltage(is: f64, nvt: f64) -> f64 {
    nvt * (nvt / (std::f64::consts::SQRT_2 * is)).ln()
}

/// SPICE's `pnjlim`: a junction moving from `old` to `new` above `vcrit` by more than two thermal
/// voltages goes only as far as a logarithmic step. Moves below `vcrit` pass unchanged.
#[inline]
fn limit_junction(new: f64, old: f64, nvt: f64, vcrit: f64) -> f64 {
    if new <= vcrit || (new - old).abs() <= 2.0 * nvt {
        return new;
    }
    if old <= 0.0 {
        return nvt * (new / nvt).ln();
    }
    let arg = 1.0 + (new - old) / nvt;
    if arg > 0.0 {
        old + nvt * arg.ln()
    } else {
        vcrit
    }
}

/// Dense row-major matrix.
#[derive(Clone)]
struct Mat {
    cols: usize,
    data: Vec<f64>,
}

impl Mat {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            cols,
            data: vec![0.0; rows * cols],
        }
    }
    fn at(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }
    fn at_mut(&mut self, r: usize, c: usize) -> &mut f64 {
        &mut self.data[r * self.cols + c]
    }
    fn row(&self, r: usize) -> &[f64] {
        &self.data[r * self.cols..(r + 1) * self.cols]
    }
}

#[derive(Clone)]
pub struct Circuit {
    elements: Vec<Element>,
    output: (Node, Node),
    nodes: usize,
    size: usize,
    rate: f64,
    dirty: bool,
    caps: Vec<(Node, Node, f64)>,
    ports: Vec<(Node, Node)>,
    /// Devices with their first port index, in solve order.
    devices: Vec<(Device, usize)>,
    /// True when one sweep of per-device solves is enough; otherwise sweeps repeat until the device
    /// currents agree (block Gauss-Seidel).
    sequential: bool,
    /// True when devices only depend on earlier ones, so later currents start each sample at zero.
    zero_currents: bool,
    /// True when devices load each other strongly enough to need one joint Newton solve.
    joint: bool,
    jac: Mat,
    input_row: usize,
    // Workspace for `update`.
    s: Mat,
    sinv: Mat,
    perm: Vec<usize>,
    w: Mat,
    // State-space over z = [x, u, 1, i]. `lin` is column-major with nx + 1 + nn rows: capacitor
    // voltages scaled by 2 Gc (so the state update is x' = row - x), the output, then the port
    // voltages. `port_v` repeats the port rows for device coupling.
    lin: Vec<f64>,
    port_v: Mat,
    /// State update x' = gc2 * v + beta * x per reactive element.
    gc2: Vec<f64>,
    beta: Vec<f64>,
    // Runtime state.
    x: Vec<f64>,
    z: Vec<f64>,
    acc: Vec<f64>,
    v: Vec<f64>,
    v_prev: Vec<f64>,
    i_used: Vec<f64>,
    i_prev: Vec<f64>,
    /// The DC solution found at construction: node voltages, then voltage-source currents.
    operating_point: Vec<f64>,
    /// Output of the last sample whose solve succeeded; a failed sample repeats it.
    y: f64,
    /// Samples whose solve failed and were held at the last valid state. Saturates.
    failed_solves: u64,
    /// Failed samples in a row. At `MAX_CONSECUTIVE_HELD` the circuit goes back to `rest`.
    consecutive_held: u32,
    /// Times the circuit went back to `rest` because it could not leave a held state. Saturates.
    rest_restores: u64,
    /// The state the DC operating point leaves, kept so that `tick` can return to it without
    /// allocating: capacitor history, port voltages, device currents, and the output. Solved once,
    /// at construction: after a pot has moved it is the operating point of the netlist as built,
    /// not of the netlist as it is now.
    rest: Vec<f64>,
    /// Iterations the joint solve may use, at most `JOINT_MAX_ITERATIONS`. Only tests lower it.
    joint_iterations: usize,
}

/// Failed samples in a row after which the circuit is put back at its DC operating point. Solves
/// that fail on a few samples and then recover (44 scattered over 49,000 for a 5e6 V sine into
/// the Feedback Pair Drive) never reach it; a state that no sample solves from would otherwise be
/// held for ever.
const MAX_CONSECUTIVE_HELD: u32 = 1024;

impl Circuit {
    pub fn new(elements: Vec<Element>, output: (Node, Node), rate: f64) -> Self {
        let node_max = |e: &Element| match *e {
            Element::Resistor(a, b, _)
            | Element::Capacitor(a, b, _)
            | Element::Inductor(a, b, _)
            | Element::Input(a, b)
            | Element::Dc(a, b, _) => a.max(b),
            Element::Vccs { p, n, cp, cn, .. } | Element::Vcvs { p, n, cp, cn, .. } => p.max(n).max(cp).max(cn),
            Element::DiodePair { a, b, .. } | Element::Diode { a, b, .. } => a.max(b),
            Element::Npn { c, b, e, .. } | Element::Pnp { c, b, e, .. } => c.max(b).max(e),
        };
        let nodes = elements
            .iter()
            .map(node_max)
            .max()
            .unwrap_or(0)
            .max(output.0)
            .max(output.1);
        let extra = elements
            .iter()
            .filter(|e| matches!(e, Element::Input(..) | Element::Dc(..) | Element::Vcvs { .. }))
            .count();
        let inputs = elements.iter().filter(|e| matches!(e, Element::Input(..))).count();
        assert_eq!(inputs, 1, "a circuit needs exactly one input");

        // Reactive elements as trapezoidal companions: conductance g and history state x, with
        // current a->b of g v - x. Capacitors update x' = 2g v - x; inductors x' = -2g v + x.
        let caps: Vec<(Node, Node, f64)> = elements
            .iter()
            .filter_map(|e| match *e {
                Element::Capacitor(a, b, c) => Some((a, b, 2.0 * c * rate)),
                Element::Inductor(a, b, l) => Some((a, b, 1.0 / (2.0 * l * rate))),
                _ => None,
            })
            .collect();
        let beta: Vec<f64> = elements
            .iter()
            .filter_map(|e| match e {
                Element::Capacitor(..) => Some(-1.0),
                Element::Inductor(..) => Some(1.0),
                _ => None,
            })
            .collect();
        let (mut ports, mut devices) = (Vec::new(), Vec::new());
        for el in &elements {
            match *el {
                Element::DiodePair { a, b, is, nvt } => {
                    devices.push((Device::DiodePair { is, nvt }, ports.len()));
                    ports.push((a, b));
                }
                Element::Diode { a, b, is, nvt } => {
                    devices.push((Device::Diode { is, nvt }, ports.len()));
                    ports.push((a, b));
                }
                Element::Npn { c, b, e, model } | Element::Pnp { c, b, e, model } => {
                    let polarity = if matches!(el, Element::Pnp { .. }) { -1.0 } else { 1.0 };
                    devices.push((Device::Bjt { model, polarity }, ports.len()));
                    ports.extend([(c, e), (b, e)]);
                }
                _ => {}
            }
        }
        let (size, nx, nn) = (nodes + extra, caps.len(), ports.len());
        let m = nx + 2 + nn;
        let mut circuit = Self {
            output,
            nodes,
            size,
            rate,
            dirty: true,
            sequential: true,
            zero_currents: true,
            joint: false,
            jac: Mat::new(nn, nn + 1),
            input_row: 0,
            s: Mat::new(size, size),
            sinv: Mat::new(size, size),
            perm: vec![0; size],
            w: Mat::new(size, m),
            lin: vec![0.0; (nx + 1 + nn) * m],
            port_v: Mat::new(nn, m),
            gc2: caps.iter().zip(&beta).map(|(&(_, _, g), &b)| -2.0 * b * g).collect(),
            beta,
            x: vec![0.0; nx],
            z: vec![0.0; m],
            acc: vec![0.0; nx + 1 + nn],
            v: vec![0.0; nn],
            v_prev: vec![0.0; nn],
            i_used: vec![0.0; nn],
            i_prev: vec![0.0; nn],
            operating_point: vec![0.0; size],
            y: 0.0,
            failed_solves: 0,
            consecutive_held: 0,
            rest_restores: 0,
            rest: vec![0.0; nx + 2 * nn + 1],
            joint_iterations: JOINT_MAX_ITERATIONS,
            caps,
            ports,
            devices,
            elements,
        };
        circuit.update();
        circuit.order_devices();
        let last_step = circuit.solve_operating_point();
        assert!(
            last_step < MAX_DC_STEP,
            "the DC operating point did not converge: the last Newton step was {last_step} V"
        );
        // `tick` keeps the state finite from here on; construction runs off the audio thread.
        assert!(
            circuit.rest.iter().all(|v| v.is_finite()),
            "operating point is not finite"
        );
        circuit.pin_operating_point();
        circuit
    }

    /// Changes the value of the resistor at `index` in the netlist. Takes effect on the next sample.
    pub fn set_resistor(&mut self, index: usize, ohms: f64) {
        if let Element::Resistor(_, _, r) = &mut self.elements[index] {
            if *r != ohms {
                *r = ohms;
                self.dirty = true;
            }
        } else {
            panic!("element {index} is not a resistor");
        }
    }

    /// Samples since construction whose solve failed: the Newton iteration ran out of steps or
    /// the result was not finite. Each was held at the last valid state and output.
    pub fn failed_solves(&self) -> u64 {
        self.failed_solves
    }

    /// Lowers the joint solve's iteration bound, to force the failure path in a test.
    #[cfg(test)]
    pub(crate) fn limit_joint_iterations(&mut self, iterations: usize) {
        assert!(self.joint, "only a jointly solved circuit has this bound");
        assert!((1..=JOINT_MAX_ITERATIONS).contains(&iterations));
        self.joint_iterations = iterations;
    }

    /// The DC operating point found at construction: node voltages (index = node - 1), then the
    /// current of each voltage source.
    #[cfg(test)]
    pub fn operating_point(&self) -> &[f64] {
        &self.operating_point
    }

    /// Fills the MNA matrix. Capacitors stamp their companion conductance, or are open when `dc`.
    fn stamp_all(&mut self, dc: bool) {
        self.s.data.fill(0.0);
        let s = &mut self.s;
        let mut extra = self.nodes;
        for e in &self.elements {
            match *e {
                Element::Resistor(a, b, r) => conductance(s, a, b, 1.0 / r),
                Element::Capacitor(a, b, c) => {
                    if !dc {
                        conductance(s, a, b, 2.0 * c * self.rate)
                    }
                }
                // At DC an inductor is a short; a large conductance keeps the matrix shape.
                Element::Inductor(a, b, l) => conductance(
                    s,
                    a,
                    b,
                    if dc {
                        INDUCTOR_DC_SHORT
                    } else {
                        1.0 / (2.0 * l * self.rate)
                    },
                ),
                Element::Vccs { p, n, cp, cn, gm } => {
                    stamp(s, p, cp, gm);
                    stamp(s, p, cn, -gm);
                    stamp(s, n, cp, -gm);
                    stamp(s, n, cn, gm);
                }
                Element::Input(a, b) | Element::Dc(a, b, _) | Element::Vcvs { p: a, n: b, .. } => {
                    extra += 1;
                    for (node, sign) in [(a, 1.0), (b, -1.0)] {
                        stamp(s, node, extra, sign);
                        stamp(s, extra, node, sign);
                    }
                    match *e {
                        Element::Vcvs { cp, cn, gain, .. } => {
                            stamp(s, extra, cp, -gain);
                            stamp(s, extra, cn, gain);
                        }
                        Element::Input(..) => self.input_row = extra - 1,
                        _ => {}
                    }
                }
                Element::DiodePair { .. } | Element::Diode { .. } | Element::Npn { .. } | Element::Pnp { .. } => {}
            }
        }
    }

    /// Value of the DC column of the right-hand side at matrix row `r`.
    fn dc_rhs(&self, r: usize) -> f64 {
        let mut extra = self.nodes;
        for e in &self.elements {
            if let Element::Input(..) | Element::Dc(..) | Element::Vcvs { .. } = e {
                if extra == r {
                    return if let Element::Dc(_, _, v) = e { *v } else { 0.0 };
                }
                extra += 1;
            }
        }
        0.0
    }

    /// Rebuilds the state-space matrices from the netlist. Allocation-free.
    fn update(&mut self) {
        self.dirty = false;
        self.stamp_all(false);
        invert(&mut self.s, &mut self.sinv, &mut self.perm);

        // W = S^-1 J; J's columns inject capacitor history, the input, DC sources and port currents.
        let (n, nx, nn) = (self.size, self.caps.len(), self.ports.len());
        let col_diff = |sinv: &Mat, r: usize, a: Node, b: Node| node_val(sinv.row(r), a) - node_val(sinv.row(r), b);
        for r in 0..n {
            for (k, &(a, b, _)) in self.caps.iter().enumerate() {
                *self.w.at_mut(r, k) = col_diff(&self.sinv, r, a, b);
            }
            *self.w.at_mut(r, nx) = self.sinv.at(r, self.input_row);
            let mut dc = 0.0;
            for c in self.nodes..n {
                let v = self.dc_rhs(c);
                if v != 0.0 {
                    dc += self.sinv.at(r, c) * v;
                }
            }
            *self.w.at_mut(r, nx + 1) = dc;
            for (k, &(a, b)) in self.ports.iter().enumerate() {
                *self.w.at_mut(r, nx + 2 + k) = -col_diff(&self.sinv, r, a, b);
            }
        }
        let row_diff = |w: &Mat, c: usize, a: Node, b: Node| {
            (if a > 0 { w.at(a - 1, c) } else { 0.0 }) - (if b > 0 { w.at(b - 1, c) } else { 0.0 })
        };
        let rows = nx + 1 + nn;
        for c in 0..nx + 2 + nn {
            for (k, &(a, b, _)) in self.caps.iter().enumerate() {
                self.lin[c * rows + k] = self.gc2[k] * row_diff(&self.w, c, a, b);
            }
            self.lin[c * rows + nx] = row_diff(&self.w, c, self.output.0, self.output.1);
            for (k, &(a, b)) in self.ports.iter().enumerate() {
                let v = row_diff(&self.w, c, a, b);
                self.lin[c * rows + nx + 1 + k] = v;
                *self.port_v.at_mut(k, c) = v;
            }
        }
    }

    /// Orders devices along their strong dependencies (upstream first), so block Gauss-Seidel
    /// sweeps only have to absorb weak feedback, such as currents through a shared bias node.
    fn order_devices(&mut self) {
        let base = self.caps.len() + 2;
        let nd = self.devices.len();
        let span = |d: &(Device, usize)| d.1..d.1 + d.0.ports();
        // coupling(a, b): largest effect of device b's currents on device a's port voltages,
        // relative to a's own driving-point impedance.
        let coupling = |a: usize, b: usize, devs: &[(Device, usize)], pv: &Mat| {
            span(&devs[a])
                .map(|r| {
                    let own = span(&devs[a]).map(|c| pv.at(r, base + c).abs()).fold(1e-30, f64::max);
                    span(&devs[b]).map(|c| pv.at(r, base + c).abs()).fold(0.0, f64::max) / own
                })
                .fold(0.0, f64::max)
        };
        let mut order: Vec<usize> = Vec::with_capacity(nd);
        let mut placed = vec![false; nd];
        while order.len() < nd {
            // Next: the unplaced device least driven by other unplaced devices.
            let next = (0..nd)
                .filter(|&a| !placed[a])
                .min_by(|&a, &b| {
                    let drive = |x: usize| {
                        (0..nd)
                            .filter(|&y| y != x && !placed[y])
                            .map(|y| coupling(x, y, &self.devices, &self.port_v))
                            .fold(0.0, f64::max)
                    };
                    drive(a).total_cmp(&drive(b))
                })
                .unwrap();
            placed[next] = true;
            order.push(next);
        }
        // One sweep suffices when devices only depend on earlier ones, or when every reverse
        // coupling's loop gain (a Gauss-Seidel sweep's convergence factor) is tiny.
        let order_ref = &order;
        let pairs = || (0..nd).flat_map(move |i| (i + 1..nd).map(move |j| (order_ref[i], order_ref[j])));
        let exact = pairs().all(|(a, b)| coupling(a, b, &self.devices, &self.port_v) < 1e-12);
        let loop_gain = pairs()
            .map(|(a, b)| coupling(a, b, &self.devices, &self.port_v) * coupling(b, a, &self.devices, &self.port_v))
            .fold(0.0, f64::max);
        self.zero_currents = exact;
        self.sequential = exact || loop_gain < 1e-3;
        // Strong mutual feedback (e.g. two-transistor fuzz circuits): solve all ports jointly.
        self.joint = !exact && loop_gain >= 0.1;
        self.devices = order.iter().map(|&i| self.devices[i]).collect();
    }

    /// Times since construction that the circuit was put back at its DC operating point because
    /// `MAX_CONSECUTIVE_HELD` samples in a row could not be solved from the state it was in.
    pub fn rest_restores(&self) -> u64 {
        self.rest_restores
    }

    /// SPICE-style DC operating point: Newton on the full nodal equations with capacitors open,
    /// stepping the DC sources up from zero. Sets the capacitor states and port voltages.
    ///
    /// Returns the size of the last Newton step with the sources at full value (volts at nodes,
    /// amperes in voltage sources), or infinity if that step was damped. The caller decides whether
    /// that is converged; the point is wherever the last step left it.
    fn solve_operating_point(&mut self) -> f64 {
        let n = self.size;
        self.stamp_all(true);
        let s_dc = self.s.clone();
        let mut z = vec![0.0; n];
        let mut jac = Mat::new(n, n);
        let mut inv = Mat::new(n, n);
        let mut perm = vec![0; n];
        let mut residual = vec![0.0; n];
        let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
        let mut last_step = f64::INFINITY;
        let port_voltage = |z: &[f64], (a, b): (Node, Node)| node_val(z, a) - node_val(z, b);
        for step in 1..=20 {
            let lambda = step as f64 / 20.0;
            for _ in 0..200 {
                // F = S z + sum over ports of (e_a - e_b) i - lambda b; J = S + port Jacobian stamps.
                jac.data.copy_from_slice(&s_dc.data);
                for r in 0..n {
                    residual[r] = (0..n).map(|c| s_dc.at(r, c) * z[c]).sum::<f64>() - lambda * self.dc_rhs(r);
                }
                for &(dev, first) in &self.devices {
                    let np = dev.ports();
                    let mut v = [0.0; 2];
                    for k in 0..np {
                        v[k] = port_voltage(&z, self.ports[first + k]);
                    }
                    dev.eval(&v[..np], &mut i, &mut j);
                    for k in 0..np {
                        let (a, b) = self.ports[first + k];
                        add_node(&mut residual, a, i[k]);
                        add_node(&mut residual, b, -i[k]);
                        for l in 0..np {
                            let (c, d) = self.ports[first + l];
                            let g = j[k * np + l];
                            for (row, rs) in [(a, 1.0), (b, -1.0)] {
                                for (col, cs) in [(c, 1.0), (d, -1.0)] {
                                    stamp(&mut jac, row, col, rs * cs * g);
                                }
                            }
                        }
                    }
                }
                invert(&mut jac, &mut inv, &mut perm);
                let delta: Vec<f64> = (0..n)
                    .map(|r| (0..n).map(|c| inv.at(r, c) * residual[c]).sum())
                    .collect();
                let max_port_step = self
                    .ports
                    .iter()
                    .map(|&p| port_voltage(&delta, p).abs())
                    .fold(0.0, f64::max);
                let damp = if max_port_step > 0.1 { 0.1 / max_port_step } else { 1.0 };
                for (zi, d) in z.iter_mut().zip(&delta) {
                    *zi -= damp * d;
                }
                // A NaN anywhere makes the step NaN, which is under no bound.
                let step_size = delta
                    .iter()
                    .fold(0.0, |m: f64, d| if d.abs() > m || d.is_nan() { d.abs() } else { m });
                last_step = if damp == 1.0 { step_size } else { f64::INFINITY };
                if last_step < 1e-12 {
                    break;
                }
            }
        }
        self.operating_point.copy_from_slice(&z);
        self.y = node_val(&z, self.output.0) - node_val(&z, self.output.1);
        for (k, &(a, b, g)) in self.caps.iter().enumerate() {
            let v = node_val(&z, a) - node_val(&z, b);
            // Steady state: no capacitor current (x = g v); inductor current i = g v - x carries the
            // DC current through the short.
            self.x[k] = if self.beta[k] < 0.0 {
                g * v
            } else {
                g * v - INDUCTOR_DC_SHORT * v
            };
        }
        for (k, &p) in self.ports.iter().enumerate() {
            self.v[k] = port_voltage(&z, p);
        }
        self.v_prev.copy_from_slice(&self.v);
        let base = self.caps.len() + 2;
        for &(dev, first) in &self.devices {
            let np = dev.ports();
            dev.eval(&self.v[first..first + np], &mut i, &mut j);
            self.z[base + first..base + first + np].copy_from_slice(&i[..np]);
            self.i_prev[first..first + np].copy_from_slice(&i[..np]);
        }
        let (nx, nn) = (self.caps.len(), self.ports.len());
        assert_eq!(self.rest.len(), nx + 2 * nn + 1);
        self.rest[..nx].copy_from_slice(&self.x);
        self.rest[nx..nx + nn].copy_from_slice(&self.v);
        self.rest[nx + nn..nx + 2 * nn].copy_from_slice(&self.z[base..]);
        self.rest[nx + 2 * nn] = self.y;
        last_step
    }

    /// How far the stored operating point is from solving the DC equations of the netlist as it is
    /// now: the worst row's residual over the sum of the magnitudes of that row's terms. A node's
    /// row is Kirchhoff's current law (amperes over amperes), a source's row its voltage (volts
    /// over volts). Rounding alone leaves about 1e-16.
    ///
    /// It shares the element stamps and device equations with `solve_operating_point` and nothing
    /// else: the matrix is stamped again from the elements, and the equations are evaluated at the
    /// point Newton stopped on, which Newton itself never does. A wrong stamp or device equation
    /// is therefore not caught here; the comparison with the SPICE reference catches that.
    fn dc_residual(&mut self) -> f64 {
        self.stamp_all(true);
        let n = self.size;
        let z = &self.operating_point;
        let (mut residual, mut terms) = (vec![0.0; n], vec![0.0; n]);
        for r in 0..n {
            let rhs = self.dc_rhs(r);
            residual[r] = -rhs;
            terms[r] = rhs.abs();
            for c in 0..n {
                let t = self.s.at(r, c) * z[c];
                residual[r] += t;
                terms[r] += t.abs();
            }
        }
        let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
        for &(dev, first) in &self.devices {
            let np = dev.ports();
            let mut v = [0.0; 2];
            for k in 0..np {
                let (a, b) = self.ports[first + k];
                v[k] = node_val(z, a) - node_val(z, b);
            }
            dev.eval(&v[..np], &mut i, &mut j);
            for k in 0..np {
                let (a, b) = self.ports[first + k];
                add_node(&mut residual, a, i[k]);
                add_node(&mut residual, b, -i[k]);
                add_node(&mut terms, a, i[k].abs());
                add_node(&mut terms, b, i[k].abs());
            }
        }
        // A row with no terms has no residual. A NaN anywhere makes the result NaN, which is under
        // no bound.
        let mut worst = 0.0f64;
        for (r, t) in residual.iter().zip(&terms) {
            if *t != 0.0 {
                let miss = r.abs() / t;
                if miss > worst || miss.is_nan() {
                    worst = miss;
                }
            }
        }
        worst
    }

    /// Makes the operating point an exact rest state of the discrete system. Returns the largest
    /// distance it removed from any row, in volts.
    ///
    /// The rows of `lin` are differences of entries of a double-precision inverse, so they agree
    /// with one another to about one part in 1e12, not exactly. At rest the history states are
    /// large (2 C rate V: 35 A for 10 uF holding 3 V at 576 kHz), so each row carries a constant
    /// error of picoamps that no element conducts. A coupling capacitor turns that into a steady
    /// current: 2e-11 A into a 10k load is 0.2 uV at the output, which settles in over 0.1 s and
    /// swamps a signal that a control has muted ahead of that stage. Folding each row's error at
    /// the operating point into the DC column removes it. What is left scales with the signal.
    ///
    /// The pin holds whatever point it is given, so the point is first checked against the DC
    /// equations. A point that fails is a bug in the DC solve or in its caller, and panics. That
    /// can only happen here, at construction: a pot change rebuilds `lin` in `update` without the
    /// pin and without a DC solve (the operating point may have moved, and pinning again needs a
    /// fresh solve), so `tick` reaches none of these checks. From the first pot change on, the
    /// rows carry their rounding error again.
    fn pin_operating_point(&mut self) -> f64 {
        let residual = self.dc_residual();
        assert!(
            residual < MAX_DC_RESIDUAL,
            "the operating point misses the DC equations by {residual} of a row's terms"
        );
        let (nx, nn) = (self.caps.len(), self.ports.len());
        let (base, rows) = (nx + 2, nx + 1 + nn);
        let dc = (nx + 1) * rows;
        let out_at_rest =
            node_val(&self.operating_point, self.output.0) - node_val(&self.operating_point, self.output.1);
        let mut worst = 0.0f64;
        for r in 0..rows {
            // At rest a capacitor's state repeats (x' = row - x = x), an inductor has no voltage
            // (x' = row + x = x), and the output and ports sit at their operating-point voltages.
            let (at_rest, volts) = if r < nx {
                ((1.0 - self.beta[r]) * self.x[r], 1.0 / self.gc2[r].abs())
            } else if r == nx {
                (out_at_rest, 1.0)
            } else {
                (self.v[r - nx - 1], 1.0)
            };
            // The row as `tick` sums it at rest: history columns in order, the DC column, then
            // the device currents. The order matters. A 100 uF capacitor at 9 V holds a history
            // near 1000 A, and one unit in its last place, 2e-13 A, is a current that a 50k bias
            // node turns into nanovolts. So the correction is repeated until that sum gives the
            // rest value itself, or `PIN_MAX_ROUNDS` corrections have been made. The last of
            // those is not checked: what it leaves, if anything, stays in the row.
            let mut removed = 0.0f64;
            for round in 0..PIN_MAX_ROUNDS {
                let mut row = 0.0;
                for c in 0..nx {
                    row += self.lin[c * rows + r] * self.x[c];
                }
                row += self.lin[dc + r];
                for k in 0..nn {
                    row += self.lin[(base + k) * rows + r] * self.z[base + k];
                }
                let error = at_rest - row;
                if round == 0 {
                    assert!(
                        error.abs() * volts < MAX_PIN_VOLTS,
                        "row {r} is {} V from the operating point",
                        error * volts
                    );
                    removed = error.abs() * volts;
                }
                if error == 0.0 {
                    break;
                }
                self.lin[dc + r] += error;
            }
            worst = worst.max(removed);
        }
        worst
    }

    /// Processes one sample at the circuit's rate. Never panics on a bad sample:
    /// - a non-finite input sample is treated as 0 V;
    /// - a sample whose solve fails (the joint Newton iteration, a device's own Newton iteration
    ///   or the rounds of per-device solves run out of steps, or the complete new capacitor
    ///   history, the output, or a port voltage or device current is not finite)
    ///   leaves the capacitor history, port voltages and device currents as the last successful
    ///   sample left them, returns that sample's output again and counts into `failed_solves`.
    pub fn tick(&mut self, u: f64) -> f64 {
        if self.dirty {
            self.update();
        }
        let u = if u.is_finite() { u } else { 0.0 };
        let (nx, nn) = (self.caps.len(), self.ports.len());
        let (base, rows) = (nx + 2, nx + 1 + nn);
        self.z[nx] = u;
        self.z[nx + 1] = 1.0;
        // Pass 1: capacitor voltages, output and port voltages from the state, input and supplies,
        // as a column-major matrix-vector product (vectorizes across rows).
        self.acc.fill(0.0);
        for (c, col) in self.lin.chunks_exact(rows).take(base).enumerate() {
            let zc = if c < nx { self.x[c] } else { self.z[c] };
            for (a, &m) in self.acc.iter_mut().zip(col) {
                *a += m * zc;
            }
        }
        // Port voltages and device currents start from a linear extrapolation of the last two samples.
        for k in 0..nn {
            let guess = 2.0 * self.v[k] - self.v_prev[k];
            self.v_prev[k] = self.v[k];
            self.v[k] = guess;
            let i = self.z[base + k];
            self.z[base + k] = if self.zero_currents {
                0.0
            } else {
                2.0 * i - self.i_prev[k]
            };
            self.i_prev[k] = i;
        }
        let mut solved = false;
        if self.joint {
            // The extrapolated guess can overshoot a junction that is just turning on (a base
            // driven through a capacitor moves 0.7 V a sample here): limit it against the last
            // solution, in `v_prev`.
            for &(dev, first) in &self.devices {
                let span = first..first + dev.ports();
                dev.limit(&self.v_prev[span.clone()], &mut self.v[span]);
            }
            solved = self.solve_joint();
        } else {
            for _ in 0..SWEEP_MAX_ROUNDS {
                self.i_used.copy_from_slice(&self.z[base..]);
                let mut devices_solved = true;
                for d in 0..self.devices.len() {
                    devices_solved &= self.solve_device(d);
                }
                if self.sequential || self.sweep_error() < SWEEP_TOLERANCE_V {
                    solved = devices_solved;
                    break;
                }
            }
        }
        // Pass 2: add the device currents' contribution to the capacitor voltages and output.
        for (col, &zc) in self.lin.chunks_exact(rows).skip(base).zip(&self.z[base..]) {
            for (a, &m) in self.acc[..nx + 1].iter_mut().zip(col) {
                *a += m * zc;
            }
        }
        // The complete candidate history x' = gc2 * v + beta * x, built in `acc` and held back
        // until the sample is accepted. Finite operands can overflow in this last step, so the
        // finiteness check below must come after it.
        for ((a, &x), &b) in self.acc.iter_mut().zip(&self.x).zip(&self.beta) {
            *a += b * x;
        }
        // Second check, on everything about to be kept: the new history, the output, and the
        // port voltages and device currents the next sample starts from.
        let finite = |s: &[f64]| s.iter().all(|v| v.is_finite());
        if !(solved && finite(&self.acc[..=nx]) && finite(&self.v) && finite(&self.z[base..])) {
            // Contain the failure. The history `x` is untouched. `v_prev` and `i_prev` hold the
            // last successful sample's port voltages and currents; go back to them. The next
            // sample's first guess then has no slope, which only costs it iterations.
            self.v.copy_from_slice(&self.v_prev);
            self.z[base..].copy_from_slice(&self.i_prev);
            self.failed_solves = self.failed_solves.saturating_add(1);
            self.consecutive_held += 1;
            if self.consecutive_held >= MAX_CONSECUTIVE_HELD {
                // An accepted history can be one that no later sample solves from (seen after a
                // 1e8 V input: every sample held from then on, at any iteration bound). Holding
                // cannot leave it, so go back to the DC operating point, as at construction.
                self.x.copy_from_slice(&self.rest[..nx]);
                self.v.copy_from_slice(&self.rest[nx..nx + nn]);
                self.v_prev.copy_from_slice(&self.rest[nx..nx + nn]);
                self.z[base..].copy_from_slice(&self.rest[nx + nn..nx + 2 * nn]);
                self.i_prev.copy_from_slice(&self.rest[nx + nn..nx + 2 * nn]);
                self.y = self.rest[nx + 2 * nn];
                self.consecutive_held = 0;
                self.rest_restores = self.rest_restores.saturating_add(1);
            }
            debug_assert!(self.y.is_finite() && finite(&self.x) && finite(&self.v));
            return self.y;
        }
        self.consecutive_held = 0;
        self.x.copy_from_slice(&self.acc[..nx]);
        self.y = self.acc[nx];
        self.y
    }

    /// Newton over all ports at once: v = p + K f(v), with p the linear part of each port voltage.
    /// Returns whether it converged within its iteration bound. When it did not, the port voltages
    /// and currents it leaves are not a solution and the caller must discard them.
    fn solve_joint(&mut self) -> bool {
        let nx = self.caps.len();
        let (base, nn) = (nx + 2, self.ports.len());
        let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
        let mut converged = false;
        for _ in 0..self.joint_iterations {
            self.jac.data.fill(0.0);
            for &(dev, first) in &self.devices {
                let np = dev.ports();
                dev.eval(&self.v[first..first + np], &mut i, &mut j);
                self.z[base + first..base + first + np].copy_from_slice(&i[..np]);
                // J = I - K Jf: column (first + b) gets -sum_a K[:, first + a] Jf[a][b].
                for r in 0..nn {
                    for b in 0..np {
                        let kj: f64 = (0..np)
                            .map(|a| self.port_v.at(r, base + first + a) * j[a * np + b])
                            .sum();
                        *self.jac.at_mut(r, first + b) -= kj;
                    }
                }
            }
            for r in 0..nn {
                *self.jac.at_mut(r, r) += 1.0;
                let ki = dot(&self.port_v.row(r)[base..], &self.z[base..]);
                *self.jac.at_mut(r, nn) = self.v[r] - self.acc[nx + 1 + r] - ki;
            }
            solve_small(&mut self.jac, nn);
            // Take the Newton step, limited per junction. Ports that stay off the exponentials
            // (a collector swinging 20 V in a sample) move the whole way: the system is linear in
            // them. One scale for all ports would hold a junction that must fall by a volt to
            // the pace of the largest port.
            // Convergence is judged on the raw Newton corrections, not on `new - old` after the
            // update, which loses digits to cancellation. A NaN correction fails the comparison.
            let small = (0..nn).all(|k| self.jac.at(k, nn).abs() < JOINT_TOLERANCE_V);
            let mut limited = false;
            for &(dev, first) in &self.devices {
                let np = dev.ports();
                let (mut old, mut new) = ([0.0; 2], [0.0; 2]);
                for k in 0..np {
                    old[k] = self.v[first + k];
                    new[k] = old[k] - self.jac.at(first + k, nn);
                }
                limited |= dev.limit(&old[..np], &mut new[..np]);
                self.v[first..first + np].copy_from_slice(&new[..np]);
            }
            if !limited && small {
                converged = true;
                break;
            }
        }
        if !converged {
            return false;
        }
        for &(dev, first) in &self.devices {
            let np = dev.ports();
            dev.eval(&self.v[first..first + np], &mut i, &mut j);
            self.z[base + first..base + first + np].copy_from_slice(&i[..np]);
        }
        true
    }

    /// Bound on how far earlier devices' port voltages are from consistent with the currents that
    /// later devices settled on during the last sweep (in volts). "Later" is later in solve
    /// order, the order of `devices`: `order_devices` permutes them, so a later device's ports
    /// can have lower indices than an earlier one's.
    fn sweep_error(&self) -> f64 {
        let base = self.caps.len() + 2;
        let mut worst = 0.0f64;
        for (d, &(dev, first)) in self.devices.iter().enumerate() {
            for r in first..first + dev.ports() {
                let row = &self.port_v.row(r)[base..];
                let mut err = 0.0;
                for &(later, later_first) in &self.devices[d + 1..] {
                    for c in later_first..later_first + later.ports() {
                        err += (row[c] * (self.z[base + c] - self.i_used[c])).abs();
                    }
                }
                worst = worst.max(err);
            }
        }
        worst
    }

    /// Newton solve of one device's ports: v = p + K_dd f(v), where p holds the linear part and the
    /// other devices' latest currents. Returns whether it converged within its iteration bound.
    fn solve_device(&mut self, d: usize) -> bool {
        let nx = self.caps.len();
        let base = nx + 2;
        let (dev, first) = self.devices[d];
        let np = dev.ports();
        let (mut p, mut k, mut v) = ([0.0; 2], [0.0; 4], [0.0; 2]);
        for r in 0..np {
            let row = &self.port_v.row(first + r)[base..];
            p[r] = self.acc[nx + 1 + first + r] + dot(row, &self.z[base..]);
            for c in 0..np {
                k[r * np + c] = row[first + c];
                p[r] -= k[r * np + c] * self.z[base + first + c];
            }
            v[r] = self.v[first + r];
        }
        let (mut i, mut j, mut cur) = ([0.0; 2], [0.0; 4], [0.0; 2]);
        let mut done = false;
        for _ in 0..DEVICE_MAX_ITERATIONS {
            dev.eval(&v[..np], &mut i, &mut j);
            let mut step = [0.0; 2];
            if np == 1 {
                step[0] = (v[0] - p[0] - k[0] * i[0]) / (1.0 - k[0] * j[0]);
            } else {
                // F = v - p - K i; J = I - K Jf (2x2, solved in closed form).
                let f0 = v[0] - p[0] - (k[0] * i[0] + k[1] * i[1]);
                let f1 = v[1] - p[1] - (k[2] * i[0] + k[3] * i[1]);
                let a = 1.0 - (k[0] * j[0] + k[1] * j[2]);
                let b = -(k[0] * j[1] + k[1] * j[3]);
                let c = -(k[2] * j[0] + k[3] * j[2]);
                let e = 1.0 - (k[2] * j[1] + k[3] * j[3]);
                let det = a * e - b * c;
                step = [(e * f0 - b * f1) / det, (a * f1 - c * f0) / det];
            }
            let big = step[..np].iter().fold(0.0f64, |m, s| m.max(s.abs()));
            let scale = if big > 0.1 { 0.1 / big } else { 1.0 };
            for r in 0..np {
                step[r] *= scale;
                v[r] -= step[r];
            }
            // A step that was scaled down says nothing about how far the solution is: a
            // transistor whose collector still has volts to travel would pass on its base step
            // alone. Only a full Newton step can end the iteration.
            done = scale == 1.0 && dev.converged(&v[..np], &step[..np]);
            // Currents at the updated v from the linearization; exact to second order in the step.
            for r in 0..np {
                cur[r] = i[r] - (0..np).map(|c| j[r * np + c] * step[c]).sum::<f64>();
            }
            if done {
                break;
            }
        }
        for r in 0..np {
            self.z[base + first + r] = cur[r];
            self.v[first + r] = v[r];
        }
        done
    }
}

/// Value at a node from a node-indexed vector (index = node - 1); ground is zero.
fn node_val(v: &[f64], node: Node) -> f64 {
    if node > 0 {
        v[node - 1]
    } else {
        0.0
    }
}

fn add_node(v: &mut [f64], node: Node, x: f64) {
    if node > 0 {
        v[node - 1] += x;
    }
}

/// Adds `v` at (r, c) of the MNA matrix; rows and columns are 1-based with 0 as ground.
fn stamp(s: &mut Mat, r: Node, c: Node, v: f64) {
    if r > 0 && c > 0 {
        *s.at_mut(r - 1, c - 1) += v;
    }
}

fn conductance(s: &mut Mat, a: Node, b: Node, g: f64) {
    stamp(s, a, a, g);
    stamp(s, b, b, g);
    stamp(s, a, b, -g);
    stamp(s, b, a, -g);
}

/// Dot product with four accumulators, so the adds are not one serial dependency chain.
#[inline]
pub(crate) fn dot(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = [0.0; 4];
    let (ac, bc) = (a.chunks_exact(4), b.chunks_exact(4));
    let tail: f64 = ac.remainder().iter().zip(bc.remainder()).map(|(x, y)| x * y).sum();
    for (x, y) in ac.zip(bc) {
        for j in 0..4 {
            acc[j] += x[j] * y[j];
        }
    }
    acc.iter().sum::<f64>() + tail
}

/// Inverts `a` into `inv` with partial-pivot LU. `a` is overwritten.
fn invert(a: &mut Mat, inv: &mut Mat, perm: &mut [usize]) {
    let n = perm.len();
    for (i, p) in perm.iter_mut().enumerate() {
        *p = i;
    }
    for k in 0..n {
        let pivot = (k..n)
            .max_by(|&i, &j| a.at(i, k).abs().total_cmp(&a.at(j, k).abs()))
            .unwrap();
        assert!(
            a.at(pivot, k).abs() > 1e-300,
            "singular circuit matrix (floating node?)"
        );
        if pivot != k {
            for c in 0..n {
                a.data.swap(k * n + c, pivot * n + c);
            }
            perm.swap(k, pivot);
        }
        for i in k + 1..n {
            let f = a.at(i, k) / a.at(k, k);
            *a.at_mut(i, k) = f;
            for c in k + 1..n {
                let v = a.at(k, c);
                *a.at_mut(i, c) -= f * v;
            }
        }
    }
    // Solve L U inv[:, j] = P e_j for each column.
    for j in 0..n {
        for i in 0..n {
            let mut v = if perm[i] == j { 1.0 } else { 0.0 };
            for k in 0..i {
                v -= a.at(i, k) * inv.at(k, j);
            }
            *inv.at_mut(i, j) = v;
        }
        for i in (0..n).rev() {
            let mut v = inv.at(i, j);
            for k in i + 1..n {
                v -= a.at(i, k) * inv.at(k, j);
            }
            *inv.at_mut(i, j) = v / a.at(i, i);
        }
    }
}

/// Gaussian elimination on an augmented n x (n + 1) system; the solution replaces the last column.
fn solve_small(m: &mut Mat, n: usize) {
    for k in 0..n {
        let pivot = (k..n)
            .max_by(|&i, &j| m.at(i, k).abs().total_cmp(&m.at(j, k).abs()))
            .unwrap();
        for c in 0..=n {
            m.data.swap(k * m.cols + c, pivot * m.cols + c);
        }
        for i in 0..n {
            if i != k {
                let f = m.at(i, k) / m.at(k, k);
                for c in k..=n {
                    let v = m.at(k, c);
                    *m.at_mut(i, c) -= f * v;
                }
            }
        }
    }
    for k in 0..n {
        *m.at_mut(k, n) /= m.at(k, k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;
    use crate::netlist::Netlist;
    use std::f64::consts::PI;

    const Q2N3904: Bjt = Bjt {
        is: 1e-14,
        bf: 300.0,
        br: 4.0,
        vaf: 100.0,
        ikf: 0.4,
        vt: 0.025_864_6,
    };

    /// Steady-state gain of a circuit at `freq`, by sine fit after settling.
    fn gain(c: &mut Circuit, freq: f64, amp: f64) -> f64 {
        let rate = c.rate;
        let n = (rate / freq * 200.0) as usize;
        let (mut s, mut co) = (0.0, 0.0);
        for i in 0..2 * n {
            let t = i as f64 / rate;
            let y = c.tick(amp * (2.0 * PI * freq * t).sin());
            if i >= n {
                s += y * (2.0 * PI * freq * t).sin();
                co += y * (2.0 * PI * freq * t).cos();
            }
        }
        2.0 * (s * s + co * co).sqrt() / n as f64 / amp
    }

    #[test]
    fn rc_lowpass_matches_bilinear_response() {
        let (r, cap, rate) = (1e3, 1e-6, 48_000.0);
        let mut c = Circuit::new(
            vec![
                Element::Input(1, 0),
                Element::Resistor(1, 2, r),
                Element::Capacitor(2, 0, cap),
            ],
            (2, 0),
            rate,
        );
        for f in [50.0, 159.15, 1000.0, 10_000.0] {
            // Trapezoidal integration is the bilinear transform: analog response at the warped frequency.
            let warped = 2.0 * rate * (PI * f / rate).tan();
            let expected = 1.0 / (1.0 + (warped * r * cap).powi(2)).sqrt();
            assert!((gain(&mut c, f, 1.0) - expected).abs() < 1e-4, "{f} Hz");
        }
    }

    #[test]
    fn history_that_overflows_from_finite_operands_is_rejected() {
        // An RC lowpass at a 1 Hz circuit rate, driven with +1e308 V and -1e308 V in turn. Both
        // terms of the history update x' = gc2 * v + beta * x stay finite while their sum
        // overflows. Such a sample must be rejected whole: the history never holds a non-finite
        // value, and silence afterwards solves normally and decays to rest.
        let elements = vec![
            Element::Input(1, 0),
            Element::Resistor(1, 2, 0.1),
            Element::Capacitor(2, 0, 0.05),
        ];
        let mut c = Circuit::new(elements, (2, 0), 1.0);
        for k in 0..100 {
            let y = c.tick(if k % 2 == 0 { 1e308 } else { -1e308 });
            assert!(y.is_finite(), "output {y} at sample {k}");
            assert!(c.x.iter().all(|x| x.is_finite()), "history {:?} at sample {k}", c.x);
        }
        let failed = c.failed_solves();
        assert!(failed > 0, "no sample overflowed, so this did not test the rejection");
        let mut y = f64::INFINITY;
        for k in 0..40_000 {
            y = c.tick(0.0);
            assert!(y.is_finite() && c.x[0].is_finite(), "sample {k} of silence");
        }
        assert_eq!(c.failed_solves(), failed, "silence solves normally afterwards");
        assert!(y.abs() < 1e-6, "output {y} V after 40000 silent samples");
    }

    #[test]
    fn rl_highpass_matches_bilinear_response_and_carries_dc() {
        // Series R into a shunt L, with a DC supply through a second resistor: the AC response is an
        // RL high-pass and the operating point puts the supply's current through the inductor.
        let (r, l, rate) = (1e3, 0.1, 96_000.0);
        let mut c = Circuit::new(
            vec![
                Element::Input(1, 0),
                Element::Resistor(1, 2, r),
                Element::Inductor(2, 0, l),
                Element::Dc(3, 0, 1.0),
                Element::Resistor(3, 2, 1e6),
            ],
            (2, 0),
            rate,
        );
        assert!(c.operating_point()[1].abs() < 1e-6, "inductor shorts its node at DC");
        let rp = r * 1e6 / (r + 1e6);
        for f in [100.0, 1591.5, 10_000.0] {
            let warped = 2.0 * rate * (PI * f / rate).tan();
            let x = warped * l / rp;
            let expected = x / (1.0 + x * x).sqrt() * 1e6 / (r + 1e6);
            assert!((gain(&mut c, f, 1.0) - expected).abs() < 1e-4, "{f} Hz");
        }
    }

    #[test]
    fn op_amp_follower_and_pot_update() {
        // Single-pole op-amp (same macro as the SPICE references) as a non-inverting amp, gain 1 + R2/R1.
        let opamp = |p, n, out, internal| {
            [
                Element::Vccs {
                    p: 0,
                    n: internal,
                    cp: p,
                    cn: n,
                    gm: 100.0,
                },
                Element::Resistor(internal, 0, 1e3),
                Element::Capacitor(internal, 0, 5.305e-6),
                Element::Vcvs {
                    p: out,
                    n: 0,
                    cp: internal,
                    cn: 0,
                    gain: 1.0,
                },
            ]
        };
        let mut els = vec![
            Element::Input(1, 0),
            Element::Resistor(2, 0, 1e3),
            Element::Resistor(3, 2, 9e3),
        ];
        els.extend(opamp(1, 2, 3, 4));
        let mut c = Circuit::new(els, (3, 0), 192_000.0);
        assert!((gain(&mut c, 1000.0, 0.1) - 10.0).abs() / 10.0 < 1e-3);
        c.set_resistor(2, 99e3);
        assert!((gain(&mut c, 1000.0, 0.01) - 100.0).abs() / 100.0 < 3e-3);
    }

    #[test]
    fn diode_clipper_satisfies_kcl() {
        // Series resistor into a diode pair to ground; check the output solves the nonlinear KCL.
        let (r, is, nvt) = (10e3, 2.52e-9, 1.752 * 0.025864);
        let mut c = Circuit::new(
            vec![
                Element::Input(1, 0),
                Element::Resistor(1, 2, r),
                Element::Capacitor(2, 0, 1e-9),
                Element::DiodePair { a: 2, b: 0, is, nvt },
            ],
            (2, 0),
            96_000.0,
        );
        for u in [0.01, 0.5, 2.0, 10.0, -3.0] {
            let mut v = 0.0;
            for _ in 0..2000 {
                v = c.tick(u);
            }
            let kcl = (u - v) / r - 2.0 * is * (v / nvt).sinh();
            assert!(kcl.abs() < 1e-12, "u={u} v={v} residual={kcl}");
        }
    }

    #[test]
    fn emitter_follower_operating_point_holds_at_rest() {
        // 9 V supply, base biased by a divider, 10k emitter resistor. The operating point satisfies
        // the transistor equations, and silence keeps the circuit there.
        let els = vec![
            Element::Dc(1, 0, 9.0),
            Element::Resistor(1, 2, 10e3),
            Element::Resistor(2, 0, 10e3),
            Element::Input(4, 0),
            Element::Capacitor(4, 2, 1e-6),
            Element::Npn {
                c: 1,
                b: 2,
                e: 3,
                model: Q2N3904,
            },
            Element::Resistor(3, 0, 10e3),
        ];
        let mut c = Circuit::new(els, (3, 0), 96_000.0);
        let op = c.operating_point().to_vec();
        let (vb, ve) = (op[1], op[2]);
        let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
        Device::Bjt {
            model: Q2N3904,
            polarity: 1.0,
        }
        .eval(&[9.0 - ve, vb - ve], &mut i, &mut j);
        assert!((ve / 10e3 - (i[0] + i[1])).abs() < 1e-12, "emitter KCL");
        assert!(((9.0 - vb) / 10e3 - vb / 10e3 - i[1]).abs() < 1e-12, "base KCL");
        for _ in 0..1000 {
            assert!((c.tick(0.0) - ve).abs() < 1e-9);
        }
    }

    #[test]
    fn pnp_is_the_mirror_image_of_npn() {
        // A PNP at negated port voltages carries the NPN's currents negated, with the same
        // Jacobian. Covers forward-active, saturation, cut-off and reverse operation.
        let (npn, pnp) = (
            Device::Bjt {
                model: Q2N3904,
                polarity: 1.0,
            },
            Device::Bjt {
                model: Q2N3904,
                polarity: -1.0,
            },
        );
        for v in [
            [5.0, 0.65],
            [0.05, 0.7],
            [0.1, 0.7],
            [11.0, -4.0],
            [-0.6, 0.1],
            [-2.0, 0.3],
            [0.0, 0.0],
        ] {
            let (mut i_n, mut j_n, mut i_p, mut j_p) = ([0.0; 2], [0.0; 4], [0.0; 2], [0.0; 4]);
            npn.eval(&v, &mut i_n, &mut j_n);
            pnp.eval(&[-v[0], -v[1]], &mut i_p, &mut j_p);
            assert_eq!([i_p[0], i_p[1]], [-i_n[0], -i_n[1]], "currents at {v:?}");
            assert_eq!(j_p, j_n, "jacobian at {v:?}");
        }
        // A forward-active PNP: current flows out of the collector and out of the base.
        let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
        pnp.eval(&[-5.0, -0.65], &mut i, &mut j);
        assert!(i[0] < 0.0 && i[1] < 0.0 && i[0] / i[1] > 100.0);
    }

    #[test]
    fn pnp_jacobian_matches_finite_differences() {
        let dev = Device::Bjt {
            model: Q2N3904,
            polarity: -1.0,
        };
        // Forward-active, then saturated (both junctions forward).
        for v in [[-5.0, -0.65], [-0.1, -0.68], [-0.1, -0.7]] {
            let (mut i0, mut j) = ([0.0; 2], [0.0; 4]);
            dev.eval(&v, &mut i0, &mut j);
            assert!(
                i0[0] < 0.0 && i0[1] < 0.0,
                "a conducting PNP's port currents are negative"
            );
            for c in 0..2 {
                let h = 1e-7;
                let (mut vp, mut vm) = (v, v);
                vp[c] += h;
                vm[c] -= h;
                let (mut ip, mut im, mut jx) = ([0.0; 2], [0.0; 2], [0.0; 4]);
                dev.eval(&vp, &mut ip, &mut jx);
                dev.eval(&vm, &mut im, &mut jx);
                for r in 0..2 {
                    let fd = (ip[r] - im[r]) / (2.0 * h);
                    assert!(
                        (fd - j[r * 2 + c]).abs() <= 1e-5 * fd.abs().max(1e-9),
                        "d i{r} / d v{c} at {v:?}: {fd} vs {}",
                        j[r * 2 + c]
                    );
                }
            }
        }
    }

    #[test]
    fn pnp_common_emitter_operating_point_holds_at_rest() {
        // 9 V supply, PNP with its emitter on the supply, base biased through 4.7M to ground, 4.7k
        // collector load to ground. The operating point satisfies the transistor equations with
        // the PNP's polarity, and silence keeps the circuit there.
        let els = vec![
            Element::Dc(1, 0, 9.0),
            Element::Resistor(2, 0, 4.7e6),
            Element::Input(4, 0),
            Element::Capacitor(4, 2, 1e-6),
            Element::Pnp {
                c: 3,
                b: 2,
                e: 1,
                model: Q2N3904,
            },
            Element::Resistor(3, 0, 4.7e3),
        ];
        let mut c = Circuit::new(els, (3, 0), 96_000.0);
        let op = c.operating_point().to_vec();
        let (vb, vc) = (op[1], op[2]);
        assert!(
            vb < 9.0 - 0.5 && vb > 9.0 - 0.8,
            "emitter-base junction is forward biased: vb={vb}"
        );
        assert!(
            vc > 0.5 && vc < 8.0 && vc < vb,
            "forward-active, collector between ground and the base: vc={vc}"
        );
        let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
        Device::Bjt {
            model: Q2N3904,
            polarity: -1.0,
        }
        .eval(&[vc - 9.0, vb - 9.0], &mut i, &mut j);
        // Port currents are counted into the collector and into the base, so both are negative
        // for a PNP.
        assert!(i[0] < 0.0 && i[1] < 0.0, "collector {} A, base {} A", i[0], i[1]);
        assert!((vc / 4.7e3 + i[0]).abs() < 1e-12, "collector KCL");
        assert!((vb / 4.7e6 + i[1]).abs() < 1e-12, "base KCL");
        for _ in 0..1000 {
            assert!((c.tick(0.0) - vc).abs() < 1e-9);
        }
    }

    #[test]
    fn junction_limiting_only_acts_far_up_the_exponential() {
        let (is, vt) = (1e-14, 0.025_864_6);
        let vcrit = critical_voltage(is, vt);
        assert!((vcrit - 0.7303).abs() < 1e-4, "vcrit {vcrit}");
        // Below vcrit, and small moves anywhere, pass unchanged.
        for (new, old) in [(0.7, -5.0), (-8.0, 0.7), (0.9, 0.86), (0.3, 0.2)] {
            assert_eq!(limit_junction(new, old, vt, vcrit), new);
        }
        // A jump into deep forward bias is cut to a logarithmic step above the old voltage.
        let v = limit_junction(1.6, 0.7, vt, vcrit);
        assert!(v > 0.7 && v < 0.8, "limited to {v}");
        // From reverse bias the limited voltage does not depend on how far back it started.
        assert_eq!(
            limit_junction(1.6, -3.0, vt, vcrit),
            limit_junction(1.6, -9.0, vt, vcrit)
        );
        assert!(limit_junction(1.6, -3.0, vt, vcrit) < 0.2);

        // A transistor: limiting vbe leaves vbc's move alone, and the mirror holds for a PNP.
        let npn = Device::Bjt {
            model: Q2N3904,
            polarity: 1.0,
        };
        let pnp = Device::Bjt {
            model: Q2N3904,
            polarity: -1.0,
        };
        let (old, mut new) = ([20.0, 0.7], [15.0, 1.6]);
        assert!(npn.limit(&old, &mut new));
        assert!(new[1] > 0.7 && new[1] < 0.8, "vbe limited to {}", new[1]);
        assert!(
            (new[1] - new[0] - (1.6 - 15.0)).abs() < 1e-12,
            "vbc keeps its proposed value"
        );
        let mut mirrored = [-15.0, -1.6];
        assert!(pnp.limit(&[-20.0, -0.7], &mut mirrored));
        assert_eq!(mirrored, [-new[0], -new[1]]);
        let mut free = [3.0, 0.65];
        assert!(
            !npn.limit(&[20.0, -4.0], &mut free),
            "a move that stays below vcrit is not limited"
        );
        assert_eq!(free, [3.0, 0.65]);
    }

    #[test]
    fn npn_jacobian_matches_finite_differences() {
        let dev = Device::Bjt {
            model: Q2N3904,
            polarity: 1.0,
        };
        let v = [5.0, 0.65];
        let (mut i0, mut j) = ([0.0; 2], [0.0; 4]);
        dev.eval(&v, &mut i0, &mut j);
        for c in 0..2 {
            let h = 1e-7;
            let mut vp = v;
            vp[c] += h;
            let (mut ip, mut jp) = ([0.0; 2], [0.0; 4]);
            dev.eval(&vp, &mut ip, &mut jp);
            for r in 0..2 {
                let fd = (ip[r] - i0[r]) / h;
                assert!(
                    (fd - j[r * 2 + c]).abs() <= 1e-4 * fd.abs().max(1e-9),
                    "d i{r} / d v{c}"
                );
            }
        }
    }

    /// Two transistors in a feedback loop (the second's emitter drives the first's base), which
    /// the Solver solves jointly.
    fn feedback_pair() -> Circuit {
        let els = vec![
            Element::Dc(1, 0, 9.0),
            Element::Input(2, 0),
            Element::Capacitor(2, 3, 2.2e-6),
            Element::Npn {
                c: 4,
                b: 3,
                e: 0,
                model: Q2N3904,
            },
            Element::Resistor(1, 4, 33e3),
            Element::Npn {
                c: 5,
                b: 4,
                e: 6,
                model: Q2N3904,
            },
            Element::Resistor(1, 5, 8.2e3),
            Element::Resistor(6, 0, 1e3),
            Element::Resistor(6, 3, 100e3),
            Element::Capacitor(5, 7, 10e-9),
            Element::Resistor(7, 0, 500e3),
        ];
        let c = Circuit::new(els, (7, 0), 576_000.0);
        assert!(c.joint, "this circuit is meant to take the joint solve");
        c
    }

    #[test]
    fn a_joint_solve_that_runs_out_of_iterations_is_held_and_counted() {
        let sine = |k: usize| 0.5 * (2.0 * PI * 1000.0 * k as f64 / 576_000.0).sin();
        let mut c = feedback_pair();
        let mut last = 0.0;
        for k in 0..5760 {
            last = c.tick(sine(k));
        }
        assert_eq!(c.failed_solves(), 0, "the driven circuit solves under the normal bound");
        // One iteration is not enough while the input moves: each such sample repeats the last
        // output and leaves the history alone.
        c.limit_joint_iterations(1);
        let history = c.x.clone();
        for k in 5760..5860 {
            assert_eq!(c.tick(sine(k)), last, "sample {k}");
        }
        assert_eq!(c.failed_solves(), 100);
        assert_eq!(c.x, history);
        assert_eq!(c.rest_restores(), 0);
        c.limit_joint_iterations(JOINT_MAX_ITERATIONS);
        for k in 5860..11_520 {
            assert!(c.tick(sine(k)).is_finite());
        }
        assert_eq!(c.failed_solves(), 100, "solves converge again under the normal bound");
    }

    #[test]
    fn a_device_solve_that_runs_out_of_iterations_is_held_and_counted() {
        // A common-emitter stage on a 100 V supply, its collector resting near 50 V, solved per
        // device. A 1 V step at the input saturates the transistor: the collector has to fall by
        // tens of volts within one sample, and the device's Newton iteration moves at most 0.1 V
        // a step, 6.4 V in all. Before the status existed that sample was committed as it stood.
        let mut c = Circuit::new(
            vec![
                Element::Dc(1, 0, 100.0),
                Element::Resistor(1, 3, 20e6),
                Element::Resistor(1, 2, 33e3),
                Element::Input(4, 0),
                Element::Resistor(4, 5, 1e3),
                Element::Capacitor(5, 3, 1e-6),
                Element::Npn {
                    c: 2,
                    b: 3,
                    e: 0,
                    model: Q2N3904,
                },
            ],
            (2, 0),
            96_000.0,
        );
        assert!(!c.joint);
        let rest = c.tick(0.0);
        assert!(rest > 30.0 && rest < 70.0, "collector rests at {rest} V");
        assert_eq!(c.failed_solves(), 0);
        let history = c.x.clone();
        // The stage cannot be solved while the step lasts: every sample is held.
        for _ in 0..MAX_CONSECUTIVE_HELD - 1 {
            assert_eq!(c.tick(1.0), rest, "a failed sample repeats the last output");
        }
        assert_eq!(c.x, history, "and the history is left alone");
        assert_eq!(c.rest_restores(), 0);
        // After `MAX_CONSECUTIVE_HELD` of them the circuit is put back at its operating point,
        // where the step still cannot be solved, and is held there.
        for _ in 0..11 {
            assert!((c.tick(1.0) - rest).abs() < 1e-9);
        }
        assert_eq!(c.failed_solves(), MAX_CONSECUTIVE_HELD as u64 + 10);
        assert_eq!(c.rest_restores(), 1);
        // Silence solves again, from the state that was held.
        for _ in 0..1000 {
            assert!((c.tick(0.0) - rest).abs() < 1e-6);
        }
        assert_eq!(c.failed_solves(), MAX_CONSECUTIVE_HELD as u64 + 10);
    }

    /// Largest distance, in volts, of any port from its own equation v = p + K i(v) after an
    /// accepted `tick`, with the currents evaluated at the accepted port voltages (not the
    /// tangent-line currents the solve kept, which satisfy the linearised equation by
    /// construction). `acc` still holds the linear part of each port voltage.
    fn port_residual(c: &Circuit) -> f64 {
        let (nx, nn) = (c.caps.len(), c.ports.len());
        let base = nx + 2;
        let mut exact = vec![0.0; nn];
        let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
        for &(dev, first) in &c.devices {
            let np = dev.ports();
            dev.eval(&c.v[first..first + np], &mut i, &mut j);
            exact[first..first + np].copy_from_slice(&i[..np]);
        }
        (0..nn)
            .map(|r| (c.v[r] - c.acc[nx + 1 + r] - dot(&c.port_v.row(r)[base..], &exact)).abs())
            .fold(0.0, f64::max)
    }

    #[test]
    fn a_damped_device_step_is_not_convergence() {
        // A transistor with its base held by a voltage source and the input coupled into its
        // collector, solved per device. An input that alternates by 2 V moves the collector by
        // volts each sample while the base does not move at all. The device's Newton step is
        // then scaled down to 0.1 V and its base part is zero: judged on the scaled step, the
        // sample was accepted with the collector volts from its equation.
        let mut c = Circuit::new(
            vec![
                Element::Dc(1, 0, 18.0),
                Element::Resistor(1, 2, 10e3),
                Element::Dc(3, 0, 0.65),
                Element::Input(4, 0),
                Element::Capacitor(4, 2, 1e-6),
                Element::Npn {
                    c: 2,
                    b: 3,
                    e: 0,
                    model: Q2N3904,
                },
            ],
            (2, 0),
            96_000.0,
        );
        assert!(!c.joint && c.devices.len() == 1);
        let rest = c.tick(0.0);
        assert!(rest > 5.0 && rest < 13.0, "collector rests at {rest} V");
        let (mut worst, mut largest_move) = (0.0f64, 0.0f64);
        for k in 0..2000 {
            let before = c.v[0];
            c.tick(if k % 2 == 0 { 1.0 } else { -1.0 });
            largest_move = largest_move.max((c.v[0] - before).abs());
            worst = worst.max(port_residual(&c));
        }
        assert_eq!(
            c.failed_solves(),
            0,
            "every sample is accepted, so every residual counts"
        );
        // Measured: 8.9e-16 V. The solve that judged the scaled step left 2.0 V.
        assert!(worst < 1e-9, "an accepted sample is {worst} V from its port equation");
        assert!(
            largest_move > 1.0,
            "the collector port moves {largest_move} V a sample, so the 0.1 V scale acts"
        );
    }

    #[test]
    fn sweep_error_sums_over_the_devices_solved_later_not_over_higher_port_indices() {
        // Three diode stages in a chain, each buffered from the one before, listed downstream
        // first: the solve order (upstream first) is the reverse of the port order. Weak
        // resistors from the later stages back to the earlier ones make the later currents count.
        let diode = |a: Node| Element::Diode {
            a,
            b: 0,
            is: 2.52e-9,
            nvt: 1.752 * 0.025_864_6,
        };
        let buffer = |from: Node, to: Node| Element::Vcvs {
            p: to,
            n: 0,
            cp: from,
            cn: 0,
            gain: 1.0,
        };
        let (x, y, w) = (2, 4, 6);
        let els = vec![
            diode(w),
            diode(y),
            diode(x),
            Element::Input(1, 0),
            Element::Resistor(1, x, 10e3),
            buffer(x, 3),
            Element::Resistor(3, y, 10e3),
            buffer(y, 5),
            Element::Resistor(5, w, 10e3),
            Element::Resistor(w, y, 1e6),
            Element::Resistor(w, x, 2e6),
            Element::Resistor(y, x, 3e6),
        ];
        let mut c = Circuit::new(els, (w, 0), 96_000.0);
        let order: Vec<usize> = c.devices.iter().map(|d| d.1).collect();
        assert_eq!(order, [2, 1, 0], "solve order by first port: upstream first");
        // The currents each device settled on in a sweep, against those the sweep started from.
        let (base, nn) = (c.caps.len() + 2, c.ports.len());
        let moved = [3e-6, 5e-6, 7e-6];
        c.i_used.copy_from_slice(&[1e-3, 2e-3, 3e-3]);
        for k in 0..nn {
            c.z[base + k] = c.i_used[k] + moved[k];
        }
        // Computed another way: each port's place in the solve order, then for every port the
        // sum over the ports that are solved after it.
        let mut place = vec![0; nn];
        for (at, &(dev, first)) in c.devices.iter().enumerate() {
            place[first..first + dev.ports()].fill(at);
        }
        let k = |r: usize, col: usize| c.port_v.at(r, base + col);
        let expected = (0..nn)
            .map(|r| {
                (0..nn)
                    .filter(|&col| place[col] > place[r])
                    .map(|col| (k(r, col) * moved[col]).abs())
                    .sum::<f64>()
            })
            .fold(0.0, f64::max);
        // The worst row is the first stage's: both later stages feed back to it.
        let first_stage = (k(2, 1) * moved[1]).abs() + (k(2, 0) * moved[0]).abs();
        assert!(
            expected > 1e-9 && expected == first_stage,
            "{expected} V, {first_stage} V"
        );
        let got = c.sweep_error();
        assert!(
            (got - expected).abs() <= 1e-12 * expected,
            "sweep_error {got} V, the devices solved later give {expected} V"
        );
    }

    #[test]
    fn diode_matches_spice_law_and_its_jacobian() {
        let (is, nvt) = (2.52e-9, 1.752 * 0.025_864_6);
        let dev = Device::Diode { is, nvt };
        let eval = |v: f64| {
            let (mut i, mut j) = ([0.0; 2], [0.0; 4]);
            dev.eval(&[v], &mut i, &mut j);
            (i[0], j[0])
        };
        // Forward and shallow reverse: Shockley plus GMIN.
        for v in [0.6, 0.1, 0.0, -0.1] {
            let want = is * ((v / nvt).exp() - 1.0) + 1e-12 * v;
            assert!((eval(v).0 - want).abs() <= 1e-12 * want.abs(), "{v} V");
        }
        // Deep reverse: the cubic tail, not yet -Is. At -0.3 V the exponential law is within
        // 0.2% of -Is and SPICE's is 0.46% short of it.
        let tail = (3.0 * nvt / (0.3 * std::f64::consts::E)).powi(3);
        assert!((0.004..0.005).contains(&tail));
        let want = -is * (1.0 - tail) - 1e-12 * 0.3;
        assert!((eval(-0.3).0 - want).abs() <= 1e-12 * want.abs());
        // The branches meet in value at -3 N Vt.
        let edge = -3.0 * nvt;
        assert!((eval(edge + 1e-9).0 - eval(edge - 1e-9).0).abs() < 1e-6 * is);
        // Jacobian against central differences, in both branches.
        for v in [0.6, 0.0, -0.1, -0.2, -0.5, -3.0] {
            let h = 1e-6;
            let fd = (eval(v + h).0 - eval(v - h).0) / (2.0 * h);
            assert!(
                (fd - eval(v).1).abs() <= 1e-5 * fd.abs().max(1e-12),
                "{v} V: {fd} vs {}",
                eval(v).1
            );
        }
    }

    /// An emitter follower biased from a divider, solved with `fault` amperes too much flowing
    /// into the bias node and then given back its own netlist: the device state is consistent
    /// with itself and belongs to another circuit. Returns it with the bias node's index.
    fn follower_solved_with_fault(fault: f64) -> (Circuit, usize) {
        let (supply, bias) = (1, 2);
        let mut els = vec![
            Element::Dc(supply, 0, 9.0),
            Element::Resistor(supply, bias, 10e3),
            Element::Resistor(bias, 0, 10e3),
            Element::Input(4, 0),
            Element::Capacitor(4, bias, 1e-6),
            Element::Npn {
                c: supply,
                b: bias,
                e: 3,
                model: Q2N3904,
            },
            Element::Resistor(3, 0, 10e3),
        ];
        if fault == 0.0 {
            return (Circuit::new(els, (3, 0), 576_000.0), bias);
        }
        els.push(Element::Vccs {
            p: 0,
            n: bias,
            cp: supply,
            cn: 0,
            gm: fault / 9.0,
        });
        let mut faulty = Circuit::new(els, (3, 0), 576_000.0);
        faulty.elements.pop();
        faulty.update();
        (faulty, bias)
    }

    /// A DC operating point that belongs to another circuit is refused, not held at rest.
    #[test]
    #[should_panic(expected = "misses the DC equations")]
    fn wrong_operating_point_is_not_pinned() {
        let (mut correct, bias) = follower_solved_with_fault(0.0);
        assert!(correct.dc_residual() < MAX_DC_RESIDUAL, "the right point is accepted");
        // 1e-13 A moves the bias by half a nanovolt and is already over the bound.
        let (mut small, _) = follower_solved_with_fault(1e-13);
        let bias_error = small.operating_point[bias - 1] - correct.operating_point[bias - 1];
        assert!(
            (bias_error - 0.5e-9).abs() < 0.02e-9,
            "1e-13 A moved the bias by {bias_error} V"
        );
        let residual = small.dc_residual();
        assert!(
            residual > 5.5e-11 && residual < 5.7e-11,
            "1e-13 A leaves a residual of {residual}"
        );
        // 100 nA moves it by half a millivolt. The pin itself would let that through: no row is
        // `MAX_PIN_VOLTS` from rest.
        let (mut faulty, _) = follower_solved_with_fault(100e-9);
        let bias_error = faulty.operating_point[bias - 1] - correct.operating_point[bias - 1];
        assert!(
            (bias_error - 0.5e-3).abs() < 0.01e-3,
            "100 nA moved the bias by {bias_error} V"
        );
        let residual = faulty.dc_residual();
        assert!(
            residual > 5.5e-5 && residual < 5.7e-5,
            "100 nA leaves a residual of {residual}"
        );
        faulty.pin_operating_point();
    }

    /// This Project's Model with its Controls at the given rotations, built as `Processor` does.
    fn model_netlist(rotations: &[f32]) -> Netlist {
        assert_eq!(rotations.len(), model::CONTROLS.len());
        let plain: Vec<f64> = model::CONTROLS
            .iter()
            .zip(rotations)
            .map(|(spec, &r)| spec.plain(r))
            .collect();
        let mut n = Netlist::default();
        model::circuit(&mut n);
        let pots = n.pots.clone();
        model::pots(&plain, &mut |name, ohms| {
            let index = pots.iter().find(|p| p.0 == name).unwrap().1;
            n.elements[index] = match n.elements[index] {
                Element::Resistor(a, b, _) => Element::Resistor(a, b, ohms.max(1.0)),
                _ => unreachable!(),
            };
        });
        n
    }

    /// Bound on the Control positions `scan_positions` returns, and so on the Circuits the DC
    /// bounds test builds at each rate.
    const SCAN_MAX_POSITIONS: usize = 4096;

    /// A Control rotation at which the Class A Transistor Drive's DC solve once ended on its
    /// largest last Newton step, 5.6e-8 (0.2496256, between the positions a coarse grid visits).
    /// Every Control of every Project is measured there.
    const SCAN_STALL_ROTATION_BITS: u32 = 0x3e7f_9dda;

    /// Control positions for the DC bounds test: at most `SCAN_MAX_POSITIONS`, the same on every
    /// run. A Model with no Control has one position. Otherwise, in this order and without
    /// repeats until the bound is reached:
    /// - every Control alike at 0, 0.1, 0.5, 0.9 and 1;
    /// - each Control in turn at those five values and at `SCAN_STALL_ROTATION_BITS`, the others
    ///   at their defaults;
    /// - every combination of the ends 0 and 1 (128 for seven Controls);
    /// - positions drawn uniformly from the 2^24 rotations k / 2^24 by a fixed pseudo-random
    ///   sequence (SplitMix64), one draw per Control, up to the bound.
    fn scan_positions() -> Vec<Vec<f32>> {
        const GRID: [f32; 5] = [0.0, 0.1, 0.5, 0.9, 1.0];
        let controls = model::CONTROLS.len();
        if controls == 0 {
            return vec![Vec::new()];
        }
        assert!(controls <= 8, "the corner combinations alone would pass the bound");
        let defaults: Vec<f32> = model::CONTROLS.iter().map(|c| c.default).collect();
        let stall = f32::from_bits(SCAN_STALL_ROTATION_BITS);
        let mut positions: Vec<Vec<f32>> = GRID.iter().map(|&g| vec![g; controls]).collect();
        for control in 0..controls {
            for value in GRID.into_iter().chain([stall]) {
                let mut position = defaults.clone();
                position[control] = value;
                positions.push(position);
            }
        }
        for corner in 0..1usize << controls {
            positions.push((0..controls).map(|c| (corner >> c & 1) as f32).collect());
        }
        // Same bit pattern, same position; no rotation here is NaN or -0.
        let key = |p: &Vec<f32>| p.iter().map(|r| r.to_bits()).collect::<Vec<u32>>();
        let mut seen = std::collections::HashSet::new();
        positions.retain(|p| seen.insert(key(p)));
        assert!(positions.len() <= SCAN_MAX_POSITIONS);
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        // Bounded: a draw repeats an earlier position with probability under 2^-12 per Control.
        for _ in 0..2 * SCAN_MAX_POSITIONS {
            if positions.len() == SCAN_MAX_POSITIONS {
                break;
            }
            let position: Vec<f32> = (0..controls)
                .map(|_| {
                    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
                    let mut x = state;
                    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                    x ^= x >> 31;
                    (x >> 40) as f32 / (1u32 << 24) as f32
                })
                .collect();
            if seen.insert(key(&position)) {
                positions.push(position);
            }
        }
        assert_eq!(positions.len(), SCAN_MAX_POSITIONS);
        positions
    }

    #[test]
    fn scan_positions_are_bounded_fixed_and_cover_the_named_ones() {
        let positions = scan_positions();
        assert_eq!(positions, scan_positions(), "the same positions on every run");
        let controls = model::CONTROLS.len();
        if controls == 0 {
            assert_eq!(positions, vec![Vec::<f32>::new()]);
            return;
        }
        assert_eq!(positions.len(), SCAN_MAX_POSITIONS);
        assert!(positions
            .iter()
            .all(|p| p.len() == controls && p.iter().all(|r| (0.0..=1.0).contains(r))));
        let defaults: Vec<f32> = model::CONTROLS.iter().map(|c| c.default).collect();
        for control in 0..controls {
            let mut stall = defaults.clone();
            stall[control] = f32::from_bits(SCAN_STALL_ROTATION_BITS);
            assert!(positions.contains(&stall), "Control {control} at the stall rotation");
            for end in [0.0f32, 1.0] {
                assert!(positions.contains(&vec![end; controls]));
            }
            // The drawn positions are spread over the travel: every sixteenth of it is visited.
            for bin in 0..16 {
                assert!(
                    positions.iter().any(|p| (p[control] * 16.0) as usize == bin),
                    "Control {control} never lands in sixteenth {bin}"
                );
            }
        }
    }

    /// What a return to rest does once a Control has moved, for every Control of this Project's
    /// Model at both ends of its travel. The rest state was solved at construction, so it is the
    /// operating point of the old Control positions: the return puts exactly that state back, and
    /// the circuit then settles to the operating point of the Controls as they are now, with no
    /// further held sample. The cost is the transient of that settling, printed here in volts at
    /// the output beside that of the pot change alone (`cargo test -p dsp -- --nocapture
    /// return_to_rest`).
    ///
    /// Run at the base rate: the DC equations do not contain the rate, and settling takes seconds.
    #[test]
    fn return_to_rest_after_a_control_change_restores_the_point_solved_at_construction() {
        const RATE: f64 = 48_000.0;
        const SETTLE_SAMPLES: usize = 4 * 48_000;
        let defaults: Vec<f32> = model::CONTROLS.iter().map(|c| c.default).collect();
        // Largest distance between two circuits' capacitor voltages, from their history states.
        let distance = |a: &Circuit, b: &Circuit| {
            (0..a.x.len())
                .map(|k| 2.0 * (a.x[k] - b.x[k]).abs() / a.gc2[k].abs())
                .fold(0.0, f64::max)
        };
        for (control, spec) in model::CONTROLS.iter().enumerate() {
            for end in [0.0f32, 1.0] {
                let mut moved = defaults.clone();
                moved[control] = end;
                let (built, target) = (model_netlist(&defaults), model_netlist(&moved));
                let fresh = Circuit::new(target.elements.clone(), (target.output, 0), RATE);
                let mut c = Circuit::new(built.elements, (built.output, 0), RATE);
                let (nx, at_construction) = (c.x.len(), c.rest.clone());
                // Move the pots as `Processor::set_control` does, and let the circuit start to
                // follow them.
                for &(_, index) in &target.pots {
                    let Element::Resistor(_, _, ohms) = target.elements[index] else {
                        unreachable!()
                    };
                    c.set_resistor(index, ohms);
                }
                // For scale: the transient of the same pot change with no return to rest.
                let mut undisturbed = c.clone();
                let mut pot_transient = 0.0f64;
                for _ in 0..SETTLE_SAMPLES {
                    pot_transient = pot_transient.max((undisturbed.tick(0.0) - fresh.y).abs());
                }
                for _ in 0..480 {
                    c.tick(0.0);
                }
                assert_eq!(c.failed_solves(), 0, "{} at {end}", spec.name);
                // A state that no sample solves from, whatever the solve path: every sample's
                // result is made non-finite. `MAX_CONSECUTIVE_HELD` of them bring the return.
                c.lin[0] = f64::NAN;
                for _ in 0..MAX_CONSECUTIVE_HELD {
                    assert!(c.tick(0.0).is_finite());
                }
                assert_eq!(
                    (c.failed_solves(), c.rest_restores()),
                    (MAX_CONSECUTIVE_HELD as u64, 1),
                    "{} at {end}",
                    spec.name
                );
                // The state is the one solved at construction, not the present Controls' own.
                assert_eq!(c.x, at_construction[..nx]);
                assert_eq!(c.y, at_construction[at_construction.len() - 1]);
                let stale_by = distance(&c, &fresh);
                // Samples solve again (the matrices are rebuilt as a pot change rebuilds them).
                c.update();
                let mut transient = 0.0f64;
                for _ in 0..SETTLE_SAMPLES {
                    let y = c.tick(0.0);
                    assert!(y.is_finite());
                    transient = transient.max((y - fresh.y).abs());
                }
                let left = distance(&c, &fresh);
                println!(
                    "{} at {end}: restored state {stale_by:e} V from the present operating point, \
                     {left:e} V after {} s, output transient {transient:e} V ({pot_transient:e} V \
                     for the pot change alone)",
                    spec.name,
                    SETTLE_SAMPLES as f64 / RATE
                );
                assert_eq!(c.failed_solves(), MAX_CONSECUTIVE_HELD as u64, "held after the return");
                assert_eq!(c.rest_restores(), 1);
                // The rows are unpinned after a pot change, so the circuit settles to where they
                // rest, which `MAX_PIN_VOLTS` allows to be that far from the operating point.
                assert!(
                    left <= 1e-3 * stale_by + MAX_PIN_VOLTS,
                    "{} at {end}: {left} V from the present operating point, started {stale_by} V away",
                    spec.name
                );
            }
        }
    }

    /// A pot change rebuilds the rows without the pin: from then on they are the rows of an
    /// unpinned build, even when the pot goes back to where it was.
    #[test]
    fn a_pot_change_drops_the_pin() {
        let defaults: Vec<f32> = model::CONTROLS.iter().map(|c| c.default).collect();
        let n = model_netlist(&defaults);
        let mut c = Circuit::new(n.elements.clone(), (n.output, 0), 576_000.0);
        let mut unpinned = c.clone();
        unpinned.update();
        for &(_, index) in &n.pots {
            let Element::Resistor(_, _, ohms) = n.elements[index] else {
                unreachable!()
            };
            c.set_resistor(index, 2.0 * ohms);
            c.set_resistor(index, ohms);
            c.tick(0.0);
            assert_eq!(c.lin, unpinned.lin);
        }
    }

    /// Room the measurement must leave under each bound, in every Project. Measured over
    /// `scan_positions`, the least room on the step is 50 times (Green Diode Overdrive) and on
    /// the pin distance 87 times (the same). The residual's is set by the Class A Transistor
    /// Drive, which leaves 35.8 times over these positions and 34 over 50,000 Level positions;
    /// the next are the Four Stage Sustainer at 180 and the Feedback Pair Drive at 669, and the
    /// other five leave more than 1000.
    const DC_STEP_HEADROOM: f64 = 10.0;
    const DC_RESIDUAL_HEADROOM: f64 = 30.0;
    const PIN_HEADROOM: f64 = 50.0;

    /// Every position of `scan_positions`, at the factor the Processor uses for each base rate:
    /// the Model's Circuit is accepted by the constructor's checks with room to spare. Each
    /// position goes through `Circuit::new`, so the constructor's own asserts run on it. Prints
    /// the worst last Newton step, DC residual and pin distance with the positions they occurred
    /// at, which is what `MAX_DC_STEP`, `MAX_DC_RESIDUAL` and `MAX_PIN_VOLTS` are set from. Up to
    /// `4 * SCAN_MAX_POSITIONS` Circuits, each solved twice, spread over the machine's cores:
    /// run it with `cargo test --release -p dsp -- --ignored --nocapture dc_bounds`. Measured on
    /// 16 cores with other work running: 4 s for a Model with no Control, 11 s to 94 s for five
    /// of the Models, and about 10 minutes each for the Green Diode Overdrive and the Four Stage
    /// Sustainer, whose DC solves use all their iterations. The plain test run still builds the
    /// Model at both ends of every Control, in `processor::tests::silence_stays_at_rest`.
    #[test]
    #[ignore]
    fn dc_bounds_over_controls_and_rates() {
        let positions = scan_positions();
        assert!(!positions.is_empty() && positions.len() <= SCAN_MAX_POSITIONS);
        let workers = std::thread::available_parallelism().map_or(1, |n| n.get()).min(16);
        let chunk = positions.len().div_ceil(workers);
        // Largest of each measurement with the position it occurred at.
        type Worst = [(f64, usize); 3];
        let larger = |a: (f64, usize), b: (f64, usize)| if b.0 > a.0 || b.0.is_nan() { b } else { a };
        let mut overall: Worst = [(0.0, 0); 3];
        for rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let circuit_rate = rate * crate::oversample::factor_for(rate) as f64;
            let worst: Worst = std::thread::scope(|scope| {
                let handles: Vec<_> = positions
                    .chunks(chunk)
                    .enumerate()
                    .map(|(k, part)| {
                        scope.spawn(move || {
                            let mut worst: Worst = [(0.0, 0); 3];
                            for (offset, rotations) in part.iter().enumerate() {
                                let at = k * chunk + offset;
                                let n = model_netlist(rotations);
                                let mut c = Circuit::new(n.elements, (n.output, 0), circuit_rate);
                                let residual = c.dc_residual();
                                let step = c.solve_operating_point();
                                // The constructor pinned `lin`; build it again without the pin
                                // to measure it.
                                c.update();
                                let pin = c.pin_operating_point();
                                for (w, value) in worst.iter_mut().zip([step, residual, pin]) {
                                    *w = larger(*w, (value, at));
                                }
                            }
                            worst
                        })
                    })
                    .collect();
                let mut worst: Worst = [(0.0, 0); 3];
                for handle in handles {
                    let part = handle.join().expect("a position failed a constructor check");
                    for (w, p) in worst.iter_mut().zip(part) {
                        *w = larger(*w, p);
                    }
                }
                worst
            });
            let [step, residual, pin] = worst;
            println!(
                "{rate} Hz, {} positions: last Newton step {:e} at {:?}, DC residual {:e} at {:?}, pin {:e} V at {:?}",
                positions.len(),
                step.0,
                positions[step.1],
                residual.0,
                positions[residual.1],
                pin.0,
                positions[pin.1]
            );
            for (o, w) in overall.iter_mut().zip(worst) {
                *o = larger(*o, w);
            }
        }
        let [step, residual, pin] = overall.map(|w| w.0);
        println!(
            "worst of all: last Newton step {step:e} ({:.0}x under the bound), DC residual {residual:e} ({:.0}x), \
             pin {pin:e} V ({:.0}x)",
            MAX_DC_STEP / step,
            MAX_DC_RESIDUAL / residual,
            MAX_PIN_VOLTS / pin
        );
        assert!(step < MAX_DC_STEP / DC_STEP_HEADROOM, "last Newton step {step}");
        assert!(
            residual < MAX_DC_RESIDUAL / DC_RESIDUAL_HEADROOM,
            "DC residual {residual}"
        );
        assert!(pin < MAX_PIN_VOLTS / PIN_HEADROOM, "pin distance {pin} V");
    }
}
