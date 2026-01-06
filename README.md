# auxide-dsp

<img src="https://raw.githubusercontent.com/Michael-A-Kuykendall/auxide-dsp/main/assets/auxide-dsp-logo.png" alt="auxide-dsp logo" width="200"/>

[![Crates.io](https://img.shields.io/crates/v/auxide-dsp.svg)](https://crates.io/crates/auxide-dsp)
[![Documentation](https://docs.rs/auxide-dsp/badge.svg)](https://docs.rs/auxide-dsp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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

DSP utilities and trait-based nodes for Auxide 0.2. This crate supplies helper functions, wavetable and window generators, and NodeDef-based DSP blocks that plug into the Auxide kernel via `NodeType::External`.

- **RT-safe**: no allocations in process paths; all buffers preallocated during init.
- Helpers: dB/linear conversions, phase increments, ms-to-samples, polyblep, interpolation.
- Tables: sine/saw/square/triangle wavetables, Hann/Hamming/Blackman windows.
- Nodes: Oscillators, Filters, Envelopes, LFO, Effects, Dynamics, Shapers, Pitch/Time, Utility.
- Builders: SynthBuilder, EffectsChainBuilder for easy graph construction.

## Auxide Ecosystem
| Crate | Description | Version |
|-------|-------------|---------|
| [auxide](https://github.com/Michael-A-Kuykendall/auxide) | Real-time-safe audio graph kernel | 0.3.0 |
| **[auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp)** | DSP nodes library | 0.2.0 |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | 0.2.0 |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | 0.2.0 |

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
auxide = "0.2"
auxide-dsp = { path = "../auxide-dsp" }
```

## Example

```rust
use auxide_dsp::builders::SynthBuilder;
use auxide_dsp::nodes::oscillators::SawOsc;
use auxide_dsp::nodes::filters::SvfFilter;
use auxide_dsp::nodes::filters::SvfMode;

let graph = SynthBuilder::new()
    .add_oscillator(SawOsc { freq: 440.0 })
    .add_filter(SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Lowpass,
    })
    .build_graph();
```

See `examples/` for more usage.

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
