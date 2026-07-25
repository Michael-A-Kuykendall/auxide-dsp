use std::sync::Arc;

use auxide::control::{PARAM_CUTOFF, PARAM_DETUNE, PARAM_FREQUENCY, PARAM_RESONANCE};
use auxide::node::NodeDef;
use auxide_dsp::generate_sine_table;
use auxide_dsp::nodes::oscillators::*;

use auxide_dsp::nodes::filters::LadderFilter;
use auxide_dsp::nodes::filters::SvfFilter;
use auxide_dsp::nodes::filters::SvfMode;

fn non_silent(buf: &[f32]) -> bool {
    buf.iter().any(|&v| v.abs() > 1e-6)
}

fn rms(buf: &[f32]) -> f32 {
    let sum: f32 = buf.iter().map(|&v| v * v).sum();
    (sum / buf.len() as f32).sqrt()
}

#[test]
fn param_surface_roundtrip() {
    // SawOsc: set_param PARAM_FREQUENCY changes pitch
    let node = SawOsc::new(440.0);
    let mut state = node.init_state(48_000.0, 64);
    let mut out_low = vec![vec![0.0; 64]];
    let mut out_high = vec![vec![0.0; 64]];

    node.set_param(&mut state, PARAM_FREQUENCY, 110.0);
    node.process_block(&mut state, &[], &mut out_low, 48_000.0);
    let rms_low = rms(&out_low[0]);

    node.set_param(&mut state, PARAM_FREQUENCY, 880.0);
    node.process_block(&mut state, &[], &mut out_high, 48_000.0);
    let rms_high = rms(&out_high[0]);

    // Higher freq should produce a different (higher energy distribution) output
    assert_ne!(rms_low, rms_high);
}

#[test]
fn param_surface_square_osc() {
    let node = SquareOsc {
        freq: 220.0,
        pulse_width: 0.5,
    };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.set_param(&mut state, PARAM_FREQUENCY, 880.0);
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn param_surface_triangle_osc() {
    let node = TriangleOsc { freq: 220.0 };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.set_param(&mut state, PARAM_FREQUENCY, 440.0);
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn param_surface_pulse_osc() {
    let node = PulseOsc {
        freq: 220.0,
        pulse_width: 0.5,
    };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.set_param(&mut state, PARAM_FREQUENCY, 660.0);
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn param_surface_wavetable_osc() {
    let table = Arc::new(generate_sine_table(64));
    let node = WavetableOsc {
        freq: 110.0,
        table: table.clone(),
    };
    let mut state = node.init_state(48_000.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.set_param(&mut state, PARAM_FREQUENCY, 880.0);
    node.process_block(&mut state, &[], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn param_surface_supersaw() {
    let node = SuperSaw {
        freq: 110.0,
        detune: 0.1,
        voices: 4,
    };
    let mut state = node.init_state(48_000.0, 64);

    let mut out1 = vec![vec![0.0; 64]];
    let mut out2 = vec![vec![0.0; 64]];

    node.set_param(&mut state, PARAM_FREQUENCY, 55.0);
    node.process_block(&mut state, &[], &mut out1, 48_000.0);
    let rms1 = rms(&out1[0]);

    node.set_param(&mut state, PARAM_FREQUENCY, 440.0);
    node.process_block(&mut state, &[], &mut out2, 48_000.0);
    let rms2 = rms(&out2[0]);

    assert_ne!(rms1, rms2);

    node.set_param(&mut state, PARAM_DETUNE, 0.5);
    let mut out3 = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out3, 48_000.0);
    assert_ne!(rms(&out3[0]), rms2);
}

#[test]
fn param_surface_svf_filter() {
    let node = SvfFilter {
        cutoff: 100.0,
        resonance: 0.1,
        mode: SvfMode::Lowpass,
    };
    let mut state = node.init_state(48_000.0, 64);

    let mut out_low = vec![vec![0.0; 64]];
    let mut out_high = vec![vec![0.0; 64]];
    let input = vec![1.0; 64];

    node.set_param(&mut state, PARAM_CUTOFF, 100.0);
    node.process_block(&mut state, &[&input], &mut out_low, 48_000.0);
    let rms_low = rms(&out_low[0]);

    node.set_param(&mut state, PARAM_CUTOFF, 10000.0);
    node.process_block(&mut state, &[&input], &mut out_high, 48_000.0);
    let rms_high = rms(&out_high[0]);

    // Higher cutoff should pass more energy (lowpass)
    assert!(rms_high > rms_low);
}

#[test]
fn param_surface_ladder_filter() {
    let node = LadderFilter {
        cutoff: 1000.0,
        resonance: 0.3,
        drive: 1.0,
    };
    let mut state = node.init_state(48_000.0, 64);

    let mut out = vec![vec![0.0; 64]];
    let input: Vec<f32> = (0..64)
        .map(|i| (2.0 * std::f32::consts::PI * 2000.0 * i as f32 / 48_000.0).sin())
        .collect();

    node.set_param(&mut state, PARAM_CUTOFF, 2000.0);
    node.process_block(&mut state, &[&input], &mut out, 48_000.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn param_surface_ladder_cutoff_changes_output() {
    let node = LadderFilter {
        cutoff: 1000.0,
        resonance: 0.0,
        drive: 1.0,
    };
    let mut state = node.init_state(48_000.0, 64);

    let mut out_low = vec![vec![0.0; 64]];
    let mut out_high = vec![vec![0.0; 64]];
    let input: Vec<f32> = (0..64)
        .map(|i| (2.0 * std::f32::consts::PI * 2000.0 * i as f32 / 48_000.0).sin())
        .collect();

    node.set_param(&mut state, PARAM_CUTOFF, 2000.0);
    // Run a priming block so filter state stabilizes
    node.process_block(&mut state, &[&input], &mut out_low, 48_000.0);
    let rms_low = rms(&out_low[0]);

    // Reset state to ensure clean comparison
    node.set_param(&mut state, PARAM_CUTOFF, 15000.0);
    node.process_block(&mut state, &[&input], &mut out_high, 48_000.0);
    let rms_high = rms(&out_high[0]);

    assert!(rms_low != rms_high);
}

#[test]
fn param_surface_ladder_resonance() {
    let node = LadderFilter {
        cutoff: 1000.0,
        resonance: 0.0,
        drive: 1.0,
    };
    let mut state = node.init_state(48_000.0, 64);

    let mut out_low = vec![vec![0.0; 64]];
    let mut out_high = vec![vec![0.0; 64]];
    // Use a sine input so resonance is audible
    let input: Vec<f32> = (0..64)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin())
        .collect();

    node.set_param(&mut state, PARAM_RESONANCE, 0.0);
    node.process_block(&mut state, &[&input], &mut out_low, 48_000.0);
    let rms_low = rms(&out_low[0]);

    node.set_param(&mut state, PARAM_RESONANCE, 0.8);
    node.process_block(&mut state, &[&input], &mut out_high, 48_000.0);
    let rms_high = rms(&out_high[0]);

    assert_ne!(rms_low, rms_high);
}
