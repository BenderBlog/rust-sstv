// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

use std::collections::VecDeque;

use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum, scaling::divide_by_N_sqrt};

use crate::mode::Mode;

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
}

impl SSTVDecoder {
    pub fn new(sample_rate: f32) -> Self {
        SSTVDecoder {
            mode: Mode::None,
            sample_rate: sample_rate,
            sample_queue: Box::new(VecDeque::new()),
            header_sample_num: (610.0 * get_sample_length_in_ms(sample_rate)) as usize,
            vis_sample_num: (300.0 * get_sample_length_in_ms(sample_rate)) as usize,
            picture: vec![vec![]],
            counter: 0,
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
    }

    /// Decoder
    fn decoder(&self, samples: &[f32]) -> f32 {
        // Adding zeros to make the length the power of 2
        // When sample rate is 96000 hz, 1ms data contains 96 samples.
        // And as far as I know, the longest sample to be dealt is 300ms.
        // Which is 28800 samples, below the 2^15(32678).
        // So this is the most samples this library can be dealt. Over.

        let len: usize = samples.len();
        let mut to_deal = Vec::from(samples);
        if len < 16 {
            to_deal.resize(16, 0.0);
        } else if len < 32 {
            to_deal.resize(32, 0.0);
        } else if len < 64 {
            to_deal.resize(64, 0.0);
        } else if len < 128 {
            to_deal.resize(128, 0.0);
        } else if len < 256 {
            to_deal.resize(256, 0.0);
        } else if len < 512 {
            to_deal.resize(512, 0.0);
        } else if len < 1024 {
            to_deal.resize(1024, 0.0);
        } else if len < 2048 {
            to_deal.resize(2048, 0.0);
        } else if len < 4096 {
            to_deal.resize(4096, 0.0);
        } else if len < 8192 {
            to_deal.resize(8192, 0.0);
        } else if len < 16384 {
            to_deal.resize(16384, 0.0);
        } else if len < 32768 {
            to_deal.resize(32768, 0.0);
        } else {
            panic!("Too many samples to handle")
        }
        //Hope there's no more, 16384 is the limit of the library.

        let spectrum_hann_window = samples_fft_to_spectrum(
            &to_deal,
            self.sample_rate as u32,
            FrequencyLimit::Range(1000.0, 3000.0),
            Some(&divide_by_N_sqrt),
        )
        .unwrap();

        // Temporarity, use max, but freq_val_closest and freq_val_exact shall be considered in the future.
        spectrum_hann_window.max().0.val()
    }

    /// Check the header
    fn get_header(&self, frequency_data: &[f32]) -> bool {
        let leader_duration_sample = (300.0 * get_sample_length_in_ms(self.sample_rate)) as usize;
        let break_duration_sample = (10.0 * get_sample_length_in_ms(self.sample_rate)) as usize;

        if frequency_data.len() != self.header_sample_num {
            eprintln!("Error: PCM data length is not equal to a VIS header's length.");
            return false;
        }

        let first_leader_tone = self.decoder(&frequency_data[0..leader_duration_sample]);
        let break_tone_freq = self.decoder(
            &frequency_data[leader_duration_sample..leader_duration_sample + break_duration_sample],
        );
        let second_leader_tone = self.decoder(
            &frequency_data[leader_duration_sample + break_duration_sample
                ..2 * leader_duration_sample + break_duration_sample],
        );
        //println!(
        //    "first_leader_tone: {}, break_tone_freq: {}, second_leader_tone: {}",
        //    first_leader_tone, break_tone_freq, second_leader_tone
        //);

        (first_leader_tone - 1900.0).abs() <= 50.0
            && (break_tone_freq - 1200.0) <= 50.0
            && (second_leader_tone - 1900.0) <= 50.0
    }

    /// Check the vis
    fn decode_vis(&self, frequency_data: &[f32]) -> Mode {
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
            let section = if self
                .decoder(&frequency_data[bit_index * bit_size..(bit_index + 1) * bit_size])
                <= 1200.0
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
        let parity = self.decoder(&frequency_data[9 * bit_size..10 * bit_size]) <= 1200.0;
        if (true_count % 2 == 1) != parity {
            eprintln!("Error decoding VIS header (invalid parity bit).");
            return Mode::None;
        }

        match vis_code {
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
        }
    }

    /// Get line info
    fn decode_line_info(&self, frequency_data: &[f32], pixels: usize) -> Vec<u8> {
        let sample_per_pixel = frequency_data.len()
            / ((get_sample_length_in_ms(self.sample_rate) * pixels as f32) as usize);
        let mut decoded_pixel = vec![0; pixels];
        for i in 0..pixels - 1 {
            let freq =
                self.decoder(&frequency_data[i * sample_per_pixel..(i + 1) * sample_per_pixel]);
            decoded_pixel[i] = ((freq - 1500.0) / 800.0) as u8;
        }
        decoded_pixel
    }

    /// Decode the stream of data, by adding 1ms data.
    pub fn decode(&mut self, pcm_data_in_1_ms: &[f32]) {
        if pcm_data_in_1_ms.len() == 0 {
            eprintln!("PCM data is empty!");
            return;
        }

        // Sample the frequency
        self.sample_queue.extend(pcm_data_in_1_ms);

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
                self.sample_queue.len(), self.header_sample_num,
            );
            //  if self.sample_queue.len() % 100000 == 0 {
            //      println!("Trigger Clear");
            //    //  for _i in 0..self.header_sample_num {
            //    //      self.sample_queue.pop_front();
            //    //  }
            //  }
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
                self.sample_queue.len(), self.vis_sample_num,
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
                self.sample_queue.len(), ((121.6 * 4.0 + 20.0 + 2.08) * get_sample_length_in_ms(self.sample_rate)),
            );
            return;
        }
    }

    fn decode_in_pd120(&mut self) {
        if self.counter >= 496 {
            self.counter = 0;
            self.mode = Mode::None;
            println!("Finish Decoding!");
        }

        let time = 121.6 * 4.0 + 20.0 + 2.08;
        let pixel_count = 640;
        let sample_count = (time * get_sample_length_in_ms(self.sample_rate) as f32) as usize;
        if self.sample_queue.len() < sample_count {
            eprintln!("Error: PCM data length is not equal to a PD120 line.");
            return;
        }

        let data_to_parse = &self
            .sample_queue
            .range(0..sample_count)
            .cloned()
            .collect::<Vec<f32>>();

        let _sync = self
            .decode(&data_to_parse[0..(20.0 * get_sample_length_in_ms(self.sample_rate)) as usize]);
        let _porch = self.decode(
            &data_to_parse[(20.0 * get_sample_length_in_ms(self.sample_rate)) as usize
                ..(22.08 * get_sample_length_in_ms(self.sample_rate) as f32) as usize],
        );
        // 0-20-22.08-143.58-265.28-386.88-end
        let line_y1 = self.decode_line_info(
            &data_to_parse[(22.08 * get_sample_length_in_ms(self.sample_rate) as f32) as usize
                ..(143.58 * get_sample_length_in_ms(self.sample_rate) as f32) as usize],
            pixel_count,
        );
        let line_ry = self.decode_line_info(
            &data_to_parse[(143.58 * get_sample_length_in_ms(self.sample_rate) as f32) as usize
                ..(265.28 * get_sample_length_in_ms(self.sample_rate) as f32) as usize],
            pixel_count,
        );
        let line_by = self.decode_line_info(
            &data_to_parse[(265.28 * get_sample_length_in_ms(self.sample_rate) as f32) as usize
                ..(386.88 * get_sample_length_in_ms(self.sample_rate) as f32) as usize],
            pixel_count,
        );
        let line_y2 = self.decode_line_info(
            &data_to_parse[(386.88 * get_sample_length_in_ms(self.sample_rate) as f32) as usize
                ..data_to_parse.len()],
            pixel_count,
        );

        for i in 0..pixel_count {
            let by_minus_128 = (line_by[i] - 128) as f32;
            let ry_minus_128 = (line_ry[i] - 128) as f32;
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
