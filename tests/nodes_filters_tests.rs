use auxide::node::NodeDef;
use auxide_dsp::{AllpassFilter, BiquadFilter, CombFilter, FormantFilter, LadderFilter, SvfFilter, SvfMode};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn svf_lowpass_runs() {
    let node = SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Lowpass,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn svf_highpass_runs() {
    let node = SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Highpass,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn svf_bandpass_runs() {
    let node = SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Bandpass,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn svf_notch_runs() {
    let node = SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Notch,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn ladder_runs() {
    let node = LadderFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        drive: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn comb_runs() {
    let node = CombFilter {
        delay_ms: 10.0,
        feedback: 0.5,
        damp: 0.1,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn formant_runs() {
    let node = FormantFilter {
        freq1: 700.0,
        freq2: 1200.0,
        bw1: 100.0,
        bw2: 100.0,
        gain1: 1.0,
        gain2: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn biquad_runs() {
    // Simple lowpass biquad coefficients
    let node = BiquadFilter {
        b0: 0.1,
        b1: 0.2,
        b2: 0.1,
        a1: -0.5,
        a2: 0.25,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn allpass_runs() {
    let node = AllpassFilter {
        delay_samples: 10,
        gain: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn svf_filter_no_panic(cutoff in 20.0..20000.0f32, resonance in 0.0..1.0f32) {
            let modes = [SvfMode::Lowpass, SvfMode::Highpass, SvfMode::Bandpass, SvfMode::Notch];
            for &mode in &modes {
                let node = SvfFilter { cutoff, resonance, mode };
                let mut state = node.init_state(44100.0, 64);
                let mut out = vec![vec![0.0; 64]];
                node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
                // Should not panic
            }
        }

        #[test]
        fn ladder_filter_no_panic(cutoff in 20.0..20000.0f32, resonance in 0.0..1.0f32, drive in 0.1..10.0f32) {
            let node = LadderFilter { cutoff, resonance, drive };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn comb_filter_no_panic(delay_ms in 1.0..100.0f32, feedback in 0.0..0.99f32, damp in 0.0..1.0f32) {
            let node = CombFilter { delay_ms, feedback, damp };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn formant_filter_no_panic(freq1 in 100.0..5000.0f32, freq2 in 100.0..5000.0f32, bw1 in 10.0..1000.0f32, bw2 in 10.0..1000.0f32, gain1 in 0.0..2.0f32, gain2 in 0.0..2.0f32) {
            let node = FormantFilter { freq1, freq2, bw1, bw2, gain1, gain2 };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn biquad_filter_no_panic(b0 in -10.0..10.0f32, b1 in -10.0..10.0f32, b2 in -10.0..10.0f32, a1 in -10.0..10.0f32, a2 in -10.0..10.0f32) {
            let node = BiquadFilter { b0, b1, b2, a1, a2 };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn allpass_filter_no_panic(delay_samples in 1..1000usize, gain in 0.0..0.99f32) {
            let node = AllpassFilter { delay_samples, gain };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }
    }
}