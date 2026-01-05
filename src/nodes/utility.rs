use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of a Ring Modulator
#[derive(Debug, Clone)]
pub struct RingModState;

/// Ring Modulator
#[derive(Debug, Clone)]
pub struct RingMod {
    pub mix: f32,
}

impl NodeDef for RingMod {
    type State = RingModState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // mod
            Port { id: PortId(2), rate: Rate::Audio }, // mix_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        2
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        RingModState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let mod_signal = &inputs[1];
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };
            let mod_val = if mod_signal.is_empty() { 1.0 } else { mod_signal[i] };
            let ring = input[i] * mod_val;
            output[i] = input[i] * (1.0 - mix) + ring * mix;
        }
    }
}

/// State of a Crossfader
#[derive(Debug, Clone)]
pub struct CrossfaderState;

/// Crossfader
#[derive(Debug, Clone)]
pub struct Crossfader {
    pub position: f32, // 0.0 = A, 1.0 = B
}

impl NodeDef for Crossfader {
    type State = CrossfaderState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // A
            Port { id: PortId(1), rate: Rate::Audio }, // B
            Port { id: PortId(2), rate: Rate::Audio }, // position_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        2
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        CrossfaderState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let a = &inputs[0];
        let b = &inputs[1];
        let pos_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..a.len() {
            let pos = self.position + if pos_mod.is_empty() { 0.0 } else { pos_mod[i] };
            let gain_a = (1.0 - pos).sqrt();
            let gain_b = pos.sqrt();
            output[i] = a[i] * gain_a + b[i] * gain_b;
        }
    }
}

/// State of a Stereo Width
#[derive(Debug, Clone)]
pub struct StereoWidthState;

/// Stereo Width
#[derive(Debug, Clone)]
pub struct StereoWidth {
    pub width: f32, // 0.0 = mono, 1.0 = wide
}

impl NodeDef for StereoWidth {
    type State = StereoWidthState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // L
            Port { id: PortId(1), rate: Rate::Audio }, // R
            Port { id: PortId(2), rate: Rate::Audio }, // width_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // L
            Port { id: PortId(1), rate: Rate::Audio }, // R
        ];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        2
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        StereoWidthState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let l = &inputs[0];
        let r = &inputs[1];
        let width_mod = if inputs.len() > 2 { inputs[2] } else { &[] };

        for i in 0..l.len() {
            let width = self.width + if width_mod.is_empty() { 0.0 } else { width_mod[i] };
            let mid = (l[i] + r[i]) * 0.5;
            let side = (l[i] - r[i]) * 0.5 * width;
            outputs[0][i] = mid + side;
            outputs[1][i] = mid - side;
        }
    }
}

/// State of a Parameter Smoother
#[derive(Debug, Clone)]
pub struct ParamSmootherState {
    pub current: f32,
}

/// Parameter Smoother
#[derive(Debug, Clone)]
pub struct ParamSmoother {
    pub smoothing: f32, // 0.0 = instant, 1.0 = slow
}

impl NodeDef for ParamSmoother {
    type State = ParamSmootherState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[Port { id: PortId(0), rate: Rate::Audio }]; // input param
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
        ParamSmootherState { current: 0.0 }
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

        let coeff = 1.0 - (-1.0 / (self.smoothing * sample_rate / 1000.0)).exp();

        for i in 0..input.len() {
            state.current += (input[i] - state.current) * coeff;
            output[i] = state.current;
        }
    }
}

/// State of a MidSideProcessor
#[derive(Debug, Clone)]
pub struct MidSideProcessorState;

/// Mid-Side Processor (encodes stereo to mid/side)
#[derive(Debug, Clone)]
pub struct MidSideProcessor;

impl NodeDef for MidSideProcessor {
    type State = MidSideProcessorState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // left
            Port { id: PortId(1), rate: Rate::Audio }, // right
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // mid
            Port { id: PortId(1), rate: Rate::Audio }, // side
        ];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        2
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        MidSideProcessorState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let left = &inputs[0];
        let right = &inputs[1];

        for i in 0..left.len() {
            let l = left[i];
            let r = right[i];
            outputs[0][i] = (l + r) * 0.5;
            outputs[1][i] = (l - r) * 0.5;
        }
    }
}

/// State of a StereoPanner
#[derive(Debug, Clone)]
pub struct StereoPannerState;

/// Stereo Panner (mono to stereo)
#[derive(Debug, Clone)]
pub struct StereoPanner {
    pub pan: f32, // -1.0 (left) to 1.0 (right)
}

impl NodeDef for StereoPanner {
    type State = StereoPannerState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // mono input
            Port { id: PortId(1), rate: Rate::Audio }, // pan_mod
        ];
        PORTS
    }

    fn output_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // left
            Port { id: PortId(1), rate: Rate::Audio }, // right
        ];
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        StereoPannerState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let pan_mod = if inputs.len() > 1 { inputs[1] } else { &[] };

        for i in 0..input.len() {
            let pan = (self.pan + if pan_mod.is_empty() { 0.0 } else { pan_mod[i] }).clamp(-1.0, 1.0);
            let left_gain = ((pan + 1.0) * 0.5).sqrt(); // equal power panning
            let right_gain = ((1.0 - pan) * 0.5).sqrt();
            outputs[0][i] = input[i] * left_gain;
            outputs[1][i] = input[i] * right_gain;
        }
    }
}

/// State of an RMSMeter
#[derive(Debug, Clone)]
pub struct RMSMeterState {
    pub rms: f32,
}

/// RMS Meter (analysis node)
#[derive(Debug, Clone)]
pub struct RMSMeter {
    pub window_size: usize,
}

impl NodeDef for RMSMeter {
    type State = RMSMeterState;

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
        RMSMeterState { rms: 0.0 }
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

        let mut sum_squares = 0.0;
        for &sample in input.iter() {
            sum_squares += sample * sample;
        }
        let block_rms = (sum_squares / input.len() as f32).sqrt();
        
        // Exponential smoothing
        let alpha = 1.0 / self.window_size as f32;
        state.rms = state.rms * (1.0 - alpha) + block_rms * alpha;

        for sample in output.iter_mut() {
            *sample = state.rms;
        }
    }
}