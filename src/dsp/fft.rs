//! In-place radix-2 Cooley-Tukey FFT with precomputed twiddle factors.

use std::f32::consts::TAU;
use std::sync::LazyLock;

const TWIDDLE_N: usize = 1024;

static TWIDDLES: LazyLock<([f32; TWIDDLE_N / 2], [f32; TWIDDLE_N / 2])> = LazyLock::new(|| {
    let mut re = [0.0f32; TWIDDLE_N / 2];
    let mut im = [0.0f32; TWIDDLE_N / 2];
    for k in 0..TWIDDLE_N / 2 {
        let angle = -TAU * k as f32 / TWIDDLE_N as f32;
        re[k] = angle.cos();
        im[k] = angle.sin();
    }
    (re, im)
});

/// Force twiddle factor initialization (call from non-audio thread).
pub fn init_twiddles() {
    let _ = &*TWIDDLES;
}

/// In-place radix-2 Cooley-Tukey FFT.
/// Arrays must have the same power-of-2 length.
pub fn fft(re: &mut [f32], im: &mut [f32], inverse: bool) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(re.len(), im.len());

    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // Butterfly stages with precomputed twiddle table for N <= 1024
    let use_table = n <= TWIDDLE_N;
    let mut len = 2;
    while len <= n {
        let half = len >> 1;
        let stride = if use_table { TWIDDLE_N / len } else { 0 };
        let angle_step = if !use_table {
            if inverse {
                TAU / len as f32
            } else {
                -TAU / len as f32
            }
        } else {
            0.0
        };
        for start in (0..n).step_by(len) {
            for k in 0..half {
                let (w_re, w_im) = if use_table {
                    let idx = k * stride;
                    if inverse {
                        (TWIDDLES.0[idx], -TWIDDLES.1[idx])
                    } else {
                        (TWIDDLES.0[idx], TWIDDLES.1[idx])
                    }
                } else {
                    let angle = angle_step * k as f32;
                    (angle.cos(), angle.sin())
                };
                let a = start + k;
                let b = start + k + half;
                let t_re = re[b] * w_re - im[b] * w_im;
                let t_im = re[b] * w_im + im[b] * w_re;
                re[b] = re[a] - t_re;
                im[b] = im[a] - t_im;
                re[a] += t_re;
                im[a] += t_im;
            }
        }
        len <<= 1;
    }

    if inverse {
        let inv_n = 1.0 / n as f32;
        for (re, im) in re.iter_mut().zip(im.iter_mut()) {
            *re *= inv_n;
            *im *= inv_n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_identity() {
        let n = 1024;
        let mut re = vec![0.0f32; n];
        let mut im = vec![0.0f32; n];
        for (i, sample) in re.iter_mut().enumerate() {
            *sample = (TAU * 3.0 * i as f32 / n as f32).sin();
        }
        let original: Vec<f32> = re.clone();

        fft(&mut re, &mut im, false);
        fft(&mut re, &mut im, true);

        for (i, ((&real, &imag), &expected)) in
            re.iter().zip(im.iter()).zip(original.iter()).enumerate()
        {
            assert!(
                (real - expected).abs() < 1e-4,
                "mismatch at {i}: {} vs {}",
                real,
                expected
            );
            assert!(imag.abs() < 1e-4, "imaginary not zero at {i}: {imag}");
        }
    }

    #[test]
    fn dc_signal() {
        let n = 256;
        let mut re = vec![1.0f32; n];
        let mut im = vec![0.0f32; n];
        fft(&mut re, &mut im, false);
        assert!((re[0] - n as f32).abs() < 1e-3);
        for (k, &bin) in re.iter().enumerate().skip(1) {
            assert!(bin.abs() < 1e-3, "bin {k} not zero: {bin}");
        }
    }
}
