#![forbid(unsafe_code)]

/// Generate a sine wavetable of the given size.
pub fn generate_sine_table(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| {
            let phase = 2.0 * std::f32::consts::PI * (i as f32) / (size as f32);
            phase.sin()
        })
        .collect()
}

/// Generate a saw wavetable of the given size (-1..1 ramp).
pub fn generate_saw_table(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| (2.0 * (i as f32) / (size as f32)) - 1.0)
        .collect()
}

/// Generate a square wavetable of the given size (-1 or 1).
pub fn generate_square_table(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| if i < size / 2 { 1.0 } else { -1.0 })
        .collect()
}

/// Generate a triangle wavetable of the given size.
pub fn generate_triangle_table(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| {
            let t = (2.0 * (i as f32) / (size as f32)) - 1.0;
            -(2.0 * (t.abs() - 0.5))
        })
        .collect()
}