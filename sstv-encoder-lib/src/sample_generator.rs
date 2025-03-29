// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

///
/// SampleGenerator is a generator to generate constant stream of pcm wave data.
///
pub struct SampleGenerator {
    /// The sample rate of the result wave file
    sample_rate: u32,

    /// The older amplitude, for continuous phase
    older_data: f32,

    /// The older cosine phase, for continuous phase
    older_cos: f32,

    /// The delta of the length, for compensating the precision related to the sample rate
    delta_length: f32,
}

impl SampleGenerator {
    fn sign(&self, num: f32) -> f32 {
        if num >= 0.0 {
            return 1.0;
        } else if num < 0.0 {
            return -1.0;
        } else {
            return 0.0;
        }
    }

    /// Create a new sample generator.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate,
            older_data: 0.0,
            older_cos: 0.0,
            delta_length: 0.0,
        }
    }

    ///
    /// Generate pcm wave data from color signal strength.
    ///
    /// Note that the library uses 8 bit unsigned integer value to store the color signal
    /// strength, and the range of the color frequency is [1500,2300].
    ///
    /// Refrence is at below:
    ///
    /// > Dayton Paper Appendix A: RGB Color Encoding
    /// >
    /// > SSTV systems use the frequency range of 1500-2300hz to represent
    /// > the range of brightness values from pure black to pure white.
    /// > Because of this, we must convert out 8-bit R,G,B values into the
    /// > appropriate frequency.
    /// >
    /// > In this formula, the RGB value is stored in 8 bit non-signed integer.
    /// >
    /// > $$ f = 1500 + s_r * 3.1372549 $$
    /// >
    /// > Where $f$ is the frequency of the strength of the value $s_r$.
    ///
    /// > Dayton Paper Appendix B: YRyBy (YCrCb) Color Encoding
    /// >
    /// > Again, as with RGB encoding, these values can be converted to frequency, for
    /// > SSTV transmission:
    /// >
    /// > $$ f = 1500 + v * 3.1372549 $$
    /// >
    /// > Where 'v' is the value of the Y, R-Y, or B-Y
    ///
    pub(crate) fn generate_color_samples(&mut self, duration_in_ms: f32, strength: u8) -> Vec<i16> {
        let frequency = 1500.0 + (strength as f32) * 800.0 / 255.0;
        self.generate_samples(duration_in_ms, frequency)
    }

    ///
    /// Generate pcm wave data.
    ///
    pub(crate) fn generate_samples(&mut self, duration_in_ms: f32, frequency: f32) -> Vec<i16> {
        // Count the amount of the sample rates
        let mut num_samples: i32 =
            (self.sample_rate as f32 * duration_in_ms / 1000.0).floor() as i32;

        // Compensating the precision if needed
        self.delta_length +=
            self.sample_rate as f32 * duration_in_ms / 1000.0 - (num_samples as f32);
        if self.delta_length >= 1.0 {
            num_samples += self.delta_length.floor() as i32;
            self.delta_length -= self.delta_length.floor();
        }

        // Generate phi sample
        let phi_samples = self.sample_rate as f32
            * (self.sign(self.older_cos) * self.older_data.asin()
                + (self.sign(self.older_cos) - 1.0).abs() / 2.0 * core::f32::consts::PI);

        // Write samples
        (0..num_samples)
            .map(move |tick| {
                // Remember older_data and older_sum
                self.older_data =
                    ((2.0 * core::f32::consts::PI * frequency * (num_samples as f32)
                        + phi_samples)
                        / self.sample_rate as f32)
                        .sin();
                self.older_cos = ((2.0 * core::f32::consts::PI * frequency * (num_samples as f32)
                    + phi_samples)
                    / self.sample_rate as f32)
                    .cos();

                (32767.0
                    * ((2.0 * core::f32::consts::PI * frequency * (tick as f32) + phi_samples)
                        / self.sample_rate as f32)
                        .sin())
                .floor() as i16
            })
            .collect()
    }
}
