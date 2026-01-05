//! Dynamics module: Compressors, limiters, and other dynamic processors.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut};

/// Compressor with RMS detection.
#[derive(Clone)]
pub struct Compressor {
    pub threshold: Sample,  // dB
    pub ratio: Sample,
    pub attack_ms: Sample,
    pub release_ms: Sample,
    pub makeup_gain: Sample, // dB
    envelope: Sample,
    attack_coeff: Sample,
    release_coeff: Sample,
}

impl Compressor {
    pub fn new(threshold: Sample, ratio: Sample, attack_ms: Sample, release_ms: Sample, makeup_gain: Sample, sample_rate: SampleRate) -> Self {
        let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();
        
        Self {
            threshold: 10.0_f32.powf(threshold / 20.0), // Convert to linear
            ratio: 1.0 / ratio,
            attack_ms,
            release_ms,
            makeup_gain: 10.0_f32.powf(makeup_gain / 20.0),
            envelope: 0.0,
            attack_coeff,
            release_coeff,
        }
    }
}

impl DspNode for Compressor {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Compressor requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            // Simple peak detection (could be RMS)
            let input_level = i_val.abs();
            
            // Update envelope
            if input_level > self.envelope {
                self.envelope = self.attack_coeff * (self.envelope - input_level) + input_level;
            } else {
                self.envelope = self.release_coeff * (self.envelope - input_level) + input_level;
            }
            
            // Calculate gain reduction
            let mut gain_reduction = 1.0;
            if self.envelope > self.threshold {
                let over_threshold = self.envelope / self.threshold;
                gain_reduction = over_threshold.powf(self.ratio - 1.0);
            }
            
            *o = i_val * gain_reduction * self.makeup_gain;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Limiter (brickwall compressor).
#[derive(Clone)]
pub struct Limiter {
    pub threshold: Sample,  // dB
    pub attack_ms: Sample,
    pub release_ms: Sample,
    envelope: Sample,
    attack_coeff: Sample,
    release_coeff: Sample,
}

impl Limiter {
    pub fn new(threshold: Sample, attack_ms: Sample, release_ms: Sample, sample_rate: SampleRate) -> Self {
        let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();
        
        Self {
            threshold: 10.0_f32.powf(threshold / 20.0),
            attack_ms,
            release_ms,
            envelope: 0.0,
            attack_coeff,
            release_coeff,
        }
    }
}

impl DspNode for Limiter {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Limiter requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let input_level = i_val.abs();
            
            // Update envelope
            if input_level > self.envelope {
                self.envelope = self.attack_coeff * (self.envelope - input_level) + input_level;
            } else {
                self.envelope = self.release_coeff * (self.envelope - input_level) + input_level;
            }
            
            // Hard limiting
            let gain_reduction = if self.envelope > self.threshold {
                self.threshold / self.envelope
            } else {
                1.0
            };
            
            *o = i_val * gain_reduction;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Expander/Gate.
#[derive(Clone)]
pub struct Gate {
    pub threshold: Sample,  // dB
    pub ratio: Sample,
    pub attack_ms: Sample,
    pub release_ms: Sample,
    envelope: Sample,
    attack_coeff: Sample,
    release_coeff: Sample,
}

impl Gate {
    pub fn new(threshold: Sample, ratio: Sample, attack_ms: Sample, release_ms: Sample, sample_rate: SampleRate) -> Self {
        let attack_coeff = (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = (-1.0 / (release_ms * 0.001 * sample_rate)).exp();
        
        Self {
            threshold: 10.0_f32.powf(threshold / 20.0),
            ratio,
            attack_ms,
            release_ms,
            envelope: 0.0,
            attack_coeff,
            release_coeff,
        }
    }
}

impl DspNode for Gate {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Gate requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let input_level = i_val.abs();
            
            // Update envelope
            if input_level > self.envelope {
                self.envelope = self.attack_coeff * (self.envelope - input_level) + input_level;
            } else {
                self.envelope = self.release_coeff * (self.envelope - input_level) + input_level;
            }
            
            // Calculate expansion
            let mut gain_reduction = 1.0;
            if self.envelope < self.threshold {
                let under_threshold = self.threshold / self.envelope.max(0.0001);
                gain_reduction = under_threshold.powf(1.0 - self.ratio);
            }
            
            *o = i_val * gain_reduction;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}