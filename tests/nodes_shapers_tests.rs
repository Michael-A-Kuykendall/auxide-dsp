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

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn waveshaper_no_panic(drive in 0.1..10.0f32, mix in 0.0..1.0f32) {
            let node = WaveShaper { drive, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn hardclip_no_panic(threshold in 0.1..1.0f32, mix in 0.0..1.0f32) {
            let node = HardClip { threshold, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn bitcrusher_no_panic(bits in 1.0..16.0f32, mix in 0.0..1.0f32) {
            let node = BitCrusher { bits, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn dc_blocker_no_panic(cutoff in 1.0..1000.0f32) {
            let node = DcBlocker { cutoff };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            let input = (0..64).map(|i| (i as f32 / 64.0) * 2.0 - 1.0).collect::<Vec<f32>>();
            node.process_block(&mut state, &[&input], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn softclip_no_panic(drive in 0.1..10.0f32, mix in 0.0..1.0f32) {
            let node = SoftClip { drive, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn tube_saturation_no_panic(drive in 0.1..10.0f32, asymmetry in 0.0..1.0f32, mix in 0.0..1.0f32) {
            let node = TubeSaturation { drive, asymmetry, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn overdrive_no_panic(drive in 0.1..10.0f32, mix in 0.0..1.0f32) {
            let node = Overdrive { drive, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }
    }
}