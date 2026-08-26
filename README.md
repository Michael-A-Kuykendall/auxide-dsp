# auxide-dsp

[![Trans rights](https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA)](https://translifeline.org/)
[![LGBTQ+ friendly](https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982)](https://www.thetrevorproject.org/)


<img src="assets/auxide-dsp-logo.png" alt="auxide-dsp logo" width="400"/>

[![Crates.io](https://img.shields.io/crates/v/auxide-dsp.svg)](https://crates.io/crates/auxide-dsp)
[![Documentation](https://docs.rs/auxide-dsp/badge.svg)](https://docs.rs/auxide-dsp)
[![CI](https://github.com/Michael-A-Kuykendall/auxide-dsp/workflows/CI/badge.svg)](https://github.com/Michael-A-Kuykendall/auxide-dsp/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 💝 Support Auxide's Growth

🚀 If Auxide helps you build amazing audio tools, consider [sponsoring](https://github.com/sponsors/Michael-A-Kuykendall) — 100% of support goes to keeping it free forever.

• $5/month: Coffee tier ☕ - Eternal gratitude + sponsor badge
• $25/month: Bug prioritizer 🐛 - Priority support + name in [SPONSORS.md](https://github.com/Michael-A-Kuykendall/auxide-dsp/blob/main/SPONSORS.md)
• $100/month: Corporate backer 🏢 - Logo placement + monthly office hours
• $500/month: Infrastructure partner 🚀 - Direct support + roadmap input

**[🎯 Become a Sponsor](https://github.com/sponsors/Michael-A-Kuykendall)** | See our amazing [sponsors](https://github.com/Michael-A-Kuykendall/auxide-dsp/blob/main/SPONSORS.md) 🙏

DSP utilities and trait-based nodes for Auxide 0.3. This crate supplies helper functions, wavetable and window generators, and NodeDef-based DSP blocks that plug into the Auxide kernel via `NodeType::External`.

- **RT-safe**: no allocations in process paths; all buffers preallocated during init.
- Helpers: dB/linear conversions, phase increments, ms-to-samples, polyblep, interpolation.
- Tables: sine/saw/square/triangle wavetables, Hann/Hamming/Blackman windows.
- Nodes: Oscillators, Filters, Envelopes, LFO, Effects, Dynamics, Shapers, Pitch/Time, Utility.
- Builders: SynthBuilder, EffectsChainBuilder for easy graph construction.

## Auxide Ecosystem
| Crate | Description | Version |
|-------|-------------|---------|
| [auxide](https://github.com/Michael-A-Kuykendall/auxide) | Real-time-safe audio graph kernel | 0.3.2 |
| **[auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp)** | DSP nodes library | 0.2.1 |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | 0.1.3 |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | 0.1.2 |

## Status

- ✅ Architecture: Clean trait-based design with proper separation of concerns
- ✅ RT-Safety: Verified zero allocations in process_block paths (dhat profiler tests)
- ✅ Test Coverage: Basic functionality tests for all nodes
- ✅ RT Allocation Tests: Comprehensive heap profiling validates RT guarantees
- 📋 Node Coverage: ~40 nodes implemented, missing ~10 for full synthesis toolkit

See [AUDIT_REPORT.md](AUDIT_REPORT.md) for detailed analysis.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
auxide = "0.3"
auxide-dsp = "0.2"
```

## Examples

### SynthBuilder with Envelope

<details>
<summary>Expand</summary>

```rust
use auxide_dsp::builders::SynthBuilder;
use auxide_dsp::nodes::oscillators::SawOsc;
use auxide_dsp::nodes::filters::SvfFilter;
use auxide_dsp::nodes::filters::SvfMode;
use auxide_dsp::nodes::envelopes::AdsrEnvelope;
use auxide::plan::Plan;

let (graph, plan) = SynthBuilder::new()
    .add_oscillator(SawOsc { freq: 220.0 })
    .add_filter(SvfFilter {
        cutoff: 2000.0,
        resonance: 0.3,
        mode: SvfMode::Lowpass,
    })
    .add_envelope(AdsrEnvelope {
        attack_ms: 100.0,
        decay_ms: 50.0,
        sustain_level: 0.7,
        release_ms: 200.0,
        curve: 2.0,
    })
    .build(64)
    .expect("synth builds");

let mut runtime = auxide::rt::Runtime::new(plan, &graph, 44100.0);
let mut out = vec![0.0f32; 64];
runtime.process_block(&mut out).unwrap();
```
</details>

### Effects Chain with Modulation

<details>
<summary>Expand</summary>

```rust
use auxide_dsp::nodes::oscillators::SawOsc;
use auxide_dsp::nodes::filters::SvfFilter;
use auxide_dsp::nodes::fx::{Delay, Tremolo, SimpleReverb};
use auxide_dsp::nodes::dynamics::Compressor;
use auxide_dsp::nodes::shapers::WaveShaper;
use auxide::graph::{Graph, NodeType, NodeType::External, PortId, Rate};
use auxide_dsp::nodes::filters::SvfMode;
use auxide::plan::Plan;

let mut graph = Graph::new();
let osc = graph.add_external_node(SawOsc { freq: 220.0 });
let filter = graph.add_external_node(SvfFilter {
    cutoff: 800.0, resonance: 0.4, mode: SvfMode::Lowpass,
});
let tremolo = graph.add_external_node(Tremolo { rate: 4.0, depth: 0.6 });
let delay = graph.add_external_node(Delay { delay_ms: 200.0, feedback: 0.4, mix: 0.5 });
let reverb = graph.add_external_node(SimpleReverb { decay: 0.5, mix: 0.3 });
let compressor = graph.add_external_node(Compressor {
    threshold: 0.5, ratio: 4.0, attack_ms: 5.0, release_ms: 50.0, makeup_gain: 2.0,
});
let waveshaper = graph.add_external_node(WaveShaper { drive: 0.3, mix: 0.7 });
let sink = graph.add_node(NodeType::OutputSink);

// Chain: osc → filter → tremolo → delay → reverb → compressor → waveshaper → output
for &(from, to) in &[
    (osc, filter), (filter, tremolo), (tremolo, delay),
    (delay, reverb), (reverb, compressor), (compressor, waveshaper), (waveshaper, sink),
] {
    graph.add_edge(auxide::graph::Edge {
        from_node: from, from_port: PortId(0),
        to_node: to, to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
}

let plan = Plan::compile(&graph, 512).unwrap();
```
</details>

### Sample Playback (ROMpler Voice)

<details>
<summary>Expand</summary>

```rust
use std::sync::Arc;
use auxide_dsp::nodes::sampler::Sampler;
use auxide::graph::{Graph, NodeType, PortId, Rate, NodeType::External};
use auxide::plan::Plan;

// Generate a test sample (440 Hz sine, 1 second)
let sr = 44100.0;
let sample: Arc<Vec<f32>> = Arc::new(
    (0..sr as usize).map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr).sin()).collect()
);

let mut graph = Graph::new();
let sampler = graph.add_external_node(Sampler::new(sample, sr, 69, false));
let sink = graph.add_node(NodeType::OutputSink);
graph.add_edge(auxide::graph::Edge {
    from_node: sampler, from_port: PortId(0),
    to_node: sink, to_port: PortId(0),
    rate: Rate::Audio,
}).unwrap();
let plan = Plan::compile(&graph, 64).unwrap();
let mut runtime = auxide::rt::Runtime::new(plan, &graph, sr);
let mut out = vec![0.0f32; 64];
runtime.process_block(&mut out).unwrap();
```
</details>

### Filter Frequency Modulation

The SVF and Ladder filters expose **modulation input ports**: connect a control-rate or audio-rate source to ports 1 (cutoff mod) or 2 (resonance mod) for dynamic filter sweeps:

```rust
use auxide_dsp::nodes::oscillators::SawOsc;
use auxide_dsp::nodes::filters::{SvfFilter, SvfMode};
use auxide_dsp::nodes::lfo::{Lfo, LfoWaveform};
use auxide::graph::{Graph, NodeType, PortId, Rate, NodeType::External};

let mut graph = Graph::new();
let osc = graph.add_external_node(SawOsc { freq: 220.0 });
let lfo = graph.add_external_node(Lfo {
    frequency: 0.5, waveform: LfoWaveform::Triangle,
    amplitude: 2000.0, offset: 1000.0,
});
let filter = graph.add_external_node(SvfFilter {
    cutoff: 1000.0, resonance: 0.3, mode: SvfMode::Lowpass,
});
let sink = graph.add_node(NodeType::OutputSink);

// Audio path: osc -> filter (audio input)
graph.add_edge(Edge { from_node: osc, from_port: PortId(0),
    to_node: filter, to_port: PortId(0), rate: Rate::Audio }).unwrap();
// Modulation: LFO -> filter cutoff mod port
graph.add_edge(Edge { from_node: lfo, from_port: PortId(0),
    to_node: filter, to_port: PortId(1), rate: Rate::Control }).unwrap();
// Output
graph.add_edge(Edge { from_node: filter, from_port: PortId(0),
    to_node: sink, to_port: PortId(0), rate: Rate::Audio }).unwrap();
```

See [`examples/`](examples/) for complete, running demos.

## 40+ Registered Nodes

All nodes are registered in the `Registry` for name-based construction:

```rust
use auxide_dsp::registry::register_dsp_ugens;
let mut reg = auxide::registry::Registry::new();
register_dsp_ugens(&mut reg); // registers "saw", "svf", "adsr", "delay", "compressor", etc.
```

| Category | Nodes |
|----------|-------|
| **Oscillators** | SawOsc, SquareOsc, TriangleOsc, PulseOsc, WavetableOsc, SuperSaw, WhiteNoise, PinkNoise, BrownNoise |
| **Filters** | SvfFilter, LadderFilter, CombFilter, FormantFilter, BiquadFilter, AllpassFilter, OnePoleFilter, ParametricEq, ResonantDrive |
| **Envelopes** | AdsrEnvelope, ArEnvelope, AdEnvelope |
| **FX** | Delay, Chorus, Flanger, Phaser, SimpleReverb, StereoReverb, MultitapDelay, ConvolutionReverb (WAV IR), Tremolo |
| **Dynamics** | Compressor, Limiter, NoiseGate, Expander |
| **Shapers** | WaveShaper, HardClip, SoftClip, BitCrusher, TubeSaturation, DcBlocker, Overdrive |
| **Pitch/Time** | PitchShifter, PitchDetector |
| **Utility** | Multiply (ring mod), RingMod, Crossfader, StereoWidth, ParamSmoother, MidSideProcessor, StereoPanner, RMSMeter, Mixer (up to 16 inputs) |
| **Modulation** | Lfo (sine/triangle/saw/square/random), Sampler (WAV playback) |

## Community & Support

• 🐛 Bug Reports: [GitHub Issues](https://github.com/Michael-A-Kuykendall/auxide-dsp/issues)
• 💬 Discussions: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/auxide-dsp/discussions)
• 📖 Documentation: [docs/](https://github.com/Michael-A-Kuykendall/auxide-dsp/tree/main/docs)
• 💝 Sponsorship: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)
• 🤝 Contributing: [CONTRIBUTING.md](https://github.com/Michael-A-Kuykendall/auxide-dsp/blob/main/CONTRIBUTING.md)
• 📜 Governance: [GOVERNANCE.md](https://github.com/Michael-A-Kuykendall/auxide-dsp/blob/main/GOVERNANCE.md)
• 🔒 Security: [SECURITY.md](https://github.com/Michael-A-Kuykendall/auxide-dsp/blob/main/SECURITY.md)

## License & Philosophy

MIT License - forever and always.

**Philosophy**: DSP infrastructure should be invisible. Auxide is infrastructure.

**Testing Philosophy**: Reliability through comprehensive validation and property-based testing.

**Forever maintainer**: Michael A. Kuykendall  
**Promise**: This will never become a paid product  
**Mission**: Making real-time audio DSP simple and reliable

## Support

This project is a safe space. Trans rights are human rights.

If you or someone you love needs support:

- [The Trevor Project](https://www.thetrevorproject.org/) — 24/7 for LGBTQ+ young people. Call 1-866-488-7386 or text START to 678-678
- [Trans Lifeline](https://translifeline.org/) — peer support run by and for trans people. US: 877-565-8860
- [988 Suicide & Crisis Lifeline](https://988lifeline.org/) — call or text 988

