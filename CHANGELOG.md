# Changelog

## [0.1.1] - 2026-01-07
- **Bug fixes**: Phase modulo guards in all oscillators for improved numerical stability
- **Documentation improvements**: Complete API documentation for all modules and public types
- **Auxide 0.2.1 compatibility**: Updated to work with latest auxide kernel
- **RT safety verification**: Confirmed all DSP nodes maintain zero allocations during process_block
- **Testing**: All unit, property-based, and integration tests passing

**Requires**: [auxide >= 0.2.1](https://github.com/Michael-A-Kuykendall/auxide/releases/tag/v0.2.1)  
**Compatible with**: [auxide-io 0.1.2](https://github.com/Michael-A-Kuykendall/auxide-io/releases/tag/v0.1.2), [auxide-midi 0.1.1](https://github.com/Michael-A-Kuykendall/auxide-midi/releases/tag/v0.1.1)

## [0.2.0] - 2026-01-05
- **Major RT-safety audit and verification** - Comprehensive heap profiling confirms zero allocations in process_block paths
- **Production readiness certification** - All 200+ tests passing across Auxide ecosystem
- **DSP node library complete** - ~40 RT-safe nodes implemented: oscillators, filters, effects, dynamics, distortion, utility, pitch, LFO
- **Helper utilities** - Audio math functions, wavetable generators, window functions, interpolation, anti-aliasing
- **Builder pattern** - SynthBuilder and EffectsChainBuilder for easy graph construction
- **Trait-based architecture** - Clean NodeDef plugin system integrating with auxide kernel
- **Comprehensive testing** - Unit tests, property-based tests, RT allocation verification, integration tests
- **Cross-platform compatibility** - Verified on multiple architectures and operating systems
- **Documentation** - Complete API docs with examples, architecture guides, and audit reports

## [0.1.0] - 2026-01-05
- **Major RT-safety audit and verification** - Comprehensive heap profiling confirms zero allocations in process_block paths
- **Production readiness certification** - All 200+ tests passing across Auxide ecosystem
- **DSP node library complete** - ~40 RT-safe nodes implemented: oscillators, filters, effects, dynamics, distortion, utility, pitch, LFO
- **Helper utilities** - Audio math functions, wavetable generators, window functions, interpolation, anti-aliasing
- **Builder pattern** - SynthBuilder and EffectsChainBuilder for easy graph construction
- **Trait-based architecture** - Clean NodeDef plugin system integrating with auxide kernel
- **Comprehensive testing** - Unit tests, property-based tests, RT allocation verification, integration tests
- **Cross-platform compatibility** - Verified on multiple architectures and operating systems
- **Documentation** - Complete API docs with examples, architecture guides, and audit reports</content>
<parameter name="filePath">c:\Users\micha\repos\auxide-dsp\CHANGELOG.md