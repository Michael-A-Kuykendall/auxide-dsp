#![forbid(unsafe_code)]

use std::sync::Arc;

use auxide::graph::Port;
use auxide::node::NodeDef;

use crate::helpers::{freq_to_phase_increment, polyblep};

const PORTS_NONE: &[Port] = &[];
const PORTS_MONO_OUT: &[Port] = &[Port {
    id: auxide::graph::PortId(0),
    rate: auxide::graph::Rate::Audio,
}];

#[derive(Clone)]
pub struct SawOsc {
    pub freq: f32,
}

#[derive(Clone)]
pub struct SquareOsc {
    pub freq: f32,
    pub pulse_width: f32,
}

#[derive(Clone)]
pub struct TriangleOsc {
    pub freq: f32,
}

#[derive(Clone)]
pub struct PulseOsc {
    pub freq: f32,
    pub pulse_width: f32,
}

#[derive(Clone)]
pub struct WavetableOsc {
    pub freq: f32,
    pub table: Arc<Vec<f32>>,
}

#[derive(Clone)]
pub struct SuperSaw {
    pub freq: f32,
    pub detune: f32,
    pub voices: usize,
}

#[derive(Clone)]
pub struct WhiteNoise;

#[derive(Clone)]
pub struct PinkNoise;

#[derive(Clone)]
pub struct BrownNoise;

pub struct OscState {
    phase: f32,
}

pub struct MultiPhaseState {
    phases: Vec<f32>,
}

pub struct NoiseState {
    rng: u64,
    pink: [f32; 7],
    brown: f32,
}

impl SawOsc {
    pub fn new(freq: f32) -> Self {
        Self { freq }
    }
}

impl NodeDef for SawOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState { phase: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let inc = freq_to_phase_increment(self.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            *sample = 2.0 * phase - 1.0;
            // PolyBLEP to reduce aliasing
            *sample -= polyblep(phase, inc);
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for SquareOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState { phase: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let pw = self.pulse_width.clamp(0.01, 0.99);
        let inc = freq_to_phase_increment(self.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            let base = if phase < pw { 1.0 } else { -1.0 };
            // Apply polyblep at both edges
            let mut val = base;
            val += polyblep(phase, inc);
            let phase_pw = (phase - pw + 1.0) % 1.0;
            val -= polyblep(phase_pw, inc);
            *sample = val;
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for TriangleOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState { phase: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let inc = freq_to_phase_increment(self.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            let saw = 2.0 * phase - 1.0;
            *sample = (2.0 / std::f32::consts::PI)
                * (saw.abs() * std::f32::consts::PI / 2.0 - std::f32::consts::PI / 4.0).sin();
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for PulseOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState { phase: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let pw = self.pulse_width.clamp(0.01, 0.99);
        let inc = freq_to_phase_increment(self.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            *sample = if phase < pw { 1.0 } else { -1.0 };
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for WavetableOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState { phase: 0.0 }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let table = &*self.table;
        if table.is_empty() {
            out.fill(0.0);
            return;
        }
        let inc = freq_to_phase_increment(self.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        let len = table.len() as f32;
        for sample in out.iter_mut() {
            let idx = (state.phase * len) as usize % table.len();
            *sample = table[idx];
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for SuperSaw {
    type State = MultiPhaseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        let voices = self.voices.max(1);
        MultiPhaseState {
            phases: vec![0.0; voices],
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let voices = state.phases.len().max(1);
        let base_inc =
            freq_to_phase_increment(self.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        let detune = self.detune.max(0.0);
        for sample in out.iter_mut() {
            let mut acc = 0.0;
            for (i, phase) in state.phases.iter_mut().enumerate() {
                let detune_factor =
                    1.0 + detune * ((i as f32) - (voices as f32 - 1.0) / 2.0) / (voices as f32);
                let inc = base_inc * detune_factor;
                acc += 2.0 * *phase - 1.0;
                *phase += inc;
                if *phase >= 1.0 {
                    *phase -= 1.0;
                }
            }
            *sample = acc / voices as f32;
        }
    }
}

impl NodeDef for WhiteNoise {
    type State = NoiseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        NoiseState {
            rng: 0x1234_5678_9abc_def0,
            pink: [0.0; 7],
            brown: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        for sample in out.iter_mut() {
            // LCG
            state.rng = state.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let v = ((state.rng >> 32) as u32) as f32 / (u32::MAX as f32);
            *sample = v * 2.0 - 1.0;
        }
    }
}

impl NodeDef for PinkNoise {
    type State = NoiseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        NoiseState {
            rng: 0x1234_5678_9abc_def0,
            pink: [0.0; 7],
            brown: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        for sample in out.iter_mut() {
            state.rng = state.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let white = ((state.rng >> 32) as u32) as f32 / (u32::MAX as f32) * 2.0 - 1.0;
            state.pink[0] = 0.99886 * state.pink[0] + white * 0.0555179;
            state.pink[1] = 0.99332 * state.pink[1] + white * 0.0750759;
            state.pink[2] = 0.96900 * state.pink[2] + white * 0.153_852;
            state.pink[3] = 0.86650 * state.pink[3] + white * 0.3104856;
            state.pink[4] = 0.55000 * state.pink[4] + white * 0.5329522;
            state.pink[5] = -0.7616 * state.pink[5] - white * 0.0168980;
            let pink = state.pink.iter().sum::<f32>() + state.pink[6] + white * 0.5362;
            state.pink[6] = white * 0.115926;
            *sample = pink * 0.1;
        }
    }
}

impl NodeDef for BrownNoise {
    type State = NoiseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        NoiseState {
            rng: 0x1234_5678_9abc_def0,
            pink: [0.0; 7],
            brown: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        for sample in out.iter_mut() {
            state.rng = state.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let white = ((state.rng >> 32) as u32) as f32 / (u32::MAX as f32) * 2.0 - 1.0;
            state.brown += white * 0.02;
            state.brown = state.brown.clamp(-1.0, 1.0);
            *sample = state.brown;
        }
    }
}

/// State of a Constant
#[derive(Debug, Clone)]
pub struct ConstantState;

/// Constant value source
#[derive(Debug, Clone)]
pub struct Constant {
    pub value: f32,
}

impl NodeDef for Constant {
    type State = ConstantState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        ConstantState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let output = &mut outputs[0];
        for sample in output.iter_mut() {
            *sample = self.value;
        }
    }
}
