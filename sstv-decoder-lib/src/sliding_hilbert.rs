use std::f32::consts::PI;

use rustfft::FftPlanner;
use rustfft::num_complex::Complex32;

/// 对单一窗进行Hilbert解析信号计算
pub fn hilbert_transform_window(x: &[f32]) -> Vec<Complex32> {
    let n = x.len();
    let mut planner = FftPlanner::<f32>::new();
    let mut input: Vec<Complex32> = x.iter().map(|&x| Complex32::new(x, 0.0)).collect();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut input);

    // 构建滤波器
    let mut h = vec![0.0; n];
    if n % 2 == 0 {
        h[0] = 1.0;
        h[n / 2] = 1.0;
        for i in 1..(n / 2) {
            h[i] = 2.0;
        }
    } else {
        h[0] = 1.0;
        for i in 1..((n + 1) / 2) {
            h[i] = 2.0;
        }
    }
    for (xi, hi) in input.iter_mut().zip(h.iter()) {
        *xi *= *hi;
    }

    // 逆变换
    let ifft = planner.plan_fft_inverse(n);
    ifft.process(&mut input);
    for x in &mut input {
        *x /= n as f32;
    }
    input
}

/// 解析信号z和采样率fs，输出每一点的瞬时频率（Hz）
pub fn instantaneous_frequency(x: &[f32], fs: f32) -> Vec<f32> {
    let z = hilbert_transform_window(x);
    let mut phase: Vec<f32> = z.iter().map(|c| c.arg()).collect();
    for i in 1..phase.len() {
        let mut dp = phase[i] - phase[i - 1];
        // unwrap
        while dp > PI {
            dp -= 2.0 * PI;
        }
        while dp < -PI {
            dp += 2.0 * PI;
        }
        phase[i] = phase[i - 1] + dp;
    }
    phase
        .windows(2)
        .map(|w| (fs * (w[1] - w[0]) / (2.0 * PI)).abs())
        .collect()
}
