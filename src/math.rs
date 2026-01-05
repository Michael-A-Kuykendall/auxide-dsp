//! Math module: Mathematical operations on audio signals.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut};

/// Add two signals.
#[derive(Clone)]
pub struct Add;

impl DspNode for Add {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.len() < 2 || outputs.is_empty() {
            return Err("Add requires at least 2 inputs and 1 output");
        }
        
        let output = &mut outputs[0];
        
        // Initialize output to first input
        if let Some(first_input) = inputs.first() {
            if first_input.len() != output.len() {
                return Err("Input and output block sizes must match");
            }
            output.copy_from_slice(first_input);
        }
        
        // Add remaining inputs
        for input in inputs.iter().skip(1) {
            if input.len() != output.len() {
                return Err("All input block sizes must match output");
            }
            for (o, &i) in output.iter_mut().zip(input.iter()) {
                *o += i;
            }
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 2 } // Minimum
    fn num_outputs(&self) -> usize { 1 }
}

/// Multiply two signals.
#[derive(Clone)]
pub struct Multiply;

impl DspNode for Multiply {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.len() < 2 || outputs.is_empty() {
            return Err("Multiply requires at least 2 inputs and 1 output");
        }
        
        let output = &mut outputs[0];
        
        // Initialize output to first input
        if let Some(first_input) = inputs.first() {
            if first_input.len() != output.len() {
                return Err("Input and output block sizes must match");
            }
            output.copy_from_slice(first_input);
        }
        
        // Multiply remaining inputs
        for input in inputs.iter().skip(1) {
            if input.len() != output.len() {
                return Err("All input block sizes must match output");
            }
            for (o, &i) in output.iter_mut().zip(input.iter()) {
                *o *= i;
            }
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 2 } // Minimum
    fn num_outputs(&self) -> usize { 1 }
}

/// Scale signal by constant.
#[derive(Clone)]
pub struct Scale {
    pub factor: Sample,
}

impl Scale {
    pub fn new(factor: Sample) -> Self {
        Self { factor }
    }
}

impl DspNode for Scale {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Scale requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = i * self.factor;
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Offset signal by constant.
#[derive(Clone)]
pub struct Offset {
    pub amount: Sample,
}

impl Offset {
    pub fn new(amount: Sample) -> Self {
        Self { amount }
    }
}

impl DspNode for Offset {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Offset requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = i + self.amount;
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Absolute value.
#[derive(Clone)]
pub struct Abs;

impl DspNode for Abs {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Abs requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = i.abs();
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Exponential function.
#[derive(Clone)]
pub struct Exp;

impl DspNode for Exp {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Exp requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = i.exp();
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Natural logarithm.
#[derive(Clone)]
pub struct Ln;

impl DspNode for Ln {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Ln requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = (i.max(0.0001)).ln();
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Power function (input ^ exponent).
#[derive(Clone)]
pub struct Pow {
    pub exponent: Sample,
}

impl Pow {
    pub fn new(exponent: Sample) -> Self {
        Self { exponent }
    }
}

impl DspNode for Pow {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Pow requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = i.powf(self.exponent);
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Sine function.
#[derive(Clone)]
pub struct Sin;

impl DspNode for Sin {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Sin requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = (i * std::f32::consts::TAU).sin();
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Cosine function.
#[derive(Clone)]
pub struct Cos;

impl DspNode for Cos {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Cos requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = (i * std::f32::consts::TAU).cos();
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Clamp signal to range.
#[derive(Clone)]
pub struct Clamp {
    pub min: Sample,
    pub max: Sample,
}

impl Clamp {
    pub fn new(min: Sample, max: Sample) -> Self {
        Self { min, max }
    }
}

impl DspNode for Clamp {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Clamp requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i, o) in input.iter().zip(output.iter_mut()) {
            *o = i.clamp(self.min, self.max);
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Subtract two signals.
#[derive(Clone)]
pub struct Sub;

impl DspNode for Sub {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.len() < 2 || outputs.is_empty() {
            return Err("Sub requires at least 2 inputs and 1 output");
        }
        
        let output = &mut outputs[0];
        
        // Initialize output to first input
        if let Some(first_input) = inputs.first() {
            if first_input.len() != output.len() {
                return Err("Input and output block sizes must match");
            }
            output.copy_from_slice(first_input);
        }
        
        // Subtract remaining inputs
        for input in inputs.iter().skip(1) {
            if input.len() != output.len() {
                return Err("All input block sizes must match output");
            }
            for (o, &i) in output.iter_mut().zip(input.iter()) {
                *o -= i;
            }
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 2 } // Minimum
    fn num_outputs(&self) -> usize { 1 }
}

/// Divide two signals.
#[derive(Clone)]
pub struct Div;

impl DspNode for Div {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.len() < 2 || outputs.is_empty() {
            return Err("Div requires at least 2 inputs and 1 output");
        }
        
        let output = &mut outputs[0];
        
        // Initialize output to first input
        if let Some(first_input) = inputs.first() {
            if first_input.len() != output.len() {
                return Err("Input and output block sizes must match");
            }
            output.copy_from_slice(first_input);
        }
        
        // Divide by remaining inputs
        for input in inputs.iter().skip(1) {
            if input.len() != output.len() {
                return Err("All input block sizes must match output");
            }
            for (o, &i) in output.iter_mut().zip(input.iter()) {
                *o /= i.max(0.0001); // Avoid division by zero
            }
        }
        
        Ok(())
    }

    fn reset(&mut self) {}
    fn num_inputs(&self) -> usize { 2 } // Minimum
    fn num_outputs(&self) -> usize { 1 }
}