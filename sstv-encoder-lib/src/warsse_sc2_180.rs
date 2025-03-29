// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use crate::{SampleGenerator, sstv_image::SSTVImage};

///
/// WRASSE SC2-180 mode
///
/// VIS Code:
///
/// |Mode | Code in Decimal|
/// |-----------------|-----|
/// | WRASSE SC2-180  | 55  |
///
/// Color Mode: RGB (1500-2300hz luminance range)
/// Scan Sequence: Red, Green, Blue
///
/// Image Size: 320x256 (including 16-line header)
///
pub(crate) fn encode_in_warsse_sc2_180(ctx: &mut SampleGenerator, image: &SSTVImage) -> Vec<i16> {
    let mut result = vec![];
    let pixel_scan_time = 0.7344;

    let image_to_send = image.resize_image(320, 256);

    for y in 0..image_to_send.get_height() {
        // Step 1: The sync pulse
        result.extend(ctx.generate_samples(5.5225, 1200.0));

        // Step 2: The sync porch
        result.extend(ctx.generate_samples(0.5, 1500.0));

        // Step 3: The red scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[0]),
            );
        });

        // Step 4: The green scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[1]),
            );
        });
        // Step 5: The blue scan
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_rgb_pixel(x, y)[2]),
            );
        });

        // Repeat the above sequence for all lines.
    }

    result
}
