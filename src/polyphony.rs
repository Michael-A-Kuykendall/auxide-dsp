//! Polyphony and note-on/off handling for envelope-gated DSP voices.
//!
//! Wraps the control-message gating of [`crate::nodes::envelopes::AdsrEnvelope`]
//! into a playable [`Voice`] that responds to MIDI-style note-on/note-off, and a
//! [`Polyphony`] allocator that sums active voices. This is the "envelope gate
//! helper abstraction" that turns a single graph into a playable, polyphonic
//! instrument without rebuilding the audio graph per note.

use crate::nodes::envelopes::{AdsrEnvelope, AdsrPhase, AdsrState};
use auxide::node::NodeDef;

/// A single playable voice: one note bound to one envelope.
pub struct Voice {
    pub note: u8,
    pub velocity: f32,
    envelope: AdsrEnvelope,
    state: AdsrState,
    released: bool,
    active: bool,
}

impl Voice {
    /// Creates a voice bound to `note` using `envelope` as its shape. The voice
    /// starts silent and must be started with [`Voice::note_on`].
    pub fn new(note: u8, velocity: f32, envelope: AdsrEnvelope) -> Self {
        let state = envelope.init_state(44_100.0, 1);
        Voice {
            note,
            velocity,
            envelope,
            state,
            released: false,
            active: true,
        }
    }

    /// Begin (or retrigger) the note: gate on.
    pub fn note_on(&mut self) {
        self.released = false;
        self.active = true;
        self.envelope.gate(&mut self.state, true);
    }

    /// Release the note: gate off. The voice keeps sounding through its release
    /// tail until the envelope returns to [`AdsrPhase::Idle`].
    pub fn note_off(&mut self) {
        self.released = true;
        self.envelope.gate(&mut self.state, false);
    }

    /// True once the release tail has fully decayed to silence.
    pub fn is_finished(&self) -> bool {
        self.released && matches!(self.state.phase, AdsrPhase::Idle)
    }

    /// Advance the voice by one sample, returning its current output (`level * velocity`).
    pub fn tick(&mut self, sample_rate: f32) -> f32 {
        let mut out = vec![vec![0.0f32; 1]];
        self.envelope
            .process_block(&mut self.state, &[], &mut out, sample_rate);
        if self.is_finished() {
            self.active = false;
        }
        out[0][0] * self.velocity
    }
}

/// Fixed-size polyphony allocator that routes note-on/note-off to voices and
/// sums their outputs. When full, the oldest voice is stolen.
pub struct Polyphony {
    voices: Vec<Voice>,
    max_voices: usize,
}

impl Polyphony {
    /// Allocates space for up to `max_voices` simultaneous voices.
    pub fn new(max_voices: usize) -> Self {
        Polyphony {
            voices: Vec::with_capacity(max_voices.max(1)),
            max_voices: max_voices.max(1),
        }
    }

    /// Starts (or retriggers) `note`. If the note is already sounding it is
    /// retriggered in place; otherwise a new voice is allocated, stealing the
    /// oldest when the pool is full.
    pub fn note_on(&mut self, note: u8, velocity: f32, envelope: AdsrEnvelope) {
        if let Some(v) = self.voices.iter_mut().find(|v| v.note == note) {
            v.envelope = envelope;
            v.velocity = velocity;
            v.note_on();
            return;
        }
        if self.voices.len() >= self.max_voices {
            self.voices.remove(0);
        }
        let mut v = Voice::new(note, velocity, envelope);
        v.note_on();
        self.voices.push(v);
    }

    /// Releases `note` (gate off); the voice continues through its release tail.
    pub fn note_off(&mut self, note: u8) {
        if let Some(v) = self.voices.iter_mut().find(|v| v.note == note) {
            v.note_off();
        }
    }

    /// Advances every active voice by one sample and returns the summed output.
    /// Finished voices are retired from the pool.
    pub fn process(&mut self, sample_rate: f32) -> f32 {
        let mut sum = 0.0f32;
        for v in self.voices.iter_mut() {
            if v.active {
                sum += v.tick(sample_rate);
            }
        }
        self.voices.retain(|v| v.active);
        sum
    }

    /// Number of voices currently resident in the pool (including releasing ones).
    pub fn active_voice_count(&self) -> usize {
        self.voices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> AdsrEnvelope {
        AdsrEnvelope {
            attack_ms: 10.0,
            decay_ms: 20.0,
            sustain_level: 0.7,
            release_ms: 10.0,
            curve: 0.0,
        }
    }

    #[test]
    fn note_on_holds_sustain_then_note_off_retires() {
        let sr = 44_100.0;
        let mut p = Polyphony::new(4);
        p.note_on(60, 1.0, env());
        // advance past attack+decay into sustain
        for _ in 0..((40.0 / 1000.0 * sr) as usize) {
            p.process(sr);
        }
        let held = p.process(sr);
        assert!(
            held > 0.6 && held <= 0.71,
            "expected sustain ~0.7, got {held}"
        );
        p.note_off(60);
        // let the release tail fully elapse
        for _ in 0..((20.0 / 1000.0 * sr) as usize + 16) {
            p.process(sr);
        }
        assert_eq!(
            p.active_voice_count(),
            0,
            "voice should retire after release"
        );
    }

    #[test]
    fn polyphony_sums_simultaneous_voices() {
        let sr = 44_100.0;
        let mut p = Polyphony::new(4);
        p.note_on(60, 1.0, env());
        p.note_on(64, 1.0, env());
        for _ in 0..((40.0 / 1000.0 * sr) as usize) {
            p.process(sr);
        }
        let two = p.process(sr);
        assert!(two > 1.3, "two sustained voices should sum ~1.4, got {two}");
    }

    #[test]
    fn retrigger_same_note_reuses_voice() {
        let mut p = Polyphony::new(4);
        p.note_on(60, 1.0, env());
        p.note_on(60, 0.5, env());
        assert_eq!(
            p.active_voice_count(),
            1,
            "retrigger should not add a voice"
        );
    }
}
