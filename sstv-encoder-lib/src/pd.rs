// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use crate::{Mode, SampleGenerator, sstv_image::SSTVImage};

///
/// PD Modes
///
/// VIS Code:
///
/// |Mode|Code in Decimal|
/// |-------|----|
/// | PD50  | 93 |
/// | PD90  | 99 |
/// | PD120 | 95 |
/// | PD160 | 98 |
/// | PD180 | 96 |
/// | PD240 | 97 |
/// | PD290 | 94 |
///
/// Color Mode: YRyBy
/// Scan Sequence: Y, Ry, By
///
/// Image Size: Varies
///
pub(crate) fn encode_in_pd(ctx: &mut SampleGenerator, image: &SSTVImage, mode: &Mode) -> Vec<i16> {
    let mut result = vec![];

    let pixels_in_line_count: usize = match mode {
        Mode::Pd50 => 320,
        Mode::Pd90 => 320,
        Mode::Pd120 => 640,
        Mode::Pd180 => 512,
        Mode::Pd160 => 640,
        Mode::Pd240 => 640,
        Mode::Pd290 => 800,
        _ => 0,
    };
    let lines_count: usize = match mode {
        Mode::Pd50 => 256,
        Mode::Pd90 => 256,
        Mode::Pd120 => 496,
        Mode::Pd180 => 400,
        Mode::Pd160 => 496,
        Mode::Pd240 => 496,
        Mode::Pd290 => 616,
        _ => 0,
    };
    let pixel_scan_time = match mode {
        Mode::Pd50 => 91.52,
        Mode::Pd90 => 170.24,
        Mode::Pd120 => 121.6,
        Mode::Pd160 => 195.584,
        Mode::Pd180 => 183.04,
        Mode::Pd240 => 244.48,
        Mode::Pd290 => 228.8,
        _ => 0.0,
    } / (pixels_in_line_count as f32);

    let image_to_send = image.resize_image(pixels_in_line_count, lines_count);

    for y in (0..image_to_send.get_height()).step_by(2) {
        // Step 1: Sync Pulse
        result.extend(ctx.generate_samples(20.0, 1200.0));

        // Step 2: Porch
        result.extend(ctx.generate_samples(2.08, 1500.0));

        // Step 3: Y scan from the odd line
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(
                ctx.generate_color_samples(pixel_scan_time, image_to_send.get_ycrcb_pixel(x, y)[0]),
            );
        });

        // Step 4: R-Y scan averaged for two lines
        (0..image_to_send.get_width()).for_each(|x| {
            let odd_pixel = image_to_send.get_ycrcb_pixel(x, y)[1];
            let even_pixel = image_to_send.get_ycrcb_pixel(x, y + 1)[1];
            let avg_freq = ((odd_pixel as u16 + even_pixel as u16) >> 1) as u8;
            result.extend(ctx.generate_color_samples(pixel_scan_time, avg_freq));
        });

        // Step 5: B-Y scan averaged for two lines
        (0..image_to_send.get_width()).for_each(|x| {
            let odd_pixel = image_to_send.get_ycrcb_pixel(x, y)[2];
            let even_pixel = image_to_send.get_ycrcb_pixel(x, y + 1)[2];
            let avg_freq = ((odd_pixel as u16 + even_pixel as u16) >> 1) as u8;
            result.extend(ctx.generate_color_samples(pixel_scan_time, avg_freq));
        });

        // Step 6: Y scan from the even line
        (0..image_to_send.get_width()).for_each(|x| {
            result.extend(ctx.generate_color_samples(
                pixel_scan_time,
                image_to_send.get_ycrcb_pixel(x, y + 1)[0],
            ));
        });

        // Repeat the above sequence for 240 lines.
    }

    result
}
