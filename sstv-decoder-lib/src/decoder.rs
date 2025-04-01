// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use core::f32;

use crate::hilbert::hilbert;

/// An decoder of various pcm samples. Get the instantaneous frequency of each samples.
pub fn decoder(sample_rate: f32, samples: &[f32]) -> Vec<f32> {
    // First, Hilbert Transform of the singles. To get the complex form of the single.
    let p = hilbert(samples);
    let mut instantaneous = vec![0.0; p.len() - 1];

    // Then calculate the instantaneous ferquency.
    for t in 1..p.len() {
        instantaneous[t - 1] = (p[t] * p[t - 1].conj()).arg() * sample_rate / f32::consts::TAU;
    }

    instantaneous
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_fm_demod() {
        let sample_rate = 8000.0;

        let freq = 1500.0;

        let t = (0..80).map(|i| i as f32 / sample_rate).collect::<Vec<_>>();
        let pcm_data: Vec<f32> = t
            .iter()
            .map(|&t| (2.0 * core::f32::consts::PI * (freq) * t).sin())
            //    .chain(
            //        t.iter()
            //            .map(|&t| (2.0 * core::f32::consts::PI * (freq * 2.0) * t).sin()),
            //    )
            .collect();

        let instantaneous_frequency = decoder(sample_rate, &pcm_data);

        println!("Instantaneous Frequency (Hz):");
        for (i, &freq) in instantaneous_frequency.iter().enumerate() {
            println!("Sample {}: {:.2}", i, freq);
        }
    }
}
