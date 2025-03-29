// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use crate::{Mode, SampleGenerator, sstv_image::SSTVImage};

///
/// PASOKON “P” modes
///
/// VIS Code:
///
/// |Mode|Code in Decimal|
/// |----|-------|
/// | P3 | 113 |
/// | P5 | 114 |
/// | P7 | 115 |
///
/// Color Mode: RGB (1500-2300hz luminance range)
/// Scan Sequence: Red,Green, Blue,
///
/// Image Size: 640x496 (including 16-line header)
///
pub(crate) fn encode_in_pasokon(
    ctx: &mut SampleGenerator,
    image: &SSTVImage,
    mode: &Mode,
) -> Vec<i16> {
    let mut result = vec![];

    let pixel_scan_time = match mode {
        Mode::P3 => 0.2083,
        Mode::P5 => 0.3125,
        Mode::P7 => 0.4167,
        _ => 0.0, // TODO: Throw illegal call exception.
    };

    let sync_period = match mode {
        Mode::P3 => 5.208,
        Mode::P5 => 7.813,
        Mode::P7 => 10.417,
        _ => 0.0, // TODO: Throw illegal call exception.
    };

    let porch_periods = match mode {
        Mode::P3 => 1.042,
        Mode::P5 => 1.563,
        Mode::P7 => 2.083,
        _ => 0.0, // TODO: Throw illegal call exception.
    };

    let image_to_send = image.resize_image(640, 496);

    for y in 0..image_to_send.get_height() {
        // Step 1: The Sync Pulse
        result.extend(ctx.generate_samples(sync_period, 1200.0));

        // Step 2: The Sync Porch
        result.extend(ctx.generate_samples(porch_periods, 1500.0));

        // Step 3: The red scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[0]),
            )
        });

        // Step 4: The Sync Porch
        result.extend(ctx.generate_samples(porch_periods, 1500.0));

        // Step 5: The green scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[1]),
            )
        });

        // Step 6: The Sync Porch
        result.extend(ctx.generate_samples(porch_periods, 1500.0));

        // Step 7: The blue scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[2]),
            )
        });

        // Step 8: The Sync Porch
        result.extend(ctx.generate_samples(porch_periods, 1500.0));

        // Repeat the above sequence for 496 lines.
    }

    result
}
