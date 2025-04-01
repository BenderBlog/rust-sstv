// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

// Copied from https://crates.io/crates/hilbert_transform
// Written by Lucas da Costa (lcscosta), licensed under MIT License.

use rustfft::{FftPlanner, num_complex::Complex};

/// Hilbert_transform is a library written in Rust to perform the hilbert transformation like
/// Matlab/Octave or scipy.signals.hilbert.
///
/// Hilbert_transform is implemented based on scipy implementation of same function.
///
/// Input and output all become f32, instead of the original f64
pub fn hilbert(input: &[f32]) -> Vec<Complex<f32>> {
    let len = input.len();
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(len);

    let mut fft_complex = input
        .iter()
        .map(|&val| Complex::new(val, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_complex);

    let mut h_spectrum = vec![Complex::new(0.0, 0.0); len];

    if len % 2 == 0 {
        h_spectrum[0] = Complex::new(1.0, 0.0);
        h_spectrum[len / 2] = Complex::new(1.0, 0.0);
        for i in 1..(len / 2) {
            h_spectrum[i] = Complex::new(2.0, 0.0);
        }
    } else {
        h_spectrum[0] = Complex::new(1.0, 0.0);
        for i in 1..((len + 1) / 2) {
            h_spectrum[i] = Complex::new(2.0, 0.0);
        }
    }

    for i in 0..len {
        fft_complex[i] = fft_complex[i] * h_spectrum[i];
    }

    let mut ifft_complex = fft_complex.clone();
    let ifft = planner.plan_fft_inverse(len);
    ifft.process(&mut ifft_complex);

    for val in ifft_complex.iter_mut() {
        *val = *val / len as f32
    }

    ifft_complex
}
