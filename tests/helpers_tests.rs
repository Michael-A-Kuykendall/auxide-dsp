#![forbid(unsafe_code)]

use auxide_dsp::*;

fn approx(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{} != {}", a, b);
}

#[test]
fn helpers_pure() {
    approx(db_to_linear(0.0), 1.0);
    approx(db_to_linear(-6.0), 0.5012);
    approx(linear_to_db(1.0), 0.0);
    approx(freq_to_phase_increment(440.0, 44100.0), 2.0 * std::f32::consts::PI * 440.0 / 44100.0);
    assert_eq!(ms_to_samples(10.0, 48000.0), 480);
    let coeff = compute_exponential_coefficient(10.0, 48000.0);
    assert!(coeff > 0.0 && coeff < 1.0);
}

#[test]
fn polyblep_continuity() {
    // At discontinuity, polyblep should soften step
    let inc = 0.1;
    let pre = polyblep(0.0, inc);
    let mid = polyblep(inc * 0.5, inc);
    let post = polyblep(inc * 1.5, inc);
    assert!(pre.abs() < 1.0);
    assert!(mid.abs() < 1.0);
    assert!(post.abs() < 1.0);
}

#[test]
fn interpolate_wraps() {
    let buf = [0.0, 1.0, 0.0, -1.0];
    approx(linear_interpolate(&buf, 0.0), 0.0);
    approx(linear_interpolate(&buf, 1.0), 1.0);
    approx(linear_interpolate(&buf, 3.5), -0.5);
    approx(linear_interpolate(&buf, 4.0), 0.0);
}
