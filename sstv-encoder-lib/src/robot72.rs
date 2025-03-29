// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use crate::{SampleGenerator, sstv_image::SSTVImage};

///
/// Robot 72 Mode
///
/// VIS Code:
///
/// |Mode|Code in Decimal|
/// |----------|----|
/// | Robot 72 | 12 |
///
/// Color Mode: YRyBy (1500-2300hz luminance range)
/// Scan Sequence: Y, Ry, By
///
/// Image Size: 320x240 (including 16-line header)
///
pub(crate) fn encode_in_robot72(ctx: &mut SampleGenerator, image: &SSTVImage) -> Vec<i16> {
    let mut result = vec![];

    let image_to_send = image.resize_image(320, 240);

    for y in 0..image_to_send.get_height() {
        // Step 1: Sync Pulse
        result.extend(ctx.generate_samples(9.0, 1200.0));

        // Step 2: Sync Porch
        result.extend(ctx.generate_samples(3.0, 1500.0));

        // Step 3: Y scan, total time 138ms
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(ctx.generate_color_samples(
                138.0 / image_to_send.get_width() as f32,
                image_to_send.get_ycrcb_pixel(x, y)[0],
            ));
        });

        // Step 6: Separator Pulse
        result.extend(ctx.generate_samples(4.5, 1500.0));

        // Step 7: Porch
        result.extend(ctx.generate_samples(1.5, 1900.0));

        // Step 8: R-Y scan, total time 69ms
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(ctx.generate_color_samples(
                69.0 / image_to_send.get_width() as f32,
                image_to_send.get_ycrcb_pixel(x, y)[1],
            ));
        });

        // Step 9: Separator Pulse
        result.extend(ctx.generate_samples(4.5, 2300.0));

        // Step 10: Porch
        result.extend(ctx.generate_samples(1.5, 1500.0));

        // Step 11: B-Y scan, total time 69ms
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(ctx.generate_color_samples(
                69.0 / image_to_send.get_width() as f32,
                image_to_send.get_ycrcb_pixel(x, y)[2],
            ));
        });

        // Repeat the above sequence for 240 lines.
    }

    result
}
