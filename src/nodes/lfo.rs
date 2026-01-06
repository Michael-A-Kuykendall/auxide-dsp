use crate::helpers::freq_to_phase_increment;
use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of an LFO
#[derive(Debug, Clone)]
pub struct LfoState {
    pub phase: f32,
}

/// LFO with multiple waveforms
#[derive(Debug, Clone)]
pub struct Lfo {
    pub frequency: f32,
    pub waveform: LfoWaveform,
    pub amplitude: f32,
    pub offset: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum LfoWaveform {
    Sine,
    Triangle,
    Saw,
    Square,
    Random, // Sample and hold
}

impl NodeDef for Lfo {
    type State = LfoState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // freq_mod
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
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        LfoState { phase: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let freq_mod = if inputs.is_empty() { &[] } else { inputs[0] };
        let output = &mut outputs[0];

        for i in 0..output.len() {
            let freq = self.frequency
                + if freq_mod.is_empty() {
                    0.0
                } else {
                    freq_mod[i]
                };
            let phase_inc = freq_to_phase_increment(freq, sample_rate);

            state.phase = (state.phase + phase_inc).fract();

            let raw = match self.waveform {
                LfoWaveform::Sine => (state.phase * std::f32::consts::TAU).sin(),
                LfoWaveform::Triangle => {
                    if state.phase < 0.25 {
                        state.phase * 4.0
                    } else if state.phase < 0.75 {
                        2.0 - state.phase * 4.0
                    } else {
                        state.phase * 4.0 - 4.0
                    }
                }
                LfoWaveform::Saw => state.phase * 2.0 - 1.0,
                LfoWaveform::Square => {
                    if state.phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                LfoWaveform::Random => {
                    // Simple pseudo-random: use phase as seed
                    ((state.phase * 12345.0).sin() - 0.5) * 2.0
                }
            };

            output[i] = raw * self.amplitude + self.offset;
        }
    }
}
