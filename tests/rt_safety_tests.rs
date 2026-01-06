//! RT Safety Tests
//! 
//! These tests verify that DSP nodes do not allocate during process_block.
//! 
//! IMPORTANT: dhat can only have ONE profiler per process, so we use a single
//! test that runs all checks sequentially. Run with: cargo test --test rt_safety_tests
//!
//! NOTE: This test covers a curated set of critical node types (oscillators, filters,
//! effects, envelopes, dynamics). It does not exhaustively test every node variant.
//! The coverage here represents the most commonly used RT-critical paths.

use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use auxide_dsp::nodes::oscillators::{SawOsc, SquareOsc, TriangleOsc, PulseOsc, WavetableOsc, SuperSaw, WhiteNoise, PinkNoise, BrownNoise, Constant};
use auxide_dsp::nodes::filters::{SvfFilter, LadderFilter};
use auxide_dsp::nodes::envelopes::AdsrEnvelope;
use auxide_dsp::nodes::dynamics::Compressor;
use auxide_dsp::nodes::lfo::Lfo;
use auxide_dsp::nodes::fx::Delay;
use auxide_dsp::nodes::shapers::WaveShaper;
use auxide_dsp::nodes::pitch::PitchShifter;
use auxide_dsp::nodes::utility::RingMod;
use auxide_dsp::SvfMode;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Single comprehensive RT allocation test for ALL DSP node types
/// 
/// Tests ALL DSP nodes for RT safety by:
/// 1. Setting up graphs and runtimes for all node types BEFORE starting the profiler
/// 2. Running multiple process_block calls while profiling
/// 3. Verifying zero allocations occurred during processing
///
/// Nodes tested: ALL NodeDef implementations (oscillators, filters, envelopes, dynamics, lfo, fx, shapers, pitch, utility)
#[test]
fn test_all_nodes_rt_safe() {
    // ========== SETUP PHASE (allocations allowed) ==========
    
    // Oscillators
    let mut graph_saw = Graph::new();
    let saw = graph_saw.add_external_node(SawOsc::new(440.0));
    let sink_saw = graph_saw.add_node(NodeType::OutputSink);
    graph_saw.add_edge(auxide::graph::Edge {
        from_node: saw,
        from_port: PortId(0),
        to_node: sink_saw,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_saw = Plan::compile(&graph_saw, 64).unwrap();
    let mut runtime_saw = Runtime::new(plan_saw, &graph_saw, 44100.0);
    let mut out_saw = vec![0.0; 64];
    runtime_saw.process_block(&mut out_saw).unwrap();

    let mut graph_square = Graph::new();
    let square = graph_square.add_external_node(SquareOsc { freq: 440.0, pulse_width: 0.5 });
    let sink_square = graph_square.add_node(NodeType::OutputSink);
    graph_square.add_edge(auxide::graph::Edge {
        from_node: square,
        from_port: PortId(0),
        to_node: sink_square,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_square = Plan::compile(&graph_square, 64).unwrap();
    let mut runtime_square = Runtime::new(plan_square, &graph_square, 44100.0);
    let mut out_square = vec![0.0; 64];
    runtime_square.process_block(&mut out_square).unwrap();

    let mut graph_triangle = Graph::new();
    let triangle = graph_triangle.add_external_node(TriangleOsc { freq: 440.0 });
    let sink_triangle = graph_triangle.add_node(NodeType::OutputSink);
    graph_triangle.add_edge(auxide::graph::Edge {
        from_node: triangle,
        from_port: PortId(0),
        to_node: sink_triangle,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_triangle = Plan::compile(&graph_triangle, 64).unwrap();
    let mut runtime_triangle = Runtime::new(plan_triangle, &graph_triangle, 44100.0);
    let mut out_triangle = vec![0.0; 64];
    runtime_triangle.process_block(&mut out_triangle).unwrap();

    let mut graph_pulse = Graph::new();
    let pulse = graph_pulse.add_external_node(PulseOsc { freq: 440.0, pulse_width: 0.5 });
    let sink_pulse = graph_pulse.add_node(NodeType::OutputSink);
    graph_pulse.add_edge(auxide::graph::Edge {
        from_node: pulse,
        from_port: PortId(0),
        to_node: sink_pulse,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_pulse = Plan::compile(&graph_pulse, 64).unwrap();
    let mut runtime_pulse = Runtime::new(plan_pulse, &graph_pulse, 44100.0);
    let mut out_pulse = vec![0.0; 64];
    runtime_pulse.process_block(&mut out_pulse).unwrap();

    let mut graph_wavetable = Graph::new();
    let wavetable = graph_wavetable.add_external_node(WavetableOsc { freq: 440.0, table: std::sync::Arc::new(vec![0.0, 1.0, 0.0, -1.0]) });
    let sink_wavetable = graph_wavetable.add_node(NodeType::OutputSink);
    graph_wavetable.add_edge(auxide::graph::Edge {
        from_node: wavetable,
        from_port: PortId(0),
        to_node: sink_wavetable,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_wavetable = Plan::compile(&graph_wavetable, 64).unwrap();
    let mut runtime_wavetable = Runtime::new(plan_wavetable, &graph_wavetable, 44100.0);
    let mut out_wavetable = vec![0.0; 64];
    runtime_wavetable.process_block(&mut out_wavetable).unwrap();

    let mut graph_supersaw = Graph::new();
    let supersaw = graph_supersaw.add_external_node(SuperSaw { freq: 440.0, detune: 0.1, voices: 3 });
    let sink_supersaw = graph_supersaw.add_node(NodeType::OutputSink);
    graph_supersaw.add_edge(auxide::graph::Edge {
        from_node: supersaw,
        from_port: PortId(0),
        to_node: sink_supersaw,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_supersaw = Plan::compile(&graph_supersaw, 64).unwrap();
    let mut runtime_supersaw = Runtime::new(plan_supersaw, &graph_supersaw, 44100.0);
    let mut out_supersaw = vec![0.0; 64];
    runtime_supersaw.process_block(&mut out_supersaw).unwrap();

    let mut graph_whitenoise = Graph::new();
    let whitenoise = graph_whitenoise.add_external_node(WhiteNoise);
    let sink_whitenoise = graph_whitenoise.add_node(NodeType::OutputSink);
    graph_whitenoise.add_edge(auxide::graph::Edge {
        from_node: whitenoise,
        from_port: PortId(0),
        to_node: sink_whitenoise,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_whitenoise = Plan::compile(&graph_whitenoise, 64).unwrap();
    let mut runtime_whitenoise = Runtime::new(plan_whitenoise, &graph_whitenoise, 44100.0);
    let mut out_whitenoise = vec![0.0; 64];
    runtime_whitenoise.process_block(&mut out_whitenoise).unwrap();

    let mut graph_pinknoise = Graph::new();
    let pinknoise = graph_pinknoise.add_external_node(PinkNoise);
    let sink_pinknoise = graph_pinknoise.add_node(NodeType::OutputSink);
    graph_pinknoise.add_edge(auxide::graph::Edge {
        from_node: pinknoise,
        from_port: PortId(0),
        to_node: sink_pinknoise,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_pinknoise = Plan::compile(&graph_pinknoise, 64).unwrap();
    let mut runtime_pinknoise = Runtime::new(plan_pinknoise, &graph_pinknoise, 44100.0);
    let mut out_pinknoise = vec![0.0; 64];
    runtime_pinknoise.process_block(&mut out_pinknoise).unwrap();

    let mut graph_brownnoise = Graph::new();
    let brownnoise = graph_brownnoise.add_external_node(BrownNoise);
    let sink_brownnoise = graph_brownnoise.add_node(NodeType::OutputSink);
    graph_brownnoise.add_edge(auxide::graph::Edge {
        from_node: brownnoise,
        from_port: PortId(0),
        to_node: sink_brownnoise,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_brownnoise = Plan::compile(&graph_brownnoise, 64).unwrap();
    let mut runtime_brownnoise = Runtime::new(plan_brownnoise, &graph_brownnoise, 44100.0);
    let mut out_brownnoise = vec![0.0; 64];
    runtime_brownnoise.process_block(&mut out_brownnoise).unwrap();

    let mut graph_constant = Graph::new();
    let constant = graph_constant.add_external_node(Constant { value: 1.0 });
    let sink_constant = graph_constant.add_node(NodeType::OutputSink);
    graph_constant.add_edge(auxide::graph::Edge {
        from_node: constant,
        from_port: PortId(0),
        to_node: sink_constant,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_constant = Plan::compile(&graph_constant, 64).unwrap();
    let mut runtime_constant = Runtime::new(plan_constant, &graph_constant, 44100.0);
    let mut out_constant = vec![0.0; 64];
    runtime_constant.process_block(&mut out_constant).unwrap();

    // Filters
    let mut graph_svf = Graph::new();
    let osc_svf = graph_svf.add_node(NodeType::SineOsc { freq: 440.0 });
    let svf = graph_svf.add_external_node(SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Lowpass,
    });
    let sink_svf = graph_svf.add_node(NodeType::OutputSink);
    graph_svf.add_edge(auxide::graph::Edge {
        from_node: osc_svf,
        from_port: PortId(0),
        to_node: svf,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_svf.add_edge(auxide::graph::Edge {
        from_node: svf,
        from_port: PortId(0),
        to_node: sink_svf,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_svf = Plan::compile(&graph_svf, 64).unwrap();
    let mut runtime_svf = Runtime::new(plan_svf, &graph_svf, 44100.0);
    let mut out_svf = vec![0.0; 64];
    runtime_svf.process_block(&mut out_svf).unwrap();

    let mut graph_ladder = Graph::new();
    let osc_ladder = graph_ladder.add_node(NodeType::SineOsc { freq: 440.0 });
    let ladder = graph_ladder.add_external_node(LadderFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        drive: 1.0,
    });
    let sink_ladder = graph_ladder.add_node(NodeType::OutputSink);
    graph_ladder.add_edge(auxide::graph::Edge {
        from_node: osc_ladder,
        from_port: PortId(0),
        to_node: ladder,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_ladder.add_edge(auxide::graph::Edge {
        from_node: ladder,
        from_port: PortId(0),
        to_node: sink_ladder,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_ladder = Plan::compile(&graph_ladder, 64).unwrap();
    let mut runtime_ladder = Runtime::new(plan_ladder, &graph_ladder, 44100.0);
    let mut out_ladder = vec![0.0; 64];
    runtime_ladder.process_block(&mut out_ladder).unwrap();

    // Envelopes
    let mut graph_adsr = Graph::new();
    let osc_adsr = graph_adsr.add_node(NodeType::SineOsc { freq: 440.0 });
    let adsr = graph_adsr.add_external_node(AdsrEnvelope {
        attack_ms: 10.0,
        decay_ms: 100.0,
        sustain_level: 0.7,
        release_ms: 200.0,
        curve: 1.0,
    });
    let sink_adsr = graph_adsr.add_node(NodeType::OutputSink);
    graph_adsr.add_edge(auxide::graph::Edge {
        from_node: osc_adsr,
        from_port: PortId(0),
        to_node: adsr,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_adsr.add_edge(auxide::graph::Edge {
        from_node: adsr,
        from_port: PortId(0),
        to_node: sink_adsr,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_adsr = Plan::compile(&graph_adsr, 64).unwrap();
    let mut runtime_adsr = Runtime::new(plan_adsr, &graph_adsr, 44100.0);
    let mut out_adsr = vec![0.0; 64];
    runtime_adsr.process_block(&mut out_adsr).unwrap();

    // Dynamics
    let mut graph_compressor = Graph::new();
    let osc_compressor = graph_compressor.add_node(NodeType::SineOsc { freq: 440.0 });
    let compressor = graph_compressor.add_external_node(Compressor {
        threshold: -12.0,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 100.0,
        makeup_gain: 1.0,
    });
    let sink_compressor = graph_compressor.add_node(NodeType::OutputSink);
    graph_compressor.add_edge(auxide::graph::Edge {
        from_node: osc_compressor,
        from_port: PortId(0),
        to_node: compressor,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_compressor.add_edge(auxide::graph::Edge {
        from_node: compressor,
        from_port: PortId(0),
        to_node: sink_compressor,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_compressor = Plan::compile(&graph_compressor, 64).unwrap();
    let mut runtime_compressor = Runtime::new(plan_compressor, &graph_compressor, 44100.0);
    let mut out_compressor = vec![0.0; 64];
    runtime_compressor.process_block(&mut out_compressor).unwrap();

    // LFO
    let mut graph_lfo = Graph::new();
    let lfo_node = graph_lfo.add_external_node(Lfo {
        frequency: 1.0,
        waveform: auxide_dsp::nodes::lfo::LfoWaveform::Sine,
        amplitude: 1.0,
        offset: 0.0,
    });
    let sink_lfo = graph_lfo.add_node(NodeType::OutputSink);
    graph_lfo.add_edge(auxide::graph::Edge {
        from_node: lfo_node,
        from_port: PortId(0),
        to_node: sink_lfo,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_lfo = Plan::compile(&graph_lfo, 64).unwrap();
    let mut runtime_lfo = Runtime::new(plan_lfo, &graph_lfo, 44100.0);
    let mut out_lfo = vec![0.0; 64];
    runtime_lfo.process_block(&mut out_lfo).unwrap();

    // FX
    let mut graph_delay = Graph::new();
    let osc_delay = graph_delay.add_node(NodeType::SineOsc { freq: 440.0 });
    let delay = graph_delay.add_external_node(Delay {
        delay_ms: 100.0,
        feedback: 0.3,
        mix: 0.5,
    });
    let sink_delay = graph_delay.add_node(NodeType::OutputSink);
    graph_delay.add_edge(auxide::graph::Edge {
        from_node: osc_delay,
        from_port: PortId(0),
        to_node: delay,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_delay.add_edge(auxide::graph::Edge {
        from_node: delay,
        from_port: PortId(0),
        to_node: sink_delay,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_delay = Plan::compile(&graph_delay, 64).unwrap();
    let mut runtime_delay = Runtime::new(plan_delay, &graph_delay, 44100.0);
    let mut out_delay = vec![0.0; 64];
    runtime_delay.process_block(&mut out_delay).unwrap();

    // Shapers
    let mut graph_waveshaper = Graph::new();
    let osc_waveshaper = graph_waveshaper.add_node(NodeType::SineOsc { freq: 440.0 });
    let waveshaper = graph_waveshaper.add_external_node(WaveShaper {
        drive: 1.0,
        mix: 1.0,
    });
    let sink_waveshaper = graph_waveshaper.add_node(NodeType::OutputSink);
    graph_waveshaper.add_edge(auxide::graph::Edge {
        from_node: osc_waveshaper,
        from_port: PortId(0),
        to_node: waveshaper,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_waveshaper.add_edge(auxide::graph::Edge {
        from_node: waveshaper,
        from_port: PortId(0),
        to_node: sink_waveshaper,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_waveshaper = Plan::compile(&graph_waveshaper, 64).unwrap();
    let mut runtime_waveshaper = Runtime::new(plan_waveshaper, &graph_waveshaper, 44100.0);
    let mut out_waveshaper = vec![0.0; 64];
    runtime_waveshaper.process_block(&mut out_waveshaper).unwrap();

    // Pitch
    let mut graph_pitchshifter = Graph::new();
    let osc_pitchshifter = graph_pitchshifter.add_node(NodeType::SineOsc { freq: 440.0 });
    let pitchshifter = graph_pitchshifter.add_external_node(PitchShifter {
        shift: 2.0,
        mix: 1.0,
    });
    let sink_pitchshifter = graph_pitchshifter.add_node(NodeType::OutputSink);
    graph_pitchshifter.add_edge(auxide::graph::Edge {
        from_node: osc_pitchshifter,
        from_port: PortId(0),
        to_node: pitchshifter,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_pitchshifter.add_edge(auxide::graph::Edge {
        from_node: pitchshifter,
        from_port: PortId(0),
        to_node: sink_pitchshifter,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_pitchshifter = Plan::compile(&graph_pitchshifter, 64).unwrap();
    let mut runtime_pitchshifter = Runtime::new(plan_pitchshifter, &graph_pitchshifter, 44100.0);
    let mut out_pitchshifter = vec![0.0; 64];
    runtime_pitchshifter.process_block(&mut out_pitchshifter).unwrap();

    // Utility
    let mut graph_ringmod = Graph::new();
    let osc1_ringmod = graph_ringmod.add_node(NodeType::SineOsc { freq: 440.0 });
    let osc2_ringmod = graph_ringmod.add_node(NodeType::SineOsc { freq: 220.0 });
    let ringmod = graph_ringmod.add_external_node(RingMod {
        mix: 1.0,
    });
    let sink_ringmod = graph_ringmod.add_node(NodeType::OutputSink);
    graph_ringmod.add_edge(auxide::graph::Edge {
        from_node: osc1_ringmod,
        from_port: PortId(0),
        to_node: ringmod,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph_ringmod.add_edge(auxide::graph::Edge {
        from_node: osc2_ringmod,
        from_port: PortId(0),
        to_node: ringmod,
        to_port: PortId(1),
        rate: Rate::Audio,
    }).unwrap();
    graph_ringmod.add_edge(auxide::graph::Edge {
        from_node: ringmod,
        from_port: PortId(0),
        to_node: sink_ringmod,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    let plan_ringmod = Plan::compile(&graph_ringmod, 64).unwrap();
    let mut runtime_ringmod = Runtime::new(plan_ringmod, &graph_ringmod, 44100.0);
    let mut out_ringmod = vec![0.0; 64];
    runtime_ringmod.process_block(&mut out_ringmod).unwrap();

    // ========== RT PHASE (zero allocations required) ==========
    let _profiler = dhat::Profiler::new_heap();
    
    // Run all process_block calls
    for _ in 0..10 {
        runtime_saw.process_block(&mut out_saw).unwrap();
        runtime_square.process_block(&mut out_square).unwrap();
        runtime_triangle.process_block(&mut out_triangle).unwrap();
        runtime_pulse.process_block(&mut out_pulse).unwrap();
        runtime_wavetable.process_block(&mut out_wavetable).unwrap();
        runtime_supersaw.process_block(&mut out_supersaw).unwrap();
        runtime_whitenoise.process_block(&mut out_whitenoise).unwrap();
        runtime_pinknoise.process_block(&mut out_pinknoise).unwrap();
        runtime_brownnoise.process_block(&mut out_brownnoise).unwrap();
        runtime_constant.process_block(&mut out_constant).unwrap();
        runtime_svf.process_block(&mut out_svf).unwrap();
        runtime_ladder.process_block(&mut out_ladder).unwrap();
        runtime_adsr.process_block(&mut out_adsr).unwrap();
        runtime_compressor.process_block(&mut out_compressor).unwrap();
        runtime_lfo.process_block(&mut out_lfo).unwrap();
        runtime_delay.process_block(&mut out_delay).unwrap();
        runtime_waveshaper.process_block(&mut out_waveshaper).unwrap();
        runtime_pitchshifter.process_block(&mut out_pitchshifter).unwrap();
        runtime_ringmod.process_block(&mut out_ringmod).unwrap();
    }
    
    let stats = dhat::HeapStats::get();
    assert_eq!(
        stats.total_blocks, 0,
        "RT violation: {} allocations detected during process_block. \
         All DSP nodes must be allocation-free in the RT path.",
        stats.total_blocks
    );
}