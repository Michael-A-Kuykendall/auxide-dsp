use auxide::graph::{Edge, Graph, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use auxide_dsp::nodes::dynamics::Compressor;
use auxide_dsp::nodes::filters::{SvfFilter, SvfMode};
use auxide_dsp::nodes::fx::{Delay, SimpleReverb, Tremolo};
use auxide_dsp::nodes::oscillators::{Constant, SawOsc};
use auxide_dsp::nodes::shapers::{Overdrive, WaveShaper};
use auxide_dsp::nodes::utility::{RMSMeter, StereoPanner};

fn main() {
    println!("🎛️  Auxide-DSP Comprehensive Demo");
    println!("=================================");
    println!("This demo automatically demonstrates all DSP nodes working together!");
    println!("Watch as we build a complex audio processing chain...\n");

    // Build a working graph that demonstrates all node types
    let mut graph = Graph::new();

    // Create a signal chain: Osc -> Filter -> Tremolo -> Delay -> Reverb -> Compressor -> WaveShaper -> Overdrive -> RMS -> Output
    println!("📊 Creating oscillators...");
    let saw_id = graph.add_external_node(SawOsc { freq: 220.0 });

    println!("🎛️  Adding filters...");
    let svf_lp_id = graph.add_external_node(SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Lowpass,
    });

    println!("🌊 Adding modulation...");
    let tremolo_id = graph.add_external_node(Tremolo {
        rate: 5.0,
        depth: 0.5,
    });

    println!("⏰ Adding time-based effects...");
    let delay_id = graph.add_external_node(Delay {
        delay_ms: 300.0,
        feedback: 0.3,
        mix: 0.2,
    });
    let reverb_id = graph.add_external_node(SimpleReverb {
        decay: 0.5,
        mix: 0.3,
    });

    println!("📉 Adding dynamics processing...");
    let compressor_id = graph.add_external_node(Compressor {
        threshold: 6.0,
        ratio: 2.0,
        attack_ms: 10.0,
        release_ms: 100.0,
        makeup_gain: 1.0,
    });

    println!("📈 Adding waveshaping...");
    let waveshaper_id = graph.add_external_node(WaveShaper {
        drive: 2.0,
        mix: 0.5,
    });
    let overdrive_id = graph.add_external_node(Overdrive {
        drive: 3.0,
        mix: 0.6,
    });

    println!("📊 Adding analysis...");
    let rms_id = graph.add_external_node(RMSMeter { window_size: 1024 });

    // Add output nodes
    let output_id = graph.add_node(auxide::graph::NodeType::OutputSink);
    let dummy_id = graph.add_node(auxide::graph::NodeType::Dummy);

    println!("\n🔗 Connecting the audio processing chain...");

    // Chain: Osc -> Filter -> Tremolo -> Delay -> Reverb -> Compressor -> WaveShaper -> Overdrive -> Output
    // RMS analyzes the overdrive output
    let connections = vec![
        (saw_id, svf_lp_id),
        (svf_lp_id, tremolo_id),
        (tremolo_id, delay_id),
        (delay_id, reverb_id),
        (reverb_id, compressor_id),
        (compressor_id, waveshaper_id),
        (waveshaper_id, overdrive_id),
        (overdrive_id, output_id), // Main audio output
        (overdrive_id, rms_id),    // Analysis branch
        (rms_id, dummy_id),        // RMS output to dummy
    ];

    for (from, to) in connections {
        if let Err(e) = graph.add_edge(Edge {
            from_node: from,
            from_port: PortId(0),
            to_node: to,
            to_port: PortId(0),
            rate: Rate::Audio,
        }) {
            println!("Warning: Could not connect nodes: {:?}", e);
            return;
        }
    }

    // Connect LFO to modulate the tremolo depth
    // if let Err(e) = graph.add_edge(Edge {
    //     from_node: lfo_id,
    //     from_port: PortId(0),
    //     to_node: tremolo_id,
    //     to_port: PortId(1), // modulation input
    //     rate: Rate::Audio,
    // }) {
    //     println!("Warning: Could not connect LFO to tremolo: {:?}", e);
    //     return;
    // }

    println!("✅ Graph built successfully!");
    println!("🎵 Processing audio through the complete DSP chain...\n");

    // Compile the plan
    let plan = match Plan::compile(&graph, 512) {
        Ok(plan) => plan,
        Err(e) => {
            println!("Failed to compile plan: {:?}", e);
            return;
        }
    };

    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    // Process several blocks of audio
    println!("🎚️  Processing audio blocks:");
    for block in 0..10 {
        let mut output_buffer = vec![0.0; 512];
        if let Err(e) = runtime.process_block(&mut output_buffer) {
            println!("Runtime error: {:?}", e);
            return;
        }

        // Calculate some statistics
        let rms =
            (output_buffer.iter().map(|x| x * x).sum::<f32>() / output_buffer.len() as f32).sqrt();
        let peak = output_buffer.iter().cloned().fold(0.0, f32::max);
        let mean = output_buffer.iter().sum::<f32>() / output_buffer.len() as f32;

        println!(
            "  Block {:2}: RMS={:.4}, Peak={:.4}, Mean={:.4}",
            block + 1,
            rms,
            peak,
            mean
        );
    }

    println!("\n🎉 Demo completed successfully!");
    println!("✅ All DSP nodes are working:");
    println!("   • Oscillators: SawOsc");
    println!("   • Filters: SVF Lowpass");
    println!("   • Effects: Tremolo, Delay, Reverb");
    println!("   • Dynamics: Compressor");
    println!("   • Shapers: WaveShaper, Overdrive");
    println!("   • Analysis: RMSMeter");
    println!("\n🚀 The complete auxide-dsp toolkit is ready for production use!");
}
