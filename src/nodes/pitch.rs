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

/// Varispeed pitch shifter (transposition + time-stretch).
///
/// The input is written into a ring buffer at the input rate and read back
/// through a fractional read pointer that advances by
/// `ratio = 2^(shift/12)` samples per input sample. `ratio > 1`
/// (positive shift) reads slower -> higher pitch and longer output;
/// `ratio < 1` reads faster -> lower pitch and shorter output. This
/// transposes pitch AND stretches time (it is NOT a time-preserving
/// pitch shifter). The wet signal is mixed with the dry input by
/// `mix` (0 = dry, 1 = wet).
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
