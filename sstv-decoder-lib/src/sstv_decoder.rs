// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use std::{
    collections::VecDeque,
    f32::consts::PI,
    fs::File,
    io::{BufWriter, Write},
    vec,
};

use image::{Rgb, RgbImage};

use crate::{bandpass_filter::bandpass_filter, hilbert::hilbert_transform, mode::Mode};

fn get_sample_length_in_ms(sample_rate: f32) -> f32 {
    sample_rate / 1000.0
}

pub struct SSTVDecoder {
    pub mode: Mode,
    pub sample_rate: f32,
    /// Need store 1000ms data.
    pub sample_queue: Box<VecDeque<f32>>,
    /// Begin of the VIS Sample:
    ///    Leader tone 1900Hz 300ms
    ///    Break 1200Hz 10ms
    ///    Leader tone 1900Hz 300ms
    header_sample_num: usize,
    /// For vis singles, total transmission time is 30 * 10 = 300ms
    vis_sample_num: usize,
    /// Optional video stuff.
    picture: Vec<Vec<[u8; 3]>>,
    counter: usize,
    /// Debug file to store sample data
    writer: BufWriter<File>,
    /// Previous phase.
    prev: f32,
}

impl SSTVDecoder {
    pub fn new(sample_rate: f32) -> Self {
        let file = std::fs::File::create("inst_freq_sliding_hilbert.csv").unwrap();
        SSTVDecoder {
            mode: Mode::None,
            sample_rate: sample_rate,
            sample_queue: Box::new(VecDeque::new()),
            header_sample_num: (610.0 * get_sample_length_in_ms(sample_rate)) as usize,
            vis_sample_num: (300.0 * get_sample_length_in_ms(sample_rate)) as usize,
            picture: vec![vec![]],
            counter: 0,
            writer: std::io::BufWriter::new(file),
            prev: 0.0,
        }
    }

    pub fn switch_sample(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.mode = Mode::None;
        self.sample_queue.clear();
        self.header_sample_num = (610.0 * get_sample_length_in_ms(sample_rate)) as usize;
        self.vis_sample_num = (300.0 * get_sample_length_in_ms(sample_rate)) as usize;
        self.picture = vec![vec![]];
        self.counter = 0;
        self.prev = 0.0;
    }

    /// Decoder
    fn decoder(&mut self, samples: &[f32]) -> Vec<f32> {
        let samples = bandpass_filter(samples, self.sample_rate);
        let z = hilbert_transform(&samples);
        let mut phase: Vec<f32> = z.iter().map(|c| c.arg()).collect();
        for i in 0..phase.len() {
            let mut dp = phase[i] - self.prev;
            while dp > PI {
                dp -= 2.0 * PI;
            }
            while dp < -PI {
                dp += 2.0 * PI;
            }
            phase[i] = self.prev + dp;
            self.prev = phase[i];
        }
        let to_return = phase
            .windows(2)
            .map(|w| {
                let to_return = (self.sample_rate * (w[1] - w[0]) / (2.0 * PI)).abs();
                if to_return > 3000.0 {
                    3000.0
                } else if to_return < 1000.0 {
                    1000.0
                } else {
                    to_return
                }
            })
            .collect();
        // let f = std::fs::File::create_new("inst_freq_hilbert.csv");
        // let mut w = std::io::BufWriter::new(f);
        for &f in &to_return {
            writeln!(self.writer, "{:.2}", f).unwrap();
        }
        to_return
    }

    /// Check the header
    fn get_header(&mut self, frequency_data: &[f32]) -> bool {
        let leader_duration_sample = (300.0 * get_sample_length_in_ms(self.sample_rate)) as usize;
        let break_duration_sample = (10.0 * get_sample_length_in_ms(self.sample_rate)) as usize;

        if frequency_data.len() != self.header_sample_num {
            eprintln!("Error: PCM data length is not equal to a VIS header's length.");
            return false;
        }

        let first_leader_tone = &frequency_data[0..leader_duration_sample]
            .iter()
            .sum::<f32>()
            / leader_duration_sample as f32;
        let break_tone_freq = &frequency_data
            [leader_duration_sample..leader_duration_sample + break_duration_sample]
            .iter()
            .sum::<f32>()
            / break_duration_sample as f32;
        let second_leader_tone = &frequency_data[leader_duration_sample + break_duration_sample
            ..2 * leader_duration_sample + break_duration_sample]
            .iter()
            .sum::<f32>()
            / leader_duration_sample as f32;

        println!(
            "first_leader_tone: {}, break_tone_freq: {}, second_leader_tone: {}",
            first_leader_tone, break_tone_freq, second_leader_tone
        );

        (first_leader_tone - 1900.0).abs() <= 50.0
            && (break_tone_freq - 1200.0) <= 50.0
            && (second_leader_tone - 1900.0) <= 50.0
    }

    /// Check the vis
    fn decode_vis(&mut self, frequency_data: &[f32]) -> Mode {
        if frequency_data.len() != self.vis_sample_num {
            eprintln!("Error: PCM data length is not equal to a VIS signal's length.");
            return Mode::None;
        }

        // Check the bitstream.
        // 1 as true, 0 as false
        let bit_size = self.vis_sample_num / 10;

        let mut vis_code: u8 = 0;
        let mut true_count: u8 = 0;

        for bit_index in 1..8 {
            let section = if (&frequency_data[bit_index * bit_size..(bit_index + 1) * bit_size]
                .iter()
                .sum::<f32>()
                / bit_size as f32
                - 1100.0)
                <= 50.0
            {
                true
            } else {
                false
            };
            println!(
                "{} bit is {} from {}.",
                bit_index,
                section,
                (&frequency_data[bit_index * bit_size..(bit_index + 1) * bit_size])
                    .iter()
                    .sum::<f32>()
                    / bit_size as f32
            );

            if section {
                true_count += 1;
            }

            vis_code += if section { 1 } else { 0 } << (bit_index - 1);
        }
        let parity = ((&frequency_data[9 * bit_size..10 * bit_size])
            .iter()
            .sum::<f32>()
            / bit_size as f32
            - 1100.0)
            .abs()
            <= 50.0;

        if (true_count % 2 == 1) != parity {
            eprintln!(
                "Error decoding VIS header (invalid parity bit). {:} {:}",
                vis_code, parity
            );
            return Mode::None;
        }

        let mode = match vis_code {
            //   60 => Mode::Scottie1,
            //   56 => Mode::Scottie2,
            //   76 => Mode::ScottieDx,
            //  44 => Mode::Martin1,
            //   40 => Mode::Martin2,
            //   8 => Mode::Robot36,
            //  12 => Mode::Robot72,
            //   55 => Mode::WrasseSc2_180,
            //   113 => Mode::P3,
            //   114 => Mode::P5,
            //   115 => Mode::P7,
            //   93 => Mode::Pd50,
            //   99 => Mode::Pd90,
            95 => Mode::Pd120,
            //   98 => Mode::Pd160,
            //   96 => Mode::Pd180,
            //   97 => Mode::Pd240,
            //   94 => Mode::Pd290,
            _ => Mode::None,
        };
        println!("VIS Mode is {:?}, code {}.", mode, vis_code);

        mode
    }

    /// Get line info
    fn decode_line_info(&mut self, frequency_data: &[f32], pixels: usize) -> Vec<u8> {
        let mut decoded_pixel = vec![0; pixels];
        let freq = &frequency_data;
        let freq_per_pixel = freq.len() / pixels;
        for i in 0..pixels - 1 {
            let freq = freq[i * freq_per_pixel..(i + 1) * freq_per_pixel]
                .iter()
                .sum::<f32>()
                / freq_per_pixel as f32;
            decoded_pixel[i] = ((freq - 1500.0) / 800.0) as u8;
        }
        decoded_pixel
    }

    /// Decode the stream of data, by adding 1ms data.
    pub fn decode(&mut self, pcm_data: &[f32]) {
        if pcm_data.len() == 0 {
            eprintln!("PCM data is empty!");
            return;
        }

        // Sample the frequency
        let decoded_data = self.decoder(pcm_data);
        self.sample_queue.extend(decoded_data);

        // If no mod and sample_time is 690
        if self.mode == Mode::None && self.sample_queue.len() >= self.header_sample_num {
            if self.get_header(
                &self
                    .sample_queue
                    .range(0..(self.header_sample_num))
                    .cloned()
                    .collect::<Vec<f32>>(),
            ) {
                self.mode = Mode::VisFind;
                for _i in 0..self.header_sample_num {
                    self.sample_queue.pop_front();
                }
            }
            println!("After VIS check, current mode: {:?}", self.mode);
            println!(
                "Length of buffer {}, minus {}",
                self.sample_queue.len(),
                self.header_sample_num,
            );
            return;
        }

        if self.mode == Mode::VisFind && self.sample_queue.len() >= self.vis_sample_num {
            self.mode = self.decode_vis(
                &self
                    .sample_queue
                    .range(0..(self.vis_sample_num))
                    .cloned()
                    .collect::<Vec<f32>>(),
            );
            for _i in 0..self.vis_sample_num {
                self.sample_queue.pop_front();
            }
            if self.mode != Mode::None {
                self.picture = vec![vec![[0; 3]; 640]; 496].into();
            }
            println!("After VIS check, current mode: {:?}", self.mode);
            println!(
                "Length of buffer {}, minus {}",
                self.sample_queue.len(),
                self.vis_sample_num,
            );
            return;
        }

        if self.mode == Mode::Pd120
            && self.sample_queue.len()
                >= ((121.6 * 4.0 + 20.0 + 2.08) * get_sample_length_in_ms(self.sample_rate))
                    as usize
        {
            self.decode_in_pd120();
            for _i in 0..((121.6 * 4.0 + 20.0 + 2.08) * get_sample_length_in_ms(self.sample_rate))
                as usize
            {
                self.sample_queue.pop_front();
            }
            println!(
                "Length of buffer {}, minus {}",
                self.sample_queue.len(),
                ((121.6 * 4.0 + 20.0 + 2.08) * get_sample_length_in_ms(self.sample_rate)),
            );
            return;
        }

        println!("Current buffer length {}", self.sample_queue.len());
        return;
    }

    fn decode_in_pd120(&mut self) {
        if self.counter >= 494 {
            self.counter = 0;
            self.mode = Mode::None;

            let height = self.picture.len();
            let width = if height > 0 { self.picture[0].len() } else { 0 };
            let mut img = RgbImage::new(width as u32, height as u32);
            for (y, row) in self.picture.iter().enumerate() {
                for (x, &rgb) in row.iter().enumerate() {
                    img.put_pixel(x as u32, y as u32, Rgb(rgb));
                }
            }
            img.save("pic.png")
                .unwrap_or_else(|x| println!("Store failed. {:?}", x));

            println!("Finish Decoding!");
        }

        let time = 121.6 * 4.0 + 20.0 + 2.08;
        let pixel_count = 640;
        let sample_count = (time * get_sample_length_in_ms(self.sample_rate) as f32) as usize;
        if self.sample_queue.len() < sample_count {
            eprintln!("Error: PCM data length is not equal to a PD120 line.");
            return;
        }

        println!("line {} decoding", self.counter);

        let data_to_parse = &self
            .sample_queue
            .range(0..sample_count)
            .cloned()
            .collect::<Vec<f32>>();

        // 0-20-22.08-143.58-265.28-386.88-end
        let division = [
            0,
            (20.0 * get_sample_length_in_ms(self.sample_rate)) as usize,
            (22.08 * get_sample_length_in_ms(self.sample_rate)) as usize,
            (143.58 * get_sample_length_in_ms(self.sample_rate)) as usize,
            (265.28 * get_sample_length_in_ms(self.sample_rate)) as usize,
            (386.88 * get_sample_length_in_ms(self.sample_rate)) as usize,
            data_to_parse.len(),
        ];

        //  let _sync = self.decoder(&data_to_parse[division[0]..division[1]], true);
        //  let _porch = self.decoder(&data_to_parse[division[1]..division[2]], true);

        let line_y1 = self.decode_line_info(&data_to_parse[division[2]..division[3]], pixel_count);
        let line_ry = self.decode_line_info(&data_to_parse[division[3]..division[4]], pixel_count);
        let line_by = self.decode_line_info(&data_to_parse[division[4]..division[5]], pixel_count);
        let line_y2 = self.decode_line_info(&data_to_parse[division[5]..division[6]], pixel_count);

        for i in 0..pixel_count {
            let by_minus_128 = line_by[i] as f32 - 128.0;
            let ry_minus_128 = line_ry[i] as f32 - 128.0;
            let y1_minus_16_mul_298_082 = (line_y1[i] as f32 - 16.0) * 298.082;
            let y2_minus_16_mul_298_082 = (line_y2[i] as f32 - 16.0) * 298.082;
            self.picture[self.counter][i] = [
                (0.003906 * (y1_minus_16_mul_298_082 + 408.583 * ry_minus_128)) as u8,
                (0.003906
                    * (y1_minus_16_mul_298_082 + -100.291 * by_minus_128 + -208.12 * ry_minus_128))
                    as u8,
                (0.003906 * (y1_minus_16_mul_298_082 + 516.411 * by_minus_128)) as u8,
            ];
            self.picture[self.counter + 1][i] = [
                (0.003906 * (y2_minus_16_mul_298_082 + 408.583 * ry_minus_128)) as u8,
                (0.003906
                    * (y2_minus_16_mul_298_082 + -100.291 * by_minus_128 + -208.12 * ry_minus_128))
                    as u8,
                (0.003906 * (y2_minus_16_mul_298_082 + 516.411 * by_minus_128)) as u8,
            ];
        }

        self.counter += 2;
    }
}
