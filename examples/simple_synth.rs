use auxide_dsp::builders::SynthBuilder;
use auxide_dsp::nodes::envelopes::AdsrEnvelope;
use auxide_dsp::nodes::filters::SvfFilter;
use auxide_dsp::nodes::filters::SvfMode;
use auxide_dsp::nodes::oscillators::SawOsc;

fn main() {
    // Build a simple synth: SawOsc -> SVF Filter -> ADSR Envelope
    let _graph = SynthBuilder::new()
        .add_oscillator(SawOsc { freq: 440.0 })
        .add_filter(SvfFilter {
            cutoff: 1000.0,
            resonance: 0.5,
            mode: SvfMode::Lowpass,
        })
        .add_envelope(AdsrEnvelope {
            attack_ms: 100.0,
            decay_ms: 200.0,
            sustain_level: 0.7,
            release_ms: 300.0,
            curve: 1.0,
        })
        .build_graph();

    println!("Synth graph built successfully");
}
