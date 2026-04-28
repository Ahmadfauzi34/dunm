use std::ops::{Add, Div, Sub};

pub const FEMTO_MULTIPLIER: i64 = 1_000_000_000_000_000; // 10^15

/// Struktur Fixed-Point (Integer i64) 100% deterministik untuk menjamin
/// tidak ada "State Drift" atau masalah FMA (Fused Multiply-Add) dari LLVM
/// saat menghitung jarak kuantum pada skala Femto (10^-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DeterministicFemto(pub i64);

impl DeterministicFemto {
    pub const ZERO: Self = Self(0);

    /// Membuat nilai Femto dari angka desimal biasa (f32/f64)
    /// HANYA SAAT INISIALISASI. Setelah masuk sistem, wajib berwujud Integer.
    pub fn from_f32(value: f32) -> Self {
        DeterministicFemto((value as f64 * FEMTO_MULTIPLIER as f64).round() as i64)
    }

    pub fn from_f64(value: f64) -> Self {
        DeterministicFemto((value * FEMTO_MULTIPLIER as f64).round() as i64)
    }

    /// Mengubah kembali ke f32/f64 HANYA untuk kebutuhan UI/Visualisasi
    /// atau fungsi FHRR yang memang membutuhkan float untuk FFT.
    pub fn to_f32(self) -> f32 {
        (self.0 as f64 / FEMTO_MULTIPLIER as f64) as f32
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / FEMTO_MULTIPLIER as f64
    }

    pub fn abs(self) -> Self {
        DeterministicFemto(self.0.abs())
    }
}

// Implementasi matematika yang aman, bebas dari Floating-Point rounding error!

impl Add for DeterministicFemto {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        DeterministicFemto(self.0.saturating_add(other.0))
    }
}

impl Sub for DeterministicFemto {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        DeterministicFemto(self.0.saturating_sub(other.0))
    }
}

// Pembagian skala Integer (membutuhkan konversi khusus jika mengalikan sesama femto)
impl Div<i64> for DeterministicFemto {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: i64) -> Self {
        if rhs == 0 {
            DeterministicFemto::ZERO
        } else {
            DeterministicFemto(self.0 / rhs)
        }
    }
}

// Topografi Gradient Energi yang baru
#[derive(Debug, Clone, Copy, Default)]
pub struct TensorGradient {
    pub x: DeterministicFemto,
    pub y: DeterministicFemto,
}

impl TensorGradient {
    /// Gradient Steering: Menuruni lembah energi dengan presisi absolut
    pub fn apply_gradient(&mut self, step_x: DeterministicFemto, step_y: DeterministicFemto) {
        // Operasi ini DIJAMIN menghasilkan bit yang persis sama lintas CPU
        self.x = self.x + step_x;
        self.y = self.y + step_y;
    }
}
