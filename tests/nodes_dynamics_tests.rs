use auxide::node::NodeDef;
use auxide_dsp::{Compressor, Expander, Limiter, NoiseGate};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn compressor_runs() {
    let node = Compressor {
        threshold: 0.5,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 100.0,
        makeup_gain: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn limiter_runs() {
    let node = Limiter {
        threshold: 0.8,
        attack_ms: 1.0,
        release_ms: 10.0,
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
fn expander_runs() {
    let node = Expander {
        threshold: 0.2,
        ratio: 2.0,
        attack_ms: 10.0,
        release_ms: 100.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[0.1; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}