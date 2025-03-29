// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use crate::{SampleGenerator, sstv_image::SSTVImage};

///
/// Robot 36 Mode
///
/// VIS Code:
///
/// |Mode|Code in Decimal|
/// |----------|-----|
/// | Robot 36 |  8  |
///
/// Color Mode: YRyBy (1500-2300hz luminance range)
/// Scan Sequence: Y, R-Y (even lines) Y, B-Y (odd lines)
///
/// Image Size: 320x240 (including 16-line header)
///
/// Reminders:
///  • The R-Y color information is averaged for two lines, and transmitted on even lines.
///  • The B-Y color information is averaged for two lines, and transmitted on odd lines.
///  • The R-Y and B-Y scans have only 1/2 the period (44ms) of the Y scan. (88ms)
///  • Even lines use a 1500hz “separator” pulse, while odd lines use 2300hz.
///
pub(crate) fn encode_in_robot36(ctx: &mut SampleGenerator, image: &SSTVImage) -> Vec<i16> {
    let mut result = vec![];

    let image_to_send = image.resize_image(320, 240);

    for y in (0..image_to_send.get_height()).step_by(2) {
        // Step 1: Sync Pulse
        result.extend(ctx.generate_samples(9.0, 1200.0));

        // Step 2: Sync Porch
        result.extend(ctx.generate_samples(3.0, 1500.0));

        // Step 3: Odd line Y scan, total time 88ms
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(ctx.generate_color_samples(
                88.0 / image_to_send.get_width() as f32,
                image_to_send.get_ycrcb_pixel(x, y)[0],
            ));
        });

        // Step 4: "Even" separator pulse
        result.extend(ctx.generate_samples(4.5, 1500.0));

        // Step 5: Porch
        result.extend(ctx.generate_samples(1.5, 1900.0));

        // Step 6: R-Y scan, total time 44ms
        (0..image_to_send.get_width()).for_each(|x| {
            let odd_pixel = image_to_send.get_ycrcb_pixel(x, y)[1];
            let even_pixel = image_to_send.get_ycrcb_pixel(x, y + 1)[1];
            let avg_freq = ((odd_pixel as u16 + even_pixel as u16) >> 1) as u8;
            result.extend(
                ctx.generate_color_samples(44.0 / image_to_send.get_width() as f32, avg_freq),
            );
        });

        // Step 7: Sync Pulse
        result.extend(ctx.generate_samples(9.0, 1200.0));

        // Step 8: Sync Porch
        result.extend(ctx.generate_samples(3.0, 1500.0));

        // Step 9: Even Y scan, total time 88ms
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(ctx.generate_color_samples(
                88.0 / image_to_send.get_width() as f32,
                image_to_send.get_ycrcb_pixel(x, y + 1)[0],
            ));
        });

        // Step 10: Separator Pulse
        result.extend(ctx.generate_samples(4.5, 2300.0));

        // Step 11: Porch
        result.extend(ctx.generate_samples(1.5, 1500.0));

        // Step 12: B-Y scan, total time 69ms
        (0..image_to_send.get_width()).for_each(|x| {
            let odd_pixel = image_to_send.get_ycrcb_pixel(x, y)[2];
            let even_pixel = image_to_send.get_ycrcb_pixel(x, y + 1)[2];
            let avg_freq = ((odd_pixel as u16 + even_pixel as u16) >> 1) as u8;
            result.extend(
                ctx.generate_color_samples(44.0 / image_to_send.get_width() as f32, avg_freq),
            );
        });

        // Repeat the above sequence for 240 lines.
    }

    result
}
