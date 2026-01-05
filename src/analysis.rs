//! Analysis module: Signal analysis and metering.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut};

/// RMS level detector.
#[derive(Clone)]
pub struct Rms {
    pub window_size: usize,
    buffer: Vec<Sample>,
    write_pos: usize,
    sum_squares: Sample,
}

impl Rms {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            buffer: vec![0.0; window_size],
            write_pos: 0,
            sum_squares: 0.0,
        }
    }
}

impl DspNode for Rms {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Rms requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            // Remove oldest sample from sum
            let oldest = self.buffer[self.write_pos];
            self.sum_squares -= oldest * oldest;
            
            // Add new sample to sum
            self.sum_squares += i * i;
            self.buffer[self.write_pos] = i;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
            
            // Calculate RMS
            *o = (self.sum_squares / self.buffer.len() as Sample).sqrt();
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.sum_squares = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Peak level detector.
#[derive(Clone)]
pub struct Peak {
    pub hold_ms: Sample,
    peak_value: Sample,
    hold_counter: Sample,
    sample_rate: SampleRate,
}

impl Peak {
    pub fn new(hold_ms: Sample, sample_rate: SampleRate) -> Self {
        Self {
            hold_ms,
            peak_value: 0.0,
            hold_counter: 0.0,
            sample_rate,
        }
    }
}

impl DspNode for Peak {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Peak requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        let hold_samples = self.hold_ms * 0.001 * self.sample_rate;
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            let abs_i = i.abs();
            
            if abs_i > self.peak_value {
                self.peak_value = abs_i;
                self.hold_counter = hold_samples;
            } else if self.hold_counter > 0.0 {
                self.hold_counter -= 1.0;
            } else {
                // Exponential decay
                self.peak_value *= 0.999;
            }
            
            *o = self.peak_value;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.peak_value = 0.0;
        self.hold_counter = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Frequency analyzer (simple zero-crossing counter).
#[derive(Clone)]
pub struct FrequencyAnalyzer {
    pub window_size: usize,
    buffer: Vec<Sample>,
    write_pos: usize,
    last_sample: Sample,
    zero_crossings: usize,
}

impl FrequencyAnalyzer {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            buffer: vec![0.0; window_size],
            write_pos: 0,
            last_sample: 0.0,
            zero_crossings: 0,
        }
    }
}

impl DspNode for FrequencyAnalyzer {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("FrequencyAnalyzer requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            // Count zero crossings
            if (self.last_sample <= 0.0 && i > 0.0) || (self.last_sample >= 0.0 && i < 0.0) {
                self.zero_crossings += 1;
            }
            
            self.buffer[self.write_pos] = i;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
            
            // Calculate frequency estimate
            if self.write_pos == 0 {
                let frequency = (self.zero_crossings as Sample * sample_rate) / (2.0 * self.buffer.len() as Sample);
                *o = frequency;
                self.zero_crossings = 0;
            } else {
                *o = 0.0; // Only output at end of window
            }
            
            self.last_sample = i;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.last_sample = 0.0;
        self.zero_crossings = 0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Spectral centroid calculator.
#[derive(Clone)]
pub struct SpectralCentroid {
    pub fft_size: usize,
    write_pos: usize,
}

impl SpectralCentroid {
    pub fn new(fft_size: usize) -> Self {
        Self {
            fft_size,
            write_pos: 0,
        }
    }
}

impl DspNode for SpectralCentroid {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("SpectralCentroid requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Simplified spectral centroid using autocorrelation
        // This is a very basic approximation
        for (&_i, o) in input.iter().zip(output.iter_mut()) {
            // For now, just output a constant (would need FFT for real implementation)
            *o = sample_rate * 0.1; // Rough estimate
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.write_pos = 0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Onset detector.
#[derive(Clone)]
pub struct OnsetDetector {
    pub threshold: Sample,
    pub sensitivity: Sample,
    buffer: Vec<Sample>,
    write_pos: usize,
    energy_history: Vec<Sample>,
}

impl OnsetDetector {
    pub fn new(threshold: Sample, sensitivity: Sample, buffer_size: usize) -> Self {
        Self {
            threshold,
            sensitivity,
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            energy_history: vec![0.0; buffer_size],
        }
    }
}

impl DspNode for OnsetDetector {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("OnsetDetector requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            // Calculate instantaneous energy
            let energy = i * i;
            
            // Store in history
            self.energy_history[self.write_pos] = energy;
            self.buffer[self.write_pos] = i;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
            
            // Simple onset detection using energy difference
            let avg_energy = self.energy_history.iter().sum::<Sample>() / self.energy_history.len() as Sample;
            let onset_strength = (energy - avg_energy).max(0.0) * self.sensitivity;
            
            *o = if onset_strength > self.threshold { 1.0 } else { 0.0 };
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.energy_history.fill(0.0);
        self.write_pos = 0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Pitch detector using autocorrelation.
#[derive(Clone)]
pub struct PitchDetector {
    pub min_freq: Sample,
    pub max_freq: Sample,
    buffer: Vec<Sample>,
    write_pos: usize,
}

impl PitchDetector {
    pub fn new(min_freq: Sample, max_freq: Sample, buffer_size: usize) -> Self {
        Self {
            min_freq,
            max_freq,
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
        }
    }
}

impl DspNode for PitchDetector {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("PitchDetector requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            self.buffer[self.write_pos] = i;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
            
            // Simple autocorrelation pitch detection
            let min_period = (sample_rate / self.max_freq) as usize;
            let max_period = (sample_rate / self.min_freq) as usize;
            
            let mut best_corr = 0.0;
            let mut best_period = min_period;
            
            for period in min_period..max_period.min(self.buffer.len() / 2) {
                let mut corr = 0.0;
                for j in 0..(self.buffer.len() - period) {
                    let idx1 = (self.write_pos + self.buffer.len() - j - 1) % self.buffer.len();
                    let idx2 = (self.write_pos + self.buffer.len() - j - period - 1) % self.buffer.len();
                    corr += self.buffer[idx1] * self.buffer[idx2];
                }
                
                if corr > best_corr {
                    best_corr = corr;
                    best_period = period;
                }
            }
            
            *o = sample_rate / best_period as Sample;
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