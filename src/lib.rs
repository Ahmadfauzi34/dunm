#![cfg_attr(
    not(test),
    warn(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::cargo,
    )
)]
#![allow(
    clippy::uninlined_format_args,
    clippy::similar_names,
    clippy::explicit_iter_loop,
    clippy::missing_const_for_fn,
    // TODO: Hapus allow ini jika refactoring dasar sudah selesai untuk optimasi FMA
    clippy::suboptimal_flops,
    // Linter Kerapian & Dokumentasi
    clippy::unused_self,
    clippy::use_self,
    clippy::doc_markdown,
    clippy::missing_panics_doc,
)]
#![cfg_attr(not(test), deny(clippy::correctness, clippy::suspicious))]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    // === DOMAIN-SPECIFIC: Tensor Math ===
    // FHRR menggunakan f32 untuk SIMD alignment (AVX2 256-bit = 8x f32).
    // Cast dari i64 (JSON) ke f32 (tensor) adalah intentional dan validated.
    // Jangan tambah allow baru tanpa signature [⬡ Carbo] di engineering journal.
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
