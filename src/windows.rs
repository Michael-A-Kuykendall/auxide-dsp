#![forbid(unsafe_code)]

/// Hann window.
pub fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| {
            let phase = (n as f32) / (size as f32);
            0.5 - 0.5 * (2.0 * std::f32::consts::PI * phase).cos()
        })
        .collect()
}

/// Hamming window.
pub fn hamming_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| {
            let phase = (n as f32) / (size as f32);
            0.54 - 0.46 * (2.0 * std::f32::consts::PI * phase).cos()
        })
        .collect()
}

/// Blackman window.
pub fn blackman_window(size: usize) -> Vec<f32> {
    const A0: f32 = 0.42;
    const A1: f32 = 0.5;
    const A2: f32 = 0.08;
    (0..size)
        .map(|n| {
            let phase = (n as f32) / (size as f32);
            A0
                - A1 * (2.0 * std::f32::consts::PI * phase).cos()
                + A2 * (4.0 * std::f32::consts::PI * phase).cos()
        })
        .collect()
}