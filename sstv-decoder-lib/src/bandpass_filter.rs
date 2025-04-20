use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type};

pub fn bandpass_filter(samples: &[f32], fs: f32) -> Vec<f32> {
    let fl = 1.khz();
    let fh = 3.khz();
    let fs = fs.hz();

    let coeffs_lp = Coefficients::<f32>::from_params(Type::LowPass, fs, fh, 1.).unwrap();
    let coeffs_hp = Coefficients::<f32>::from_params(Type::HighPass, fs, fl, 1.).unwrap();

    let mut biquad_lp = DirectForm1::<f32>::new(coeffs_lp);
    let mut biquad_hp = DirectForm1::<f32>::new(coeffs_hp);
    samples
        .iter()
        .map(|&x| biquad_lp.run(biquad_hp.run(x)))
        .collect()
}
