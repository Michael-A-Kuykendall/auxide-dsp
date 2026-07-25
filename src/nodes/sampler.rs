#![forbid(unsafe_code)]

use std::sync::Arc;

use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

use crate::helpers::linear_interpolate;

/// State of a Sampler voice.
#[derive(Debug, Clone)]
pub struct SamplerState {
    /// Current read position (in source samples, fractional).
    pub pos: f32,
    /// Whether the voice is currently playing.
    pub playing: bool,
    /// Current playback rate multiplier (file_sr/stream_sr * pitch ratio).
    pub rate: f32,
    /// Anchor frequency (Hz) the sample was recorded at; pitch is relative to this.
    pub anchor_freq: f32,
    /// Base rate = file_sr / stream_sr (plays sample at original pitch).
    pub base_rate: f32,
}

/// Sample player / ROMpler voice.
///
/// Plays a recorded buffer (`sample`) at a pitch selected by `PARAM_FREQUENCY`
/// (or `set_param`), looping or one-shot. Triggering is via `gate` (i.e. the
/// kernel `TriggerGate` ControlMsg). Amplitude/ADSR should be applied by an
/// external `AdsrEnvelope` (or `Gain`) node downstream — this node only
/// produces the pitched sample.
#[derive(Debug, Clone)]
pub struct Sampler {
    /// The recorded source. Shared; cloned cheaply.
    pub sample: Arc<Vec<f32>>,
    /// Sample rate the buffer was recorded at (Hz). 0 = assume stream rate.
    pub file_sample_rate: f32,
    /// MIDI note the sample represents (pitch reference).
    pub anchor_note: u8,
    /// Loop the sample while playing (sustained sounds) vs. one-shot (percussive).
    pub loop_mode: bool,
}

impl Sampler {
    /// Create a sampler from an in-memory sample buffer.
    ///
    /// * `sample` - mono PCM samples.
    /// * `file_sample_rate` - the rate `sample` was captured at (use 0 to mean
    ///   "same as the stream").
    /// * `anchor_note` - MIDI note the recording corresponds to (pitch center).
    /// * `loop_mode` - loop while held (true) or play once (false).
    pub fn new(
        sample: Arc<Vec<f32>>,
        file_sample_rate: f32,
        anchor_note: u8,
        loop_mode: bool,
    ) -> Self {
        Self {
            sample,
            file_sample_rate,
            anchor_note,
            loop_mode,
        }
    }
}

impl NodeDef for Sampler {
    type State = SamplerState;

    fn input_ports(&self) -> &'static [Port] {
        const PORTS: &[Port] = &[];
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

    fn init_state(&self, sample_rate: f32, _block_size: usize) -> Self::State {
        let base_rate = if self.file_sample_rate > 0.0 {
            self.file_sample_rate / sample_rate
        } else {
            1.0
        };
        let anchor_freq = 440.0 * 2.0f32.powf((self.anchor_note as f32 - 69.0) / 12.0);
        SamplerState {
            pos: 0.0,
            playing: false,
            rate: base_rate,
            anchor_freq,
            base_rate,
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
        let len = self.sample.len();
        if len == 0 {
            for s in out.iter_mut() {
                *s = 0.0;
            }
            return;
        }
        for s in out.iter_mut() {
            if state.playing {
                *s = linear_interpolate(&self.sample, state.pos);
                state.pos += state.rate;
                if state.pos >= len as f32 {
                    if self.loop_mode {
                        state.pos -= len as f32;
                        // Guard against pathological tiny samples.
                        if state.pos >= len as f32 {
                            state.pos = 0.0;
                        }
                    } else {
                        state.pos = 0.0;
                        state.playing = false;
                    }
                }
            } else {
                *s = 0.0;
            }
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        use auxide::control::PARAM_FREQUENCY;
        if param == PARAM_FREQUENCY && state.anchor_freq > 0.0 && value > 0.0 {
            // Pitch ratio relative to the recorded anchor, scaled by the
            // file/stream sample-rate correction.
            state.rate = state.base_rate * (value / state.anchor_freq);
        }
    }

    fn gate(&self, state: &mut Self::State, on: bool) {
        if on {
            state.playing = true;
            state.pos = 0.0;
        } else {
            state.playing = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auxide::control::ControlMsg;
    use auxide::graph::{Graph, NodeType, PortId, Rate};
    use auxide::plan::Plan;
    use auxide::rt::{RuntimeControl, RuntimeCore, RuntimeHandle};

    /// Build a mono sine sample at `freq` Hz lasting `dur_s` seconds.
    fn make_sample(freq: f32, dur_s: f32, sr: f32) -> Arc<Vec<f32>> {
        let n = (dur_s * sr) as usize;
        let mut v = Vec::with_capacity(n);
        for i in 0..n {
            v.push((2.0 * std::f32::consts::PI * freq * (i as f32) / sr).sin());
        }
        Arc::new(v)
    }

    fn build_runtime(
        sample: Arc<Vec<f32>>,
        anchor: u8,
        loop_mode: bool,
    ) -> (RuntimeHandle, RuntimeControl, auxide::graph::NodeId) {
        let mut graph = Graph::new();
        let node = graph.add_external_node(Sampler::new(sample, 44100.0, anchor, loop_mode));
        let sink = graph.add_node(NodeType::OutputSink);
        graph
            .add_edge(auxide::graph::Edge {
                from_node: node,
                from_port: PortId(0),
                to_node: sink,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
        let plan = Plan::compile(&graph, 64).unwrap();
        let (handle, control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);
        (handle, control, node)
    }

    /// Zero-crossing frequency estimate for a signed signal.
    fn zc_freq(out: &[f32], sr: f32) -> f32 {
        let mut zc = 0u32;
        for w in out.windows(2) {
            if (w[0] <= 0.0 && w[1] > 0.0) || (w[0] >= 0.0 && w[1] < 0.0) {
                zc += 1;
            }
        }
        (zc as f32) / 2.0 / (out.len() as f32 / sr)
    }

    #[test]
    fn sampler_plays_at_anchor_pitch() {
        let (mut handle, mut control, node) =
            build_runtime(make_sample(440.0, 0.5, 44100.0), 69, false);
        control
            .send(ControlMsg::SetFrequency { node, hz: 440.0 })
            .unwrap();
        control
            .send(ControlMsg::TriggerGate { node, on: true })
            .unwrap();
        let mut out = vec![0.0; 64];
        let mut all = Vec::new();
        for _ in 0..40 {
            handle.process_block(&mut out).unwrap();
            all.extend_from_slice(&out);
        }
        assert!(
            all.iter().any(|&x| x.abs() > 0.1),
            "sampler should produce audio"
        );
        let f = zc_freq(&all, 44100.0);
        assert!((f - 440.0).abs() < 40.0, "freq was {f}");
    }

    #[test]
    fn sampler_pitch_shifts_up() {
        let (mut handle, mut control, node) =
            build_runtime(make_sample(440.0, 0.5, 44100.0), 69, false);
        control
            .send(ControlMsg::SetFrequency { node, hz: 880.0 })
            .unwrap();
        control
            .send(ControlMsg::TriggerGate { node, on: true })
            .unwrap();
        let mut out = vec![0.0; 64];
        let mut all = Vec::new();
        for _ in 0..40 {
            handle.process_block(&mut out).unwrap();
            all.extend_from_slice(&out);
        }
        let f = zc_freq(&all, 44100.0);
        assert!((f - 880.0).abs() < 60.0, "freq was {f}");
    }

    #[test]
    fn sampler_loop_sustains() {
        let (mut handle, mut control, node) =
            build_runtime(make_sample(220.0, 0.05, 44100.0), 57, true);
        control
            .send(ControlMsg::SetFrequency { node, hz: 220.0 })
            .unwrap();
        control
            .send(ControlMsg::TriggerGate { node, on: true })
            .unwrap();
        let mut out = vec![0.0; 64];
        // Run far longer than the 0.05 s sample -> looping must keep it alive.
        for _ in 0..200 {
            handle.process_block(&mut out).unwrap();
        }
        assert!(
            out.iter().any(|x| x.abs() > 0.05),
            "looped sampler should still sound"
        );
    }

    #[test]
    fn sampler_one_shot_stops() {
        let (mut handle, mut control, node) =
            build_runtime(make_sample(220.0, 0.02, 44100.0), 57, false);
        control
            .send(ControlMsg::SetFrequency { node, hz: 220.0 })
            .unwrap();
        control
            .send(ControlMsg::TriggerGate { node, on: true })
            .unwrap();
        let mut out = vec![0.0; 64];
        // 0.02 s @ 44.1k = 882 samples = ~14 blocks. Run 60 blocks.
        for _ in 0..60 {
            handle.process_block(&mut out).unwrap();
        }
        assert!(
            out.iter().all(|x| x.abs() < 1e-4),
            "one-shot should have fallen silent"
        );
    }
}
