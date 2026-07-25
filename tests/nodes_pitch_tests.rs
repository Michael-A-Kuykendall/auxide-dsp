use auxide::node::NodeDef;
use auxide_dsp::{dynamics::NoiseGate, PitchDetector, PitchShifter};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn pitch_shifter_runs() {
    let node = PitchShifter {
        shift: 2.0,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn noise_gate_runs() {
    let node = NoiseGate {
        threshold: 0.1,
        ratio: 10.0,
        attack_ms: 1.0,
        release_ms: 10.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn pitch_detector_runs() {
    let node = PitchDetector;
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    // Sine wave at 440 Hz
    let input = (0..64)
        .map(|i| (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin())
        .collect::<Vec<f32>>();
    node.process_block(&mut state, &[&input], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

/// Render a 440 Hz sine through the pitch shifter for `seconds`.
fn render_pitch(shift: f32, seconds: f32) -> Vec<f32> {
    let sr = 44100.0;
    let node = PitchShifter { shift, mix: 1.0 };
    let mut state = node.init_state(sr, 64);
    let total = (sr * seconds) as usize;
    let mut out = vec![0.0f32; total];
    let mut written = 0;
    while written < total {
        let take = 64usize.min(total - written);
        let mut inp = vec![0.0f32; take];
        for (k, slot) in inp.iter_mut().enumerate() {
            let idx = written + k;
            *slot = (2.0 * std::f32::consts::PI * 440.0 * idx as f32 / sr).sin();
        }
        let mut block_out = vec![vec![0.0f32; take]];
        node.process_block(&mut state, &[&inp], &mut block_out, sr);
        out[written..written + take].copy_from_slice(&block_out[0]);
        written += take;
    }
    out
}

/// Dominant (non-DC) frequency via a realfft magnitude spectrum.
fn dominant_freq(samples: &[f32], sr: f32) -> f32 {
    use realfft::RealFftPlanner;
    let n = samples.len();
    let mut planner = RealFftPlanner::<f32>::new();
    let r2c = planner.plan_fft_forward(n);
    let mut indata = samples.to_vec();
    let mut out = r2c.make_output_vec();
    r2c.process(&mut indata, &mut out).unwrap();
    let mut best_bin = 1usize;
    let mut best = 0.0f32;
    for (b, v) in out.iter().enumerate().take(n / 2).skip(1) {
        let m = v.norm_sqr();
        if m > best {
            best = m;
            best_bin = b;
        }
    }
    best_bin as f32 * sr / n as f32
}

#[test]
fn pitch_shift_up() {
    let out = render_pitch(12.0, 0.5);
    let f = dominant_freq(&out, 44100.0);
    assert!(
        (f - 880.0).abs() / 880.0 < 0.05,
        "shift +12 should ~880 Hz, got {f}"
    );
}

#[test]
fn pitch_shift_down() {
    let out = render_pitch(-12.0, 0.5);
    let f = dominant_freq(&out, 44100.0);
    assert!(
        (f - 220.0).abs() / 220.0 < 0.05,
        "shift -12 should ~220 Hz, got {f}"
    );
}

#[test]
fn pitch_shift_finite() {
    let out = render_pitch(7.0, 0.5);
    assert!(out.iter().all(|&s| s.is_finite()), "output must be finite");
    let peak_in = 1.0f32; // sine input peak
    let peak_out = out.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        peak_out < 2.0 * peak_in,
        "peak gain must be < 2x, got {peak_out}"
    );
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn pitch_shifter_no_panic(shift in 0.5..2.0f32, mix in 0.0..1.0f32) {
            let node = PitchShifter { shift, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn noise_gate_no_panic(threshold in 0.0..1.0f32, ratio in 1.0..20.0f32) {
            let node = NoiseGate { threshold, ratio, attack_ms: 1.0, release_ms: 10.0 };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }
    }
}
