use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use auxide_dsp::nodes::oscillators::SawOsc;
use auxide_dsp::nodes::filters::SvfFilter;
use auxide_dsp::nodes::fx::Delay;
use auxide_dsp::nodes::envelopes::AdsrEnvelope;
use auxide_dsp::nodes::dynamics::Compressor;
use auxide_dsp::SvfMode;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Test that SawOsc doesn't allocate in process_block
#[test]
fn test_saw_osc_no_rt_alloc() {
    let mut graph = Graph::new();
    let osc = graph.add_external_node(SawOsc::new(440.0));
    let sink = graph.add_node(NodeType::OutputSink);

    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();

    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    // Process several blocks to ensure we're past initialization
    let mut out = vec![0.0; 64];

    // Start profiling only for process_block calls
    let _profiler = dhat::Profiler::new_heap();
    for _ in 0..10 {
        runtime.process_block(&mut out).unwrap();
    }

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 0, "SawOsc should not allocate in process_block");
    assert_eq!(stats.total_bytes, 0, "SawOsc should not allocate in process_block");
}

/// Test that SVF Lowpass doesn't allocate in process_block
#[test]
fn test_svf_lowpass_no_rt_alloc() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let filter = graph.add_external_node(SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Lowpass,
    });
    let sink = graph.add_node(NodeType::OutputSink);

    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: filter,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    graph.add_edge(auxide::graph::Edge {
        from_node: filter,
        from_port: PortId(0),
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    // Process several blocks to ensure we're past initialization
    let mut out = vec![0.0; 64];

    // Start profiling only for process_block calls
    let _profiler = dhat::Profiler::new_heap();
    for _ in 0..10 {
        runtime.process_block(&mut out).unwrap();
    }

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 0, "SVF Lowpass should not allocate in process_block");
    assert_eq!(stats.total_bytes, 0, "SVF Lowpass should not allocate in process_block");
}

/// Test that Delay doesn't allocate in process_block
#[test]
fn test_delay_no_rt_alloc() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let delay = graph.add_external_node(Delay {
        delay_ms: 100.0,
        feedback: 0.3,
        mix: 0.5,
    });
    let sink = graph.add_node(NodeType::OutputSink);

    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: delay,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    graph.add_edge(auxide::graph::Edge {
        from_node: delay,
        from_port: PortId(0),
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    let mut out = vec![0.0; 64];

    // Start profiling only for process_block calls
    let _profiler = dhat::Profiler::new_heap();
    for _ in 0..100 { // More blocks for delay to fill
        runtime.process_block(&mut out).unwrap();
    }

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 0, "Delay should not allocate in process_block");
    assert_eq!(stats.total_bytes, 0, "Delay should not allocate in process_block");
}

/// Test that ADSR envelope doesn't allocate in process_block
#[test]
fn test_adsr_no_rt_alloc() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let adsr = graph.add_external_node(AdsrEnvelope {
        attack_ms: 10.0,
        decay_ms: 100.0,
        sustain_level: 0.7,
        release_ms: 200.0,
        curve: 1.0,
    });
    let gain = graph.add_node(NodeType::Gain { gain: 1.0 });
    let sink = graph.add_node(NodeType::OutputSink);

    // Osc -> ADSR -> Gain -> Sink
    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: adsr,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    graph.add_edge(auxide::graph::Edge {
        from_node: adsr,
        from_port: PortId(0),
        to_node: gain,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    graph.add_edge(auxide::graph::Edge {
        from_node: gain,
        from_port: PortId(0),
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    let mut out = vec![0.0; 64];

    // Start profiling only for process_block calls
    let _profiler = dhat::Profiler::new_heap();
    for _ in 0..50 {
        runtime.process_block(&mut out).unwrap();
    }

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 0, "ADSR should not allocate in process_block");
    assert_eq!(stats.total_bytes, 0, "ADSR should not allocate in process_block");
}

/// Test that Compressor doesn't allocate in process_block
#[test]
fn test_compressor_no_rt_alloc() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let compressor = graph.add_external_node(Compressor {
        threshold: -12.0,
        ratio: 4.0,
        attack_ms: 10.0,
        release_ms: 100.0,
        makeup_gain: 0.0,
    });
    let sink = graph.add_node(NodeType::OutputSink);

    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: compressor,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    graph.add_edge(auxide::graph::Edge {
        from_node: compressor,
        from_port: PortId(0),
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    let mut out = vec![0.0; 64];

    // Start profiling only for process_block calls
    let _profiler = dhat::Profiler::new_heap();
    for _ in 0..50 {
        runtime.process_block(&mut out).unwrap();
    }

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 0, "Compressor should not allocate in process_block");
    assert_eq!(stats.total_bytes, 0, "Compressor should not allocate in process_block");
}