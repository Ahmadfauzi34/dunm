use crate::core::config::GLOBAL_DIMENSION;
use crate::core::entity_manifold::EntityManifold;
use ndarray::Array1;
use std::time::Instant;

// ═══════════════════════════════════════════════════════════════════════════════
// QUANTUM VISUALIZER v2.0 — Deep Transparency for FHRR/VSA Debugging
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Visualizer;

/// Mode transparansi — semakin tinggi, semakin detail
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum TransparencyLevel {
    Silent,       // Hanya error
    Minimal,      // Summary saja
    Standard,     // Default (energy + prob)
    Verbose,      // + tensor stats + interference
    Diagnostic,   // + memory tracking + anomalies
    QuantumDebug, // EVERYTHING: FFT, coherence, full state dump
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct TensorStatsDetailed {
    pub mean: f32,
    pub std_dev: f32,
    pub variance: f32,
    pub min: f32,
    pub max: f32,
    pub energy: f32,
    pub l1_norm: f32,
    pub sparsity: f32,
    pub entropy: f32,
    pub non_zeros: usize,
    pub zero_crossings: usize,
    pub skewness: f32,
    pub kurtosis: f32,
}

#[derive(Debug, Clone)]
pub struct SpectralInfo {
    pub dc: f32,
    pub low_freq: f32,
    pub high_freq: f32,
    pub dominant_band: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum AnomalySeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct MctsNodeInfo {
    pub id: usize,
    pub depth: usize,
    pub probability: f32,
    pub pragmatic_error: f32,
    pub epistemic_value: f32,
    pub complexity: f32,
    pub threshold: f32,
    pub is_pruned: bool,
    pub is_ground_state: bool,
    pub is_expanding: bool,
    pub path: Vec<String>,
    pub axiom_type: String,
}

// Platform-specific memory tracking (Implemented)
fn get_heap_usage() -> usize {
    crate::memory::allocator::TrackingAllocator::get_allocated()
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. REAL-TIME PROBABILITY & ENERGY VISUALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

impl Visualizer {
    /// Mencetak 40 elemen pertama Tensor sebagai Barcode Holografik (Legacy Port)
    pub fn print_tensor_barcode(name: &str, tensor: &Array1<f32>) {
        Self::print_tensor_standard(name, tensor);
    }

    /// Mencetak Memory Map Partikel: █ = Hidup, _ = Dark Matter (Massa 0) (Legacy Port)
    pub fn print_particle_memory_map(manifold: &EntityManifold) {
        let mut mem_map = String::new();
        let cap = manifold.masses.len();
        let limit = std::cmp::min(manifold.active_count + 10, cap);

        for i in 0..limit {
            if manifold.masses[i] > 0.0 {
                mem_map.push('█'); // Partikel aktif (Entitas riil)
            } else {
                mem_map.push('_'); // Dark Matter (Vakum)
            }
        }

        if limit < cap {
            mem_map.push_str("... (truncated)");
        }

        println!(
            "  [Memory]  Map ({}/{}): [{}]",
            manifold.active_count, cap, mem_map
        );
    }

    /// Visualisasi rentang probabilitas dengan gradient color
    pub fn print_probability_range(
        min_prob: f32,
        max_prob: f32,
        current: f32,
        threshold: f32,
        context: &str,
    ) {
        let width = 40;
        let denom = (max_prob - min_prob).max(0.001);
        let pos = (((current - min_prob) / denom) * width as f32).clamp(0.0, width as f32) as usize;
        let threshold_pos =
            (((threshold - min_prob) / denom) * width as f32).clamp(0.0, width as f32) as usize;

        let mut bar = String::with_capacity(width);
        for i in 0..width {
            if i == threshold_pos {
                bar.push('┴'); // Threshold marker
            } else if i < pos {
                bar.push('█');
            } else {
                bar.push('░');
            }
        }

        let status = if current > threshold * 1.2 {
            "\x1b[32m🟢 HIGH\x1b[0m"
        } else if current > threshold {
            "\x1b[33m🟡 MARGINAL\x1b[0m"
        } else {
            "\x1b[31m🔴 PRUNED\x1b[0m"
        };

        println!(
            "  [PROB] {:<20} [{:>6.3}] {} | {} | range[{:.3}, {:.3}] thresh={:.3}",
            context, current, bar, status, min_prob, max_prob, threshold
        );
    }

    /// Visualisasi interference pattern (constructive/destructive)
    pub fn print_interference(before_prob: f32, after_prob: f32, energy: f32, depth: usize) {
        let interference = if before_prob > 0.0 {
            after_prob / before_prob
        } else {
            0.0
        };
        let phase = if interference > 1.0 {
            "CONSTRUCTIVE"
        } else if interference > 0.5 {
            "NEUTRAL"
        } else {
            "DESTRUCTIVE"
        };

        let wave_chars: &[char] = &['▁', '▃', '▅', '▇', '█', '▇', '▅', '▃'];
        let wave_idx = ((interference * 4.0).min(7.0) as usize).max(0);

        println!(
            "  [INTERF] d{} │{}{}{}│ {} | γ={:.3} | E={:.2} | Δp={:+.3e}",
            depth,
            wave_chars[wave_idx % 8],
            wave_chars[(wave_idx + 1) % 8],
            wave_chars[(wave_idx + 2) % 8],
            phase,
            interference,
            energy,
            after_prob - before_prob
        );
    }

    /// Timeline probabilitas untuk tracking evolution
    pub fn print_probability_timeline(
        history: &[(usize, f32, f32)], // (depth, prob, energy)
        level: TransparencyLevel,
    ) {
        if history.is_empty() {
            return;
        }

        println!("  ╔══════════════════════════════════════════════════════════════╗");
        println!("  ║  QUANTUM TRAJECTORY                                          ║");
        println!("  ╠══════════════════════════════════════════════════════════════╣");

        let max_prob = history
            .iter()
            .map(|(_, p, _)| *p)
            .fold(0.0f32, f32::max)
            .max(0.001);

        for (depth, prob, energy) in history {
            let normalized = ((prob / max_prob) * 30.0).clamp(0.0, 30.0) as usize;
            let bar = "█".repeat(normalized) + &"░".repeat(30 - normalized);

            let symbol = if *prob > 0.9 {
                "★"
            } else if *prob > 0.5 {
                "◆"
            } else if *prob > 0.1 {
                "◇"
            } else {
                "✕"
            };

            let detail = match level {
                TransparencyLevel::QuantumDebug => format!("E={:.2e} S={:.3}", energy, -prob.ln()),
                TransparencyLevel::Diagnostic => format!("E={:.1}", energy),
                _ => String::new(),
            };

            println!(
                "  ║  d{:>2} {} │{}│ {:>6.4} {:<20} ║",
                depth, symbol, bar, prob, detail
            );
        }

        println!("  ╚══════════════════════════════════════════════════════════════╝");
    }

    // ═════════════════════════════════════════════════════════════════════════
    // 2. MEMORY TRANSPARENCY — TRACKING ALLOCATION & FRAGMENTATION
    // ═════════════════════════════════════════════════════════════════════════

    pub fn print_memory_transparency(
        manifolds: &[&EntityManifold],
        tensor_ops_count: usize,
        level: TransparencyLevel,
    ) {
        let total_capacity: usize = manifolds
            .iter()
            .map(|m| m.masses.len() * GLOBAL_DIMENSION * 3 * 4)
            .sum();
        let total_active: usize = manifolds
            .iter()
            .map(|m| m.active_count * GLOBAL_DIMENSION * 3 * 4)
            .sum();

        let efficiency = total_active as f32 / total_capacity.max(1) as f32;

        println!("  ┌──────────────────────────────────────────────────────────────┐");
        println!("  │  MEMORY TRANSPARENCY                                         │");
        println!("  ├──────────────────────────────────────────────────────────────┤");
        println!(
            "  │  Manifolds: {:>3} │ Active: {:>6}KB │ Allocated: {:>6}KB     │",
            manifolds.len(),
            total_active / 1024,
            total_capacity / 1024
        );
        println!(
            "  │  Efficiency: {:>5.1}% │ Tensor Ops: {:>6} │ Fragmentation: {:.1}% │",
            efficiency * 100.0,
            tensor_ops_count,
            (1.0 - efficiency) * 100.0
        );

        if level >= TransparencyLevel::Diagnostic {
            // Per-manifold breakdown
            for (i, m) in manifolds.iter().enumerate() {
                let cap = m.masses.len().max(1);
                let density = m.active_count as f32 / cap as f32;
                let mem_actual = m.active_count * GLOBAL_DIMENSION * 3 * 4;
                let mem_wasted = (m.masses.len() - m.active_count) * GLOBAL_DIMENSION * 3 * 4;

                let density_bar = Self::density_gradient_bar(density, 20);
                println!(
                    "  │  [{}] {} entities {} │ +{:>5}KB -{:>5}KB waste          │",
                    i,
                    m.active_count,
                    density_bar,
                    mem_actual / 1024,
                    mem_wasted / 1024
                );
            }
        }

        println!("  └──────────────────────────────────────────────────────────────┘");
    }

    /// Real-time allocation tracker
    pub fn track_allocation<T, F>(name: &str, operation: F, level: TransparencyLevel) -> T
    where
        F: FnOnce() -> T,
    {
        if level < TransparencyLevel::Diagnostic {
            return operation();
        }

        let start = Instant::now();
        let pre_mem = get_heap_usage(); // Platform-specific

        let result = operation();

        let post_mem = get_heap_usage();
        let duration = start.elapsed();
        let allocated = post_mem.saturating_sub(pre_mem);

        println!(
            "  [ALLOC] {:<20} │ +{:>6}KB │ {:>6}µs │ {}",
            name,
            allocated / 1024,
            duration.as_micros(),
            if allocated > 1024 * 1024 {
                "⚠️ LARGE"
            } else {
                "✓"
            }
        );

        result
    }

    // ═════════════════════════════════════════════════════════════════════════
    // 3. TENSOR DEEP INSPECTION — FHRR/VSA SPECIFIC
    // ═════════════════════════════════════════════════════════════════════════

    pub fn print_tensor_quantum(
        name: &str,
        tensor: &Array1<f32>,
        level: TransparencyLevel,
        context: Option<&str>,
    ) {
        match level {
            TransparencyLevel::Silent => {}
            TransparencyLevel::Minimal => {
                let energy = tensor.iter().map(|x| x * x).sum::<f32>().sqrt();
                println!("  [T] {} E={:.2e}", name, energy);
            }
            TransparencyLevel::Standard => Self::print_tensor_standard(name, tensor),
            TransparencyLevel::Verbose => Self::print_tensor_verbose(name, tensor, context),
            TransparencyLevel::Diagnostic => Self::print_tensor_diagnostic(name, tensor),
            TransparencyLevel::QuantumDebug => Self::print_tensor_full_quantum(name, tensor),
        }
    }

    fn print_tensor_standard(name: &str, tensor: &Array1<f32>) {
        let stats = Self::compute_stats(tensor);
        let sample_size = std::cmp::min(40, tensor.len());

        let mut barcode = String::with_capacity(sample_size * 3);
        for i in 0..sample_size {
            barcode.push_str(Self::amplitude_to_unicode(tensor[i]));
        }

        // Energy indicator dengan threshold
        let energy_status = if stats.energy > 100.0 {
            "🔥"
        } else if stats.energy > 10.0 {
            "⚡"
        } else {
            "○"
        };

        println!(
            "  [T] {:<18} │{}│ {} μ={:+.2} σ={:.2} S={:.0}%",
            name,
            barcode,
            energy_status,
            stats.mean,
            stats.std_dev,
            stats.sparsity * 100.0
        );
    }

    fn print_tensor_verbose(name: &str, tensor: &Array1<f32>, context: Option<&str>) {
        let stats = Self::compute_stats(tensor);

        // Spectral analysis (simulated FFT magnitude)
        let spectral = Self::estimate_spectral_content(tensor);

        println!("  ┌─ TENSOR: {:<50} ─┐", name);
        if let Some(ctx) = context {
            println!("  │  Context: {:<48} │", ctx);
        }
        println!("  │  Visual: {}", Self::tensor_to_heatmap(tensor, 60));
        println!(
            "  │  Stats:  μ={:+.3} σ={:.3} [{:.2}, {:.2}]          │",
            stats.mean, stats.std_dev, stats.min, stats.max
        );
        println!(
            "  │  Energy: {:.2e} | Sparsity: {:.1}% | Entropy: {:.2} bits │",
            stats.energy,
            stats.sparsity * 100.0,
            stats.entropy
        );
        println!(
            "  │  Spectral: DC={:.2} LF={:.2} HF={:.2} (dominant: {})     │",
            spectral.dc, spectral.low_freq, spectral.high_freq, spectral.dominant_band
        );
        println!("  └──────────────────────────────────────────────────────────────┘");
    }

    fn print_tensor_diagnostic(name: &str, tensor: &Array1<f32>) {
        let stats = Self::compute_stats(tensor);
        let anomalies = Self::detect_anomalies(tensor, &stats);

        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║  TENSOR DIAGNOSTIC: {:<42} ║", name);
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!(
            "║  Dimensions: {} | Layout: Standard | Alignment: 4-byte         ║",
            tensor.len()
        );
        println!("║  ───────────────────────────────────────────────────────────── ║");
        println!(
            "║  First Moment:  {:>+10.4}  │  Second Moment: {:>10.4}         ║",
            stats.mean, stats.variance
        );
        println!(
            "║  Skewness:      {:>10.2}  │  Kurtosis:      {:>10.2}         ║",
            stats.skewness, stats.kurtosis
        );
        println!(
            "║  L1 Norm:       {:>10.4}  │  L2 Norm:        {:>10.4}         ║",
            stats.l1_norm, stats.energy
        );
        println!("║  ───────────────────────────────────────────────────────────── ║");
        println!(
            "║  Sparsity: {:.1}% │ Non-zeros: {:>6} │ Zero-crossings: {:>6}    ║",
            stats.sparsity * 100.0,
            stats.non_zeros,
            stats.zero_crossings
        );

        if !anomalies.is_empty() {
            println!(
                "║  ⚠️  ANOMALIES ({})                                             ║",
                anomalies.len()
            );
            for (idx, val, reason, severity) in anomalies.iter().take(3) {
                let sev_char = match severity {
                    AnomalySeverity::Critical => '🔴',
                    AnomalySeverity::Warning => '🟡',
                    AnomalySeverity::Info => '🔵',
                };
                println!(
                    "║    {} [{}] = {:+.4} ({})                           ║",
                    sev_char, idx, val, reason
                );
            }
        }

        // Pattern classification
        let pattern = Self::classify_pattern(tensor);
        println!(
            "║  Pattern: {}                                                    ║",
            pattern
        );
        println!("╚════════════════════════════════════════════════════════════════╝");
    }

    fn print_tensor_full_quantum(name: &str, tensor: &Array1<f32>) {
        // Full state dump untuk debugging quantum
        Self::print_tensor_diagnostic(name, tensor);

        println!("  ─── FULL STATE DUMP (first 100 elements) ───");
        for (i, _chunk) in tensor.iter().take(100).enumerate().step_by(10) {
            let values: Vec<String> = tensor
                .iter()
                .skip(i)
                .take(10)
                .map(|v| format!("{:+.2e}", v))
                .collect();
            println!("    [{:>4}] {}", i, values.join(" "));
        }

        // FFT plan (simulated)
        println!("  ─── FREQUENCY DOMAIN (simulated FFT) ───");
        let fft_sim = Self::simulate_fft_magnitude(tensor);
        println!(
            "    Bin 0-10:  {}",
            Self::magnitude_bar(&fft_sim[0..10], 40)
        );
        println!(
            "    Bin 10-20: {}",
            Self::magnitude_bar(&fft_sim[10..20], 40)
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // 4. MCTS TREE TRANSPARENCY — DECISION POINT VISUALIZATION
    // ═════════════════════════════════════════════════════════════════════════

    pub fn print_mcts_transparent(
        node: &MctsNodeInfo,
        siblings: &[MctsNodeInfo],
        level: TransparencyLevel,
    ) {
        if level < TransparencyLevel::Standard {
            return;
        }

        // Context: posisi relatif terhadap siblings
        let rank = siblings
            .iter()
            .filter(|s| s.probability > node.probability)
            .count()
            + 1;
        let total = siblings.len().max(1);

        println!("  ╭──────────────────────────────────────────────────────────────╮");
        println!(
            "  │  MCTS NODE [{}] — Rank {}/{} │ Depth {}                       │",
            node.id, rank, total, node.depth
        );
        println!("  ├──────────────────────────────────────────────────────────────┤");

        // Probability dengan konteks kompetisi
        Self::print_probability_competition(node.probability, siblings, node.threshold);

        // Energy breakdown
        println!("  │  Energy Components:                                          │");
        println!(
            "  │    Pragmatic (E):  {:>8.3}  │  Goal-seeking error             │",
            node.pragmatic_error
        );
        println!(
            "  │    Epistemic (I):  {:>8.3}  │  Information gain               │",
            node.epistemic_value
        );
        println!(
            "  │    Complexity (C):  {:>7.3}  │  Model complexity               │",
            node.complexity
        );
        println!("  │    ───────────────────────────────────────────────────────── │");
        println!(
            "  │    Total Free Energy: {:>7.3}  │  G = E - I + C                  │",
            node.pragmatic_error - node.epistemic_value + node.complexity
        );

        if level >= TransparencyLevel::Verbose {
            println!("  │                                                              │");
            println!("  │  Policy Path: {:<46} │", node.path.join(" → "));
            println!("  │  Axiom: {:<52} │", node.axiom_type);
        }

        // Decision visualization
        let decision = if node.is_pruned {
            "❌ PRUNED (destructive interference)"
        } else if node.is_ground_state {
            "✅ GROUND STATE (energy minimized)"
        } else if node.is_expanding {
            "🌿 EXPANDING (exploring policies...)"
        } else {
            "⏸️  PENDING (queued for evaluation)"
        };

        println!("  ├──────────────────────────────────────────────────────────────┤");
        println!("  │  Status: {:<53} │", decision);
        println!("  ╰──────────────────────────────────────────────────────────────╯");
    }

    fn print_probability_competition(prob: f32, siblings: &[MctsNodeInfo], threshold: f32) {
        if siblings.is_empty() {
            println!(
                "  │  Probability: {:.4} (no competition)                         │",
                prob
            );
            return;
        }

        let all_probs: Vec<f32> = siblings.iter().map(|s| s.probability).collect();
        let min = all_probs.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = all_probs.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let mean = all_probs.iter().sum::<f32>() / all_probs.len() as f32;

        let width = 30;
        let denom = (max - min).max(0.001);
        let pos = (((prob - min) / denom) * width as f32).clamp(0.0, width as f32) as usize;

        let mut bar = String::with_capacity(width);
        for i in 0..width {
            match i.cmp(&pos) {
                std::cmp::Ordering::Equal => bar.push('●'),
                std::cmp::Ordering::Less => bar.push('━'),
                std::cmp::Ordering::Greater => bar.push('┅'),
            }
        }

        println!(
            "  │  P(dist): {} ●={:.3} μ={:.3} range[{:.3}, {:.3}]   │",
            bar, prob, mean, min, max
        );
        println!(
            "  │  Threshold: {:.3} {:<41} │",
            threshold,
            if prob > threshold {
                "✓ SURVIVES"
            } else {
                "✗ PRUNED"
            }
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // 5. UTILITY & HELPER FUNCTIONS
    // ═════════════════════════════════════════════════════════════════════════

    fn compute_stats(tensor: &Array1<f32>) -> TensorStatsDetailed {
        let n = tensor.len() as f32;
        let sum: f32 = tensor.iter().sum();
        let mean = sum / n;

        let variance = tensor.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / n;
        let std_dev = variance.sqrt();

        let min = tensor.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = tensor.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let energy = tensor.iter().map(|x| x * x).sum::<f32>().sqrt();
        let l1_norm = tensor.iter().map(|x| x.abs()).sum();

        let non_zeros = tensor.iter().filter(|&&x| x.abs() > 1e-6).count();
        let sparsity = 1.0 - (non_zeros as f32 / n);

        let zero_crossings = tensor
            .iter()
            .zip(tensor.iter().skip(1))
            .filter(|(a, b)| a.signum() != b.signum())
            .count();

        // Higher moments
        let skewness = if std_dev > 0.0 {
            tensor
                .iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f32>()
                / n
        } else {
            0.0
        };
        let kurtosis = if std_dev > 0.0 {
            tensor
                .iter()
                .map(|x| ((x - mean) / std_dev).powi(4))
                .sum::<f32>()
                / n
                - 3.0
        } else {
            0.0
        };

        // Entropy (Shannon)
        let entropy = if std_dev > 0.0 {
            0.5 * (2.0 * std::f32::consts::PI * std::f32::consts::E * variance).ln()
        } else {
            0.0
        };

        TensorStatsDetailed {
            mean,
            std_dev,
            variance,
            min,
            max,
            energy,
            l1_norm,
            sparsity,
            entropy,
            non_zeros,
            zero_crossings,
            skewness,
            kurtosis,
        }
    }

    fn amplitude_to_unicode(val: f32) -> &'static str {
        // Extended unicode blocks untuk lebih granular
        const BLOCKS: &[&str] = &[
            " ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▉", "▊", "▋", "▌", "▍", "▎", "▏",
        ];
        let idx = ((val.abs() * 8.0 + 8.0) as usize).clamp(0, 15);
        BLOCKS[idx]
    }

    fn density_gradient_bar(density: f32, width: usize) -> String {
        const GRADIENT: &[char] = &[' ', '░', '▒', '▓', '█'];
        (0..width)
            .map(|i| {
                let threshold = i as f32 / width as f32;
                if density > threshold {
                    GRADIENT[((density - threshold) * 4.0 * width as f32).min(4.0) as usize]
                } else {
                    ' '
                }
            })
            .collect()
    }

    fn tensor_to_heatmap(tensor: &Array1<f32>, width: usize) -> String {
        tensor
            .iter()
            .take(width)
            .map(|&v| {
                let intensity = (v.tanh().midpoint(1.0) * 255.0) as u8;
                // Simplified: gunakan ANSI colors
                if intensity > 200 {
                    "█"
                } else if intensity > 150 {
                    "▓"
                } else if intensity > 100 {
                    "▒"
                } else if intensity > 50 {
                    "░"
                } else {
                    " "
                }
            })
            .collect()
    }

    fn simulate_fft_magnitude(tensor: &Array1<f32>) -> Vec<f32> {
        // Simulasi sederhana: binning berdasarkan frekuensi spatial
        let mut bins = vec![0.0f32; 20];
        for (i, &val) in tensor.iter().enumerate() {
            let bin = (i * 20 / tensor.len()).min(19);
            bins[bin] += val.abs();
        }
        bins
    }

    fn magnitude_bar(mags: &[f32], width: usize) -> String {
        let max = mags.iter().fold(0.0f32, |a, &b| a.max(b)).max(0.001);
        mags.iter()
            .map(|&m| {
                let filled = ((m / max) * width as f32) as usize;
                "█".repeat(filled / mags.len().max(1))
            })
            .collect::<String>()
    }

    fn classify_pattern(tensor: &Array1<f32>) -> String {
        let len = tensor.len().max(4);
        let first_half: f32 = tensor.iter().take(len / 2).sum();
        let second_half: f32 = tensor.iter().skip(len / 2).sum();
        let first_quarter: f32 = tensor.iter().take(len / 4).sum();
        let last_quarter: f32 = tensor.iter().skip(len * 3 / 4).sum();

        let patterns = [
            ((first_half - second_half).abs() < 0.1, "Symmetric"),
            (first_half > second_half * 2.0, "Front-loaded"),
            (second_half > first_half * 2.0, "Back-loaded"),
            (first_quarter > last_quarter * 3.0, "Decay"),
            (last_quarter > first_quarter * 3.0, "Growth"),
        ];

        patterns
            .iter()
            .filter(|(cond, _)| *cond)
            .map(|(_, name)| *name)
            .next()
            .unwrap_or("Random/Uniform")
            .to_string()
    }

    fn detect_anomalies(
        tensor: &Array1<f32>,
        stats: &TensorStatsDetailed,
    ) -> Vec<(usize, f32, &'static str, AnomalySeverity)> {
        let mut anomalies = Vec::new();

        for (i, &val) in tensor.iter().enumerate() {
            if stats.std_dev > 0.0 {
                let z_score = (val - stats.mean).abs() / stats.std_dev;

                if z_score > 5.0 {
                    anomalies.push((i, val, "extreme outlier (>5σ)", AnomalySeverity::Critical));
                } else if z_score > 3.0 {
                    anomalies.push((i, val, "outlier (>3σ)", AnomalySeverity::Warning));
                }
            }
            if val.is_nan() {
                anomalies.push((i, val, "NaN detected", AnomalySeverity::Critical));
            } else if val.is_infinite() {
                anomalies.push((i, val, "Infinite value", AnomalySeverity::Critical));
            }
        }

        anomalies
    }

    fn estimate_spectral_content(tensor: &Array1<f32>) -> SpectralInfo {
        // Approximasi: low-freq = slow variation, high-freq = rapid variation
        let dc = tensor.iter().sum::<f32>().abs() / tensor.len() as f32;

        let diffs: Vec<f32> = tensor
            .iter()
            .zip(tensor.iter().skip(1))
            .map(|(a, b)| (b - a).abs())
            .collect();

        let avg_diff = diffs.iter().sum::<f32>() / diffs.len().max(1) as f32;
        let high_freq_content = diffs.iter().filter(|&&d| d > avg_diff * 2.0).count() as f32
            / diffs.len().max(1) as f32;

        SpectralInfo {
            dc,
            low_freq: 1.0 - high_freq_content,
            high_freq: high_freq_content,
            dominant_band: if high_freq_content > 0.5 { "HF" } else { "LF" },
        }
    }
}
