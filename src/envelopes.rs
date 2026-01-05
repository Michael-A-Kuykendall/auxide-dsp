//! Envelopes module: ADSR, AR, and other envelope generators.

use crate::{DspNode, Sample, SampleRate, AudioBlock, AudioBlockMut};

/// ADSR envelope generator.
#[derive(Clone)]
pub struct Adsr {
    pub attack_ms: Sample,
    pub decay_ms: Sample,
    pub sustain_level: Sample, // 0.0 to 1.0
    pub release_ms: Sample,
    pub gate: bool,
    state: EnvelopeState,
    level: Sample,
    attack_rate: Sample,
    decay_rate: Sample,
    release_rate: Sample,
}

#[derive(Clone, Copy, PartialEq)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Adsr {
    pub fn new(attack_ms: Sample, decay_ms: Sample, sustain_level: Sample, release_ms: Sample, sample_rate: SampleRate) -> Self {
        let attack_rate = 1.0 / (attack_ms * 0.001 * sample_rate);
        let decay_rate = 1.0 / (decay_ms * 0.001 * sample_rate);
        let release_rate = 1.0 / (release_ms * 0.001 * sample_rate);
        
        Self {
            attack_ms,
            decay_ms,
            sustain_level: sustain_level.clamp(0.0, 1.0),
            release_ms,
            gate: false,
            state: EnvelopeState::Idle,
            level: 0.0,
            attack_rate,
            decay_rate,
            release_rate,
        }
    }
    
    pub fn trigger(&mut self, gate: bool) {
        self.gate = gate;
        if gate && self.state == EnvelopeState::Idle {
            self.state = EnvelopeState::Attack;
        } else if !gate && self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
        }
    }
}

impl DspNode for Adsr {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("Adsr requires 1 output");
        }
        
        let output = &mut outputs[0];
        
        for o in output.iter_mut() {
            match self.state {
                EnvelopeState::Idle => {
                    self.level = 0.0;
                },
                EnvelopeState::Attack => {
                    self.level += self.attack_rate;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                        self.state = EnvelopeState::Decay;
                    }
                },
                EnvelopeState::Decay => {
                    self.level -= self.decay_rate * (1.0 - self.sustain_level);
                    if self.level <= self.sustain_level {
                        self.level = self.sustain_level;
                        self.state = EnvelopeState::Sustain;
                    }
                },
                EnvelopeState::Sustain => {
                    // Hold sustain level
                },
                EnvelopeState::Release => {
                    self.level -= self.release_rate * self.sustain_level;
                    if self.level <= 0.0 {
                        self.level = 0.0;
                        self.state = EnvelopeState::Idle;
                    }
                },
            }
            
            *o = self.level;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.level = 0.0;
        self.gate = false;
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// AR (Attack-Release) envelope.
#[derive(Clone)]
pub struct Ar {
    pub attack_ms: Sample,
    pub release_ms: Sample,
    pub gate: bool,
    state: ArState,
    level: Sample,
    attack_rate: Sample,
    release_rate: Sample,
}

#[derive(Clone, Copy, PartialEq)]
enum ArState {
    Idle,
    Attack,
    Release,
}

impl Ar {
    pub fn new(attack_ms: Sample, release_ms: Sample, sample_rate: SampleRate) -> Self {
        let attack_rate = 1.0 / (attack_ms * 0.001 * sample_rate);
        let release_rate = 1.0 / (release_ms * 0.001 * sample_rate);
        
        Self {
            attack_ms,
            release_ms,
            gate: false,
            state: ArState::Idle,
            level: 0.0,
            attack_rate,
            release_rate,
        }
    }
    
    pub fn trigger(&mut self, gate: bool) {
        self.gate = gate;
        if gate && self.state == ArState::Idle {
            self.state = ArState::Attack;
        } else if !gate && self.state == ArState::Attack {
            self.state = ArState::Release;
        }
    }
}

impl DspNode for Ar {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("Ar requires 1 output");
        }
        
        let output = &mut outputs[0];
        
        for o in output.iter_mut() {
            match self.state {
                ArState::Idle => {
                    self.level = 0.0;
                },
                ArState::Attack => {
                    self.level += self.attack_rate;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                    }
                },
                ArState::Release => {
                    self.level -= self.release_rate;
                    if self.level <= 0.0 {
                        self.level = 0.0;
                        self.state = ArState::Idle;
                    }
                },
            }
            
            *o = self.level;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.state = ArState::Idle;
        self.level = 0.0;
        self.gate = false;
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Exponential decay envelope.
#[derive(Clone)]
pub struct Decay {
    pub decay_ms: Sample,
    level: Sample,
    decay_rate: Sample,
    triggered: bool,
}

impl Decay {
    pub fn new(decay_ms: Sample, sample_rate: SampleRate) -> Self {
        let decay_rate = 1.0 / (decay_ms * 0.001 * sample_rate);
        
        Self {
            decay_ms,
            level: 0.0,
            decay_rate,
            triggered: false,
        }
    }
    
    pub fn trigger(&mut self) {
        self.level = 1.0;
        self.triggered = true;
    }
}

impl DspNode for Decay {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("Decay requires 1 output");
        }
        
        let output = &mut outputs[0];
        
        for o in output.iter_mut() {
            if self.triggered {
                self.level -= self.decay_rate;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.triggered = false;
                }
            } else {
                self.level = 0.0;
            }
            
            *o = self.level;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.level = 0.0;
        self.triggered = false;
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}

/// Multi-stage envelope.
#[derive(Clone)]
pub struct MultiStageEnvelope {
    pub stages: Vec<EnvelopeStage>,
    current_stage: usize,
    level: Sample,
    stage_time: Sample,
    sample_rate: SampleRate,
}

#[derive(Clone, Copy)]
pub struct EnvelopeStage {
    pub duration_ms: Sample,
    pub target_level: Sample,
    pub curve: EnvelopeCurve,
}

#[derive(Clone, Copy)]
pub enum EnvelopeCurve {
    Linear,
    Exponential,
    Logarithmic,
}

impl MultiStageEnvelope {
    pub fn new(stages: Vec<EnvelopeStage>, sample_rate: SampleRate) -> Self {
        Self {
            stages,
            current_stage: 0,
            level: 0.0,
            stage_time: 0.0,
            sample_rate,
        }
    }
    
    pub fn trigger(&mut self) {
        self.current_stage = 0;
        self.stage_time = 0.0;
        self.level = 0.0;
    }
}

impl DspNode for MultiStageEnvelope {
    fn process(&mut self, _inputs: &[AudioBlock], outputs: &mut [AudioBlockMut], _sample_rate: SampleRate) -> Result<(), &'static str> {
        if outputs.is_empty() {
            return Err("MultiStageEnvelope requires 1 output");
        }
        
        let output = &mut outputs[0];
        
        for o in output.iter_mut() {
            if self.current_stage < self.stages.len() {
                let stage = &self.stages[self.current_stage];
                let stage_duration_samples = stage.duration_ms * 0.001 * self.sample_rate;
                
                if self.stage_time < stage_duration_samples {
                    let t = self.stage_time / stage_duration_samples;
                    let start_level = if self.current_stage == 0 { 0.0 } else { self.stages[self.current_stage - 1].target_level };
                    
                    match stage.curve {
                        EnvelopeCurve::Linear => {
                            self.level = start_level + (stage.target_level - start_level) * t;
                        },
                        EnvelopeCurve::Exponential => {
                            let ratio = stage.target_level / start_level.max(0.001);
                            self.level = start_level * ratio.powf(t);
                        },
                        EnvelopeCurve::Logarithmic => {
                            let ratio = (stage.target_level / start_level.max(0.001)).ln();
                            self.level = start_level * (1.0 + ratio * t).exp();
                        },
                    }
                    
                    self.stage_time += 1.0;
                } else {
                    self.level = stage.target_level;
                    self.current_stage += 1;
                    self.stage_time = 0.0;
                }
            } else {
                self.level = self.stages.last().map(|s| s.target_level).unwrap_or(0.0);
            }
            
            *o = self.level;
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        self.current_stage = 0;
        self.level = 0.0;
        self.stage_time = 0.0;
    }

    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
}