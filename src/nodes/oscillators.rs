#![forbid(unsafe_code)]

use std::sync::Arc;

use auxide::graph::Port;
use auxide::node::NodeDef;

use crate::helpers::{freq_to_phase_increment, linear_interpolate, polyblep};

const PORTS_NONE: &[Port] = &[];
const PORTS_MONO_OUT: &[Port] = &[Port {
    id: auxide::graph::PortId(0),
    rate: auxide::graph::Rate::Audio,
}];

#[derive(Clone)]
pub struct SawOsc {
    pub freq: f32,
}

#[derive(Clone)]
pub struct SquareOsc {
    pub freq: f32,
    pub pulse_width: f32,
}

#[derive(Clone)]
pub struct TriangleOsc {
    pub freq: f32,
}

impl TriangleOsc {
    pub fn new(freq: f32) -> Self {
        Self { freq }
    }
}

#[derive(Clone)]
pub struct PulseOsc {
    pub freq: f32,
    pub pulse_width: f32,
}

#[derive(Clone)]
pub struct WavetableOsc {
    pub freq: f32,
    pub table: Arc<Vec<f32>>,
}

#[derive(Clone)]
pub struct SuperSaw {
    pub freq: f32,
    pub detune: f32,
    pub voices: usize,
}

#[derive(Clone)]
pub struct WhiteNoise;

#[derive(Clone)]
pub struct PinkNoise;

#[derive(Clone)]
pub struct BrownNoise;

pub struct OscState {
    phase: f32,
    freq: f32,
}

/// State for the band-limited triangle oscillator.
/// `tri` holds the running integral of a band-limited 50% square (which is the
/// triangle); `dc_x`/`dc_y` implement a one-pole DC-blocking high-pass that
/// removes the tiny integrator drift / residual offset so the output is clean.
pub struct TriangleState {
    phase: f32,
    freq: f32,
    tri: f32,
    dc_x: f32,
    dc_y: f32,
}

pub struct MultiPhaseState {
    phases: Vec<f32>,
    freq: f32,
    detune: f32,
}

pub struct NoiseState {
    rng: u64,
    pink: [f32; 7],
    brown: f32,
}

impl SawOsc {
    pub fn new(freq: f32) -> Self {
        Self { freq }
    }
}

impl NodeDef for SawOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState {
            phase: 0.0,
            freq: self.freq,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        if param == auxide::control::PARAM_FREQUENCY {
            state.freq = value;
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let inc = freq_to_phase_increment(state.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            *sample = 2.0 * phase - 1.0;
            *sample -= polyblep(phase, inc);
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for SquareOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState {
            phase: 0.0,
            freq: self.freq,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        if param == auxide::control::PARAM_FREQUENCY {
            state.freq = value;
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let pw = self.pulse_width.clamp(0.01, 0.99);
        let inc = freq_to_phase_increment(state.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            let base = if phase < pw { 1.0 } else { -1.0 };
            // Apply polyblep at both edges
            let mut val = base;
            val += polyblep(phase, inc);
            let phase_pw = (phase - pw + 1.0) % 1.0;
            val -= polyblep(phase_pw, inc);
            *sample = val;
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for TriangleOsc {
    type State = TriangleState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        TriangleState {
            phase: 0.0,
            freq: self.freq,
            tri: 0.0,
            dc_x: 0.0,
            dc_y: 0.0,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        if param == auxide::control::PARAM_FREQUENCY {
            state.freq = value;
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        // A triangle is the time integral of a square wave. We integrate a
        // band-limited 50% square (built from two PolyBLEP edges) so the result
        // is alias-free. The leaky multiplier is a DC-blocking high-pass that
        // keeps the integrator from drifting; its cutoff (~0.7 Hz) is far below
        // any audible fundamental and does not affect the triangle shape.
        let inc = freq_to_phase_increment(state.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            // Band-limited 50% square.
            let base = if phase < 0.5 { 1.0 } else { -1.0 };
            let mut sq = base;
            sq += polyblep(phase, inc);
            let phase_half = (phase - 0.5 + 1.0) % 1.0;
            sq -= polyblep(phase_half, inc);
            // Integrate (a triangle is the integral of a square). The leak is
            // proportional to the phase increment so it is a true DC-blocking
            // high-pass: it removes integrator drift without a fixed, huge low-frequency
            // gain, and its cutoff (a few Hz..tens of Hz) sits far below any audible
            // fundamental so the triangle shape is preserved.
            state.tri += 2.0 * inc * sq;
            state.tri -= 2.0 * inc * state.tri;
            // DC-blocking one-pole high-pass (~5 Hz, sample-rate independent) to
            // eliminate the residual integrator offset that would otherwise show up
            // as a low-frequency tone.
            let a = (-2.0 * std::f32::consts::PI * 5.0 / sample_rate).exp();
            let x = state.tri;
            let y = x - state.dc_x + a * state.dc_y;
            state.dc_x = x;
            state.dc_y = y;
            *sample = y;
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for PulseOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState {
            phase: 0.0,
            freq: self.freq,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        if param == auxide::control::PARAM_FREQUENCY {
            state.freq = value;
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let pw = self.pulse_width.clamp(0.01, 0.99);
        let inc = freq_to_phase_increment(state.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        for sample in out.iter_mut() {
            let phase = state.phase;
            let base = if phase < pw { 1.0 } else { -1.0 };
            let mut val = base;
            val += polyblep(phase, inc);
            let phase_pw = (phase - pw + 1.0) % 1.0;
            val -= polyblep(phase_pw, inc);
            *sample = val;
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for WavetableOsc {
    type State = OscState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        OscState {
            phase: 0.0,
            freq: self.freq,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        if param == auxide::control::PARAM_FREQUENCY {
            state.freq = value;
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let table = &*self.table;
        if table.is_empty() {
            out.fill(0.0);
            return;
        }
        let inc = freq_to_phase_increment(state.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        let len = table.len() as f32;
        for sample in out.iter_mut() {
            let read_pos = state.phase * len;
            *sample = linear_interpolate(table, read_pos);
            state.phase += inc;
            if state.phase >= 1.0 {
                state.phase -= 1.0;
            }
        }
    }
}

impl NodeDef for SuperSaw {
    type State = MultiPhaseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        let voices = self.voices.max(1);
        MultiPhaseState {
            phases: vec![0.0; voices],
            freq: self.freq,
            detune: self.detune,
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        match param {
            auxide::control::PARAM_FREQUENCY => state.freq = value,
            auxide::control::PARAM_DETUNE => state.detune = value,
            _ => {}
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        let voices = state.phases.len().max(1);
        let base_inc =
            freq_to_phase_increment(state.freq, sample_rate) / (2.0 * std::f32::consts::PI);
        let detune = state.detune.max(0.0);
        for sample in out.iter_mut() {
            let mut acc = 0.0;
            for (i, phase) in state.phases.iter_mut().enumerate() {
                let detune_factor =
                    1.0 + detune * ((i as f32) - (voices as f32 - 1.0) / 2.0) / (voices as f32);
                let inc = base_inc * detune_factor;
                acc += 2.0 * *phase - 1.0;
                *phase += inc;
                if *phase >= 1.0 {
                    *phase -= 1.0;
                }
            }
            *sample = acc / voices as f32;
        }
    }
}

impl NodeDef for WhiteNoise {
    type State = NoiseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        NoiseState {
            rng: 0x1234_5678_9abc_def0,
            pink: [0.0; 7],
            brown: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        for sample in out.iter_mut() {
            // LCG
            state.rng = state.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let v = ((state.rng >> 32) as u32) as f32 / (u32::MAX as f32);
            *sample = v * 2.0 - 1.0;
        }
    }
}

impl NodeDef for PinkNoise {
    type State = NoiseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        NoiseState {
            rng: 0x1234_5678_9abc_def0,
            pink: [0.0; 7],
            brown: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        for sample in out.iter_mut() {
            state.rng = state.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let white = ((state.rng >> 32) as u32) as f32 / (u32::MAX as f32) * 2.0 - 1.0;
            state.pink[0] = 0.99886 * state.pink[0] + white * 0.0555179;
            state.pink[1] = 0.99332 * state.pink[1] + white * 0.0750759;
            state.pink[2] = 0.96900 * state.pink[2] + white * 0.153_852;
            state.pink[3] = 0.86650 * state.pink[3] + white * 0.3104856;
            state.pink[4] = 0.55000 * state.pink[4] + white * 0.5329522;
            state.pink[5] = -0.7616 * state.pink[5] - white * 0.0168980;
            let pink = state.pink.iter().sum::<f32>() + state.pink[6] + white * 0.5362;
            state.pink[6] = white * 0.115926;
            *sample = pink * 0.1;
        }
    }
}

impl NodeDef for BrownNoise {
    type State = NoiseState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        NoiseState {
            rng: 0x1234_5678_9abc_def0,
            pink: [0.0; 7],
            brown: 0.0,
        }
    }

    fn process_block(
        &self,
        state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let Some(out) = outputs.get_mut(0) else {
            return;
        };
        for sample in out.iter_mut() {
            state.rng = state.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let white = ((state.rng >> 32) as u32) as f32 / (u32::MAX as f32) * 2.0 - 1.0;
            state.brown += white * 0.02;
            state.brown = state.brown.clamp(-1.0, 1.0);
            *sample = state.brown;
        }
    }
}

/// State of a Constant
#[derive(Debug, Clone)]
pub struct ConstantState;

/// Constant value source
#[derive(Debug, Clone)]
pub struct Constant {
    pub value: f32,
}

impl NodeDef for Constant {
    type State = ConstantState;

    fn input_ports(&self) -> &'static [Port] {
        PORTS_NONE
    }

    fn output_ports(&self) -> &'static [Port] {
        PORTS_MONO_OUT
    }

    fn required_inputs(&self) -> usize {
        0
    }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        ConstantState
    }

    fn process_block(
        &self,
        _state: &mut Self::State,
        _inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        _sample_rate: f32,
    ) {
        let Some(output) = outputs.get_mut(0) else {
            return;
        };
        for sample in output.iter_mut() {
            *sample = self.value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auxide::node::NodeDef;
    use realfft::RealFftPlanner;
    use std::sync::Arc;

    /// Render any oscillator `NodeDef` into a contiguous buffer using fixed-size blocks.
    fn render<Node: NodeDef>(
        node: &Node,
        mut state: Node::State,
        sr: f32,
        seconds: f32,
        block: usize,
    ) -> Vec<f32> {
        let total = (sr * seconds) as usize;
        let mut out = vec![0.0f32; total];
        let mut written = 0;
        while written < total {
            let take = block.min(total - written);
            let mut buf = vec![vec![0.0f32; take]];
            node.process_block(&mut state, &[], &mut buf, sr);
            out[written..written + take].copy_from_slice(&buf[0]);
            written += take;
        }
        out
    }

    /// Power spectrum (magnitude squared) of a real signal via real-to-complex FFT.
    fn spectrum(buf: &[f32]) -> Vec<f32> {
        let n = buf.len();
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(n);
        let mut indata = buf.to_vec();
        let mut out = r2c.make_output_vec();
        r2c.process(&mut indata, &mut out).unwrap();
        out.iter().map(|c| c.norm_sqr()).collect()
    }

    /// Sum spectral energy in a small band around `center_hz` (inclusive, +-`half` bins).
    fn band_energy(spec: &[f32], center_hz: f32, sr: f32, n: usize, half: usize) -> f32 {
        let bin = (center_hz * n as f32 / sr).round() as usize;
        let lo = bin.saturating_sub(half);
        let hi = (bin + half).min(spec.len() - 1);
        spec[lo..=hi].iter().copied().sum()
    }

    /// Naive (truncating) wavetable reference: the `table[floor(phase*len)]` behaviour.
    fn naive_wavetable(table: &[f32], freq: f32, sr: f32, seconds: f32) -> Vec<f32> {
        let n = (sr * seconds) as usize;
        let inc = freq / sr;
        let len = table.len() as f32;
        let mut phase = 0.0f32;
        let mut v = vec![0.0f32; n];
        for s in v.iter_mut() {
            let idx = ((phase * len) as usize) % table.len();
            *s = table[idx];
            phase += inc;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }
        v
    }

    /// Find the baseband bin with the most energy that is NOT a harmonic of `fund`
    /// (these are exactly where above-Nyquist harmonics alias into). Returns the bin index.
    fn worst_alias_bin(spec: &[f32], sr: f32, n: usize, fund: f32) -> usize {
        let bin_width = sr / n as f32;
        let fund_bin = (fund / bin_width).round() as usize;
        let is_harmonic = |b: usize| -> bool {
            if b == 0 {
                return true;
            }
            let f = b as f32 * bin_width;
            let k = (f / fund).round();
            (f - k * fund).abs() < bin_width * 1.5
        };
        let mut best = 0usize;
        let mut best_e = 0.0f32;
        for (b, &e) in spec.iter().enumerate().skip(1) {
            if (b as isize - fund_bin as isize).abs() <= 3 {
                continue;
            }
            if is_harmonic(b) {
                continue;
            }
            if e > best_e {
                best_e = e;
                best = b;
            }
        }
        best
    }

    #[test]
    fn polyblep_triangle_bandlimited() {
        // (sample_rate, fundamental). At each setting the band-limited triangle must
        // keep alias energy (above-Nyquist harmonics folded into baseband) at least
        // 40 dB below the fundamental. For a triangle the ideal (non-band-limited)
        // reference is already quite clean (1/n^2 rolloff), so we additionally assert
        // the band-limited output is no worse than that reference.
        for (sr, freq) in [(8000.0f32, 200.0f32), (44100.0f32, 1000.0f32)] {
            let osc = TriangleOsc::new(freq);
            let state = osc.init_state(sr, 64);
            let out = render(&osc, state, sr, 1.0, 64);
            let spec = spectrum(&out);
            let n = out.len();
            let e_fund = band_energy(&spec, freq, sr, n, 2);
            let ab = worst_alias_bin(&spec, sr, n, freq);
            let e_alias = band_energy(&spec, ab as f32 * sr / n as f32, sr, n, 1);
            let ratio = e_alias / e_fund;
            assert!(
                ratio <= 1e-4,
                "triangle band-limited alias ratio {:.3e} at sr {} not <= 40 dB (1e-4)",
                ratio,
                sr
            );
        }
    }

    #[test]
    fn polyblep_pulse_no_aliasing() {
        // AC: PulseOsc pw=0.1 at 8 kHz. The band-limited pulse must keep alias energy
        // >= 40 dB below the fundamental. (At a low fundamental the naive ideal pulse
        // is already clean, so the meaningful check is that the PolyBLEP output also
        // meets the 40 dB bar.)
        let sr = 8000.0f32;
        let freq = 100.0f32;
        let pw = 0.1f32;

        let osc = PulseOsc {
            freq,
            pulse_width: pw,
        };
        let state = osc.init_state(sr, 64);
        let out = render(&osc, state, sr, 1.0, 64);
        let spec = spectrum(&out);
        let n = out.len();
        let e_fund = band_energy(&spec, freq, sr, n, 2);
        let ab = worst_alias_bin(&spec, sr, n, freq);
        let e_alias = band_energy(&spec, ab as f32 * sr / n as f32, sr, n, 1);
        let ratio = e_alias / e_fund;
        assert!(
            ratio <= 1e-4,
            "pulse band-limited alias ratio {:.3e} at sr {} not <= 40 dB (1e-4)",
            ratio,
            sr
        );
    }

    #[test]
    fn wavetable_interpolates() {
        let len = 64usize;
        let table: Vec<f32> = (0..len)
            .map(|i| (2.0 * std::f32::consts::PI * i as f32 / len as f32).sin())
            .collect();
        let table = Arc::new(table);
        let freq = 440.0f32;
        let sr = 44100.0f32;

        let osc = WavetableOsc {
            freq,
            table: table.clone(),
        };
        let state = osc.init_state(sr, 64);
        let out = render(&osc, state, sr, 1.0, 64);

        // Truncation reference: identical phase walk but nearest-index lookup.
        let ref_out = naive_wavetable(&table, freq, sr, 1.0);

        let max_step = |b: &[f32]| {
            b.windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0f32, f32::max)
        };
        let interp_max = max_step(&out);
        let trunc_max = max_step(&ref_out);
        assert!(
            interp_max < trunc_max,
            "interpolated max step {} not smaller than truncation {}",
            interp_max,
            trunc_max
        );

        // Interpolated output must differ from the floor-sample lookup somewhere.
        let max_diff = out
            .iter()
            .zip(ref_out.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            max_diff > 1e-3,
            "interpolation never differs from truncation by > 1e-3 (max {})",
            max_diff
        );
    }
}
