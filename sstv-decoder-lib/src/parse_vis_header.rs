// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

/// Get average frequency in a sample.
fn get_average_frequency(frequency_data: &[f32]) -> f32 {
    frequency_data.iter().sum::<f32>() / frequency_data.len() as f32
}

fn get_sample_length_in_ms(sample_rate: f32) -> usize {
    (1.0 / sample_rate * 1000.0) as usize
}
/*
/// Check the header
fn get_header(frequency_data: &[f32], sample_rate: f32) -> bool {
    // Begin of the VIS Sample:
    //    Leader tone 1900Hz 300ms
    //    Break 1200Hz 10ms
    //    Leader tone 1900Hz 300ms
    let vis_samples = (sample_rate * vis_duration_ms / 1000.0) as usize;

    let first_leader_tone = frequency_data[0..];

    false
}
*/
/// Check the vis
fn decode_vis(frequency_data: &[f32], sample_rate: f32) -> Option<u8> {
    // For vis singles, total transmission time is 30 * 10 = 300ms
    // Apart from the first VIS Start bit, it should be 30 * 9 =  270ms
    let vis_duration_ms = 300.0;
    let vis_samples = (sample_rate * vis_duration_ms / 1000.0) as usize;

    if frequency_data.len() != vis_samples {
        eprintln!("Error: PCM data length is not equal to a VIS signal's length.");
        return None;
    }

    // Check the bitstream.
    // 1 as true, 0 as false
    let bit_size = vis_samples / 10;

    let mut vis_code: u8 = 0;
    let mut true_count: u8 = 0;

    for bit_index in 1..8 {
        let section = if get_average_frequency(
            &frequency_data[bit_index * bit_size..(bit_index + 1) * bit_size],
        ) <= 1200.0
        {
            true
        } else {
            false
        };

        if section {
            true_count += 1;
        }

        vis_code += if section { 1 } else { 0 } << (bit_index - 1);
    }
    let parity = get_average_frequency(&frequency_data[9 * bit_size..9 * bit_size]) <= 1200.0;
    if (true_count % 2 == 1) != parity {
        eprintln!("Error decoding VIS header (invalid parity bit).");
        return None;
    }

    Some(vis_code)
}
