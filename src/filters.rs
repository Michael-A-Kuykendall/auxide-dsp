//! Filters module: Real-time safe filter implementations.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut, BiquadState};

/// Low-pass filter using biquad implementation.
#[derive(Clone)]
pub struct Lpf {
    pub cutoff: Sample,
    pub q: Sample,
    state: BiquadState,
}

impl Lpf {
    pub fn new(cutoff: Sample, q: Sample) -> Self {
        Self {
            cutoff,
            q,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for Lpf {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Lpf requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Calculate biquad coefficients for low-pass filter
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let norm = 1.0 / (1.0 + k / self.q + k * k);
        let a0 = k * k * norm;
        let a1 = 2.0 * a0;
        let a2 = a0;
        let b1 = 2.0 * (k * k - 1.0) * norm;
        let b2 = (1.0 - k / self.q + k * k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            // Update state
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// High-pass filter using biquad implementation.
#[derive(Clone)]
pub struct Hpf {
    pub cutoff: Sample,
    pub q: Sample,
    state: BiquadState,
}

impl Hpf {
    pub fn new(cutoff: Sample, q: Sample) -> Self {
        Self {
            cutoff,
            q,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for Hpf {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Hpf requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Calculate biquad coefficients for high-pass filter
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let norm = 1.0 / (1.0 + k / self.q + k * k);
        let a0 = 1.0 * norm;
        let a1 = -2.0 * a0;
        let a2 = a0;
        let b1 = 2.0 * (k * k - 1.0) * norm;
        let b2 = (1.0 - k / self.q + k * k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            // Update state
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Band-pass filter using biquad implementation.
#[derive(Clone)]
pub struct Bpf {
    pub cutoff: Sample,
    pub q: Sample,
    state: BiquadState,
}

impl Bpf {
    pub fn new(cutoff: Sample, q: Sample) -> Self {
        Self {
            cutoff,
            q,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for Bpf {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Bpf requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Calculate biquad coefficients for band-pass filter
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let norm = 1.0 / (1.0 + k / self.q + k * k);
        let a0 = k / self.q * norm;
        let a1 = 0.0;
        let a2 = -a0;
        let b1 = 2.0 * (k * k - 1.0) * norm;
        let b2 = (1.0 - k / self.q + k * k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            // Update state
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Notch filter using biquad implementation.
#[derive(Clone)]
pub struct Notch {
    pub cutoff: Sample,
    pub q: Sample,
    state: BiquadState,
}

impl Notch {
    pub fn new(cutoff: Sample, q: Sample) -> Self {
        Self {
            cutoff,
            q,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for Notch {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Notch requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Calculate biquad coefficients for notch filter
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let norm = 1.0 / (1.0 + k / self.q + k * k);
        let a0 = (1.0 + k * k) * norm;
        let a1 = 2.0 * (k * k - 1.0) * norm;
        let a2 = a0;
        let b1 = a1;
        let b2 = (1.0 - k / self.q + k * k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            // Update state
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// All-pass filter using biquad implementation.
#[derive(Clone)]
pub struct Allpass {
    pub cutoff: Sample,
    pub q: Sample,
    state: BiquadState,
}

impl Allpass {
    pub fn new(cutoff: Sample, q: Sample) -> Self {
        Self {
            cutoff,
            q,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for Allpass {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Allpass requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Calculate biquad coefficients for all-pass filter
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let norm = 1.0 / (1.0 + k / self.q + k * k);
        let a0 = (1.0 - k / self.q + k * k) * norm;
        let a1 = 2.0 * (k * k - 1.0) * norm;
        let a2 = 1.0;
        let b1 = a1;
        let b2 = a0;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            // Update state
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Low shelf filter.
#[derive(Clone)]
pub struct ShelfLow {
    pub cutoff: Sample,
    pub gain: Sample,
    state: BiquadState,
}

impl ShelfLow {
    pub fn new(cutoff: Sample, gain: Sample) -> Self {
        Self {
            cutoff,
            gain,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for ShelfLow {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("ShelfLow requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Simplified low shelf implementation
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let v = 10.0_f32.powf(self.gain / 20.0);
        let norm = 1.0 / (1.0 + k / 0.7 + k * k);
        
        let a0 = (1.0 + (v.sqrt()) * k / 0.7 + v * k * k) * norm;
        let a1 = 2.0 * (v * k * k - 1.0) * norm;
        let a2 = (1.0 - (v.sqrt()) * k / 0.7 + v * k * k) * norm;
        let b1 = 2.0 * (k * k - 1.0) * norm;
        let b2 = (1.0 - k / 0.7 + k * k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// High shelf filter.
#[derive(Clone)]
pub struct ShelfHigh {
    pub cutoff: Sample,
    pub gain: Sample,
    state: BiquadState,
}

impl ShelfHigh {
    pub fn new(cutoff: Sample, gain: Sample) -> Self {
        Self {
            cutoff,
            gain,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for ShelfHigh {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("ShelfHigh requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Simplified high shelf implementation
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let v = 10.0_f32.powf(self.gain / 20.0);
        let norm = 1.0 / (1.0 + k / 0.7 + k * k);
        
        let a0 = (v + (v.sqrt()) * k / 0.7 + k * k) * norm;
        let a1 = 2.0 * (k * k - v) * norm;
        let a2 = (v - (v.sqrt()) * k / 0.7 + k * k) * norm;
        let b1 = 2.0 * (k * k - 1.0) * norm;
        let b2 = (1.0 - k / 0.7 + k * k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Peak filter (parametric EQ).
#[derive(Clone)]
pub struct Peak {
    pub cutoff: Sample,
    pub q: Sample,
    pub gain: Sample,
    state: BiquadState,
}

impl Peak {
    pub fn new(cutoff: Sample, q: Sample, gain: Sample) -> Self {
        Self {
            cutoff,
            q,
            gain,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for Peak {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("Peak requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        let k = (std::f32::consts::PI * self.cutoff / sample_rate).tan();
        let v = 10.0_f32.powf(self.gain / 20.0);
        let norm = 1.0 / (1.0 + k / self.q + k * k);
        
        let a0 = (1.0 + v * k / self.q + k * k) * norm;
        let a1 = 2.0 * (k * k - 1.0) * norm;
        let a2 = (1.0 - v * k / self.q + k * k) * norm;
        let b1 = a1;
        let b2 = (1.0 - k / self.q + k * k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// State variable filter (LPF, HPF, BPF, notch in one).
#[derive(Clone)]
pub struct StateVariableFilter {
    pub cutoff: Sample,
    pub resonance: Sample,
    pub mode: SvfMode,
    state: SvfState,
}

#[derive(Clone, Copy)]
pub enum SvfMode {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
}

#[derive(Clone)]
struct SvfState {
    z1: Sample,
    z2: Sample,
}

impl Default for SvfState {
    fn default() -> Self {
        Self { z1: 0.0, z2: 0.0 }
    }
}

impl StateVariableFilter {
    pub fn new(cutoff: Sample, resonance: Sample, mode: SvfMode) -> Self {
        Self {
            cutoff,
            resonance,
            mode,
            state: SvfState::default(),
        }
    }
}

impl DspNode for StateVariableFilter {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("StateVariableFilter requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        let f = 2.0 * (std::f32::consts::PI * self.cutoff / sample_rate).sin();
        let q = 1.0 / self.resonance;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let hp = i_val - self.state.z1 * q - self.state.z2;
            let bp = hp * f + self.state.z1;
            let lp = bp * f + self.state.z2;
            
            *o = match self.mode {
                SvfMode::Lowpass => lp,
                SvfMode::Highpass => hp,
                SvfMode::Bandpass => bp,
                SvfMode::Notch => i_val - bp,
            };
            
            self.state.z1 = bp;
            self.state.z2 = lp;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = SvfState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Ladder filter (Moog-style).
#[derive(Clone)]
pub struct LadderFilter {
    pub cutoff: Sample,
    pub resonance: Sample,
    state: LadderState,
}

#[derive(Clone)]
struct LadderState {
    z1: Sample,
    z2: Sample,
    z3: Sample,
    z4: Sample,
}

impl Default for LadderState {
    fn default() -> Self {
        Self { z1: 0.0, z2: 0.0, z3: 0.0, z4: 0.0 }
    }
}

impl LadderFilter {
    pub fn new(cutoff: Sample, resonance: Sample) -> Self {
        Self {
            cutoff,
            resonance,
            state: LadderState::default(),
        }
    }
}

impl DspNode for LadderFilter {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("LadderFilter requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        let fc = self.cutoff / sample_rate;
        let k = 3.6 * fc - 1.6 * fc * fc - 1.0;
        let p = (k + 1.0) * 0.5;
        let scale = (1.0 - p).powi(4);
        let r = self.resonance * scale;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x = i_val - r * self.state.z4;
            
            // Four cascaded one-pole filters
            let y1 = x * p + self.state.z1 * (1.0 - p);
            let y2 = y1 * p + self.state.z2 * (1.0 - p);
            let y3 = y2 * p + self.state.z3 * (1.0 - p);
            let y4 = y3 * p + self.state.z4 * (1.0 - p);
            
            *o = y4;
            
            self.state.z1 = y1;
            self.state.z2 = y2;
            self.state.z3 = y3;
            self.state.z4 = y4;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = LadderState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Comb filter.
#[derive(Clone)]
pub struct CombFilter {
    pub delay_samples: usize,
    pub feedback: Sample,
    pub damp: Sample,
    buffer: Vec<Sample>,
    write_pos: usize,
    last_output: Sample,
}

impl CombFilter {
    pub fn new(delay_samples: usize, feedback: Sample, damp: Sample) -> Self {
        Self {
            delay_samples,
            feedback,
            damp,
            buffer: vec![0.0; delay_samples],
            write_pos: 0,
            last_output: 0.0,
        }
    }
}

impl DspNode for CombFilter {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("CombFilter requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let read_pos = (self.write_pos + self.buffer.len() - self.delay_samples) % self.buffer.len();
            let delayed = self.buffer[read_pos];
            
            let filtered = delayed * (1.0 - self.damp) + self.last_output * self.damp;
            *o = i_val + filtered * self.feedback;
            
            self.buffer[self.write_pos] = *o;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
            self.last_output = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.last_output = 0.0;
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Formant filter.
#[derive(Clone)]
pub struct FormantFilter {
    pub formant_freq: Sample,
    pub bandwidth: Sample,
    state: BiquadState,
}

impl FormantFilter {
    pub fn new(formant_freq: Sample, bandwidth: Sample) -> Self {
        Self {
            formant_freq,
            bandwidth,
            state: BiquadState::default(),
        }
    }
}

impl DspNode for FormantFilter {
    fn process(&mut self, inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], sample_rate: SampleRate) -> Result<(), &'static str> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err("FormantFilter requires 1 input and 1 output");
        }
        
        let input = &inputs[0];
        let output = &mut outputs[0];
        
        if input.len() != output.len() {
            return Err("Input and output block sizes must match");
        }
        
        // Simple formant filter using bandpass
        let k = (std::f32::consts::PI * self.bandwidth / sample_rate).tan();
        let center = 2.0 * (std::f32::consts::PI * self.formant_freq / sample_rate).cos();
        let norm = 1.0 / (1.0 + k);
        
        let a0 = k * norm;
        let a1 = 0.0;
        let a2 = -k * norm;
        let b1 = -center * norm;
        let b2 = (1.0 - k) * norm;
        
        for (&i_val, o) in input.iter().zip(output.iter_mut()) {
            let x0 = i_val;
            *o = a0 * x0 + a1 * self.state.x1 + a2 * self.state.x2 
                - b1 * self.state.y1 - b2 * self.state.y2;
            
            self.state.x2 = self.state.x1;
            self.state.x1 = x0;
            self.state.y2 = self.state.y1;
            self.state.y1 = *o;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = BiquadState::default();
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}