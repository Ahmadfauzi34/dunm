use crate::core::config::GLOBAL_DIMENSION;
use ndarray::Array1;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FftPlanner;
use std::cell::RefCell;
use std::f32::consts::PI;

thread_local! {
    static SEED: RefCell<u64> = const { RefCell::new(42) };
    static PLANNER: RefCell<FftPlanner<f32>> = RefCell::new(FftPlanner::new());
}

fn seeded_random(custom_seed: Option<u64>) -> f32 {
    SEED.with(|s| {
        let mut seed = s.borrow_mut();
        if let Some(cs) = custom_seed {
            *seed = cs;
        }
        // Sama dengan JS LCG: seed = (seed * 16807) % 2147483647
        *seed = (*seed * 16807) % 2_147_483_647;
        (*seed as f32 - 1.0) / 2_147_483_646.0
    })
}

pub struct FHRR;

impl FHRR {
    /// 1. CREATE: Membuat Vektor Unitary (Flat-Spectrum)
    pub fn create(custom_seed: Option<u64>) -> Array1<f32> {
        let dim = GLOBAL_DIMENSION;
        let mut freqs = vec![Complex::zero(); dim];

        // DC (0) & Nyquist (N/2) harus Real
        freqs[0] = Complex::new(1.0, 0.0);
        freqs[dim / 2] = Complex::new(1.0, 0.0);

        if custom_seed.is_some() {
            seeded_random(custom_seed);
        }

        // Isi frekuensi dengan fase acak (Unit Magnitude)
        for k in 1..(dim / 2) {
            let phase = seeded_random(None) * PI * 2.0;
            let cos_p = phase.cos();
            let sin_p = phase.sin();

            freqs[k] = Complex::new(cos_p, sin_p);
            let sym_k = dim - k;
            freqs[sym_k] = Complex::new(cos_p, -sin_p); // Conjugate
        }

        // Inverse FFT ke Real Space
        let mut out = vec![Complex::zero(); dim];

        PLANNER.with(|p| {
            let mut planner = p.borrow_mut();
            let fft = planner.plan_fft_inverse(dim);

            out.copy_from_slice(&freqs);
            fft.process(&mut out);
        });

        // Di FFT.js, output IFFT sudah disederhanakan, di Rust kita ambil real() dan normalize
        let scale = 1.0 / (dim as f32).sqrt();
        Array1::from_iter(out.into_iter().map(|c| c.re * scale))
    }

    /// 2. BIND: Konvolusi Sirkular (FFT -> Mul -> IFFT)
    pub fn bind(a: &Array1<f32>, b: &Array1<f32>) -> Array1<f32> {
        let dim = GLOBAL_DIMENSION;
        let mut cx_a: Vec<Complex<f32>> = a.iter().map(|&x| Complex::new(x, 0.0)).collect();
        let mut cx_b: Vec<Complex<f32>> = b.iter().map(|&x| Complex::new(x, 0.0)).collect();

        PLANNER.with(|p| {
            let mut planner = p.borrow_mut();
            let fft_fwd = planner.plan_fft_forward(dim);

            fft_fwd.process(&mut cx_a);
            fft_fwd.process(&mut cx_b);
        });

        let mut freqs: Vec<Complex<f32>> =
            cx_a.iter().zip(cx_b.iter()).map(|(a, b)| a * b).collect();

        PLANNER.with(|p| {
            let mut planner = p.borrow_mut();
            let fft_inv = planner.plan_fft_inverse(dim);
            fft_inv.process(&mut freqs);
        });

        let scale = 1.0 / (dim as f32);
        Array1::from_iter(freqs.into_iter().map(|c| c.re * scale))
    }

    /// 3. FRACTIONAL BIND: Memutar Fasa Fraksional
    pub fn fractional_bind(a: &Array1<f32>, power: f32) -> Array1<f32> {
        let dim = GLOBAL_DIMENSION;
        let mut cx_a: Vec<Complex<f32>> = a.iter().map(|&x| Complex::new(x, 0.0)).collect();

        PLANNER.with(|p| {
            let mut planner = p.borrow_mut();
            let fft_fwd = planner.plan_fft_forward(dim);
            fft_fwd.process(&mut cx_a);
        });

        for k in 0..dim {
            let amp = cx_a[k].norm();
            let phase = cx_a[k].arg() * power;
            cx_a[k] = Complex::new(amp * phase.cos(), amp * phase.sin());
        }

        PLANNER.with(|p| {
            let mut planner = p.borrow_mut();
            let fft_inv = planner.plan_fft_inverse(dim);
            fft_inv.process(&mut cx_a);
        });

        let scale = 1.0 / (dim as f32);
        Array1::from_iter(cx_a.into_iter().map(|c| c.re * scale))
    }

    /// 3.5 FUSI SIMD: Fractional Bind & Convolve (Zero-Cost Abstraction)
    /// Menghitung rotasi fasa untuk sumbu X dan Y sekaligus di domain frekuensi,
    /// lalu langsung mengalikannya (binding) sebelum IFFT. Ini menghemat 2x proses FFT/IFFT
    /// dan memicu register SIMD karena intervensi zip iterator.
    pub fn fractional_bind_2d(
        x_seed: &Array1<f32>,
        power_x: f32,
        y_seed: &Array1<f32>,
        power_y: f32,
    ) -> Array1<f32> {
        let dim = GLOBAL_DIMENSION;
        let mut cx_x: Vec<Complex<f32>> = x_seed.iter().map(|&x| Complex::new(x, 0.0)).collect();
        let mut cx_y: Vec<Complex<f32>> = y_seed.iter().map(|&y| Complex::new(y, 0.0)).collect();

        PLANNER.with(|p| {
            let mut planner = p.borrow_mut();
            let fft_fwd = planner.plan_fft_forward(dim);
            fft_fwd.process(&mut cx_x);
            fft_fwd.process(&mut cx_y);
        });

        // FUSI MATEMATIKA: Iterasi paralel di vektor SIMD (O(N) pass tunggal)
        for (kx, ky) in cx_x.iter_mut().zip(cx_y.iter_mut()) {
            let phase_x = kx.arg() * power_x;
            let rot_x = Complex::new(kx.norm() * phase_x.cos(), kx.norm() * phase_x.sin());

            let phase_y = ky.arg() * power_y;
            let rot_y = Complex::new(ky.norm() * phase_y.cos(), ky.norm() * phase_y.sin());

            // Bind (Konvolusi di Time Domain = Perkalian di Freq Domain)
            *kx = rot_x * rot_y;
        }

        PLANNER.with(|p| {
            let mut planner = p.borrow_mut();
            let fft_inv = planner.plan_fft_inverse(dim);
            // cx_x sekarang berisi hasil binding X ⊗ Y di domain frekuensi
            fft_inv.process(&mut cx_x);
        });

        let scale = 1.0 / (dim as f32);
        Array1::from_iter(cx_x.into_iter().map(|c| c.re * scale))
    }

    /// 4. INVERSE: Kebalikan (Involution)
    pub fn inverse(a: &Array1<f32>) -> Array1<f32> {
        let mut out = Array1::zeros(GLOBAL_DIMENSION);
        out[0] = a[0];
        for i in 1..GLOBAL_DIMENSION {
            out[i] = a[GLOBAL_DIMENSION - i];
        }
        out
    }

    /// 5. SIMILARITY: Cosine Similarity
    pub fn similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|&x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|&x| x * x).sum::<f32>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        dot_product / (mag_a * mag_b)
    }
}
