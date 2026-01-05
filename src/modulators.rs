//! Modulators module: LFOs, ring modulators, and other modulation effects.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut};

/// Low Frequency Oscillator.
#[derive(Clone)]
pub struct Lfo {
    pub frequency: Sample,
    pub waveform: LfoWaveform,
    pub phase: Sample, // 0.0 to 1.0
    phase_accum: Sample,
}

#[derive(Clone, Copy)]
pub enum LfoWaveform {
    Sine,
    Triangle,
    Square,
    Sawtooth,
    Random, // Sample and hold
}

impl Lfo {
    pub fn new(frequency: Sample, waveform: LfoWaveform) -> Self {
        Self {
            frequency,
            waveform,
            phase: 0.0,
            phase_accum: 0.0,
        }
    }
    
    fn generate_sample(&self, phase: Sample) -> Sample {
        match self.waveform {
            LfoWaveform::Sine => (phase * std::f32::consts::TAU).sin(),
            LfoWaveform::Triangle => {
                if phase < 0.25 {
                    phase * 4.0
                } else if phase < 0.75 {
                    1.0 - (phase - 0.25) * 4.0
                } else {
                    (phase - 0.75) * 4.0 - 1.0
                }
            },
            LfoWaveform::Square => if phase < 0.5 { 1.0 } else { -1.0 },
            LfoWaveform::Sawtooth => phase * 2.0 - 1.0,
            LfoWaveform::Random => {
                // Simple pseudo-random using phase
                ((phase * 12345.0).sin() * 43758.0).fract() * 2.0 - 1.0
            },
        }
    }
}

impl DspNode for Lfo {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("Lfo requires 1 output");
        }
        
        let output = &mut outputs[0];
        
        for o in output.iter_mut() {
            *o = self.generate_sample(self.phase_accum);
            self.phase_accum = (self.phase_accum + self.frequency / sample_rate).fract();
        }
        
        self.phase = self.phase_accum;
        
        Ok(())
    }

    fn reset(&mut self) {
        self.phase_accum = self.phase;
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Ring modulator.
#[derive(Clone)]
pub struct RingModulator {
    pub frequency: Sample,
    phase_accum: Sample,
}

impl RingModulator {
    pub fn new(frequency: Sample) -> Self {
        Self {
            frequency,
            phase_accum: 0.0,
        }
    }
}

impl DspNode for RingModulator {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("RingModulator requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let modulator = (self.phase_accum * std::f32::consts::TAU).sin();
            *o = i_val * modulator;
            self.phase_accum = (self.phase_accum + self.frequency / sample_rate).fract();
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.phase_accum = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Frequency shifter using Hilbert transform approximation.
#[derive(Clone)]
pub struct FrequencyShifter {
    pub shift_hz: Sample,
    phase_accum: Sample,
    hilbert_delay: Vec<Sample>,
    write_pos: usize,
}

impl FrequencyShifter {
    pub fn new(shift_hz: Sample, max_delay_samples: usize) -> Self {
        Self {
            shift_hz,
            phase_accum: 0.0,
            hilbert_delay: vec![0.0; max_delay_samples],
            write_pos: 0,
        }
    }
}

impl DspNode for FrequencyShifter {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("FrequencyShifter requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        let delay_samples = (sample_rate / (2.0 * self.shift_hz.abs())).round() as usize;
        let delay_samples = delay_samples.min(self.hilbert_delay.len() - 1);
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            // Simple 90-degree phase shift approximation
            let read_pos = (self.write_pos + self.hilbert_delay.len() - delay_samples) % self.hilbert_delay.len();
            let hilbert = self.hilbert_delay[read_pos];
            
            // Frequency shifting
            let shift_phase = self.phase_accum * std::f32::consts::TAU;
            let cos_shift = shift_phase.cos();
            let sin_shift = shift_phase.sin();
            
            *o = i_val * cos_shift + hilbert * sin_shift;
            
            // Update delay line
            self.hilbert_delay[self.write_pos] = i_val;
            self.write_pos = (self.write_pos + 1) % self.hilbert_delay.len();
            
            self.phase_accum = (self.phase_accum + self.shift_hz / sample_rate).fract();
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.phase_accum = 0.0;
        self.hilbert_delay.fill(0.0);
        self.write_pos = 0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Tremolo effect.
#[derive(Clone)]
pub struct Tremolo {
    pub frequency: Sample,
    pub depth: Sample,
    phase_accum: Sample,
}

impl Tremolo {
    pub fn new(frequency: Sample, depth: Sample) -> Self {
        Self {
            frequency,
            depth: depth.clamp(0.0, 1.0),
            phase_accum: 0.0,
        }
    }
}

impl DspNode for Tremolo {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Tremolo requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let lfo = (self.phase_accum * std::f32::consts::TAU).sin() * 0.5 + 0.5; // 0 to 1
            let modulation = 1.0 - self.depth + self.depth * lfo;
            *o = i_val * modulation;
            self.phase_accum = (self.phase_accum + self.frequency / sample_rate).fract();
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.phase_accum = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}