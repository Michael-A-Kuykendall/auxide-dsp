use crate::helpers::{compute_exponential_coefficient, freq_to_phase_increment};
use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of a State Variable Filter (SVF)
#[derive(Debug, Clone)]
pub struct SvfState {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
    pub y3: f32,
    pub y4: f32,
}

/// State Variable Filter (SVF) - Lowpass, Highpass, Bandpass, Notch
#[derive(Debug, Clone)]
pub struct SvfFilter {
    pub cutoff: f32,
    pub resonance: f32,
    pub mode: SvfMode,
}

#[derive(Debug, Clone, Copy)]
pub enum SvfMode {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
}

impl NodeDef for SvfFilter {
    type State = SvfState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio },
            Port { id: PortId(1), rate: Rate::Audio },
            Port { id: PortId(2), rate: Rate::Audio },
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
        SvfState {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            y3: 0.0,
            y4: 0.0,
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
        let cutoff_mod = if inputs.len() > 1 { inputs[1] } else { &[][..] };
        let resonance_mod = if inputs.len() > 2 { inputs[2] } else { &[][..] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let cutoff = self.cutoff + if cutoff_mod.is_empty() { 0.0 } else { cutoff_mod[i] };
            let resonance = self.resonance + if resonance_mod.is_empty() { 0.0 } else { resonance_mod[i] };

            let f = freq_to_phase_increment(cutoff, sample_rate) * 2.0;
            let k = 2.0 - 2.0 * resonance.clamp(0.0, 1.0);

            let x = input[i];
            let x1 = state.x1;
            let x2 = state.x2;
            let y1 = state.y1;
            let y2 = state.y2;

            let y_hp = (x - x2) - k * y1;
            let y_bp = y_hp * f + y1;
            let y_lp = y_bp * f + y2;

            state.x1 = x;
            state.x2 = x1;
            state.y1 = y_hp;
            state.y2 = y_bp;
            state.y3 = y_lp;
            state.y4 = y_hp + y_lp; // notch

            output[i] = match self.mode {
                SvfMode::Lowpass => y_lp,
                SvfMode::Highpass => y_hp,
                SvfMode::Bandpass => y_bp,
                SvfMode::Notch => state.y4,
            };
        }
    }
}

/// State of a Ladder Filter
#[derive(Debug, Clone)]
pub struct LadderState {
    pub z1: f32,
    pub z2: f32,
    pub z3: f32,
    pub z4: f32,
}

/// Ladder Filter (Moog-style)
#[derive(Debug, Clone)]
pub struct LadderFilter {
    pub cutoff: f32,
    pub resonance: f32,
    pub drive: f32,
}

impl NodeDef for LadderFilter {
    type State = LadderState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio },
            Port { id: PortId(1), rate: Rate::Audio },
            Port { id: PortId(2), rate: Rate::Audio },
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
        LadderState {
            z1: 0.0,
            z2: 0.0,
            z3: 0.0,
            z4: 0.0,
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
        let cutoff_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let resonance_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let cutoff = self.cutoff + if cutoff_mod.is_empty() { 0.0 } else { cutoff_mod[i] };
            let resonance = self.resonance + if resonance_mod.is_empty() { 0.0 } else { resonance_mod[i] };

            let fc = cutoff / sample_rate;
            let k = resonance * 4.0;
            let p = fc * (1.8 - 0.8 * fc);
            let t = (1.0 - p) * 1.386249;
            let _t2 = 12.0 + t * t;

            let x = input[i] * self.drive;

            let y4 = x - k * (state.z4 + state.z3 + state.z2 + state.z1);
            let y3 = y4 * t + state.z4;
            let y2 = y3 * t + state.z3;
            let y1 = y2 * t + state.z2;
            let y0 = y1 * t + state.z1;

            state.z1 += y0 * t;
            state.z2 += y1 * t;
            state.z3 += y2 * t;
            state.z4 += y3 * t;

            output[i] = y4;
        }
    }
}

/// State of a Comb Filter
#[derive(Debug, Clone)]
pub struct CombState {
    pub buffer: Vec<f32>,
    pub index: usize,
}

/// Comb Filter
#[derive(Debug, Clone)]
pub struct CombFilter {
    pub delay_ms: f32,
    pub feedback: f32,
    pub damp: f32,
}

impl NodeDef for CombFilter {
    type State = CombState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio },
            Port { id: PortId(1), rate: Rate::Audio },
            Port { id: PortId(2), rate: Rate::Audio },
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
        CombState {
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
        let feedback_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let damp_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        let delay_samples = state.buffer.len();
        let mut damp = self.damp;
        let mut feedback = self.feedback;

        for i in 0..input.len() {
            if !feedback_mod.is_empty() {
                feedback = self.feedback + feedback_mod[i];
            }
            if !damp_mod.is_empty() {
                damp = self.damp + damp_mod[i];
            }

            let delayed = state.buffer[state.index];
            let damped = delayed * (1.0 - damp);
            let out = input[i] + damped * feedback;
            output[i] = out;

            state.buffer[state.index] = out;
            state.index = (state.index + 1) % delay_samples;
        }
    }
}

/// State of a Formant Filter
#[derive(Debug, Clone)]
pub struct FormantState {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
}

/// Formant Filter (simple vowel formant)
#[derive(Debug, Clone)]
pub struct FormantFilter {
    pub freq1: f32,
    pub freq2: f32,
    pub bw1: f32,
    pub bw2: f32,
    pub gain1: f32,
    pub gain2: f32,
}

impl NodeDef for FormantFilter {
    type State = FormantState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
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
        FormantState {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
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
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let x = input[i];

            // Simple formant: two bandpass filters in parallel
            let c1 = compute_exponential_coefficient(self.freq1, self.bw1);
            let c2 = compute_exponential_coefficient(self.freq2, self.bw2);

            let y1 = x * self.gain1 + state.x1 * c1 - state.y1 * c1;
            let y2 = x * self.gain2 + state.x2 * c2 - state.y2 * c2;

            state.x1 = x;
            state.x2 = x;
            state.y1 = y1;
            state.y2 = y2;

            output[i] = y1 + y2;
        }
    }
}

/// State of a BiquadFilter
#[derive(Debug, Clone)]
pub struct BiquadFilterState {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
}

/// Biquad Filter (second-order IIR)
#[derive(Debug, Clone)]
pub struct BiquadFilter {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl NodeDef for BiquadFilter {
    type State = BiquadFilterState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
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
        BiquadFilterState {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
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
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let x = input[i];
            let y = self.b0 * x + self.b1 * state.x1 + self.b2 * state.x2
                  - self.a1 * state.y1 - self.a2 * state.y2;

            state.x2 = state.x1;
            state.x1 = x;
            state.y2 = state.y1;
            state.y1 = y;

            output[i] = y;
        }
    }
}

/// State of an AllpassFilter
#[derive(Debug, Clone)]
pub struct AllpassFilterState {
    pub buffer: Vec<f32>,
    pub index: usize,
}

/// Allpass Filter
#[derive(Debug, Clone)]
pub struct AllpassFilter {
    pub delay_samples: usize,
    pub gain: f32,
}

impl NodeDef for AllpassFilter {
    type State = AllpassFilterState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
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
        AllpassFilterState {
            buffer: vec![0.0; self.delay_samples],
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
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let delayed = state.buffer[state.index];
            let y = -self.gain * input[i] + delayed + self.gain * delayed;
            state.buffer[state.index] = input[i] + self.gain * delayed;
            state.index = (state.index + 1) % self.delay_samples;
            output[i] = y;
        }
    }
}