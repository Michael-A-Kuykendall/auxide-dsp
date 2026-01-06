#![forbid(unsafe_code)]

/// Convert decibels to linear gain.
pub fn db_to_linear(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

/// Convert linear gain to decibels; clamps at a tiny floor to avoid -inf.
pub fn linear_to_db(linear: f32) -> f32 {
    let v = linear.max(1.0e-20);
    20.0 * v.log10()
}

/// Phase increment for a given frequency and sample rate.
pub fn freq_to_phase_increment(freq: f32, sample_rate: f32) -> f32 {
    2.0 * std::f32::consts::PI * freq / sample_rate
}

/// Milliseconds to samples (rounded down).
pub fn ms_to_samples(ms: f32, sample_rate: f32) -> usize {
    ((ms * sample_rate) / 1000.0).floor() as usize
}

/// PolyBLEP correction for band-limited waveforms.
pub fn polyblep(phase: f32, phase_inc: f32) -> f32 {
    let t = phase / phase_inc;
    if t < 1.0 {
        let t2 = t * t;
        return t + t2 - t2 * t;
    } else if t > 1.0 && t < 2.0 {
        let t = t - 2.0;
        let t2 = t * t;
        return t + t2 + t2 * t;
    }
    0.0
}

/// Linear interpolation from a buffer using fractional index.
pub fn linear_interpolate(buffer: &[f32], read_pos: f32) -> f32 {
    let len = buffer.len();
    if len == 0 {
        return 0.0;
    }
    let idx = read_pos.floor() as usize;
    let frac = read_pos - read_pos.floor();
    let a = buffer[idx % len];
    let b = buffer[(idx + 1) % len];
    a + (b - a) * frac
}

/// Exponential smoothing coefficient for time constant in milliseconds.
pub fn compute_exponential_coefficient(time_ms: f32, sample_rate: f32) -> f32 {
    if time_ms <= 0.0 {
        return 0.0;
    }
    (-1.0 / (time_ms / 1000.0 * sample_rate)).exp()
}
