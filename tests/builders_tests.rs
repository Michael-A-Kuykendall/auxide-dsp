use auxide_dsp::builders::{EffectsChainBuilder, SynthBuilder};
use auxide_dsp::nodes::oscillators::SawOsc;

#[test]
fn synth_builder_runs() {
    let builder = SynthBuilder::new().add_oscillator(SawOsc { freq: 440.0 });
    let _graph = builder.build_graph();
    // Just check it builds without error
}

#[test]
fn synth_builder_build_succeeds() {
    let builder = SynthBuilder::new().add_oscillator(SawOsc { freq: 440.0 });
    let result = builder.build(64);
    assert!(result.is_ok());
}

#[test]
fn effects_chain_builder_runs() {
    let builder = EffectsChainBuilder::new().add_input().add_output();
    let _graph = builder.build_graph();
    // Just check it builds without error
}

#[test]
fn effects_chain_builder_connects_nodes() {
    // The builder now wires edges automatically, so a chain with an input and
    // output compiles and actually contains an edge between them.
    let (graph, _plan) = EffectsChainBuilder::new()
        .add_input()
        .add_output()
        .build(64)
        .expect("effects chain should build");
    assert!(
        !graph.edges.is_empty(),
        "effects chain must contain at least one edge after build"
    );
}
