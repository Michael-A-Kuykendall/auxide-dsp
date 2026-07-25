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

#[test]
fn envelope_gate_live_ar() {
    let node = ArEnvelope {
        attack_ms: 1.0,
        release_ms: 10.0,
        curve: 0.0,
    };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];

    // No audio gate connected — process with empty inputs
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert_eq!(out[0].iter().sum::<f32>(), 0.0); // Still idle

    // Trigger gate on via control message
    node.gate(&mut state, true);
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(out[0].iter().any(|&v| v > 0.0)); // Attack triggered — non-zero output
}

#[test]
fn envelope_gate_live_ar_off() {
    let node = ArEnvelope {
        attack_ms: 2.0,
        release_ms: 5.0,
        curve: 0.0,
    };
    let mut state = node.init_state(48_000.0, 64);

    // Gate on — process attack for one block
    node.gate(&mut state, true);
    let mut out1 = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out1, 48_000.0);
    assert!(out1[0].iter().any(|&v| v > 0.0)); // Attack produced non-zero

    // Gate off — process release
    node.gate(&mut state, false);
    let mut out2 = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out2, 48_000.0);
    assert!(out2[0].iter().any(|&v| v > 0.0)); // Release still has trailing output
    assert!(out1[0][out1[0].len() - 1] > out2[0][out2[0].len() - 1]); // Level decreased
}

#[test]
fn envelope_gate_live_ad() {
    let node = AdEnvelope {
        attack_ms: 5.0,
        decay_ms: 10.0,
        curve: 0.0,
    };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];

    // No audio gate — process with empty inputs
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert_eq!(out[0].iter().sum::<f32>(), 0.0); // Idle

    // Gate on — process attack for one block
    node.gate(&mut state, true);
    let mut out1 = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out1, 48_000.0);
    assert!(out1[0].iter().any(|&v| v > 0.0)); // Attack produced non-zero

    // Gate off — process decay
    node.gate(&mut state, false);
    let mut out2 = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out2, 48_000.0);
    assert!(out2[0].iter().any(|&v| v > 0.0)); // Decay still has trailing output
    assert!(out1[0][out1[0].len() - 1] > out2[0][out2[0].len() - 1]); // Level decreased
}

#[test]
fn envelope_gate_live_adsr() {
    let node = AdsrEnvelope {
        attack_ms: 1.0,
        decay_ms: 10.0,
        sustain_level: 0.5,
        release_ms: 10.0,
        curve: 0.0,
    };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];

    // No audio gate
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert_eq!(out[0].iter().sum::<f32>(), 0.0); // Idle

    // Gate on — attack
    node.gate(&mut state, true);
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(out[0].iter().any(|&v| v > 0.0)); // Attack triggered — non-zero output
}

#[test]
fn adsr_reaches_peak_then_sustain_then_decays() {
    let node = AdsrEnvelope {
        attack_ms: 10.0,
        decay_ms: 20.0,
        sustain_level: 0.5,
        release_ms: 30.0,
        curve: 0.0,
    };
    let sr = 44100.0;
    let mut state = node.init_state(sr, 64);
    let attack_blocks = ((10.0 / 1000.0) * sr / 64.0).ceil() as usize + 1;
    let decay_blocks = ((20.0 / 1000.0) * sr / 64.0).ceil() as usize + 1;

    let mut peak = 0.0f32;
    let mut sustain_val = 0.0f32;
    for b in 0..(attack_blocks + decay_blocks + 3) {
        let gate = vec![1.0; 64];
        let mut out = vec![vec![0.0; 64]];
        node.process_block(&mut state, &[&gate], &mut out, sr);
        for &v in &out[0] {
            peak = peak.max(v);
        }
        if b == attack_blocks + decay_blocks + 1 {
            sustain_val = out[0][0];
        }
    }
    assert!(
        (peak - 1.0).abs() < 0.05,
        "attack should reach ~1.0, got {peak}"
    );
    assert!(
        (sustain_val - 0.5).abs() < 0.1,
        "sustain should be ~0.5, got {sustain_val}"
    );

    let mut finalv = 1.0f32;
    for _ in 0..60 {
        let gate = vec![0.0; 64];
        let mut out = vec![vec![0.0; 64]];
        node.process_block(&mut state, &[&gate], &mut out, sr);
        finalv = out[0][0];
    }
    assert!(
        finalv.abs() < 0.05,
        "release should decay to ~0, got {finalv}"
    );
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
