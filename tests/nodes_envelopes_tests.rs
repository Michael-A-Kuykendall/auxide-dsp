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