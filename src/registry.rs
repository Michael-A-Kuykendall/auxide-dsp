//! Server integration: register all `auxide-dsp` UGens into a kernel [`Registry`].
//!
//! `auxide-server` calls [`register_dsp_ugens`] at init so that DSP nodes can
//! be referenced by name inside a `SynthDef` (e.g. `"saw"`, `"svf"`). The
//! [`Registry`] type lives in the kernel (`auxide`) so this crate — which the
//! server depends on — can populate it without inverting the dependency
//! direction.

#![forbid(unsafe_code)]

use auxide::registry::{external, param_or, ParamMap, Registry};

use crate::nodes::dynamics::{Compressor, Expander, Limiter, NoiseGate};
use crate::nodes::envelopes::{AdEnvelope, AdsrEnvelope, ArEnvelope};
use crate::nodes::filters::{
    AllpassFilter, BiquadFilter, CombFilter, FormantFilter, LadderFilter, OnePoleFilter,
    ParametricEq, ResonantDrive, SvfFilter, SvfMode,
};
use crate::nodes::fx::{Chorus, Delay, Flanger, Phaser, SimpleReverb, StereoReverb, Tremolo};
use crate::nodes::lfo::{Lfo, LfoWaveform};
use crate::nodes::oscillators::{
    BrownNoise, PinkNoise, PulseOsc, SawOsc, SquareOsc, SuperSaw, TriangleOsc, WhiteNoise,
};
use crate::nodes::shapers::{BitCrusher, DcBlocker, HardClip, Overdrive, SoftClip, WaveShaper};
use crate::nodes::utility::{Crossfader, Mixer, RingMod, StereoPanner};

/// Register every `auxide-dsp` UGen into `reg`.
///
/// Each entry maps a stable, SuperCollider-flavored name to a factory that
/// builds the corresponding [`NodeType::External`] from a [`ParamMap`]. Missing
/// parameters fall back to musically sensible defaults (see [`param_or`]), so a
/// `SynthDef` only specifies what it needs.
///
/// New UGen categories are added here as their `auxide-dsp-*` beads land.
pub fn register_dsp_ugens(reg: &mut Registry) {
    // ---- Oscillators -------------------------------------------------------
    reg.register("saw", |p: &ParamMap| {
        external(SawOsc {
            freq: param_or(p, "freq", 440.0),
        })
    });
    reg.register("square", |p: &ParamMap| {
        external(SquareOsc {
            freq: param_or(p, "freq", 440.0),
            pulse_width: param_or(p, "pulse_width", 0.5),
        })
    });
    reg.register("triangle", |p: &ParamMap| {
        external(TriangleOsc {
            freq: param_or(p, "freq", 440.0),
        })
    });
    reg.register("pulse", |p: &ParamMap| {
        external(PulseOsc {
            freq: param_or(p, "freq", 440.0),
            pulse_width: param_or(p, "pulse_width", 0.5),
        })
    });
    reg.register("supersaw", |p: &ParamMap| {
        external(SuperSaw {
            freq: param_or(p, "freq", 440.0),
            detune: param_or(p, "detune", 0.1),
            voices: param_or(p, "voices", 7.0) as usize,
        })
    });
    reg.register("white_noise", |_: &ParamMap| external(WhiteNoise));
    reg.register("pink_noise", |_: &ParamMap| external(PinkNoise));
    reg.register("brown_noise", |_: &ParamMap| external(BrownNoise));

    // ---- Filters -----------------------------------------------------------
    reg.register("svf", |p: &ParamMap| {
        external(SvfFilter {
            cutoff: param_or(p, "cutoff", 1000.0),
            resonance: param_or(p, "resonance", 0.5),
            mode: SvfMode::Lowpass,
        })
    });
    reg.register("ladder", |p: &ParamMap| {
        external(LadderFilter {
            cutoff: param_or(p, "cutoff", 1000.0),
            resonance: param_or(p, "resonance", 0.5),
            drive: param_or(p, "drive", 1.0),
        })
    });
    reg.register("one_pole", |p: &ParamMap| {
        external(OnePoleFilter {
            cutoff: param_or(p, "cutoff", 1000.0),
            highpass: param_or(p, "highpass", 0.0) > 0.5,
        })
    });
    reg.register("biquad", |p: &ParamMap| {
        external(BiquadFilter {
            b0: param_or(p, "b0", 1.0),
            b1: param_or(p, "b1", 0.0),
            b2: param_or(p, "b2", 0.0),
            a1: param_or(p, "a1", 0.0),
            a2: param_or(p, "a2", 0.0),
        })
    });
    reg.register("comb", |p: &ParamMap| {
        external(CombFilter {
            delay_ms: param_or(p, "delay_ms", 10.0),
            feedback: param_or(p, "feedback", 0.5),
            damp: param_or(p, "damp", 0.5),
        })
    });
    reg.register("allpass", |p: &ParamMap| {
        external(AllpassFilter {
            delay_samples: param_or(p, "delay_samples", 64.0) as usize,
            gain: param_or(p, "gain", 0.7),
        })
    });
    reg.register("formant", |p: &ParamMap| {
        external(FormantFilter {
            freq1: param_or(p, "freq1", 700.0),
            freq2: param_or(p, "freq2", 1200.0),
            bw1: param_or(p, "bw1", 80.0),
            bw2: param_or(p, "bw2", 90.0),
            gain1: param_or(p, "gain1", 1.0),
            gain2: param_or(p, "gain2", 1.0),
        })
    });
    reg.register("param_eq", |p: &ParamMap| {
        external(ParametricEq {
            freq: param_or(p, "freq", 1000.0),
            q: param_or(p, "q", 1.0),
            gain_db: param_or(p, "gain_db", 0.0),
        })
    });
    reg.register("resonant_drive", |p: &ParamMap| {
        external(ResonantDrive {
            drive: param_or(p, "drive", 1.0),
            mix: param_or(p, "mix", 0.5),
        })
    });

    // ---- Reverbs ---------------------------------------------------------
    // `convolution_reverb` is loaded from an IR file via `ConvolutionReverb::from_wav`
    // (see `auxide-dsp-m13`); it is not built from a `ParamMap` here.
    reg.register("simple_reverb", |p: &ParamMap| {
        external(SimpleReverb {
            decay: param_or(p, "decay", 0.5),
            mix: param_or(p, "mix", 0.3),
        })
    });
    reg.register("stereo_reverb", |p: &ParamMap| {
        external(StereoReverb {
            decay: param_or(p, "decay", 0.5),
            mix: param_or(p, "mix", 0.3),
            width: param_or(p, "width", 0.5),
        })
    });

    // ---- Envelopes ---------------------------------------------------------
    reg.register("adsr", |p: &ParamMap| {
        external(AdsrEnvelope {
            attack_ms: param_or(p, "attack_ms", 10.0),
            decay_ms: param_or(p, "decay_ms", 100.0),
            sustain_level: param_or(p, "sustain_level", 0.7),
            release_ms: param_or(p, "release_ms", 200.0),
            curve: param_or(p, "curve", 0.0),
        })
    });
    reg.register("ar", |p: &ParamMap| {
        external(ArEnvelope {
            attack_ms: param_or(p, "attack_ms", 10.0),
            release_ms: param_or(p, "release_ms", 200.0),
            curve: param_or(p, "curve", 0.0),
        })
    });
    reg.register("ad", |p: &ParamMap| {
        external(AdEnvelope {
            attack_ms: param_or(p, "attack_ms", 10.0),
            decay_ms: param_or(p, "decay_ms", 200.0),
            curve: param_or(p, "curve", 0.0),
        })
    });

    // ---- Time / modulation -------------------------------------------------
    reg.register("lfo", |p: &ParamMap| {
        external(Lfo {
            frequency: param_or(p, "frequency", 5.0),
            waveform: LfoWaveform::Sine,
            amplitude: param_or(p, "amplitude", 1.0),
            offset: param_or(p, "offset", 0.0),
        })
    });

    // ---- FX ----------------------------------------------------------------
    reg.register("delay", |p: &ParamMap| {
        external(Delay {
            delay_ms: param_or(p, "delay_ms", 250.0),
            feedback: param_or(p, "feedback", 0.4),
            mix: param_or(p, "mix", 0.5),
        })
    });
    reg.register("chorus", |p: &ParamMap| {
        external(Chorus {
            delay_ms: param_or(p, "delay_ms", 20.0),
            depth_ms: param_or(p, "depth_ms", 5.0),
            rate: param_or(p, "rate", 0.5),
            mix: param_or(p, "mix", 0.5),
        })
    });
    reg.register("flanger", |p: &ParamMap| {
        external(Flanger {
            delay_ms: param_or(p, "delay_ms", 5.0),
            depth_ms: param_or(p, "depth_ms", 2.0),
            rate: param_or(p, "rate", 0.3),
            feedback: param_or(p, "feedback", 0.3),
            mix: param_or(p, "mix", 0.5),
        })
    });
    reg.register("phaser", |p: &ParamMap| {
        external(Phaser {
            rate: param_or(p, "rate", 0.3),
            depth: param_or(p, "depth", 0.5),
            feedback: param_or(p, "feedback", 0.3),
            mix: param_or(p, "mix", 0.5),
        })
    });
    reg.register("tremolo", |p: &ParamMap| {
        external(Tremolo {
            rate: param_or(p, "rate", 5.0),
            depth: param_or(p, "depth", 0.5),
        })
    });

    // ---- Dynamics ----------------------------------------------------------
    reg.register("compressor", |p: &ParamMap| {
        external(Compressor {
            threshold: param_or(p, "threshold", 0.2),
            ratio: param_or(p, "ratio", 4.0),
            attack_ms: param_or(p, "attack_ms", 5.0),
            release_ms: param_or(p, "release_ms", 100.0),
            makeup_gain: param_or(p, "makeup_gain", 1.0),
        })
    });
    reg.register("limiter", |p: &ParamMap| {
        external(Limiter {
            threshold: param_or(p, "threshold", 0.9),
            attack_ms: param_or(p, "attack_ms", 1.0),
            release_ms: param_or(p, "release_ms", 50.0),
        })
    });
    reg.register("noise_gate", |p: &ParamMap| {
        external(NoiseGate {
            threshold: param_or(p, "threshold", 0.01),
            ratio: param_or(p, "ratio", 2.0),
            attack_ms: param_or(p, "attack_ms", 1.0),
            release_ms: param_or(p, "release_ms", 50.0),
        })
    });
    reg.register("expander", |p: &ParamMap| {
        external(Expander {
            threshold: param_or(p, "threshold", 0.1),
            ratio: param_or(p, "ratio", 2.0),
            attack_ms: param_or(p, "attack_ms", 1.0),
            release_ms: param_or(p, "release_ms", 50.0),
        })
    });

    // ---- Waveshapers -------------------------------------------------------
    reg.register("waveshaper", |p: &ParamMap| {
        external(WaveShaper {
            drive: param_or(p, "drive", 1.0),
            mix: param_or(p, "mix", 1.0),
        })
    });
    reg.register("hard_clip", |p: &ParamMap| {
        external(HardClip {
            threshold: param_or(p, "threshold", 0.7),
            mix: param_or(p, "mix", 1.0),
        })
    });
    reg.register("soft_clip", |p: &ParamMap| {
        external(SoftClip {
            drive: param_or(p, "drive", 1.0),
            mix: param_or(p, "mix", 1.0),
        })
    });
    reg.register("bitcrusher", |p: &ParamMap| {
        external(BitCrusher {
            bits: param_or(p, "bits", 8.0),
            mix: param_or(p, "mix", 1.0),
        })
    });
    reg.register("dc_blocker", |p: &ParamMap| {
        external(DcBlocker {
            cutoff: param_or(p, "cutoff", 20.0),
        })
    });
    reg.register("overdrive", |p: &ParamMap| {
        external(Overdrive {
            drive: param_or(p, "drive", 2.0),
            mix: param_or(p, "mix", 0.5),
        })
    });

    // ---- Utility -----------------------------------------------------------
    reg.register("ring_mod", |p: &ParamMap| {
        external(RingMod {
            mix: param_or(p, "mix", 1.0),
        })
    });
    reg.register("crossfader", |p: &ParamMap| {
        external(Crossfader {
            position: param_or(p, "position", 0.5),
        })
    });
    reg.register("stereo_panner", |p: &ParamMap| {
        external(StereoPanner {
            pan: param_or(p, "pan", 0.0),
        })
    });
    reg.register("mixer", |p: &ParamMap| {
        external(Mixer {
            inputs: param_or(p, "inputs", 2.0) as usize,
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use auxide::graph::NodeType;

    #[test]
    fn registers_a_representative_set_of_ugens() {
        let mut reg = Registry::new();
        register_dsp_ugens(&mut reg);
        // Oscillators, filters, envelopes, fx, dynamics, shapers, utility.
        for name in [
            "saw",
            "square",
            "triangle",
            "pulse",
            "supersaw",
            "white_noise",
            "pink_noise",
            "brown_noise",
            "svf",
            "ladder",
            "one_pole",
            "biquad",
            "comb",
            "allpass",
            "formant",
            "param_eq",
            "resonant_drive",
            "adsr",
            "ar",
            "ad",
            "lfo",
            "delay",
            "chorus",
            "flanger",
            "phaser",
            "tremolo",
            "compressor",
            "limiter",
            "noise_gate",
            "expander",
            "waveshaper",
            "hard_clip",
            "soft_clip",
            "bitcrusher",
            "dc_blocker",
            "overdrive",
            "ring_mod",
            "crossfader",
            "stereo_panner",
            "mixer",
        ] {
            assert!(reg.contains(name), "registry missing UGen: {name}");
        }
        // Params flow through: an svf built with a custom cutoff honours it.
        let mut p = ParamMap::new();
        p.insert("cutoff".to_string(), 320.0);
        match reg.create("svf", &p).unwrap() {
            NodeType::External { .. } => {}
            other => panic!("svf should be External, got {other:?}"),
        }
    }
}
