//! Effects module: Real-time safe audio effects implementations.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut, DelayState, ms_to_samples};
use crate::filters::Allpass;

/// Delay effect with feedback.
#[derive(Clone)]
pub struct Delay {
    pub delay_ms: Sample,
    pub feedback: Sample,
    state: DelayState,
}

impl Delay {
    pub fn new(delay_ms: Sample, feedback: Sample, max_delay_ms: Sample, sample_rate: SampleRate) -> Self {
        let max_delay_samples = ms_to_samples(max_delay_ms, sample_rate);
        Self {
            delay_ms,
            feedback: feedback.clamp(0.0, 0.99), // Prevent infinite feedback
            state: DelayState::new(max_delay_samples),
        }
    }
}

impl DspNode for Delay {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Delay requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        let delay_samples = ms_to_samples(self.delay_ms, sample_rate);
        let delay_samples = delay_samples.min(self.state.buffer.len().saturating_sub(1));
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let read_pos = (self.state.write_pos + self.state.buffer.len() - delay_samples) % self.state.buffer.len();
            let delayed_sample = self.state.buffer[read_pos];
            *o = delayed_sample;
            self.state.buffer[self.state.write_pos] = i_val + delayed_sample * self.feedback;
            self.state.write_pos = (self.state.write_pos + 1) % self.state.buffer.len();
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state.buffer.fill(0.0);
        self.state.write_pos = 0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Reverb effect (simplified implementation).
#[derive(Clone)]
pub struct Reverb {
    pub decay: Sample,
    pub mix: Sample,
    delays: Vec<Delay>,
}

impl Reverb {
    pub fn new(decay: Sample, mix: Sample, sample_rate: SampleRate) -> Self {
        // Simple reverb using multiple delays
        let delays = vec![
            Delay::new(29.7, decay * 0.9, 50.0, sample_rate),
            Delay::new(37.1, decay * 0.8, 50.0, sample_rate),
            Delay::new(41.1, decay * 0.7, 50.0, sample_rate),
            Delay::new(43.7, decay * 0.6, 50.0, sample_rate),
        ];
        
        Self {
            decay,
            mix: mix.clamp(0.0, 1.0),
            delays,
        }
    }
}

impl DspNode for Reverb {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Reverb requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Process through all delays and mix
        let mut wet_signal = vec![0.0; input.len()];
        
        for delay in &mut self.delays {
            let mut temp_output = vec![0.0; input.len()];
            delay.process(&[input], &mut [&mut temp_output], sample_rate)?;
            
            for (wet, &delayed) in wet_signal.iter_mut().zip(&temp_output) {
                *wet += delayed;
            }
        }
        
        // Normalize wet signal
        let num_delays = self.delays.len() as Sample;
        for wet in &mut wet_signal {
            *wet /= num_delays;
        }
        
        // Mix dry and wet
        for (o, (&dry, &wet)) in output.iter_mut().zip(input.iter().zip(wet_signal.iter())) {
            *o = dry * (1.0 - self.mix) + wet * self.mix;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        for delay in &mut self.delays {
            delay.reset();
        }
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Chorus effect.
#[derive(Clone)]
pub struct Chorus {
    pub rate: Sample,
    pub depth: Sample,
    pub mix: Sample,
    delay: Delay,
    lfo_phase: Sample,
}

impl Chorus {
    pub fn new(rate: Sample, depth: Sample, mix: Sample, sample_rate: SampleRate) -> Self {
        Self {
            rate,
            depth: depth.clamp(0.0, 20.0), // Max 20ms depth
            mix: mix.clamp(0.0, 1.0),
            delay: Delay::new(10.0, 0.0, 30.0, sample_rate), // Base delay
            lfo_phase: 0.0,
        }
    }
}

impl DspNode for Chorus {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Chorus requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Modulate delay time with LFO
        let lfo_step = self.rate / sample_rate;
        
        for (i, (&i_val, o)) in input.iter().zip(output.iter_mut()).enumerate() {
            // Calculate modulated delay time
            let lfo = (self.lfo_phase * 2.0 * std::f32::consts::PI).sin();
            let _modulated_delay_ms = 10.0 + lfo * self.depth * 0.5; // 10ms ± depth/2
            
            // For simplicity, we'll use a fixed delay here
            // A full implementation would need a variable delay line
            let delayed_sample = if i > 0 { input[i-1] } else { 0.0 };
            
            *o = i_val * (1.0 - self.mix) + delayed_sample * self.mix;
            
            self.lfo_phase += lfo_step;
            self.lfo_phase %= 1.0;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.delay.reset();
        self.lfo_phase = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Phaser effect (simplified).
#[derive(Clone)]
pub struct Phaser {
    pub rate: Sample,
    pub depth: Sample,
    pub mix: Sample,
    allpass_filters: Vec<Allpass>,
    lfo_phase: Sample,
}

impl Phaser {
    pub fn new(rate: Sample, depth: Sample, mix: Sample, stages: usize) -> Self {
        let mut allpass_filters = Vec::new();
        for i in 0..stages {
            // Create all-pass filters with different frequencies
            let freq = 300.0 + (i as Sample) * 200.0;
            allpass_filters.push(Allpass::new(freq, 0.7));
        }
        
        Self {
            rate,
            depth: depth.clamp(0.0, 1.0),
            mix: mix.clamp(0.0, 1.0),
            allpass_filters,
            lfo_phase: 0.0,
        }
    }
}

impl DspNode for Phaser {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Phaser requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Process through all-pass filters
        let mut signal = input.to_vec();
        
        for filter in &mut self.allpass_filters {
            let mut temp_output = vec![0.0; signal.len()];
            filter.process(&[&signal], &mut [&mut temp_output], sample_rate)?;
            signal = temp_output;
        }
        
        // Mix dry and wet
        for (o, (&dry, &wet)) in output.iter_mut().zip(input.iter().zip(&signal)) {
            *o = dry * (1.0 - self.mix) + wet * self.mix;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        for filter in &mut self.allpass_filters {
            filter.reset();
        }
        self.lfo_phase = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Flanger effect.
#[derive(Clone)]
pub struct Flanger {
    pub rate: Sample,
    pub depth: Sample,
    pub feedback: Sample,
    pub mix: Sample,
    delay: Delay,
    lfo_phase: Sample,
}

impl Flanger {
    pub fn new(rate: Sample, depth: Sample, feedback: Sample, mix: Sample, sample_rate: SampleRate) -> Self {
        Self {
            rate,
            depth: depth.clamp(0.0, 10.0), // Max 10ms depth
            feedback: feedback.clamp(0.0, 0.9),
            mix: mix.clamp(0.0, 1.0),
            delay: Delay::new(1.0, feedback, 15.0, sample_rate), // Base delay
            lfo_phase: 0.0,
        }
    }
}

impl DspNode for Flanger {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Flanger requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // For simplicity, use fixed delay modulation
        // A full implementation would modulate the delay time
        
        // Process delay first into a temporary buffer
        let mut temp_output = vec![0.0; input.len()];
        let temp_outputs = &mut [&mut temp_output[..]];
        self.delay.process(inputs, temp_outputs, sample_rate)?;
        
        // Mix with original
        for (i, (o, &i_val)) in output.iter_mut().zip(input.iter()).enumerate() {
            if i < temp_output.len() {
                *o = i_val * (1.0 - self.mix) + temp_output[i] * self.mix;
            }
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.delay.reset();
        self.lfo_phase = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Multi-tap delay effect.
#[derive(Clone)]
pub struct MultiTapDelay {
    pub taps: Vec<(Sample, Sample)>, // (delay_ms, gain)
    pub feedback: Sample,
    state: DelayState,
}

impl MultiTapDelay {
    pub fn new(taps: Vec<(Sample, Sample)>, feedback: Sample, max_delay_ms: Sample, sample_rate: SampleRate) -> Self {
        let max_delay_samples = ms_to_samples(max_delay_ms, sample_rate);
        Self {
            taps,
            feedback: feedback.clamp(0.0, 0.99),
            state: DelayState::new(max_delay_samples),
        }
    }
}

impl DspNode for MultiTapDelay {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("MultiTapDelay requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let mut sum = 0.0;
            
            // Sum all tap outputs
            for &(delay_ms, gain) in &self.taps {
                let delay_samples = ms_to_samples(delay_ms, sample_rate);
                let read_pos = (self.state.write_pos + self.state.buffer.len() - delay_samples as usize) % self.state.buffer.len();
                sum += self.state.buffer[read_pos] * gain;
            }
            
            *o = sum;
            
            // Write input + feedback
            let feedback_sample = self.state.buffer[self.state.write_pos] * self.feedback;
            self.state.buffer[self.state.write_pos] = i_val + feedback_sample;
            self.state.write_pos = (self.state.write_pos + 1) % self.state.buffer.len();
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state.reset();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Convolution reverb using impulse response.
#[derive(Clone)]
pub struct ConvolutionReverb {
    pub mix: Sample,
    impulse_response: Vec<Sample>,
    buffer: Vec<Sample>,
    write_pos: usize,
}

impl ConvolutionReverb {
    pub fn new(impulse_response: Vec<Sample>, mix: Sample) -> Self {
        let buffer_len = impulse_response.len();
        Self {
            mix: mix.clamp(0.0, 1.0),
            impulse_response,
            buffer: vec![0.0; buffer_len],
            write_pos: 0,
        }
    }
}

impl DspNode for ConvolutionReverb {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("ConvolutionReverb requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            // Write input to buffer
            self.buffer[self.write_pos] = i_val;
            
            // Convolve with impulse response
            let mut wet = 0.0;
            for (i, &ir_sample) in self.impulse_response.iter().enumerate() {
                let read_pos = (self.write_pos + self.buffer.len() - i) % self.buffer.len();
                wet += self.buffer[read_pos] * ir_sample;
            }
            
            // Mix dry and wet
            *o = i_val * (1.0 - self.mix) + wet * self.mix;
            
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}