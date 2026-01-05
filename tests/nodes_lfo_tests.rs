use auxide::node::NodeDef;
use auxide_dsp::{Lfo, LfoWaveform};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn lfo_sine_runs() {
    let node = Lfo {
        frequency: 1.0,
        waveform: LfoWaveform::Sine,
        amplitude: 1.0,
        offset: 0.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn lfo_triangle_runs() {
    let node = Lfo {
        frequency: 1.0,
        waveform: LfoWaveform::Triangle,
        amplitude: 1.0,
        offset: 0.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn lfo_saw_runs() {
    let node = Lfo {
        frequency: 1.0,
        waveform: LfoWaveform::Saw,
        amplitude: 1.0,
        offset: 0.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn lfo_square_runs() {
    let node = Lfo {
        frequency: 1.0,
        waveform: LfoWaveform::Square,
        amplitude: 1.0,
        offset: 0.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn lfo_random_runs() {
    let node = Lfo {
        frequency: 1.0,
        waveform: LfoWaveform::Random,
        amplitude: 1.0,
        offset: 0.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}