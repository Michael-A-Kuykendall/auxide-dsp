use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of an ADSR Envelope
#[derive(Debug, Clone)]
pub struct AdsrState {
    pub phase: AdsrPhase,
    pub level: f32,
    pub time_accum: f32,
}

/// Phases of ADSR
#[derive(Debug, Clone, Copy)]
pub enum AdsrPhase {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR Envelope
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain_level: f32,
    pub release_ms: f32,
    pub curve: f32, // 0.0 = linear, >0 exponential
}

impl NodeDef for AdsrEnvelope {
    type State = AdsrState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // gate
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
        AdsrState {
            phase: AdsrPhase::Idle,
            level: 0.0,
            time_accum: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let gate = &inputs[0];
        let output = &mut outputs[0];

        let dt = 1.0 / sample_rate;

        for i in 0..gate.len() {
            let gate_on = gate[i] > 0.5;

            match state.phase {
                AdsrPhase::Idle => {
                    if gate_on {
                        state.phase = AdsrPhase::Attack;
                        state.time_accum = 0.0;
                    }
                    state.level = 0.0;
                }
                AdsrPhase::Attack => {
                    state.time_accum += dt;
                    let t = (state.time_accum / (self.attack_ms / 1000.0)).min(1.0);
                    state.level = if self.curve > 0.0 {
                        1.0 - (-t * self.curve).exp()
                    } else {
                        t
                    };
                    if state.time_accum >= self.attack_ms / 1000.0 {
                        state.phase = AdsrPhase::Decay;
                        state.time_accum = 0.0;
                    }
                    if !gate_on {
                        state.phase = AdsrPhase::Release;
                        state.time_accum = 0.0;
                    }
                }
                AdsrPhase::Decay => {
                    state.time_accum += dt;
                    let t = (state.time_accum / (self.decay_ms / 1000.0)).min(1.0);
                    let decay_factor = if self.curve > 0.0 {
                        (-t * self.curve).exp()
                    } else {
                        1.0 - t
                    };
                    state.level = 1.0 + (self.sustain_level - 1.0) * decay_factor;
                    if state.time_accum >= self.decay_ms / 1000.0 {
                        state.phase = AdsrPhase::Sustain;
                    }
                    if !gate_on {
                        state.phase = AdsrPhase::Release;
                        state.time_accum = 0.0;
                    }
                }
                AdsrPhase::Sustain => {
                    state.level = self.sustain_level;
                    if !gate_on {
                        state.phase = AdsrPhase::Release;
                        state.time_accum = 0.0;
                    }
                }
                AdsrPhase::Release => {
                    state.time_accum += dt;
                    let t = (state.time_accum / (self.release_ms / 1000.0)).min(1.0);
                    let release_factor = if self.curve > 0.0 {
                        (-t * self.curve).exp()
                    } else {
                        1.0 - t
                    };
                    state.level = self.sustain_level * release_factor;
                    if state.time_accum >= self.release_ms / 1000.0 {
                        state.phase = AdsrPhase::Idle;
                        state.level = 0.0;
                    }
                }
            }

            output[i] = state.level;
        }
    }
}

/// State of an AR Envelope
#[derive(Debug, Clone)]
pub struct ArState {
    pub phase: ArPhase,
    pub level: f32,
    pub time_accum: f32,
}

/// Phases of AR
#[derive(Debug, Clone, Copy)]
pub enum ArPhase {
    Idle,
    Attack,
    Release,
}

/// AR Envelope
#[derive(Debug, Clone)]
pub struct ArEnvelope {
    pub attack_ms: f32,
    pub release_ms: f32,
    pub curve: f32,
}

impl NodeDef for ArEnvelope {
    type State = ArState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // gate
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
        ArState {
            phase: ArPhase::Idle,
            level: 0.0,
            time_accum: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let gate = &inputs[0];
        let output = &mut outputs[0];

        let dt = 1.0 / sample_rate;

        for i in 0..gate.len() {
            let gate_on = gate[i] > 0.5;

            match state.phase {
                ArPhase::Idle => {
                    if gate_on {
                        state.phase = ArPhase::Attack;
                        state.time_accum = 0.0;
                    }
                    state.level = 0.0;
                }
                ArPhase::Attack => {
                    state.time_accum += dt;
                    let t = (state.time_accum / (self.attack_ms / 1000.0)).min(1.0);
                    state.level = if self.curve > 0.0 {
                        1.0 - (-t * self.curve).exp()
                    } else {
                        t
                    };
                    if state.time_accum >= self.attack_ms / 1000.0 {
                        state.level = 1.0;
                    }
                    if !gate_on {
                        state.phase = ArPhase::Release;
                        state.time_accum = 0.0;
                    }
                }
                ArPhase::Release => {
                    state.time_accum += dt;
                    let t = (state.time_accum / (self.release_ms / 1000.0)).min(1.0);
                    let release_factor = if self.curve > 0.0 {
                        (-t * self.curve).exp()
                    } else {
                        1.0 - t
                    };
                    state.level *= release_factor;
                    if state.time_accum >= self.release_ms / 1000.0 {
                        state.phase = ArPhase::Idle;
                        state.level = 0.0;
                    }
                }
            }

            output[i] = state.level;
        }
    }
}

/// State of an AD Envelope
#[derive(Debug, Clone)]
pub struct AdState {
    pub phase: AdPhase,
    pub level: f32,
    pub time_accum: f32,
}

/// Phases of AD
#[derive(Debug, Clone, Copy)]
pub enum AdPhase {
    Idle,
    Attack,
    Decay,
}

/// AD Envelope
#[derive(Debug, Clone)]
pub struct AdEnvelope {
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub curve: f32,
}

impl NodeDef for AdEnvelope {
    type State = AdState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            }, // gate
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
        AdState {
            phase: AdPhase::Idle,
            level: 0.0,
            time_accum: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let gate = &inputs[0];
        let output = &mut outputs[0];

        let dt = 1.0 / sample_rate;

        for i in 0..gate.len() {
            let gate_on = gate[i] > 0.5;

            match state.phase {
                AdPhase::Idle => {
                    if gate_on {
                        state.phase = AdPhase::Attack;
                        state.time_accum = 0.0;
                    }
                    state.level = 0.0;
                }
                AdPhase::Attack => {
                    state.time_accum += dt;
                    let t = (state.time_accum / (self.attack_ms / 1000.0)).min(1.0);
                    state.level = if self.curve > 0.0 {
                        1.0 - (-t * self.curve).exp()
                    } else {
                        t
                    };
                    if state.time_accum >= self.attack_ms / 1000.0 {
                        state.phase = AdPhase::Decay;
                        state.time_accum = 0.0;
                    }
                    if !gate_on {
                        state.phase = AdPhase::Decay;
                        state.time_accum = 0.0;
                    }
                }
                AdPhase::Decay => {
                    state.time_accum += dt;
                    let t = (state.time_accum / (self.decay_ms / 1000.0)).min(1.0);
                    let decay_factor = if self.curve > 0.0 {
                        (-t * self.curve).exp()
                    } else {
                        1.0 - t
                    };
                    state.level = decay_factor;
                    if state.time_accum >= self.decay_ms / 1000.0 {
                        state.phase = AdPhase::Idle;
                        state.level = 0.0;
                    }
                }
            }

            output[i] = state.level;
        }
    }
}
