use auxide::node::NodeDef;
use auxide_dsp::{Chorus, ConvolutionReverb, Delay, Flanger, MultitapDelay, Phaser, SimpleReverb, Tremolo};

fn non_silent(output: &[f32]) -> bool {
    output.iter().any(|&x| x.abs() > 1e-6)
}

#[test]
fn delay_runs() {
    let node = Delay {
        delay_ms: 50.0,
        feedback: 0.5,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn chorus_runs() {
    let node = Chorus {
        delay_ms: 10.0,
        depth_ms: 5.0,
        rate: 0.5,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn flanger_runs() {
    let node = Flanger {
        delay_ms: 1.0,
        depth_ms: 1.0,
        rate: 0.5,
        feedback: 0.5,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn phaser_runs() {
    let node = Phaser {
        rate: 0.5,
        depth: 1.0,
        feedback: 0.5,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn simple_reverb_runs() {
    let node = SimpleReverb {
        decay: 0.5,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn multitap_delay_runs() {
    let node = MultitapDelay {
        taps: vec![(50.0, 0.5), (100.0, 0.3), (150.0, 0.2)],
        feedback: 0.2,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    assert!(non_silent(&out[0]));
}

#[test]
fn convolution_reverb_runs() {
    // Simple short IR
    let ir: Vec<f32> = (0..32).map(|i| (1.0 - i as f32 / 32.0).powf(2.0)).collect();
    let node = ConvolutionReverb {
        ir,
        mix: 0.5,
    };
    let mut state = node.init_state(44100.0, 64);
    let mut out = vec![vec![0.0; 64]];
    // Process multiple blocks to get convolution output
    for _ in 0..4 {
        node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
    }
    assert!(non_silent(&out[0]));
}

// TODO: Add RT allocation test using dhat or similar tool
// The ConvolutionReverb has been fixed to preallocate scratch buffers in init_state
// to ensure RT-safety. Manual testing with dhat shows no allocations in process_block.

#[test]
fn tremolo_runs() {
    let node = Tremolo {
        rate: 5.0,
        depth: 0.5,
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
        fn delay_no_panic(delay_ms in 1.0..1000.0f32, feedback in 0.0..0.99f32, mix in 0.0..1.0f32) {
            let node = Delay { delay_ms, feedback, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn chorus_no_panic(delay_ms in 1.0..50.0f32, depth_ms in 0.1..10.0f32, rate in 0.1..10.0f32, mix in 0.0..1.0f32) {
            let node = Chorus { delay_ms, depth_ms, rate, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn flanger_no_panic(delay_ms in 0.1..10.0f32, depth_ms in 0.1..5.0f32, rate in 0.1..10.0f32, feedback in 0.0..0.99f32, mix in 0.0..1.0f32) {
            let node = Flanger { delay_ms, depth_ms, rate, feedback, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn phaser_no_panic(rate in 0.1..10.0f32, depth in 0.1..1.0f32, feedback in 0.0..0.99f32, mix in 0.0..1.0f32) {
            let node = Phaser { rate, depth, feedback, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn simple_reverb_no_panic(decay in 0.0..0.99f32, mix in 0.0..1.0f32) {
            let node = SimpleReverb { decay, mix };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }

        #[test]
        fn tremolo_no_panic(rate in 0.1..20.0f32, depth in 0.0..1.0f32) {
            let node = Tremolo { rate, depth };
            let mut state = node.init_state(44100.0, 64);
            let mut out = vec![vec![0.0; 64]];
            node.process_block(&mut state, &[&[1.0; 64]], &mut out, 44100.0);
            // Should not panic
        }
    }
}