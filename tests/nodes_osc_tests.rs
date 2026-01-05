#![forbid(unsafe_code)]

use std::sync::Arc;

use auxide::node::NodeDef;
use auxide_dsp::nodes::oscillators::*;
use auxide_dsp::{generate_sine_table};

fn non_silent(buf: &[f32]) -> bool {
    buf.iter().any(|&v| v.abs() > 1e-6)
}

#[test]
fn saw_runs() {
    let node = SawOsc::new(440.0);
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
    // Golden check: Saw wave should start near 0 and rise
    assert!(out[0][0].abs() < 0.1); // Near 0 at phase 0
    assert!(out[0][16] > 0.0); // Positive in first quadrant
    assert!(out[0][32] > 0.5); // High in second quadrant
}

#[test]
fn square_runs() {
    let node = SquareOsc { freq: 220.0, pulse_width: 0.5 };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn wavetable_runs() {
    let table = Arc::new(generate_sine_table(64));
    let node = WavetableOsc { freq: 110.0, table };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn supersaw_runs() {
    let node = SuperSaw { freq: 110.0, detune: 0.1, voices: 4 };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn noise_runs() {
    let mut out = vec![vec![0.0; 64]];

    let white = WhiteNoise;
    let mut st = white.init_state(48_000.0, 64);
    white.process_block(&mut st, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));

    let pink = PinkNoise;
    let mut st = pink.init_state(48_000.0, 64);
    pink.process_block(&mut st, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));

    let brown = BrownNoise;
    let mut st = brown.init_state(48_000.0, 64);
    brown.process_block(&mut st, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn constant_runs() {
    let node = Constant { value: 0.5 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 44100.0);
    assert!(out[0].iter().all(|&v| (v - 0.5).abs() < 1e-6));
}
