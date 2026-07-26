use crate::helpers;
use auxide::graph::{Port, PortId, Rate};
use auxide::node::NodeDef;

/// State of a varispeed pitch shifter.
#[derive(Debug, Clone)]
pub struct PitchShifterState {
    pub ring: Vec<f32>,
    pub write_idx: usize,
    pub read_pos: f32,
}

/// Pitch shifter — **basic varispeed stub** (scoped as a basic stub per
/// `auxide-dsp-6jy`; NOT a phase-vocoder / time-preserving shifter).
///
/// The input is written into a ring buffer at the input rate and read back
/// through a fractional read pointer that advances by
/// `ratio = 2^(shift/12)` samples per input sample. `ratio > 1`
/// (positive shift) reads slower -> higher pitch and LONGER output;
/// `ratio < 1` reads faster -> lower pitch and SHORTER output.
///
/// # Documented limits
/// - Transposes pitch AND stretches time (it is NOT time-preserving). A
///   one-octave-up shift makes the signal twice as long.
/// - No formant preservation and no cross-fading: pitch jumps at the ring
///   wrap boundary produce audible discontinuities/artifacts.
/// - Ring length is fixed at ~50 ms of history; very low pitches or long
///   correlations exceed the ring and alias.
/// - For a proper, artifact-free, time-preserving shift, replace this with a
///   phase-vocoder or granular algorithm.
#[derive(Debug, Clone)]
pub struct PitchShifter {
    pub shift: f32, // semitones
    pub mix: f32,   // 0 = dry, 1 = wet
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
        // Ring sized for up to ~50 ms of history.
        let len = (sample_rate * 0.05).max(64.0) as usize;
        PitchShifterState {
            ring: vec![0.0; len],
            write_idx: 0,
            read_pos: 0.0,
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
        let n = input.len();
        let len = state.ring.len();

        for i in 0..n {
            let shift = self.shift
                + if shift_mod.is_empty() {
                    0.0
                } else {
                    shift_mod[i]
                };
            let mix =
                (self.mix + if mix_mod.is_empty() { 0.0 } else { mix_mod[i] }).clamp(0.0, 1.0);

            let ratio = 2.0_f32.powf(shift / 12.0);

            // Write current input into the ring at the write head.
            state.ring[state.write_idx] = input[i];

            // Read the (fractional) varispeed position.
            let wet = helpers::linear_interpolate(&state.ring, state.read_pos);

            output[i] = input[i] * (1.0 - mix) + wet * mix;

            // Advance the read pointer by the pitch ratio (wraps the ring).
            state.read_pos += ratio;
            while state.read_pos >= len as f32 {
                state.read_pos -= len as f32;
            }
            state.write_idx = (state.write_idx + 1) % len;
        }
        let _ = sample_rate;
    }
}

// Note: a time-domain NoiseGate (envelope-following gate) already lives in
// `crate::dynamics`. The previously misnamed `SpectralGate` here was a duplicate
// of it and has been removed; use `crate::dynamics::NoiseGate` instead.

/// State of a Pitch Detector
#[derive(Debug, Clone)]
pub struct PitchDetectorState {
    pub prev_sample: f32,
    pub period: f32,
}

/// Pitch Detector — **crude** zero-crossing estimator (scoped as crude per
/// `auxide-dsp-1ps`; NOT autocorrelation/FFT).
///
/// # Documented limits
/// - Estimates fundamental frequency by measuring the period between
///   consecutive **positive-going** zero crossings. This yields a full-cycle
///   period (a half-cycle trigger would double the reported frequency).
/// - Assumes a **monophonic, single-tone** input with one zero crossing per
///   period. Polyphonic or harmonically rich signals, DC offset, or noise
///   near the zero axis will produce incorrect readings.
/// - Output is held constant between crossings; it is not a smoothed or
///   interpolated pitch estimate. Accuracy degrades for very low frequencies
///   (few samples per period) and for frequencies above `sample_rate / 2`.
/// - For robust polyphonic/low-latency detection, replace this with an
///   autocorrelation or FFT-based estimator.
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
            // Trigger only on the positive-going zero crossing so the measured
            // period is a full cycle (not a half-cycle, which would double the
            // reported frequency). This is the "crude" scope of the bead: a
            // zero-crossing detector, correct for simple tones.
            if state.prev_sample <= 0.0 && input[i] > 0.0 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_unity_shift_is_passthrough() {
        // shift = 0 => ratio = 1 => wet read equals input; mix = 1 => output == input.
        let node = PitchShifter {
            shift: 0.0,
            mix: 1.0,
        };
        let sr = 44100.0;
        let input: Vec<f32> = (0..2048)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr).sin())
            .collect();
        let mut state = node.init_state(sr, 2048);
        let mut out = vec![vec![0.0; 2048]];
        node.process_block(&mut state, &[&input], &mut out, sr);
        for (a, b) in input.iter().zip(out[0].iter()) {
            assert!((a - b).abs() < 1e-5, "shift=0/mix=1 must pass through");
        }
    }

    #[test]
    fn stub_dry_is_passthrough_at_any_shift() {
        // mix = 0 => only dry signal reaches output.
        let node = PitchShifter {
            shift: 12.0,
            mix: 0.0,
        };
        let sr = 44100.0;
        let input: Vec<f32> = (0..2048)
            .map(|i| (2.0 * std::f32::consts::PI * 220.0 * i as f32 / sr).sin())
            .collect();
        let mut state = node.init_state(sr, 2048);
        let mut out = vec![vec![0.0; 2048]];
        node.process_block(&mut state, &[&input], &mut out, sr);
        for (a, b) in input.iter().zip(out[0].iter()) {
            assert!((a - b).abs() < 1e-5, "mix=0 must pass through dry");
        }
    }

    #[test]
    fn stub_positive_shift_transposes_signal() {
        // A positive shift must alter (raise) the pitch, i.e. the output must
        // differ from a dry passthrough.
        let dry = PitchShifter {
            shift: 0.0,
            mix: 1.0,
        };
        let wet = PitchShifter {
            shift: 12.0,
            mix: 1.0,
        };
        let sr = 44100.0;
        let input: Vec<f32> = (0..8192)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr).sin())
            .collect();
        let mut sd = dry.init_state(sr, 8192);
        let mut sw = wet.init_state(sr, 8192);
        let mut out_dry = vec![vec![0.0; 8192]];
        let mut out_wet = vec![vec![0.0; 8192]];
        dry.process_block(&mut sd, &[&input], &mut out_dry, sr);
        wet.process_block(&mut sw, &[&input], &mut out_wet, sr);
        let diff: f32 = out_dry[0]
            .iter()
            .zip(out_wet[0].iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 1.0, "octave-up shift must transpose the signal");
    }
}
