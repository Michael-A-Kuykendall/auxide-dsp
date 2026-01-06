use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of a Pitch Shifter
#[derive(Debug, Clone)]
pub struct PitchShifterState {
    pub buffer: Vec<f32>,
    pub index: usize,
    pub phase: f32,
}

/// Pitch Shifter (simple delay-based)
#[derive(Debug, Clone)]
pub struct PitchShifter {
    pub shift: f32, // semitones
    pub mix: f32,
}

impl NodeDef for PitchShifter {
    type State = PitchShifterState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // input
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            }, // shift_mod
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

    fn init_state(&self, sample_rate: f32, _block_size: usize) -> Self::State {
        let max_delay = (sample_rate / 50.0) as usize; // up to 20ms
        PitchShifterState {
            buffer: vec![0.0; max_delay],
            index: 0,
            phase: 0.0,
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
        let shift_mod = if inputs.len() > 1 { inputs[1] } else { &[] };
        let mix_mod = if inputs.len() > 2 { inputs[2] } else { &[] };
        let output = &mut outputs[0];

        for i in 0..input.len() {
            let shift = self.shift
                + if shift_mod.is_empty() {
                    0.0
                } else {
                    shift_mod[i]
                };
            let mix = self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] };

            let ratio = 2.0_f32.powf(shift / 12.0);
            let delay_samples = (sample_rate / 440.0 / ratio) as usize; // approximate for A4

            let delayed_idx = (state.index + state.buffer.len()
                - delay_samples.min(state.buffer.len() - 1))
                % state.buffer.len();
            let delayed = state.buffer[delayed_idx];

            output[i] = input[i] * (1.0 - mix) + delayed * mix;

            state.buffer[state.index] = input[i];
            state.index = (state.index + 1) % state.buffer.len();
        }
    }
}

/// State of a Spectral Gate
#[derive(Debug, Clone)]
pub struct SpectralGateState {
    pub envelope: f32,
}

/// Spectral Gate (simple noise gate)
#[derive(Debug, Clone)]
pub struct SpectralGate {
    pub threshold: f32,
    pub ratio: f32,
}

impl NodeDef for SpectralGate {
    type State = SpectralGateState;

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
        SpectralGateState { envelope: 0.0 }
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

        let attack_coeff = (-1.0 / (1.0 * sample_rate / 1000.0)).exp();
        let release_coeff = (-1.0 / (10.0 * sample_rate / 1000.0)).exp();

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
            let gain_linear = if state.envelope > 0.0 {
                gain / state.envelope
            } else {
                0.0
            };

            output[i] = input[i] * gain_linear;
        }
    }
}

/// State of a Pitch Detector
#[derive(Debug, Clone)]
pub struct PitchDetectorState {
    pub prev_sample: f32,
    pub period: f32,
}

/// Pitch Detector (simple zero-crossing)
#[derive(Debug, Clone)]
pub struct PitchDetector;

impl NodeDef for PitchDetector {
    type State = PitchDetectorState;

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
        }]; // pitch in Hz
        PORTS
    }

    fn required_inputs(&self) -> usize {
        1
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        PitchDetectorState {
            prev_sample: 0.0,
            period: 0.0,
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

        for i in 0..input.len() {
            if (state.prev_sample <= 0.0 && input[i] > 0.0)
                || (state.prev_sample >= 0.0 && input[i] < 0.0)
            {
                // zero crossing
                let freq = sample_rate / state.period.max(1.0);
                output[i] = freq;
                state.period = 0.0;
            } else {
                output[i] = output.get(i.saturating_sub(1)).copied().unwrap_or(0.0);
            }
            state.period += 1.0;
            state.prev_sample = input[i];
        }
    }
}
