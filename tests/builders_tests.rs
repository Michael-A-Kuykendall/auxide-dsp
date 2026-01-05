use auxide_dsp::builders::{EffectsChainBuilder, SynthBuilder};
use auxide_dsp::nodes::oscillators::SawOsc;

#[test]
fn synth_builder_runs() {
    let builder = SynthBuilder::new()
        .add_oscillator(SawOsc { freq: 440.0 });
    let _graph = builder.build_graph();
    // Just check it builds without error
}

#[test]
fn synth_builder_build_succeeds() {
    let builder = SynthBuilder::new()
        .add_oscillator(SawOsc { freq: 440.0 });
    let result = builder.build(64);
    assert!(result.is_ok());
}

#[test]
fn effects_chain_builder_runs() {
    let builder = EffectsChainBuilder::new()
        .add_input()
        .add_output();
    let _graph = builder.build_graph();
    // Just check it builds without error
}

#[test]
fn effects_chain_builder_build_fails_without_connections() {
    let builder = EffectsChainBuilder::new()
        .add_input()
        .add_output();
    let result = builder.build(64);
    assert!(result.is_err()); // Fails because no edges
}