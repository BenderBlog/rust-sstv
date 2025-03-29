// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use crate::{Mode, SampleGenerator, sstv_image::SSTVImage};

///
/// Martin mode
///
/// VIS Code:
///
/// |Mode|Code in Decimal|
/// |------------|-------|
/// | Martin 1  |   44  |
/// | Martin 2  |   40  |
///
/// Color Mode: RGB (1500-2300hz luminance range)
/// Scan Sequence: Green, Blue, Red
///
/// Image Size: 320x256 (including 16-line header)
///
pub(crate) fn encode_in_martin(
    ctx: &mut SampleGenerator,
    image: &SSTVImage,
    mode: &Mode,
) -> Vec<i16> {
    let mut result = vec![];

    let pixel_scan_time = match mode {
        Mode::Martin1 => 0.4576,
        Mode::Martin2 => 0.2288,
        _ => 0.0, // TODO: Throw illegal call exception.
    };

    let image_to_send = image.resize_image(320, 256);

    for y in 0..image_to_send.get_height() {
        // Step 1: The Sync Pulse
        result.extend(ctx.generate_samples(4.862, 1200.0));

        // Step 2: The Sync Porch
        result.extend(ctx.generate_samples(0.572, 1500.0));

        // Step 3: The green scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[1]),
            )
        });

        // Step 4: The separator pulse
        result.extend(ctx.generate_samples(0.572, 1500.0));

        // Step 5: The blue scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[2]),
            )
        });

        // Step 6: The separator pulse
        result.extend(ctx.generate_samples(0.572, 1500.0));

        // Step 7: The red scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[0]),
            )
        });

        // Step 8: The separator pulse
        result.extend(ctx.generate_samples(0.572, 1500.0));

        // Repeat the above sequence for 256 lines.
    }

    result
}
