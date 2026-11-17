use crate::circuit::Circuit;
use crate::model;
use crate::netlist::Netlist;
use crate::oversample::{factor_for, Oversampler};

const MIN_POT_OHMS: f64 = 1.0;

/// Stereo processor: one circuit per channel, oversampled.
pub struct Processor {
    channels: Vec<(Circuit, Oversampler)>,
    pots: Vec<(&'static str, usize)>,
    controls: Vec<f64>,
}

impl Processor {
    /// Builds the circuit at its DC operating point for the given knob rotations. A NaN rotation
    /// is replaced by that Control's default. Allocates; call off the audio thread.
    pub fn new(sample_rate: f64, rotations: &[f32]) -> Self {
        let controls: Vec<f64> = model::CONTROLS
            .iter()
            .zip(rotations)
            .map(|(spec, &r)| spec.plain(if r.is_nan() { spec.default } else { r }))
            .collect();
        let mut netlist = Netlist::default();
        model::circuit(&mut netlist);
        let pots = netlist.pots.clone();
        model::pots(&controls, &mut |name, ohms| {
            let index = pots
                .iter()
                .find(|p| p.0 == name)
                .unwrap_or_else(|| panic!("unknown pot {name}"))
                .1;
            if let crate::circuit::Element::Resistor(_, _, r) = &mut netlist.elements[index] {
                *r = ohms.max(MIN_POT_OHMS);
            }
        });
        let factor = factor_for(sample_rate);
        let circuit = Circuit::new(netlist.elements, (netlist.output, 0), sample_rate * factor as f64);
        let channels = vec![
            (circuit.clone(), Oversampler::new(sample_rate, factor)),
            (circuit, Oversampler::new(sample_rate, factor)),
        ];
        Self {
            channels,
            pots,
            controls,
        }
    }

    /// Sets a control by knob rotation. Allocation-free. A NaN rotation is rejected: the Control
    /// keeps its value. (Infinite rotations clamp to the ends of travel like any out-of-range one.)
    pub fn set_control(&mut self, index: usize, rotation: f32) {
        if rotation.is_nan() {
            return;
        }
        let value = model::CONTROLS[index].plain(rotation);
        if self.controls[index] == value {
            return;
        }
        self.controls[index] = value;
        let (pots, channels) = (&self.pots, &mut self.channels);
        model::pots(&self.controls, &mut |name, ohms| {
            if let Some(&(_, index)) = pots.iter().find(|p| p.0 == name) {
                for (circuit, _) in channels.iter_mut() {
                    circuit.set_resistor(index, ohms.max(MIN_POT_OHMS));
                }
            }
        });
    }

    /// Processes one block in place. A non-finite input sample is treated as silence (0) before
    /// it reaches the oversampling filters, so it cannot linger in their history or in the circuit.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let scale = model::VOLTS_PER_FULL_SCALE;
        for ((circuit, os), buf) in self.channels.iter_mut().zip([left, right]) {
            for x in buf.iter_mut() {
                let u = if x.is_finite() { *x as f64 * scale } else { 0.0 };
                *x = (os.process(u, |u| circuit.tick(u)) / scale) as f32;
            }
        }
    }

    /// Circuit samples since construction, over both channels, whose solve failed and were held
    /// at the last valid state and output (see `Circuit::tick`). Saturates. Zero in normal use.
    pub fn failed_solves(&self) -> u64 {
        self.channels
            .iter()
            .fold(0, |n, (circuit, _)| n.saturating_add(circuit.failed_solves()))
    }

    /// Delay in samples from the oversampling filters.
    pub fn latency(&self) -> usize {
        self.channels[0].1.latency()
    }

    /// Control values in physical units, as substituted into the SPICE netlist.
    pub fn plain_controls(&self) -> &[f64] {
        &self.controls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48_000.0;

    fn sine(len: usize, peak: f32) -> Vec<f32> {
        (0..len)
            .map(|k| peak * (std::f32::consts::TAU * 1000.0 * k as f32 / RATE as f32).sin())
            .collect()
    }

    #[test]
    fn non_finite_input_samples_are_silence() {
        // One NaN and one infinity in a driven block, then silence. The output must equal, bit
        // for bit, that of a twin fed 0 in their place: nothing lingers in the filters or circuit.
        let mut dirty = sine(2048, 0.5);
        dirty.extend(std::iter::repeat_n(0.0, 2048));
        let mut clean = dirty.clone();
        (dirty[700], dirty[1300]) = (f32::NAN, f32::INFINITY);
        (clean[700], clean[1300]) = (0.0, 0.0);
        let (mut a, mut b) = (Processor::new(RATE, &[1.0]), Processor::new(RATE, &[1.0]));
        let (mut dirty_r, mut clean_r) = (dirty.clone(), clean.clone());
        a.process(&mut dirty, &mut dirty_r);
        b.process(&mut clean, &mut clean_r);
        assert!(dirty.iter().all(|y| y.is_finite()));
        assert_eq!(dirty, clean);
        assert_eq!(dirty_r, clean_r);
        assert_eq!(a.failed_solves(), 0);
    }

    #[test]
    fn nan_input_then_silence_returns_to_rest() {
        // A NaN first sample, then two seconds of silence: every output is finite and the last
        // tenth of a second is at rest.
        let mut left = vec![0.0f32; 96_000];
        left[0] = f32::NAN;
        let mut right = left.clone();
        let mut p = Processor::new(RATE, &[1.0]);
        p.process(&mut left, &mut right);
        assert!(left.iter().chain(&right).all(|y| y.is_finite()));
        assert!(left[91_200..].iter().chain(&right[91_200..]).all(|y| y.abs() < 1e-6));
        assert_eq!(p.failed_solves(), 0);
    }

    #[test]
    fn nan_control_is_rejected_without_a_panic() {
        // NaN at construction means the default; NaN later keeps the value in force.
        let mut p = Processor::new(RATE, &[f32::NAN]);
        assert_eq!(
            p.plain_controls(),
            [model::CONTROLS[0].plain(model::CONTROLS[0].default)]
        );
        p.set_control(0, 0.9);
        let before = p.plain_controls().to_vec();
        p.set_control(0, f32::NAN);
        assert_eq!(p.plain_controls(), before);
        p.set_control(0, f32::INFINITY);
        assert_eq!(p.plain_controls(), [model::CONTROLS[0].plain(1.0)]);
        let (mut left, mut right) = (sine(512, 0.5), sine(512, 0.5));
        p.process(&mut left, &mut right);
        assert!(left.iter().chain(&right).all(|y| y.is_finite()));
    }
}
