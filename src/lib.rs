#![cfg_attr(
    not(test),
    warn(
        clippy::all,
        clippy::pedantic,
        clippy::cargo,
    )
)]

// ==========================================
// ⛔ STRICT DENY (Keamanan & Anti-Mangkir)
// ==========================================
#![cfg_attr(not(test), deny(
    clippy::correctness,
    clippy::suspicious,
    clippy::unwrap_used,   // Wajib handle error (jangan pakai panics)
    clippy::expect_used,   // Sama seperti unwrap
    clippy::todo,          // Cegah AI/Developer meninggalkan placeholder
    clippy::unimplemented, // Cegah fungsi kosong masuk ke production
))]

// ==========================================
// 🚧 TEMPORARY ALLOW (Tersisa Prioritas Merah & Struktural)
// ==========================================
#![allow(
    clippy::suboptimal_flops,      // 🔴 Paling tinggi: tensor math (Belum dioptimasi)
    
    // ⚠️ STRUKTURAL: Dipertahankan agar AI tidak merusak arsitektur hot-path SoA
    clippy::too_many_lines,
    clippy::too_many_arguments,
)]

// ==========================================
// 🛡️ PERMANENT ALLOW (Domain Tensor)
// ==========================================
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,

    clippy::unreadable_literal, // FHRR random generator constant
    
    // [⬡ Carbo] FHRR: i64→f32 cast intentional untuk AVX2 256-bit alignment
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
)]

pub mod core;
pub mod memory;
pub mod perception;
pub mod quantum_topology;
pub mod reasoning;
pub mod self_awareness;
pub mod shared;

// =============================================================================
// Re-exports untuk binary crate (main.rs) dan external consumers
// =============================================================================
// Tujuan: main.rs harus tetap thin wrapper. Semua logic heavy tetap di lib.
// Agen tidak boleh menambah modul declaration di main.rs.

pub use crate::core::entity_manifold::EntityManifold;
pub use crate::perception::anomalous_extractor::extract_anomalous_quadrant;
pub use crate::reasoning::rrm_agent::RrmAgent;
pub use crate::self_awareness::immortal_loop::KVImmortalEngine;
