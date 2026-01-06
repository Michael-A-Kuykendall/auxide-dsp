use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;
use num_complex::Complex;
use realfft::RealFftPlanner;

/// State of a Delay
#[derive(Debug, Clone)]
pub struct DelayState {
    pub buffer: Vec<f32>,
    pub index: usize,
}

/// Delay Effect
#[derive(Debug, Clone)]
pub struct Delay {
    pub delay_ms: f32,
    pub feedback: f32,
    pub mix: f32,
}

impl NodeDef for Delay {
    type State = DelayState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // feedback_mod
            Port { id: PortId(2), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, sample_rate: f32, _block_size: usize) -> Self::State {
        let delay_samples = (self.delay_ms * sample_rate / 1000.0) as usize;
        DelayState {
            buffer: vec![0.0; delay_samples],
            index: 0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let feedback_mod = if inputs.len() > 1 { inputs[1] } else { &[][..] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[][..] };
        let output = &mut outputs[0];

        let delay_samples = state.buffer.len();

        for i in 0..input.len() {
            let feedback = self.feedback + if feedback_mod.is_empty() { 0.0 } else { feedback_mod[i] };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let delayed = state.buffer[state.index];
            let out = input[i] + delayed * feedback * mix + input[i] * (1.0 - mix);
            output[i] = out;

            state.buffer[state.index] = input[i] + delayed * feedback;
            state.index = (state.index + 1) % delay_samples;
        }
    }
}

/// State of a Chorus
#[derive(Debug, Clone)]
pub struct ChorusState {
    pub buffer: Vec<f32>,
    pub index: usize,
    pub lfo_phase: f32,
}

/// Chorus Effect
#[derive(Debug, Clone)]
pub struct Chorus {
    pub delay_ms: f32,
    pub depth_ms: f32,
    pub rate: f32,
    pub mix: f32,
}

impl NodeDef for Chorus {
    type State = ChorusState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // rate_mod
            Port { id: PortId(2), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, sample_rate: f32, _block_size: usize) -> Self::State {
        let max_delay = (self.delay_ms + self.depth_ms) * sample_rate / 1000.0;
        ChorusState {
            buffer: vec![0.0; max_delay as usize + 1],
            index: 0,
            lfo_phase: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let input = &inputs[0];
        let rate_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        let base_delay_samples = (self.delay_ms * sample_rate / 1000.0) as usize;
        let depth_samples = self.depth_ms * sample_rate / 1000.0;

        for i in 0..input.len() {
            let rate = self.rate + if rate_mod.is_empty() { 0.0 } else { rate_mod[i] };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let lfo_inc = rate / sample_rate;
            state.lfo_phase = (state.lfo_phase + lfo_inc).fract();
            let lfo = (state.lfo_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5; // 0 to 1

            let delay_samples = base_delay_samples as f32 + lfo * depth_samples;
            let delay_int = delay_samples as usize;
            let frac = delay_samples.fract();

            let idx1 = (state.index + state.buffer.len() - delay_int) % state.buffer.len();
            let idx2 = (idx1 + state.buffer.len() - 1) % state.buffer.len();

            let delayed = state.buffer[idx1] * (1.0 - frac) + state.buffer[idx2] * frac;

            let out = input[i] * (1.0 - mix) + delayed * mix;
            output[i] = out;

            state.buffer[state.index] = input[i];
            state.index = (state.index + 1) % state.buffer.len();
        }
    }
}

/// State of a Flanger
#[derive(Debug, Clone)]
pub struct FlangerState {
    pub buffer: Vec<f32>,
    pub index: usize,
    pub lfo_phase: f32,
}

/// Flanger Effect
#[derive(Debug, Clone)]
pub struct Flanger {
    pub delay_ms: f32,
    pub depth_ms: f32,
    pub rate: f32,
    pub feedback: f32,
    pub mix: f32,
}

impl NodeDef for Flanger {
    type State = FlangerState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // rate_mod
            Port { id: PortId(2), rate: Rate::Audio }, // feedback_mod
            Port { id: PortId(3), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, sample_rate: f32, _block_size: usize) -> Self::State {
        let max_delay = (self.delay_ms + self.depth_ms) * sample_rate / 1000.0;
        FlangerState {
            buffer: vec![0.0; max_delay as usize + 1],
            index: 0,
            lfo_phase: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let input = &inputs[0];
        let rate_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let feedback_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let mix_mod = if inputs.len() > 3 { inputs[3] } else { &[] };
        let output = &mut outputs[0];

        let base_delay_samples = (self.delay_ms * sample_rate / 1000.0) as usize;
        let depth_samples = self.depth_ms * sample_rate / 1000.0;

        for i in 0..input.len() {
            let rate = self.rate + if rate_mod.is_empty() { 0.0 } else { rate_mod[i] };
            let feedback = self.feedback + if feedback_mod.is_empty() { 0.0 } else { feedback_mod[i] };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let lfo_inc = rate / sample_rate;
            state.lfo_phase = (state.lfo_phase + lfo_inc).fract();
            let lfo = (state.lfo_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5; // 0 to 1

            let delay_samples = base_delay_samples as f32 + lfo * depth_samples;
            let delay_int = delay_samples as usize;
            let frac = delay_samples.fract();

            let idx1 = (state.index + state.buffer.len() - delay_int) % state.buffer.len();
            let idx2 = (idx1 + state.buffer.len() - 1) % state.buffer.len();

            let delayed = state.buffer[idx1] * (1.0 - frac) + state.buffer[idx2] * frac;

            let out = input[i] + delayed * mix;
            output[i] = out;

            state.buffer[state.index] = input[i] + delayed * feedback;
            state.index = (state.index + 1) % state.buffer.len();
        }
    }
}

/// State of a Phaser
#[derive(Debug, Clone)]
pub struct PhaserState {
    pub lfo_phase: f32,
    pub x1: f32,
    pub y1: f32,
}

/// Phaser Effect (simple allpass-based)
#[derive(Debug, Clone)]
pub struct Phaser {
    pub rate: f32,
    pub depth: f32,
    pub feedback: f32,
    pub mix: f32,
}

impl NodeDef for Phaser {
    type State = PhaserState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // rate_mod
            Port { id: PortId(2), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        PhaserState {
            lfo_phase: 0.0,
            x1: 0.0,
            y1: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let input = &inputs[0];
        let rate_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let rate = self.rate + if rate_mod.is_empty() { 0.0 } else { rate_mod[i] };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let lfo_inc = rate / sample_rate;
            state.lfo_phase = (state.lfo_phase + lfo_inc).fract();
            let lfo = (state.lfo_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5; // 0 to 1

            let freq = 300.0 + lfo * 2000.0; // sweep 300-2300 Hz
            let c = (std::f32::consts::PI * freq / sample_rate).tan();
            let a1_coeff = (1.0 - c) / (1.0 + c);
            let b0 = a1_coeff;
            let b1 = 1.0;
            let a0 = 1.0;

            let x = input[i] + state.y1 * self.feedback;
            let y = b0 / a0 * x + b1 / a0 * state.x1 - a1_coeff / a0 * state.y1;

            state.x1 = x;
            state.y1 = y;

            output[i] = input[i] * (1.0 - mix) + y * mix;
        }
    }
}

/// State of a Simple Reverb
#[derive(Debug, Clone)]
pub struct SimpleReverbState {
    pub comb1: Vec<f32>,
    pub comb2: Vec<f32>,
    pub comb3: Vec<f32>,
    pub comb4: Vec<f32>,
    pub idx1: usize,
    pub idx2: usize,
    pub idx3: usize,
    pub idx4: usize,
}

/// Simple Reverb (4 comb filters)
#[derive(Debug, Clone)]
pub struct SimpleReverb {
    pub decay: f32,
    pub mix: f32,
}

impl NodeDef for SimpleReverb {
    type State = SimpleReverbState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, sample_rate: f32, _block_size: usize) -> Self::State {
        let len1 = (0.0297 * sample_rate) as usize;
        let len2 = (0.0371 * sample_rate) as usize;
        let len3 = (0.0411 * sample_rate) as usize;
        let len4 = (0.0437 * sample_rate) as usize;
        SimpleReverbState {
            comb1: vec![0.0; len1],
            comb2: vec![0.0; len2],
            comb3: vec![0.0; len3],
            comb4: vec![0.0; len4],
            idx1: 0,
            idx2: 0,
            idx3: 0,
            idx4: 0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let mix_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let d1 = state.comb1[state.idx1];
            let d2 = state.comb2[state.idx2];
            let d3 = state.comb3[state.idx3];
            let d4 = state.comb4[state.idx4];

            let rev = (d1 + d2 + d3 + d4) * 0.25;
            let out = input[i] * (1.0 - mix) + rev * mix;
            output[i] = out;

            let fb = input[i] + rev * self.decay;
            state.comb1[state.idx1] = fb;
            state.comb2[state.idx2] = fb;
            state.comb3[state.idx3] = fb;
            state.comb4[state.idx4] = fb;

            state.idx1 = (state.idx1 + 1) % state.comb1.len();
            state.idx2 = (state.idx2 + 1) % state.comb2.len();
            state.idx3 = (state.idx3 + 1) % state.comb3.len();
            state.idx4 = (state.idx4 + 1) % state.comb4.len();
        }
    }
}

/// State of a MultitapDelay
#[derive(Debug, Clone)]
pub struct MultitapDelayState {
    pub buffer: Vec<f32>,
    pub index: usize,
}

/// Multitap Delay Effect
#[derive(Debug, Clone)]
pub struct MultitapDelay {
    pub taps: Vec<(f32, f32)>, // (delay_ms, gain)
    pub feedback: f32,
    pub mix: f32,
}

impl NodeDef for MultitapDelay {
    type State = MultitapDelayState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // feedback_mod
            Port { id: PortId(2), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, sample_rate: f32, _block_size: usize) -> Self::State {
        let max_delay_samples = self.taps.iter().map(|(ms, _)| (ms * sample_rate / 1000.0) as usize).max().unwrap_or(1);
        MultitapDelayState {
            buffer: vec![0.0; max_delay_samples],
            index: 0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let input = &inputs[0];
        let feedback_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let feedback = self.feedback + if feedback_mod.is_empty() { 0.0 } else { feedback_mod[i] };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            // Write input + feedback
            let buf_len = state.buffer.len();
            state.buffer[state.index] = input[i] + state.buffer[state.index] * feedback;

            // Sum taps
            let mut tap_sum = 0.0;
            for (delay_ms, gain) in &self.taps {
                let delay_samples = (delay_ms * sample_rate / 1000.0) as usize;
                let read_idx = (state.index + buf_len - delay_samples) % buf_len;
                tap_sum += state.buffer[read_idx] * gain;
            }

            output[i] = input[i] * (1.0 - mix) + tap_sum * mix;

            state.index = (state.index + 1) % buf_len;
        }
    }
}

/// State of a ConvolutionReverb
#[derive(Clone)]
pub struct ConvolutionReverbState {
    pub fft_size: usize,
    pub ir_fft: Vec<Complex<f32>>,
    pub input_buffer: Vec<f32>,
    pub output_buffer: Vec<f32>,
    pub overlap: Vec<f32>,
    pub input_pos: usize,
    pub scratch_fft: Vec<Complex<f32>>,
    pub forward_fft: std::sync::Arc<dyn realfft::RealToComplex<f32>>,
    pub inverse_fft: std::sync::Arc<dyn realfft::ComplexToReal<f32>>,
}

/// Convolution Reverb Effect using FFT convolution
#[derive(Debug, Clone)]
pub struct ConvolutionReverb {
    pub ir: Vec<f32>, // impulse response
    pub mix: f32,
}

impl NodeDef for ConvolutionReverb {
    type State = ConvolutionReverbState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, block_size: usize) -> Self::State {
        let ir_len = self.ir.len();
        let fft_size = (ir_len + block_size - 1).next_power_of_two();
        
        let mut planner = RealFftPlanner::<f32>::new();
        let forward_fft = planner.plan_fft_forward(fft_size);
        let inverse_fft = planner.plan_fft_inverse(fft_size);
        
        let fft_output_size = fft_size / 2 + 1;
        
        // Pre-compute IR FFT
        let mut ir_padded = vec![0.0; fft_size];
        ir_padded[..ir_len].copy_from_slice(&self.ir);
        let mut ir_fft = vec![Complex::new(0.0, 0.0); fft_output_size];
        
        // Handle FFT failure gracefully - fall back to impulse response (pass-through)
        if forward_fft.process(&mut ir_padded, &mut ir_fft).is_err() {
            // Fallback: impulse response (DC = 1.0, others = 0.0 for pass-through)
            ir_fft[0] = Complex::new(1.0, 0.0);
            for freq in ir_fft.iter_mut().skip(1) {
                *freq = Complex::new(0.0, 0.0);
            }
        }
        
        ConvolutionReverbState {
            fft_size,
            ir_fft,
            input_buffer: vec![0.0; fft_size],
            output_buffer: vec![0.0; fft_size],
            overlap: vec![0.0; fft_size - block_size],
            input_pos: 0,
            scratch_fft: vec![Complex::new(0.0, 0.0); fft_output_size],
            forward_fft,
            inverse_fft,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let mix_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let output = &mut outputs[0];
        let block_size = input.len();
        
        // Copy input to buffer
        for i in 0..block_size {
            state.input_buffer[state.input_pos + i] = input[i];
        }
        state.input_pos += block_size;
        
        // If we have enough samples, process convolution
        if state.input_pos >= state.fft_size {
            // FFT input
            if state.forward_fft.process(&mut state.input_buffer, &mut state.scratch_fft).is_err() {
                // Fail-closed: output silence
                output[..block_size].fill(0.0);
                return;
            }
            
            // Multiply with IR FFT
            for j in 0..state.scratch_fft.len() {
                state.scratch_fft[j] *= state.ir_fft[j];
            }
            
            // Inverse FFT
            if state.inverse_fft.process(&mut state.scratch_fft, &mut state.output_buffer).is_err() {
                // Fail-closed: output silence
                output[..block_size].fill(0.0);
                return;
            }
            
            // Normalize
            let norm = 1.0 / state.fft_size as f32;
            for sample in state.output_buffer.iter_mut().take(state.fft_size) {
                *sample *= norm;
            }
            
            // Overlap-add to output
            for (i, out_sample) in output.iter_mut().enumerate().take(block_size) {
                *out_sample = state.output_buffer[i] + state.overlap[i];
            }
            
            // Update overlap
            let overlap_len = state.overlap.len();
            for i in 0..overlap_len {
                state.overlap[i] = state.output_buffer[i + block_size];
            }
            
            // Shift input buffer
            for i in 0..(state.fft_size - block_size) {
                state.input_buffer[i] = state.input_buffer[i + block_size];
            }
            state.input_pos -= block_size;
        } else {
            // Not enough samples, output silence or previous
            output[..block_size].fill(0.0);
        }
        
        // Mix dry/wet
        for i in 0..block_size {
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };
            output[i] = input[i] * (1.0 - mix) + output[i] * mix;
        }
    }
}

/// State of a Tremolo
#[derive(Debug, Clone)]
pub struct TremoloState {
    pub phase: f32,
}

/// Tremolo Effect (amplitude modulation)
#[derive(Debug, Clone)]
pub struct Tremolo {
    pub rate: f32,
    pub depth: f32,
}

impl NodeDef for Tremolo {
    type State = TremoloState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // rate_mod
            Port { id: PortId(2), rate: Rate::Audio }, // depth_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        TremoloState { phase: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let input = &inputs[0];
        let rate_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let depth_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let rate = self.rate + if rate_mod.is_empty() { 0.0 } else { rate_mod[i] };
            let depth = self.depth + if depth_mod.is_empty() { 0.0 } else { depth_mod[i] };
            
            let modulation = (state.phase * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;
            let gain = 1.0 - depth * (1.0 - modulation);
            
            output[i] = input[i] * gain;
            
            state.phase += rate / sample_rate;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}