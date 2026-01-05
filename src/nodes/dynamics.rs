use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of a Compressor
#[derive(Debug, Clone)]
pub struct CompressorState {
    pub envelope: f32,
}

/// Compressor
#[derive(Debug, Clone)]
pub struct Compressor {
    pub threshold: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_gain: f32,
}

impl NodeDef for Compressor {
    type State = CompressorState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port { id: PortId(0), rate: Rate::Audio }, // input
            Port { id: PortId(1), rate: Rate::Audio }, // sidechain
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
        CompressorState { envelope: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let input = &inputs[0];
        let sidechain = if inputs.len() > 1 { inputs[1] } else { input };
        let output = &mut outputs[0];

        let attack_coeff = (-1.0 / (self.attack_ms * sample_rate / 1000.0)).exp();
        let release_coeff = (-1.0 / (self.release_ms * sample_rate / 1000.0)).exp();

        for i in 0..input.len() {
            let key = sidechain[i].abs();
            if key > state.envelope {
                state.envelope = attack_coeff * (state.envelope - key) + key;
            } else {
                state.envelope = release_coeff * (state.envelope - key) + key;
            }

            let gain = if state.envelope > self.threshold {
                self.threshold + (state.envelope - self.threshold) / self.ratio
            } else {
                state.envelope
            };
            let gain_db = 20.0 * (gain / state.envelope).log10();
            let gain_linear = 10.0_f32.powf(gain_db / 20.0) * self.makeup_gain;

            output[i] = input[i] * gain_linear;
        }
    }
}

/// State of a Limiter
#[derive(Debug, Clone)]
pub struct LimiterState {
    pub envelope: f32,
}

/// Limiter (hard knee compressor with high ratio)
#[derive(Debug, Clone)]
pub struct Limiter {
    pub threshold: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
}

impl NodeDef for Limiter {
    type State = LimiterState;

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
        LimiterState { envelope: 0.0 }
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

        let attack_coeff = (-1.0 / (self.attack_ms * sample_rate / 1000.0)).exp();
        let release_coeff = (-1.0 / (self.release_ms * sample_rate / 1000.0)).exp();

        for i in 0..input.len() {
            let key = input[i].abs();
            if key > state.envelope {
                state.envelope = attack_coeff * (state.envelope - key) + key;
            } else {
                state.envelope = release_coeff * (state.envelope - key) + key;
            }

            let gain = if state.envelope > self.threshold {
                self.threshold
            } else {
                state.envelope
            };
            let gain_linear = if state.envelope > 0.0 { gain / state.envelope } else { 1.0 };

            output[i] = input[i] * gain_linear;
        }
    }
}

/// State of a Noise Gate
#[derive(Debug, Clone)]
pub struct GateState {
    pub envelope: f32,
}

/// Noise Gate
#[derive(Debug, Clone)]
pub struct NoiseGate {
    pub threshold: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
}

impl NodeDef for NoiseGate {
    type State = GateState;

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
        GateState { envelope: 0.0 }
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

        let attack_coeff = (-1.0 / (self.attack_ms * sample_rate / 1000.0)).exp();
        let release_coeff = (-1.0 / (self.release_ms * sample_rate / 1000.0)).exp();

        for i in 0..input.len() {
            let key = input[i].abs();
            if key > state.envelope {
                state.envelope = attack_coeff * (state.envelope - key) + key;
            } else {
                state.envelope = release_coeff * (state.envelope - key) + key;
            }

            let gain = if state.envelope < self.threshold {
                0.0
            } else {
                state.envelope
            };
            let gain_linear = if state.envelope > 0.0 { gain / state.envelope } else { 0.0 };

            output[i] = input[i] * gain_linear;
        }
    }
}

/// State of an Expander
#[derive(Debug, Clone)]
pub struct ExpanderState {
    pub envelope: f32,
}

/// Expander
#[derive(Debug, Clone)]
pub struct Expander {
    pub threshold: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
}

impl NodeDef for Expander {
    type State = ExpanderState;

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
        ExpanderState { envelope: 0.0 }
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

        let attack_coeff = (-1.0 / (self.attack_ms * sample_rate / 1000.0)).exp();
        let release_coeff = (-1.0 / (self.release_ms * sample_rate / 1000.0)).exp();

        for i in 0..input.len() {
            let key = input[i].abs();
            if key > state.envelope {
                state.envelope = attack_coeff * (state.envelope - key) + key;
            } else {
                state.envelope = release_coeff * (state.envelope - key) + key;
            }

            let gain = if state.envelope < self.threshold {
                self.threshold + (state.envelope - self.threshold) * self.ratio
            } else {
                state.envelope
            };
            let gain_linear = if state.envelope > 0.0 { gain / state.envelope } else { 0.0 };

            output[i] = input[i] * gain_linear;
        }
    }
}