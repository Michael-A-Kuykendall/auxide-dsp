# auxide-dsp

<img src="https://raw.githubusercontent.com/Michael-A-Kuykendall/auxide-dsp/master/assets/auxide-dsp-logo.png" alt="auxide-dsp logo" width="200"/>

DSP utilities and trait-based nodes for Auxide 0.2. This crate supplies helper functions, wavetable and window generators, and NodeDef-based DSP blocks that plug into the Auxide kernel via `NodeType::External`.

- **RT-safe**: no allocations in process paths; all buffers preallocated during init.
- Helpers: dB/linear conversions, phase increments, ms-to-samples, polyblep, interpolation.
- Tables: sine/saw/square/triangle wavetables, Hann/Hamming/Blackman windows.
- Nodes: Oscillators, Filters, Envelopes, LFO, Effects, Dynamics, Shapers, Pitch/Time, Utility.
- Builders: SynthBuilder, EffectsChainBuilder for easy graph construction.

## Status

- ✅ Architecture: Clean trait-based design with proper separation of concerns
- ✅ RT-Safety: Verified no allocations in process_block paths (audit completed)
- ✅ Test Coverage: Basic functionality tests for all nodes
- ⚠️ Advanced Testing: Missing RT allocation counting tests, golden value regression tests
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
