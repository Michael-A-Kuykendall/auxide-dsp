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

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn lfo_no_panic(frequency in 0.1..20.0f32, amplitude in 0.0..10.0f32, offset in -10.0..10.0f32) {
            let waveforms = [LfoWaveform::Sine, LfoWaveform::Triangle, LfoWaveform::Saw, LfoWaveform::Square, LfoWaveform::Random];
            for &waveform in &waveforms {
                let node = Lfo { frequency, waveform, amplitude, offset };
                let mut state = node.init_state(44100.0, 64);
                let mut out = vec![vec![0.0; 64]];
                node.process_block(&mut state, &[], &mut out, 44100.0);
                // Should not panic
            }
        }
    }
}