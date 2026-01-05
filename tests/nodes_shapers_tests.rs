use auxide::node::NodeDef;
use auxide_dsp::{BitCrusher, DcBlocker, HardClip, Overdrive, SoftClip, TubeSaturation, WaveShaper};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn waveshaper_runs() {
    let node = WaveShaper {
        drive: 2.0,
        mix: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn hardclip_runs() {
    let node = HardClip {
        threshold: 0.5,
        mix: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn bitcrusher_runs() {
    let node = BitCrusher {
        bits: 4.0,
        mix: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn dc_blocker_runs() {
    let node = DcBlocker { cutoff: 10.0 };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    let input = (0..64).map(|i| (i as f32 / 64.0) * 2.0 - 1.0).collect::<Vec<f32>>();
    node.process_block(&mut state, &[&input], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn softclip_runs() {
    let node = SoftClip {
        drive: 2.0,
        mix: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn tube_saturation_runs() {
    let node = TubeSaturation {
        drive: 2.0,
        asymmetry: 0.1,
        mix: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn overdrive_runs() {
    let node = Overdrive {
        drive: 2.0,
        mix: 1.0,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}