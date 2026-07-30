# Changelog

## [0.2.1] - 2026-07-29
- **Ecosystem docs**: Updated AGENTS.md with full 7-crate Auxide ecosystem overview

## [0.2.0] - 2026-07-25
- **Band-limited oscillators**: Triangle and pulse waveshapes now use PolyBLEP anti-aliasing (integrate a band-limited square + 5 Hz DC-block); WavetableOsc uses linear interpolation instead of truncation.
- **RT safety verification**: Confirmed all DSP nodes maintain zero allocations during process_block.
- **DSP node library**: RT-safe nodes for oscillators, filters, effects, dynamics, distortion, utility, pitch, LFO.
- **Helper utilities** - Audio math functions, wavetable generators, window functions, interpolation, anti-aliasing.
- **Builder pattern** - SynthBuilder and EffectsChainBuilder for easy graph construction.
- **Trait-based architecture** - Clean NodeDef plugin system integrating with auxide kernel.
- **Comprehensive testing** - Unit tests (including FFT golden tests for alias suppression), property-based tests, and RT allocation verification.

**Requires**: [auxide >= 0.3.1](https://github.com/Michael-A-Kuykendall/auxide/releases/tag/v0.3.1)  
**Compatible with**: [auxide-io 0.1.2](https://github.com/Michael-A-Kuykendall/auxide-io/releases/tag/v0.1.2), [auxide-midi 0.1.1](https://github.com/Michael-A-Kuykendall/auxide-midi/releases/tag/v0.1.1)

## [0.1.1] - 2026-01-07
- **Bug fixes**: Phase modulo guards in all oscillators for improved numerical stability
- **Documentation improvements**: Complete API documentation for all modules and public types
- **Auxide 0.2.1 compatibility**: Updated to work with latest auxide kernel
- **RT safety verification**: Confirmed all DSP nodes maintain zero allocations during process_block
- **Testing**: All unit, property-based, and integration tests passing

**Requires**: [auxide >= 0.2.1](https://github.com/Michael-A-Kuykendall/auxide/releases/tag/v0.2.1)  
**Compatible with**: [auxide-io 0.1.2](https://github.com/Michael-A-Kuykendall/auxide-io/releases/tag/v0.1.2), [auxide-midi 0.1.1](https://github.com/Michael-A-Kuykendall/auxide-midi/releases/tag/v0.1.1)

## [0.1.0] - 2026-01-05
- Initial public release of the DSP node library.
- RT-safe nodes for oscillators, filters, effects, dynamics, distortion, utility, pitch, LFO.
- Helper utilities: audio math, wavetable generators, window functions, interpolation, anti-aliasing.
- SynthBuilder and EffectsChainBuilder for graph construction.
- Trait-based NodeDef plugin system integrating with the auxide kernel.</content>
<parameter name="filePath">c:\Users\micha\repos\auxide-dsp\CHANGELOG.md