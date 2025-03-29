// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use std::i16;

use crate::{Mode, sample_generator::SampleGenerator};

///
/// Generate signals from bool.
///
/// In the header, 1100 Hz means '1', 1300 Hz means '0'.
/// And the true stands for 1, false stands for 0.
///
/// Refrence: Dayton Paper - VIS Code and Robot calibration header
///
fn generate_signals_from_bool(ctx: &mut SampleGenerator, f: bool) -> Vec<i16> {
    ctx.generate_samples(30.0, if f { 1100.0 } else { 1300.0 })
}

///
/// Generate the beginning of the header.
///
/// Refrence: Dayton Paper - VIS Code and Robot calibration header
///
pub(crate) fn generate_header(ctx: &mut SampleGenerator, mode: &Mode) -> Vec<i16> {
    let mode_vis = match mode {
        Mode::Scottie1 => [false, false, true, true, true, true, false],
        Mode::Scottie2 => [false, false, false, true, true, true, false],
        Mode::ScottieDx => [false, false, true, true, false, false, true],
        Mode::Martin1 => [false, false, true, true, false, true, false],
        Mode::Martin2 => [false, false, false, true, false, true, false],
        Mode::Robot36 => [false, false, false, true, false, false, false],
        Mode::Robot72 => [false, false, true, true, false, false, false],
        Mode::WrasseSc2_180 => [true, true, true, false, true, true, false],
        Mode::P3 => [true, false, false, false, true, true, true],
        Mode::P5 => [false, true, false, false, true, true, true],
        Mode::P7 => [true, true, false, false, true, true, true],
        Mode::Pd50 => [true, false, true, true, true, false, true],
        Mode::Pd90 => [true, true, false, false, false, true, true],
        Mode::Pd120 => [true, true, true, true, true, false, true],
        Mode::Pd180 => [false, false, false, false, false, true, true],
        Mode::Pd160 => [false, true, false, false, false, true, true],
        Mode::Pd240 => [true, false, false, false, false, true, true],
        Mode::Pd290 => [false, true, true, true, true, false, true],
    }
    .to_vec();

    let mut samples = vec![
        // Leader tone
        ctx.generate_samples(300.0, 1900.0),
        // Break
        ctx.generate_samples(10.0, 1200.0),
        // Leader tone
        ctx.generate_samples(300.0, 1900.0),
        // VIS start bit
        ctx.generate_samples(30.0, 1200.0),
    ];

    for f in &mode_vis {
        samples.extend([generate_signals_from_bool(ctx, *f)]);
    }

    samples.extend([
        // The seven-bit code is transmitted least-significant-bit (LSB) first, and uses “even” parity.
        generate_signals_from_bool(ctx, mode_vis.iter().filter(|f| **f).count() % 2 == 1),
        // VIS stop bit
        ctx.generate_samples(30.0, 1200.0),
    ]);

    samples.concat()
}
