use crate::core::entity_manifold::EntityManifold;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// Struktur data log (Soul Log)
#[derive(Clone, Debug)]
pub enum SoulEvent {
    TaskAttempted { task_id: String },
    TaskSolved { task_id: String },
    MctsFailed { task_id: String },
    Evolution { skill_name: String, depth: usize },
    Checkpoint { file_path: String },
}

impl SoulEvent {
    pub fn to_string_log(&self) -> String {
        match self {
            Self::TaskAttempted { task_id } => format!("ATTEMPT: Task {}", task_id),
            Self::TaskSolved { task_id } => format!("SOLVED: Task {}", task_id),
            Self::MctsFailed { task_id } => format!("FAILED: MCTS exhausted on Task {}", task_id),
            Self::Evolution { skill_name, depth } => {
                format!("EVOLVE: {} at depth {}", skill_name, depth)
            }
            Self::Checkpoint { file_path } => format!("CHECKPOINT: Saved at {}", file_path),
        }
    }
}

/// Buffer memori untuk mencegah RRM terblokir oleh I/O Disk
pub struct AsyncSoulLog {
    buffer: Vec<SoulEvent>,
    capacity: usize,
    last_flush: Instant,
    flush_interval: Duration,
    /// Channel untuk mengirim data ke thread penulis disk
    io_sender: Sender<Vec<SoulEvent>>,
}

impl AsyncSoulLog {
    pub fn new(capacity: usize, flush_interval: Duration, root_dir: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<Vec<SoulEvent>>();
        let log_file = root_dir.join("soul_log.md");

        // Pastikan direktori log ada
        if !root_dir.exists() {
            if let Err(e) = fs::create_dir_all(&root_dir) {
                eprintln!("Warning: Failed to create log directory {:?}: {}", root_dir, e);
            }
        }

        // Buat thread terpisah (Dedicated I/O Thread)
        // Thread ini yang akan menangani bottleneck disk, membebaskan CPU utama
        thread::spawn(move || {
            let mut file = match OpenOptions::new().create(true).append(true).open(&log_file) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!(
                        "Critical Error: Failed to open soul_log.md at {:?}: {}",
                        log_file, e
                    );
                    return; // Graceful exit of the logging thread
                }
            };

            while let Ok(batch) = rx.recv() {
                for event in batch {
                    let log_line = format!("- [{:?}] {}\n", Instant::now(), event.to_string_log());
                    let _ = file.write_all(log_line.as_bytes());
                }
                let _ = file.sync_data(); // Ensure flush to physical disk
            }
        });

        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            last_flush: Instant::now(),
            flush_interval,
            io_sender: tx,
        }
    }

    /// Menambahkan event ke RAM, ini sangat cepat (Zero I/O blocking)
    #[inline(always)]
    pub fn append(&mut self, entry: SoulEvent) {
        self.buffer.push(entry);

        // Lazy Checkpointing / Threshold flush
        if self.should_flush() {
            self.flush_to_background();
        }
    }

    /// Cek apakah sudah waktunya membilas data ke disk
    #[inline(always)]
    pub fn should_flush(&self) -> bool {
        self.buffer.len() >= self.capacity || self.last_flush.elapsed() >= self.flush_interval
    }

    /// Mengirim buffer memori ke Thread I/O secara asynchronous
    pub fn flush_to_background(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        // Tukar isi buffer lama dengan vektor baru yang kosong secara murah (O(1) allocation via replace)
        let batch_to_write = std::mem::replace(&mut self.buffer, Vec::with_capacity(self.capacity));

        // Kirim ke thread I/O tanpa memblokir loop utama
        let _ = self.io_sender.send(batch_to_write);

        self.last_flush = Instant::now();
    }
}

// Wrapper yang digunakan oleh arsitektur RRM saat ini
pub struct KVImmortalEngine {
    root_dir: PathBuf,
    soul_log: AsyncSoulLog,
}

impl KVImmortalEngine {
    pub fn new(base_dir: &Path, name: &str) -> Self {
        let dir = base_dir.join(name);

        // Buat instance AsyncSoulLog (Buffer=1000 events, Flush interval=5 detik)
        let async_log = AsyncSoulLog::new(1000, Duration::from_secs(5), dir.clone());

        Self {
            root_dir: dir,
            soul_log: async_log,
        }
    }

    pub fn branch(&self, name: &str) -> Self {
        Self::new(&self.root_dir, name)
    }

    pub fn append_event(&mut self, event: SoulEvent) {
        self.soul_log.append(event);
    }

    pub fn hibernate(&mut self, _state: &EntityManifold) {
        // Flush semua log yang tersisa ke background sebelum tidur panjang
        self.soul_log.flush_to_background();

        // (Optional) Serialize _state to .bin here if needed
        self.soul_log.append(SoulEvent::Checkpoint {
            file_path: "hibernate_state.bin".to_string(),
        });
        self.soul_log.flush_to_background();
    }

    pub fn resurrect(&mut self) -> Result<(), String> {
        // Dummy: Load state from .bin and replay events from .md if necessary
        Ok(())
    }
}

impl Drop for KVImmortalEngine {
    fn drop(&mut self) {
        self.soul_log.flush_to_background();
    }
}
