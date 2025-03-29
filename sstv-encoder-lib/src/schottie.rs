// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use crate::{Mode, SampleGenerator, sstv_image::SSTVImage};

///
/// Scottie mode
///
/// VIS Code:
///
/// |Mode|Code in Decimal|
/// |------------|-------|
/// | Scottie 1  |   60  |
/// | Scottie 2  |   56  |
/// | Scottie DX |   76  |
///
/// Color Mode: RGB (1500-2300hz luminance range)
/// Scan Sequence: Green, Blue, Red
///
/// Image Size: 320x256 (including 16-line header)
///
pub(crate) fn encode_in_schottie(
    ctx: &mut SampleGenerator,
    image: &SSTVImage,
    mode: &Mode,
) -> Vec<i16> {
    let mut result = vec![];
    let pixel_scan_time = match mode {
        Mode::Scottie1 => 0.4320,
        Mode::Scottie2 => 0.2752,
        Mode::ScottieDx => 1.08,
        _ => 0.0, // TODO: Throw illegal call exception.
    };

    let image_to_send = image.resize_image(320, 256);

    // Step 1: Add out-sync "Starting" sync pulse (first line only!)
    result.extend(ctx.generate_samples(9.0, 1200.0));

    for y in 0..image_to_send.get_height() {
        // Step 2: The separator pulse
        result.extend(ctx.generate_samples(1.5, 1500.0));

        // Step 3: The green scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[1]),
            )
        });

        // Step 4: The separator pulse
        result.extend(ctx.generate_samples(1.5, 1500.0));

        // Step 5: The blue scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[2]),
            )
        });
        // Step 6: The sync pulse
        result.extend(ctx.generate_samples(9.0, 1200.0));

        // Step 7: The sync porch
        result.extend(ctx.generate_samples(1.5, 1500.0));

        // Step 8: The red scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[0]),
            )
        });

        // Repeat step 2 to step 8 for 256 lines.
    }

    result
}
