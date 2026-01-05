use auxide::node::NodeDef;
use auxide_dsp::nodes::utility::{Crossfader, MidSideProcessor, ParamSmoother, RingMod, RMSMeter, StereoPanner, StereoWidth};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn ring_mod_runs() {
    let node = RingMod { mix: 0.5 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    let input = [1.0; 64];
    let mod_signal = [0.5; 64];
    node.process_block(&mut state, &[&input, &mod_signal], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn crossfader_runs() {
    let node = Crossfader { position: 0.5 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    let a = [1.0; 64];
    let b = [0.0; 64];
    node.process_block(&mut state, &[&a, &b], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn stereo_width_runs() {
    let node = StereoWidth { width: 0.5 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64], vec![0.0; 64]];
    let l = [1.0; 64];
    let r = [0.5; 64];
    node.process_block(&mut state, &[&l, &r], &mut out, 44100.0);
    assert!(non_silent(&out[0]) || non_silent(&out[1]));
}

#[test]
fn param_smoother_runs() {
    let node = ParamSmoother { smoothing: 0.1 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    let input = [1.0; 64];
    node.process_block(&mut state, &[&input], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn mid_side_processor_runs() {
    let node = MidSideProcessor;
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64], vec![0.0; 64]];
    let l = [1.0; 64];
    let r = [0.5; 64];
    node.process_block(&mut state, &[&l, &r], &mut out, 44100.0);
    assert!(non_silent(&out[0]) || non_silent(&out[1]));
}

#[test]
fn stereo_panner_runs() {
    let node = StereoPanner { pan: 0.5 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64], vec![0.0; 64]];
    let input = [1.0; 64];
    node.process_block(&mut state, &[&input], &mut out, 44100.0);
    assert!(non_silent(&out[0]) && non_silent(&out[1]));
}

#[test]
fn rms_meter_runs() {
    let node = RMSMeter { window_size: 64 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    let input = [1.0; 64];
    node.process_block(&mut state, &[&input], &mut out, 44100.0);
    assert!(out[0][0] > 0.0); // RMS should be positive
}