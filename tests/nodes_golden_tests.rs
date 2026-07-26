//! Golden-reference tests for filters and envelopes (auxide-dsp-xlw).
//!
//! Each test compares the node output against a reference computed directly from
//! the published DSP formula, so a regression in the math (not just a panic)
//! is caught.

use auxide::node::NodeDef;
use auxide_dsp::{AdsrEnvelope, OnePoleFilter};

#[test]
fn one_pole_lowpass_impulse_golden() {
    // Canonical one-pole LP: y[n] = (1-g)*x[n] + g*y[n-1].
    // Feed a unit impulse and compare every sample to the closed form
    // y[n] = (1-g) * g^n.
    let fs = 44100.0;
    let cutoff = 1000.0;
    let g = (-2.0 * std::f32::consts::PI * cutoff / fs)
        .exp()
        .clamp(0.0, 0.999_999);
    let a = 1.0 - g;

    let node = OnePoleFilter {
        cutoff,
        highpass: false,
    };
    let mut state = node.init_state(fs, 64);
    let input: Vec<f32> = (0..64).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&input], &mut out, fs);

    let mut expected = 0.0;
    for (i, &y) in out[0].iter().enumerate() {
        if i == 0 {
            expected = a;
        } else {
            expected *= g; // (1-g)*0 + g*y[n-1]
        }
        assert!(
            (y - expected).abs() < 1e-4,
            "one-pole impulse sample {i}: got {y}, expected {expected}"
        );
    }
}

#[test]
fn adsr_linear_golden_attack_decay_release() {
    // Linear ADSR, fs = 1000 Hz, A = D = R = 10 ms, sustain = 0.5.
    // Hand-computed reference below. Decay must fall 1.0 -> 0.5.
    let fs = 1000.0;
    let node = AdsrEnvelope {
        attack_ms: 10.0,
        decay_ms: 10.0,
        sustain_level: 0.5,
        release_ms: 10.0,
        curve: 0.0,
    };
    let mut state = node.init_state(fs, 64);

    // Gate on -> Attack -> Decay (internally transitions to Sustain at sample 19).
    node.gate(&mut state, true);
    let mut out1 = vec![0.0; 20];
    node.process_block(&mut state, &[], std::slice::from_mut(&mut out1), fs);

    // Attack: level = (i+1)*0.1 for i in 0..10 (peak 1.0 at sample 9).
    // Decay (linear): level = 1.0 + (S-1)*t = 1.0 - 0.5*t, t=(j+1)*0.1.
    let expected_attack: [f32; 10] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let expected_decay: [f32; 10] = [0.95, 0.90, 0.85, 0.80, 0.75, 0.70, 0.65, 0.60, 0.55, 0.50];
    for i in 0..10 {
        assert!(
            (out1[i] - expected_attack[i]).abs() < 1e-4,
            "attack sample {i}: got {}, expected {}",
            out1[i],
            expected_attack[i]
        );
    }
    for j in 0..10 {
        assert!(
            (out1[10 + j] - expected_decay[j]).abs() < 1e-4,
            "decay sample {j}: got {}, expected {}",
            out1[10 + j],
            expected_decay[j]
        );
    }

    // Gate off -> Release: level = S * (1 - t), t=(j+1)*0.1, ending at 0.0.
    node.gate(&mut state, false);
    let mut out2 = vec![0.0; 10];
    node.process_block(&mut state, &[], std::slice::from_mut(&mut out2), fs);
    let expected_release: [f32; 10] = [0.45, 0.40, 0.35, 0.30, 0.25, 0.20, 0.15, 0.10, 0.05, 0.00];
    for (j, &y) in out2.iter().enumerate() {
        assert!(
            (y - expected_release[j]).abs() < 1e-4,
            "release sample {j}: got {y}, expected {}",
            expected_release[j]
        );
    }
}
