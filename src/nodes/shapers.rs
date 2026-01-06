use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of a WaveShaper
#[derive(Debug, Clone)]
pub struct WaveShaperState;

/// Wave Shaper (tanh soft clip)
#[derive(Debug, Clone)]
pub struct WaveShaper {
    pub drive: f32,
    pub mix: f32,
}

impl NodeDef for WaveShaper {
    type State = WaveShaperState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // input
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            }, // drive_mod
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            }, // mix_mod
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
        WaveShaperState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let drive_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let drive = self.drive
                + if drive_mod.is_empty() {
                    0.0
                } else {
                    drive_mod[i]
                };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let shaped = (input[i] * drive).tanh();
            output[i] = input[i] * (1.0 - mix) + shaped * mix;
        }
    }
}

/// State of a HardClip
#[derive(Debug, Clone)]
pub struct HardClipState;

/// Hard Clip
#[derive(Debug, Clone)]
pub struct HardClip {
    pub threshold: f32,
    pub mix: f32,
}

impl NodeDef for HardClip {
    type State = HardClipState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // input
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            }, // threshold_mod
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            }, // mix_mod
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
        HardClipState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let threshold_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let threshold = self.threshold
                + if threshold_mod.is_empty() {
                    0.0
                } else {
                    threshold_mod[i]
                };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let shaped = input[i].clamp(-threshold, threshold);
            output[i] = input[i] * (1.0 - mix) + shaped * mix;
        }
    }
}

/// State of a BitCrusher
#[derive(Debug, Clone)]
pub struct BitCrusherState;

/// Bit Crusher
#[derive(Debug, Clone)]
pub struct BitCrusher {
    pub bits: f32,
    pub mix: f32,
}

impl NodeDef for BitCrusher {
    type State = BitCrusherState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // input
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            }, // bits_mod
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            }, // mix_mod
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
        BitCrusherState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let bits_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let bits = self.bits
                + if bits_mod.is_empty() {
                    0.0
                } else {
                    bits_mod[i]
                };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let steps = 2.0_f32.powf(bits);
            let shaped = (input[i] * steps).round() / steps;
            output[i] = input[i] * (1.0 - mix) + shaped * mix;
        }
    }
}

/// State of a SoftClip
#[derive(Debug, Clone)]
pub struct SoftClipState;

/// Soft Clip (cubic)
#[derive(Debug, Clone)]
pub struct SoftClip {
    pub drive: f32,
    pub mix: f32,
}

impl NodeDef for SoftClip {
    type State = SoftClipState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // input
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            }, // drive_mod
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            }, // mix_mod
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
        SoftClipState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let drive_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let drive = self.drive
                + if drive_mod.is_empty() {
                    0.0
                } else {
                    drive_mod[i]
                };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let x = input[i] * drive;
            let shaped = if x.abs() < 1.0 / 3.0 {
                x - (1.0 / 3.0) * x * x * x
            } else {
                x.signum() * (2.0 / 3.0)
            };
            output[i] = input[i] * (1.0 - mix) + shaped * mix;
        }
    }
}

/// State of a TubeSaturation
#[derive(Debug, Clone)]
pub struct TubeSaturationState;

/// Tube Saturation (asymmetric tanh)
#[derive(Debug, Clone)]
pub struct TubeSaturation {
    pub drive: f32,
    pub asymmetry: f32,
    pub mix: f32,
}

impl NodeDef for TubeSaturation {
    type State = TubeSaturationState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // input
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            }, // drive_mod
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            }, // asymmetry_mod
            Port {
                id: PortId(3),
                rate: Rate::Audio,
            }, // mix_mod
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
        TubeSaturationState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let drive_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let asymmetry_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let mix_mod = if inputs.len() > 3 { inputs[3] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let drive = self.drive
                + if drive_mod.is_empty() {
                    0.0
                } else {
                    drive_mod[i]
                };
            let asymmetry = self.asymmetry
                + if asymmetry_mod.is_empty() {
                    0.0
                } else {
                    asymmetry_mod[i]
                };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let x = input[i] * drive;
            let shaped = if x > 0.0 {
                (x * (1.0 + asymmetry)).tanh()
            } else {
                (x * (1.0 - asymmetry)).tanh()
            };
            output[i] = input[i] * (1.0 - mix) + shaped * mix;
        }
    }
}

/// State of a DC Blocker
#[derive(Debug, Clone)]
pub struct DcBlockerState {
    pub x1: f32,
    pub y1: f32,
}

/// DC Blocker (highpass filter at very low freq)
#[derive(Debug, Clone)]
pub struct DcBlocker {
    pub cutoff: f32,
}

impl NodeDef for DcBlocker {
    type State = DcBlockerState;

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
        DcBlockerState { x1: 0.0, y1: 0.0 }
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

        let r = 0.995; // close to 1 for low cutoff

        for i in 0..input.len() {
            let x = input[i];
            let y = x - state.x1 + r * state.y1;
            state.x1 = x;
            state.y1 = y;
            output[i] = y;
        }
    }
}

/// State of an Overdrive
#[derive(Debug, Clone)]
pub struct OverdriveState;

/// Overdrive Distortion
#[derive(Debug, Clone)]
pub struct Overdrive {
    pub drive: f32,
    pub mix: f32,
}

impl NodeDef for Overdrive {
    type State = OverdriveState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // input
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            }, // drive_mod
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            }, // mix_mod
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
        OverdriveState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let input = &inputs[0];
        let drive_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let drive = self.drive
                + if drive_mod.is_empty() {
                    0.0
                } else {
                    drive_mod[i]
                };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let x = input[i] * drive;
            // Simple overdrive: tanh with pre-gain
            let shaped = (x * 2.0).tanh() * 0.8;
            output[i] = input[i] * (1.0 - mix) + shaped * mix;
        }
    }
}
