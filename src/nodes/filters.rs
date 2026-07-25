use crate::helpers::compute_exponential_coefficient;
use auxide::control::{PARAM_CUTOFF, PARAM_RESONANCE};
use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of a State Variable Filter (SVF)
///
/// Runtime-mutable parameters (`cutoff`, `resonance`) live in state so
/// [`set_param`](NodeDef::set_param) can update them from a control message.
///
/// Uses the ZDF (Zero-Delay Feedback / Trapezoidal) SVF topology, which is
/// guaranteed stable for all parameter values:
/// ```text
/// g  = tan(π · cutoff / sr)
/// R  = resonance * 2              (0 → no resonance, 1 → self-oscillation)
/// hp = (x - lp - R · bp) / (1 + g·(g + R))
/// bp += g · hp
/// lp += g · bp
/// ```
#[derive(Debug, Clone)]
pub struct SvfState {
    /// Current cutoff frequency (Hz), mutable via control message.
    pub cutoff: f32,
    /// Current resonance (0–1), mutable via control message.
    pub resonance: f32,
    pub bp: f32,
    pub lp: f32,
}

/// State Variable Filter (SVF) - Lowpass, Highpass, Bandpass, Notch
///
/// `cutoff` and `resonance` are initial values copied into state at init;
/// runtime changes arrive via `PARAM_CUTOFF` / `PARAM_RESONANCE` control
/// messages.
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
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            },
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        SvfState {
            cutoff: self.cutoff,
            resonance: self.resonance,
            bp: 0.0,
            lp: 0.0,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        match param {
            PARAM_CUTOFF => state.cutoff = value,
            PARAM_RESONANCE => state.resonance = value,
            _ => {}
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
            let cutoff = state.cutoff
                + if cutoff_mod.is_empty() {
                    0.0
                } else {
                    cutoff_mod[i]
                };
            let resonance = state.resonance
                + if resonance_mod.is_empty() {
                    0.0
                } else {
                    resonance_mod[i]
                };

            let g = (std::f32::consts::PI * cutoff / sample_rate).tan();
            let r = resonance.clamp(0.0, 1.0) * 2.0;
            let norm = 1.0 / (1.0 + g * (g + r));

            let x = input[i];
            let hp = (x - state.lp - r * state.bp) * norm;
            state.bp += g * hp;
            state.lp += g * state.bp;

            output[i] = match self.mode {
                SvfMode::Lowpass => state.lp,
                SvfMode::Highpass => hp,
                SvfMode::Bandpass => state.bp,
                SvfMode::Notch => hp + state.lp,
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
    pub cutoff: f32,
    pub resonance: f32,
    pub drive: f32,
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
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            },
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
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
            cutoff: self.cutoff,
            resonance: self.resonance,
            drive: self.drive,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        match param {
            // Clamp the cutoff to a sane audio range so a runaway control message
            // cannot push the filter past Nyquist (which would alias) or below
            // DC (degenerate).
            PARAM_CUTOFF => state.cutoff = value.clamp(20.0, 20_000.0),
            PARAM_RESONANCE => state.resonance = value.clamp(0.0, 1.0),
            _ => {}
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
            let cutoff = state.cutoff
                + if cutoff_mod.is_empty() {
                    0.0
                } else {
                    cutoff_mod[i]
                };
            let resonance = state.resonance
                + if resonance_mod.is_empty() {
                    0.0
                } else {
                    resonance_mod[i]
                };

            let fc = cutoff / sample_rate;
            let k = resonance * 4.0;
            let p = fc * (1.8 - 0.8 * fc);
            let t = (1.0 - p) * 1.386249;
            let _t2 = 12.0 + t * t;

            let x = input[i] * state.drive;

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
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            },
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
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
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
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
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
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
                - self.a1 * state.y1
                - self.a2 * state.y2;

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
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
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

/// State of a 1-pole utility filter.
#[derive(Debug, Clone)]
pub struct OnePoleState {
    y: f32,
}

/// 1-pole utility filter (lowpass / highpass).
///
/// Cheap, unconditionally stable first-order filter useful for DC blockers,
/// slews, and gentle tone shaping where a full SVF is overkill.
#[derive(Debug, Clone, Copy)]
pub struct OnePoleFilter {
    pub cutoff: f32,
    pub highpass: bool,
}

impl NodeDef for OnePoleFilter {
    type State = OnePoleState;

    fn input_ports(&self) -> &'static [Port] {
        &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }]
    }

    fn output_ports(&self) -> &'static [Port] {
        &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }]
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OnePoleState { y: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let input = &inputs[0];
        let output = &mut outputs[0];
        // One-pole coefficient: g in (0,1). Larger g = brighter (LP) / darker (HP).
        let g = (-2.0 * std::f32::consts::PI * self.cutoff / sample_rate).exp();
        let g = g.clamp(0.0, 0.999_999);
        for i in 0..input.len() {
            let lp = state.y + g * (input[i] - state.y);
            state.y = lp;
            output[i] = if self.highpass { input[i] - lp } else { lp };
        }
    }
}

/// State of a parametric (peaking) EQ.
#[derive(Debug, Clone)]
pub struct ParametricEqState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    freq: f32,
    q: f32,
    gain_db: f32,
}

/// Parametric EQ (peaking filter) via the Audio EQ Cookbook biquad.
#[derive(Debug, Clone, Copy)]
pub struct ParametricEq {
    pub freq: f32,
    pub q: f32,
    pub gain_db: f32,
}

impl NodeDef for ParametricEq {
    type State = ParametricEqState;

    fn input_ports(&self) -> &'static [Port] {
        &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }]
    }

    fn output_ports(&self) -> &'static [Port] {
        &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }]
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        ParametricEqState {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            freq: self.freq,
            q: self.q,
            gain_db: self.gain_db,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        match param {
            PARAM_CUTOFF => state.freq = value,
            // Reuse PARAM_RESONANCE slot to carry Q for the EQ.
            PARAM_RESONANCE => state.q = value.max(0.1),
            _ => {}
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
        let output = &mut outputs[0];
        let a0;
        let a1;
        let a2;
        let b0;
        let b1;
        let b2;
        {
            let a = (2.0 * std::f32::consts::PI * state.freq / sample_rate).cos();
            let alpha = state.freq / sample_rate.max(1.0) / (2.0 * state.q);
            let a0d = 1.0 + alpha;
            a0 = 1.0;
            a1 = -2.0 * a;
            a2 = 1.0 - alpha;
            let g = 10.0_f32.powf(state.gain_db / 40.0);
            b0 = (1.0 + alpha * g) / a0d;
            b1 = -2.0 * a / a0d;
            b2 = (1.0 - alpha * g) / a0d;
            let _ = a0d;
        }
        let (a1, a2) = (a1 / a0, a2 / a0);
        for i in 0..input.len() {
            let x = input[i];
            let y = b0 * x + b1 * state.x1 + b2 * state.x2 - a1 * state.y1 - a2 * state.y2;
            state.x2 = state.x1;
            state.x1 = x;
            state.y2 = state.y1;
            state.y1 = y;
            output[i] = y;
        }
    }
}

/// State of a resonant drive (saturation) node.
#[derive(Debug, Clone)]
pub struct ResonantDriveState {
    slope: f32,
}

/// Resonant drive: a tanh saturator that adds harmonics ("drive") and blends
/// the driven signal with the dry. Useful to push a resonant filter into
/// self-oscillation-like character or to warm up a voice.
#[derive(Debug, Clone, Copy)]
pub struct ResonantDrive {
    pub drive: f32,
    pub mix: f32,
}

impl NodeDef for ResonantDrive {
    type State = ResonantDriveState;

    fn input_ports(&self) -> &'static [Port] {
        &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }]
    }

    fn output_ports(&self) -> &'static [Port] {
        &[Port {
            id: PortId(0),
            rate: Rate::Audio,
        }]
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        ResonantDriveState { slope: 0.0 }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        if param == PARAM_CUTOFF {
            state.slope = value;
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
        // `drive` determines saturation amount; `mix` blends dry/wet.
        let drive = self.drive.max(0.0);
        let mix = self.mix.clamp(0.0, 1.0);
        for i in 0..input.len() {
            let wet = (state.slope + input[i] * drive).tanh();
            output[i] = input[i] * (1.0 - mix) + wet * mix;
        }
    }
}
