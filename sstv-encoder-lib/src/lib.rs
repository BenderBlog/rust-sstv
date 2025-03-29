//! # sstv-encoder-lib
//!
//! sstv-encoder-lib is a library to convert images into audio using the Slow Scan Television algorithms.
//!
//! Currently only support outputing 16 bit unsigned integer pcm samples.
//!
//! For more detail, see encode_picture_into_pcm and encode_picture_into_file.

mod header;
mod martin;
mod pasokon;
mod pd;
mod robot36;
mod robot72;
pub mod sample_generator;
mod schottie;
pub mod sstv_image;
mod warsse_sc2_180;

use std::fs::File;

use header::generate_header;
use robot36::encode_in_robot36;
use sample_generator::SampleGenerator;
use sstv_image::SSTVImage;
use thiserror::Error;
use wav_io::writer::i16samples_to_file;

use crate::{
    martin::encode_in_martin, pasokon::encode_in_pasokon, pd::encode_in_pd,
    robot72::encode_in_robot72, schottie::encode_in_schottie,
    warsse_sc2_180::encode_in_warsse_sc2_180,
};

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Create output file error: {0}")]
    CreateFileError(String),

    #[error("Write output file error: {0}")]
    WriteFileError(String),
}

/// Support modes.
pub enum Mode {
    Scottie1,
    Scottie2,
    ScottieDx,
    Martin1,
    Martin2,
    Robot36,
    Robot72,
    WrasseSc2_180,
    P3,
    P5,
    P7,
    Pd50,
    Pd90,
    Pd120,
    Pd160,
    Pd180,
    Pd240,
    Pd290,
}

/// Generate pure pcm data.
pub fn encode_picture_into_pcm(
    ctx: &mut SampleGenerator,
    image: &SSTVImage,
    mode: &Mode,
) -> Vec<i16> {
    [
        generate_header(ctx, mode),
        match mode {
            Mode::Scottie1 => encode_in_schottie(ctx, image, mode),
            Mode::Scottie2 => encode_in_schottie(ctx, image, mode),
            Mode::ScottieDx => encode_in_schottie(ctx, image, mode),
            Mode::Martin1 => encode_in_martin(ctx, image, mode),
            Mode::Martin2 => encode_in_martin(ctx, image, mode),
            Mode::Robot36 => encode_in_robot36(ctx, image),
            Mode::Robot72 => encode_in_robot72(ctx, image),
            Mode::WrasseSc2_180 => encode_in_warsse_sc2_180(ctx, image),
            Mode::P3 => encode_in_pasokon(ctx, image, mode),
            Mode::P5 => encode_in_pasokon(ctx, image, mode),
            Mode::P7 => encode_in_pasokon(ctx, image, mode),
            Mode::Pd50 => encode_in_pd(ctx, image, mode),
            Mode::Pd90 => encode_in_pd(ctx, image, mode),
            Mode::Pd120 => encode_in_pd(ctx, image, mode),
            Mode::Pd160 => encode_in_pd(ctx, image, mode),
            Mode::Pd180 => encode_in_pd(ctx, image, mode),
            Mode::Pd240 => encode_in_pd(ctx, image, mode),
            Mode::Pd290 => encode_in_pd(ctx, image, mode),
        },
    ]
    .concat()
}

/// Encode the picture into a file.
pub fn encode_picture_into_file(
    image: &SSTVImage,
    mode: &Mode,
    name: &str,
    sample_rate: u32,
) -> Result<File, FileError> {
    match std::fs::File::create(name) {
        Ok(mut v) => {
            let head = wav_io::new_header(sample_rate, 16, false, true);
            let ctx = &mut SampleGenerator::new(8000);
            let samples = encode_picture_into_pcm(ctx, image, mode);
            let result = i16samples_to_file(&mut v, &head, &samples);
            if result.is_err() {
                return Err(FileError::WriteFileError(result.err().unwrap().to_string()));
            }

            Ok(v)
        }
        Err(e) => {
            return Err(FileError::CreateFileError(e.to_string()));
        }
    }
}
