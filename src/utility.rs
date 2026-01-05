//! Utility module: Basic audio utility nodes.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut};

/// Gain control node.
#[derive(Clone)]
pub struct Gain {
    pub gain: Sample,
}

impl Gain {
    pub fn new(gain: Sample) -> Self {
        Self { gain }
    }
}

impl DspNode for Gain {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Gain requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            *o = i_val * self.gain;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Audio mixer node.
#[derive(Clone)]
pub struct Mix;

impl Mix {
    pub fn new() -> Self {
        Self
    }
}

impl DspNode for Mix {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Mix requires at least 1 input and 1 output");
        }
        
        if outputs.is_empty() {
            return Err("Mix requires at least 1 output");
        }
        
        let output = &mut outputs[0];
        output.fill(0.0);
        
        for input in inputs {
            if input.len() != output.len() {
                return Err("All input blocks must match output block size");
            }
            
            for (o, &i_val) in output.iter_mut().zip(input.iter()) {
                *o += i_val;
            }
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn num_inputs(&self) -> usize { 2 } // Minimum 2 inputs
    fn num_outputs(&self) -> usize { 1 }
}

/// Output sink node (consumes audio without producing output).
#[derive(Clone)]
pub struct OutputSink;

impl OutputSink {
    pub fn new() -> Self {
        Self
    }
}

impl DspNode for OutputSink {
    fn process(&mut self, inputs: &[AudioBlock], _outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() {
            return Err("OutputSink requires 1 input");
        }
        
        // Just consume the input - no processing needed
        Ok(())
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 0 }
}

/// Dummy node for testing.
#[derive(Clone)]
pub struct Dummy;

impl Dummy {
    pub fn new() -> Self {
        Self
    }
}

impl DspNode for Dummy {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Dummy requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Just copy input to output
        output.copy_from_slice(input);
        
        Ok(())
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}