use auxide::node::NodeDef;
use auxide_dsp::{PitchDetector, PitchShifter, SpectralGate};

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
fn spectral_gate_runs() {
    let node = SpectralGate {
        threshold: 0.1,
        ratio: 10.0,
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
    let input = (0..64).map(|i| (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin()).collect::<Vec<f32>>();
    node.process_block(&mut state, &[&input], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
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
        fn spectral_gate_no_panic(threshold in 0.0..1.0f32, ratio in 1.0..20.0f32) {
            let node = SpectralGate { threshold, ratio };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }
    }
}