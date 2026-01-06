use auxide::node::NodeDef;
use auxide_dsp::{AdEnvelope, AdsrEnvelope, ArEnvelope};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn adsr_runs() {
    let node = AdsrEnvelope {
        attack_ms: 10.0,
        decay_ms: 20.0,
        sustain_level: 0.5,
        release_ms: 30.0,
        curve: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    // Gate on for first half, off for second
    let mut gate = vec![1.0; 32];
    gate.extend(vec![0.0; 32]);
    node.process_block(&mut state, &[&gate], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn ar_runs() {
    let node = ArEnvelope {
        attack_ms: 10.0,
        release_ms: 20.0,
        curve: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    let mut gate = vec![1.0; 32];
    gate.extend(vec![0.0; 32]);
    node.process_block(&mut state, &[&gate], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn ad_runs() {
    let node = AdEnvelope {
        attack_ms: 10.0,
        decay_ms: 20.0,
        curve: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    let mut gate = vec![1.0; 32];
    gate.extend(vec![0.0; 32]);
    node.process_block(&mut state, &[&gate], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn adsr_envelope_no_panic(attack_ms in 0.1..1000.0f32, decay_ms in 0.1..1000.0f32, sustain_level in 0.0..1.0f32, release_ms in 0.1..1000.0f32, curve in 0.1..10.0f32) {
            let node = AdsrEnvelope { attack_ms, decay_ms, sustain_level, release_ms, curve };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            let gate = vec![1.0; 64]; // Full gate
            node.process_block(&mut state, &[&gate], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn ar_envelope_no_panic(attack_ms in 0.1..1000.0f32, release_ms in 0.1..1000.0f32, curve in 0.1..10.0f32) {
            let node = ArEnvelope { attack_ms, release_ms, curve };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            let gate = vec![1.0; 64]; // Full gate
            node.process_block(&mut state, &[&gate], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn ad_envelope_no_panic(attack_ms in 0.1..1000.0f32, decay_ms in 0.1..1000.0f32, curve in 0.1..10.0f32) {
            let node = AdEnvelope { attack_ms, decay_ms, curve };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            let gate = vec![1.0; 64]; // Full gate
            node.process_block(&mut state, &[&gate], &mut out, 44100.0);
            // Should not panic
        }
    }
}