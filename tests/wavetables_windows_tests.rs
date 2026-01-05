#![forbid(unsafe_code)]

use auxide_dsp::*;

fn approx(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{} != {}", a, b);
}

#[test]
fn wavetable_golden() {
    let table = generate_sine_table(8);
    let expected = [0.0, 0.7071, 1.0, 0.7071, 0.0, -0.7071, -1.0, -0.7071];
    for (t, e) in table.iter().zip(expected.iter()) {
        approx(*t, *e);
    }

    let saw = generate_saw_table(4);
    approx(saw[0], -1.0);
    approx(saw[2], 0.0);

    let tri = generate_triangle_table(4);
    approx(tri[0], -1.0);
    approx(tri[1], 0.0);
    approx(tri[2], 1.0);
    approx(tri[3], 0.0);
}

#[test]
fn windows_sum_correctly() {
    let n = 8;
    let hann = hann_window(n);
    let sum: f32 = hann.iter().sum();
    approx(sum, (n as f32) / 2.0);

    let hamming = hamming_window(n);
    let blackman = blackman_window(n);
    assert!(hamming.iter().all(|v| *v >= 0.0));
    // Blackman can dip slightly negative; just ensure energy is positive.
    let energy: f32 = blackman.iter().map(|v| v * v).sum();
    assert!(energy > 0.0);
}
