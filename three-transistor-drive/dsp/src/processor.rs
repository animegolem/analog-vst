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
    /// Builds the circuit at its DC operating point for the given knob rotations. Allocates; call
    /// off the audio thread.
    pub fn new(sample_rate: f64, rotations: &[f32]) -> Self {
        let controls: Vec<f64> = model::CONTROLS
            .iter()
            .zip(rotations)
            .map(|(spec, &r)| spec.plain(r))
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

    /// Sets a control by knob rotation. Allocation-free.
    pub fn set_control(&mut self, index: usize, rotation: f32) {
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

    /// Processes both channels in place. A non-finite input sample (NaN or infinity) is treated
    /// as 0 before it reaches the oversampling filter, so it cannot linger in the filter's
    /// history. Never panics; a sample the solver cannot solve is held (see `failed_samples`).
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let scale = model::VOLTS_PER_FULL_SCALE;
        for ((circuit, os), buf) in self.channels.iter_mut().zip([left, right]) {
            for x in buf.iter_mut() {
                let u = if x.is_finite() { *x as f64 * scale } else { 0.0 };
                *x = (os.process(u, |u| circuit.tick(u)) / scale) as f32;
            }
        }
    }

    /// Circuit-rate samples, over both channels since construction, that the solver did not
    /// accept. Each one kept the circuit's state and repeated the previous circuit-rate output,
    /// so the audio stays finite but is not the Circuit's response there. 0 means every sample
    /// was solved.
    pub fn failed_samples(&self) -> u64 {
        self.channels
            .iter()
            .fold(0, |n, (circuit, _)| n.saturating_add(circuit.failures()))
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

    /// NaN and both infinities at the Plugin's input, then silence, at full drive: every output
    /// sample is finite and bit-identical to a processor that was fed silence throughout.
    #[test]
    fn non_finite_input_is_silence_and_does_not_linger() {
        let mut dirty = Processor::new(48_000.0, &[1.0]);
        let mut clean = Processor::new(48_000.0, &[1.0]);
        let n = dirty.latency() + 64;
        let (mut left, mut right) = (vec![0.0f32; n], vec![0.0f32; n]);
        (left[1], left[2], left[5]) = (f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
        right[0] = f32::NAN;
        dirty.process(&mut left, &mut right);
        let (mut left_ref, mut right_ref) = (vec![0.0f32; n], vec![0.0f32; n]);
        clean.process(&mut left_ref, &mut right_ref);
        for (got, want) in left.iter().chain(&right).zip(left_ref.iter().chain(&right_ref)) {
            assert!(got.is_finite());
            assert_eq!(got.to_bits(), want.to_bits());
        }
        assert_eq!(dirty.failed_samples(), 0);
    }
}
