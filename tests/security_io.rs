use rrm_rust::KVImmortalEngine;
use std::path::PathBuf;
use std::time::Duration;
use std::thread;

#[test]
fn test_immortal_engine_io_failure_resilience() {
    // Jalur yang tidak valid atau tidak dapat ditulis (di sebagian besar sistem Unix/Linux)
    let invalid_path = PathBuf::from("/proc/invalid_path_rrm_test");

    // Inisialisasi tidak boleh panic meskipun pembuatan direktori gagal
    let mut engine = KVImmortalEngine::new(&invalid_path, "security_test");

    // Menambahkan event juga tidak boleh panic meskipun thread I/O mungkin sudah keluar
    use rrm_rust::self_awareness::immortal_loop::SoulEvent;
    engine.append_event(SoulEvent::TaskAttempted { task_id: "test".to_string() });

    // Tunggu sebentar untuk memastikan thread I/O sempat berjalan dan mencoba membuka file
    thread::sleep(Duration::from_millis(100));

    // Hibernate juga harus berjalan tanpa panic
    let dummy_manifold = rrm_rust::EntityManifold::default();
    engine.hibernate(&dummy_manifold);

    // Jika sampai di sini tanpa panic, test lulus
}
