//! Oscillators module: Real-time safe oscillator implementations.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut, OscillatorState};

/// Sine wave oscillator.
#[derive(Clone)]
pub struct SineOsc {
    pub freq: Sample,
    state: OscillatorState,
}

impl SineOsc {
    pub fn new(freq: Sample) -> Self {
        Self {
            freq,
            state: OscillatorState::default(),
        }
    }
}

impl DspNode for SineOsc {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("SineOsc requires at least 1 output");
        }
        
        let step = 2.0 * std::f32::consts::PI * self.freq / sample_rate;
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                *sample = self.state.phase.sin();
                self.state.phase += step;
                // Wrap phase to prevent precision loss
                self.state.phase %= 2.0 * std::f32::consts::PI;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.state = OscillatorState::default();
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Sawtooth wave oscillator.
#[derive(Clone)]
pub struct SawOsc {
    pub freq: Sample,
    state: OscillatorState,
}

impl SawOsc {
    pub fn new(freq: Sample) -> Self {
        Self {
            freq,
            state: OscillatorState::default(),
        }
    }
}

impl DspNode for SawOsc {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("SawOsc requires at least 1 output");
        }
        
        let step = self.freq / sample_rate;
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                *sample = 2.0 * (self.state.phase - self.state.phase.floor()) - 1.0;
                self.state.phase += step;
                self.state.phase %= 1.0;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.state = OscillatorState::default();
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Square wave oscillator.
#[derive(Clone)]
pub struct SquareOsc {
    pub freq: Sample,
    pub duty: Sample,
    state: OscillatorState,
}

impl SquareOsc {
    pub fn new(freq: Sample, duty: Sample) -> Self {
        Self {
            freq,
            duty: duty.clamp(0.0, 1.0),
            state: OscillatorState::default(),
        }
    }
}

impl DspNode for SquareOsc {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("SquareOsc requires at least 1 output");
        }
        
        let step = self.freq / sample_rate;
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                *sample = if (self.state.phase % 1.0) < self.duty { 1.0 } else { -1.0 };
                self.state.phase += step;
                self.state.phase %= 1.0;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.state = OscillatorState::default();
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Triangle wave oscillator.
#[derive(Clone)]
pub struct TriangleOsc {
    pub freq: Sample,
    state: OscillatorState,
}

impl TriangleOsc {
    pub fn new(freq: Sample) -> Self {
        Self {
            freq,
            state: OscillatorState::default(),
        }
    }
}

impl DspNode for TriangleOsc {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("TriangleOsc requires at least 1 output");
        }
        
        let step = self.freq / sample_rate;
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                let phase = self.state.phase % 1.0;
                *sample = if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                };
                self.state.phase += step;
                self.state.phase %= 1.0;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.state = OscillatorState::default();
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// White noise oscillator.
#[derive(Clone)]
pub struct NoiseOsc {
    seed: u32,
}

impl NoiseOsc {
    pub fn new() -> Self {
        Self { seed: 12345 }
    }
}

impl DspNode for NoiseOsc {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("NoiseOsc requires at least 1 output");
        }
        
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                // Simple LCG for white noise (safe implementation)
                self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
                *sample = (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.seed = 12345;
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Pulse wave oscillator.
#[derive(Clone)]
pub struct PulseOsc {
    pub freq: Sample,
    pub width: Sample,
    state: OscillatorState,
}

impl PulseOsc {
    pub fn new(freq: Sample, width: Sample) -> Self {
        Self {
            freq,
            width: width.clamp(0.0, 1.0),
            state: OscillatorState::default(),
        }
    }
}

impl DspNode for PulseOsc {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("PulseOsc requires at least 1 output");
        }
        
        let step = self.freq / sample_rate;
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                let phase = self.state.phase % 1.0;
                *sample = if phase < self.width { 1.0 } else { -1.0 };
                self.state.phase += step;
                self.state.phase %= 1.0;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.state = OscillatorState::default();
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// FM oscillator (simple implementation).
#[derive(Clone)]
pub struct SineFm {
    pub carrier: Sample,
    pub modulator: Sample,
    pub index: Sample,
    carrier_phase: Sample,
    modulator_phase: Sample,
}

impl SineFm {
    pub fn new(carrier: Sample, modulator: Sample, index: Sample) -> Self {
        Self {
            carrier,
            modulator,
            index,
            carrier_phase: 0.0,
            modulator_phase: 0.0,
        }
    }
}

impl DspNode for SineFm {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("SineFm requires at least 1 output");
        }
        
        let carrier_step = self.carrier / sample_rate;
        let modulator_step = self.modulator / sample_rate;
        
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                let modulation = (self.modulator_phase * 2.0 * std::f32::consts::PI).sin() * self.index;
                let carrier_freq = self.carrier + modulation;
                let instant_phase = self.carrier_phase * 2.0 * std::f32::consts::PI * carrier_freq / self.carrier;
                
                *sample = instant_phase.sin();
                
                self.carrier_phase += carrier_step;
                self.modulator_phase += modulator_step;
                
                self.carrier_phase %= 1.0;
                self.modulator_phase %= 1.0;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.carrier_phase = 0.0;
        self.modulator_phase = 0.0;
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}