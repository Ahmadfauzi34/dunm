use crate::core::config::GLOBAL_DIMENSION;
use crate::memory::maintenance_engine::MaintenanceEngine;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolicComponent {
    pub seed: String,
    pub weight: Option<f32>,
    pub phase: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryTrace {
    pub axiom_type: String,
    pub composition: Vec<SymbolicComponent>,
    pub entropy_at_creation: f32,
    pub timestamp: u64,
    pub version: String,
    pub dimension_at_creation: usize,
}

pub struct KnowledgeBase {
    memory_dir: String,
    version: String,
}

impl KnowledgeBase {
    pub fn new(dir: &str) -> Self {
        fs::create_dir_all(dir).unwrap_or(());
        Self {
            memory_dir: dir.to_string(),
            version: "v1.0".to_string(),
        }
    }

    /// Membaca semua memori yang ada, melakukan anneal_memory, dan menyimpan kembali hasilnya
    pub fn anneal_all_memories(&self) {
        let mut base_names = Vec::new();
        let mut tensors = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.memory_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                    if let Some(base_name) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(bytes) = fs::read(&path) {
                            // Cek jika ukuran valid
                            if bytes.len() == GLOBAL_DIMENSION * 4 {
                                let mut tensor = Array1::zeros(GLOBAL_DIMENSION);
                                for (i, chunk) in bytes.chunks(4).enumerate() {
                                    if chunk.len() == 4 {
                                        tensor[i] = f32::from_ne_bytes(chunk.try_into().unwrap());
                                    }
                                }
                                tensors.push(tensor);
                                base_names.push(base_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        if tensors.is_empty() {
            println!("[KnowledgeBase] Tidak ada axiom binari untuk di-anneal.");
            return;
        }

        let engine = MaintenanceEngine::new();
        let (avg_before, avg_after) = engine.anneal_memory(&mut tensors, 0.5, 30);

        // Simpan kembali
        for (i, base_name) in base_names.iter().enumerate() {
            let bin_path = Path::new(&self.memory_dir).join(format!("{}.bin", base_name));
            let bytes: Vec<u8> = tensors[i].iter().flat_map(|&f| f.to_ne_bytes()).collect();
            fs::write(bin_path, bytes).unwrap_or(());
        }

        println!(
            "[KnowledgeBase] Selesai Annealing {} memori (Noise: {:.4} -> {:.4})",
            tensors.len(),
            avg_before,
            avg_after
        );
    }

    pub fn save_axiom(
        &self,
        base_name: &str,
        axiom_type: &str,
        composition: Vec<SymbolicComponent>,
        tensor: &Array1<f32>,
        entropy: f32,
    ) {
        let trace_path = Path::new(&self.memory_dir).join(format!("{}.json", base_name));
        let bin_path = Path::new(&self.memory_dir).join(format!("{}.bin", base_name));

        let trace = MemoryTrace {
            axiom_type: axiom_type.to_string(),
            composition,
            entropy_at_creation: entropy,
            timestamp: 0, // Should be actual epoch time
            version: self.version.clone(),
            dimension_at_creation: GLOBAL_DIMENSION,
        };

        // Save JSON
        let json_str = serde_json::to_string_pretty(&trace).unwrap();
        fs::write(trace_path, json_str).unwrap();

        // Save Bin (Float32Array bytes equivalent)
        let bytes: Vec<u8> = tensor.iter().flat_map(|&f| f.to_ne_bytes()).collect();
        fs::write(bin_path, bytes).unwrap();

        println!(
            "[Rust KnowledgeBase] Eksport Axiom '{}' berhasil.",
            axiom_type
        );
    }
}
