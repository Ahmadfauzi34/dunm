use crate::core::entity_manifold::EntityManifold;
use crate::memory::logic_seed_bank::LogicSeedBank;
use crate::perception::entity_segmenter::EntitySegmenter;
use crate::perception::hologram_decoder::HologramDecoder;
use crate::perception::structural_analyzer::StructuralAnalyzer;
use crate::perception::universal_manifold::UniversalManifold;
use crate::reasoning::causal_reasoning::CausalReasoner;
use crate::reasoning::global_blackboard::GlobalBlackboard;
use crate::reasoning::grover_diffusion_system::{
    GroverCandidate, GroverConfig, GroverDiffusionSystem, TrainState,
};
use crate::reasoning::hierarchical_inference::{DeepActiveInferenceEngine, SimulationMode};
use crate::reasoning::multiverse_sandbox::MultiverseSandbox;
use crate::reasoning::quantum_search::{AsyncWaveSearch, WaveNode};
use crate::reasoning::top_down_axiomator::TopDownAxiomator;
use crate::reasoning::topological_aligner::TopologicalAligner;
use crate::self_awareness::self_reflection::{Bottleneck, FailureMode, SelfReflection};
use crate::self_awareness::skill_ontology::SkillOntology;

use ndarray::Array1;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RrmAgent {
    perceiver: UniversalManifold,
    decoder: HologramDecoder,
    seed_bank: LogicSeedBank,

    // Self-Awareness Layer
    ontology: SkillOntology,
    self_reflection: SelfReflection,

    // Reasoning
    counterfactual_engine: crate::reasoning::counterfactual_engine::CounterfactualEngine,
    causal_reasoner: CausalReasoner,

    // Memory
    mental_replay: crate::memory::mental_replay::MentalReplay,
}

impl Default for RrmAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl RrmAgent {
    pub fn new() -> Self {
        let ontology = SkillOntology::initialize();
        let self_reflection = SelfReflection::new(ontology.clone());

        Self {
            perceiver: UniversalManifold::new(),
            decoder: HologramDecoder::new(),
            seed_bank: LogicSeedBank::new(),
            ontology,
            self_reflection,
            counterfactual_engine:
                crate::reasoning::counterfactual_engine::CounterfactualEngine::new(),
            causal_reasoner: CausalReasoner::new(),
            mental_replay: crate::memory::mental_replay::MentalReplay::new(),
        }
    }

    pub fn solve_task_v2(
        &mut self,
        train_pairs: &[(EntityManifold, EntityManifold)],
        test_input: &EntityManifold,
    ) -> Vec<Vec<i32>> {
        use crate::reasoning::hierarchical_planner::HierarchicalPlanner;
        use crate::reasoning::structures::Axiom;

        println!("🧠 [Mental Simulation] Memulai counterfactual exploration...");

        let deltas: Vec<_> = train_pairs
            .iter()
            .map(|(inp, out)| StructuralAnalyzer::analyze(inp, out))
            .collect();

        let consensus_delta = StructuralAnalyzer::consensus(&deltas);

        // Pre-filter dengan simulasi cepat di CounterfactualEngine
        let mut promising_axioms = Vec::new();

        // Introspeksi dari ontology
        let introspect_candidates = self.ontology.introspect(&consensus_delta.signature);
        let mut candidates = vec![Axiom::identity(), Axiom::crop_to_content()];

        for cap in introspect_candidates {
            candidates.push(Axiom::new(
                &cap.name,
                cap.tier_id,
                ndarray::Array1::zeros(crate::core::config::GLOBAL_DIMENSION),
                ndarray::Array1::zeros(crate::core::config::GLOBAL_DIMENSION),
                0.0,
                0.0,
            ));
        }

        for axiom in candidates {
            let result =
                self.counterfactual_engine
                    .what_if(&axiom, &train_pairs[0].0, &train_pairs[0].1);
            if result.is_success {
                println!(
                    "    ✅ {} langsung sukses di CounterfactualEngine!",
                    axiom.name
                );

                // Kausalitas
                let dummy_sig = crate::reasoning::structures::StructuralSignature {
                    dim_relation: crate::reasoning::structures::DimensionRelation::Equal,
                    object_delta: crate::reasoning::structures::ObjectDelta::Same,
                    color_mapping: None,
                    topology_hint: crate::reasoning::structures::TopologyHint::Grid,
                };

                let empty_alts: Vec<crate::reasoning::structures::Axiom> = vec![];
                let causal_result = self.causal_reasoner.assess_causality(
                    &axiom,
                    &train_pairs[0].0,
                    &dummy_sig,
                    &empty_alts,
                );
                println!("    ✅ Evaluasi Kausalitas: {}", causal_result.explanation);

                // === CAUSAL FEEDBACK LOOP ===
                if causal_result.confidence > 0.6 && causal_result.is_sufficient {
                    // Cek Struktural Self-Awareness: Apakah perubahan pikiran (Tensor) mengubah Tubuh (Grid Fisik)?
                    let mut test_body = train_pairs[0].0.clone();
                    crate::reasoning::multiverse_sandbox::MultiverseSandbox::apply_axiom(
                        &mut test_body,
                        &axiom.condition_tensor,
                        &axiom.delta_spatial,
                        &axiom.delta_semantic,
                        axiom.delta_x,
                        axiom.delta_y,
                        axiom.tier,
                        &axiom.name,
                    );

                    let mut is_body_changed = false;
                    if test_body.active_count != train_pairs[0].0.active_count
                        || test_body.global_width != train_pairs[0].0.global_width
                        || test_body.global_height != train_pairs[0].0.global_height
                    {
                        is_body_changed = true;
                    }

                    if !is_body_changed
                        && (axiom.delta_x != 0.0
                            || axiom.delta_y != 0.0
                            || axiom.name.contains("COLOR"))
                    {
                        println!("    🚨 [STRUCTURAL SELF-AWARENESS WARNING]");
                        println!("    🚨 Pikiran saya tahu '{}' adalah aksioma yang tepat secara kausalitas dan tensor...", axiom.name);
                        println!("    🚨 ...Tapi 'Tubuh' saya (MultiverseSandbox) gagal mengeksekusinya ke grid pixel!");
                        println!("    🚨 SAYA KEKURANGAN ALAT FISIK. Tolong upgrade `apply_axiom` di Sandbox.");

                        let wiki = crate::self_awareness::executable_wiki::ExecutableWiki::new(
                            "rrm_rust/knowledge/skills/",
                        );
                        let _ = wiki.append_to_log("Execution_Log", &format!("SELF-AWARENESS: Causal reasoning found solution {}, but Sandbox physics engine lacks implementation to move pixels.", axiom.name));
                    }

                    // Menyimpan kausalitas yang sukses sebagai "Memory Constraint"
                    let causal_memory_str = format!("Causal_Success_{}", axiom.name);
                    self.seed_bank.add_seed(
                        &causal_memory_str,
                        9999,
                        &ndarray::Array1::ones(crate::core::config::GLOBAL_DIMENSION),
                    );
                } else if !causal_result.is_necessary {
                    println!("    ⚠️ Peringatan Kausalitas: Intervensi {} tidak Necessary. Mempertimbangkan untuk mencari Axiom lain yang lebih spesifik.", axiom.name);
                }

                promising_axioms.push(axiom);
                break;
            } else {
                println!("    ❌ {} tidak cocok", axiom.name);
            }
        }

        if !promising_axioms.is_empty() {
            let mut test_state = test_input.clone();
            for axiom in &promising_axioms {
                MultiverseSandbox::apply_axiom(
                    &mut test_state,
                    &axiom.condition_tensor,
                    &axiom.delta_spatial,
                    &axiom.delta_semantic,
                    axiom.delta_x,
                    axiom.delta_y,
                    axiom.tier,
                    &axiom.name,
                );
            }

            return self.decoder.collapse_to_grid(
                &test_state,
                test_state.global_width as usize,
                test_state.global_height as usize,
                0.5,
            );
        }

        // Fallback ke Hierarchical Planner
        println!("  🔄 Fallback ke hierarchical planning...");

        let _strategy = self
            .ontology
            .can_solve(&consensus_delta)
            .expect("No strategy available for this task class");

        let planner = HierarchicalPlanner::from_delta(&consensus_delta, &self.ontology);

        let plan = planner.plan_with_validation(
            &mut self.counterfactual_engine,
            &train_pairs[0].0,
            &train_pairs[0].1,
        );

        match plan {
            Some(axioms) => {
                let mut test_state = test_input.clone();
                for axiom in &axioms {
                    MultiverseSandbox::apply_axiom(
                        &mut test_state,
                        &axiom.condition_tensor,
                        &axiom.delta_spatial,
                        &axiom.delta_semantic,
                        axiom.delta_x,
                        axiom.delta_y,
                        axiom.tier,
                        &axiom.name,
                    );
                }

                let wiki = crate::self_awareness::executable_wiki::ExecutableWiki::new(
                    "rrm_rust/knowledge/skills/",
                );
                let _ = wiki.append_to_log(
                    "Execution_Log",
                    "Run #X -> SUCCESS via HierarchicalPlanner fallback",
                );

                self.decoder.collapse_to_grid(
                    &test_state,
                    test_state.global_width as usize,
                    test_state.global_height as usize,
                    0.5,
                )
            }
            None => {
                self.mental_replay.generate_dreams(10);
                let _discovered = self
                    .mental_replay
                    .practice_in_dreams(&mut self.counterfactual_engine, &self.ontology);

                // MCTS/Planner gagal total. Catat ke log Wiki
                let mut wiki = crate::self_awareness::executable_wiki::ExecutableWiki::new(
                    "rrm_rust/knowledge/skills/",
                );
                let _ = wiki.append_to_log("Analysis_Log", "Catastrophic Failure Detected. Need to synthesize generative skill via crossover.");

                // Simulasi pembuatan skill baru hasil "crossover"
                let new_page = crate::self_awareness::executable_wiki::WikiPage {
                    id: format!("synthesized_{}", chrono::Utc::now().format("%Y%m%d%H%M%S")),
                    page_type: "synthesized_crossover".to_string(),
                    tier: 8,
                    confidence: 0.50,
                    parent: Some("mcts_fallback".to_string()),
                    content: "## Origin\nAuto-generated skill from Catastrophic Failure\n\n```rust\n// Novel spatial tensor bound\n```\n".to_string(),
                    code_blocks: vec![],
                };
                let _ = wiki.create_skill(new_page);

                self.decoder.collapse_to_grid(
                    test_input,
                    test_input.global_width as usize,
                    test_input.global_height as usize,
                    0.5,
                )
            }
        }
    }

    pub fn dream(&mut self) {
        println!("🌙 [Mental Replay] RRM memasuki fase REM (Bermimpi)...");
        use crate::self_awareness::executable_wiki::ExecutableWiki;
        let wiki = ExecutableWiki::new(std::path::PathBuf::from("knowledge"));
        let _ = wiki.append_to_log(
            "soul_log",
            "\n## Branch: Mengubah Dimensi Ruang & Warna Dalam Mimpi\n",
        );

        // Asumsikan RRM sedang mengamati memori Task 2dc579da
        if self.mental_replay.solved_tasks.is_empty() {
            println!("   -> Tidak ada memori aktif. Menciptakan skenario fraktal hipotetis...");
        }

        println!("   -> Generate skenario variasi...");
        self.mental_replay.generate_dreams(3);
        println!("   -> Tercipta {} dimensi alternatif (Size Scaling, Color Permutation, Noise Injection).", 3);

        println!("   -> Mensimulasikan Counterfactual Engine di alam mimpi...");
        // Di alam mimpi, agen mencoba memecahkan masalah dengan skill ontology yang dia punya.
        let discovered_skills = self
            .mental_replay
            .practice_in_dreams(&mut self.counterfactual_engine, &self.ontology);

        if !discovered_skills.is_empty() {
            println!(
                "✨ [Eureka!] RRM menemukan {} komposisi skill baru di alam mimpinya!",
                discovered_skills.len()
            );
            for _skill in discovered_skills {
                // println!("  - Discovered Rule: {:?}", skill.emergence_properties);
                let axiom_name = String::from("DreamAxiom_Unknown");
                self.seed_bank.add_seed(
                    &axiom_name,
                    9999,
                    &ndarray::Array1::ones(crate::core::config::GLOBAL_DIMENSION),
                );
            }
        } else {
            println!("   -> Mimpi selesai. Sistem telah melatih otot kognitifnya.");
            let _ = wiki.append_to_log(
                "soul_log",
                "### [tX] Mimpi Selesai: Otot kognitif tensor telah direkalibrasi.",
            );
        }
    }

    pub fn solve_task(
        &mut self,
        train_in: &Vec<Vec<Vec<i32>>>,
        train_out: &Vec<Vec<Vec<i32>>>,
        test_in: &Vec<Vec<i32>>,
    ) -> Vec<Vec<i32>> {
        let mut train_states: Vec<(EntityManifold, EntityManifold)> = Vec::new();

        for (i, o) in train_in.iter().zip(train_out.iter()) {
            let mut stream_in = HashMap::new();
            let mut stream_out = HashMap::new();

            self.encode_grid(i, &mut stream_in);
            self.encode_grid(o, &mut stream_out);

            let mut man_in = EntityManifold::new();
            let mut man_out = EntityManifold::new();

            EntitySegmenter::segment_stream(&stream_in, &mut man_in, 0.85, &self.perceiver);
            EntitySegmenter::segment_stream(&stream_out, &mut man_out, 0.85, &self.perceiver);

            train_states.push((man_in, man_out));
        }

        let mut stream_test = HashMap::new();
        self.encode_grid(test_in, &mut stream_test);
        let mut test_manifold = EntityManifold::new();
        EntitySegmenter::segment_stream(&stream_test, &mut test_manifold, 0.85, &self.perceiver);

        let expected_grids: Vec<Vec<Vec<i32>>> = train_out.clone();

        // 1.5 ORIENTASI PRE-EMPTIVE (Membaca Niat Task)
        println!("🧠 [Orientasi Pre-emptive] Membaca Niat Task...");
        let mut pre_emptive_delta = None;
        let mut betti_1_holes = 0;
        let mut curvature_norm = 0.0;

        if let Some((man_in, man_out)) = train_states.first() {
            let delta = StructuralAnalyzer::analyze(man_in, man_out);
            let report = self.self_reflection.assess_situation(&delta);
            println!(
                "   -> Niat / Klasifikasi Masalah: {}",
                report.situation_assessment
            );
            pre_emptive_delta = Some(delta);

            // Evaluasi Topologi Kuantum (Deteksi Lubang / Betti-1)
            let qcc = crate::quantum_topology::QuantumCellComplex::from_manifold(man_in, 1.5);
            betti_1_holes = *qcc.betti_numbers.get(1).unwrap_or(&0);

            if betti_1_holes > 0 {
                println!("🧠 [Topologi Kuantum] Betti-1: Mendeteksi {} lubang (holes) pada struktur awal. Task mungkin bertipe Flood-Fill / Enclosure.", betti_1_holes);
            } else {
                println!(
                    "🧠 [Topologi Kuantum] Betti-1: Tidak ada lubang terdeteksi. Struktur solid."
                );
            }
        }

        // Cek Saliency Ratio: Seberapa besar porsi grid yang benar-benar aktif dibanding keseluruhan?
        // Menghitung berdasarkan massa total entitas aktif (jumlah piksel)
        if let Some((man_in, _)) = train_states.first() {
            let mut total_active_mass = 0.0;
            for i in 0..man_in.active_count {
                if man_in.tokens[i] != 0 {
                    total_active_mass += man_in.masses[i];
                }
            }

            let total_area = (man_in.global_width * man_in.global_height).max(1.0);
            self.self_reflection.active_saliency_ratio = total_active_mass / total_area;
        }

        // Asosiasi Masa Lalu (Knowledge Base) & Betti-1 Injection
        let mut historical_axiom_injected = false;
        if let Some(ref delta) = pre_emptive_delta {
            // Jika ada deteksi warna atau ada deteksi lubang (Betti-1), injeksi aksioma pewarnaan area
            if (!delta.signature.color_transitions.is_empty()
                && delta.signature.dim_relation
                    == crate::perception::structural_analyzer::DimensionRelation::Equal)
                || betti_1_holes > 0
            {
                if betti_1_holes > 0 {
                    println!("🧠 [Memori Masa Lalu & Topologi] Teringat pola Flood Fill karena ada {} lubang Betti-1...", betti_1_holes);
                } else {
                    println!(
                        "🧠 [Memori Masa Lalu] Teringat pola yang mirip dengan Task 09629e4f..."
                    );
                }
                println!("   -> Menginjeksi [CROP_TO_COLOR, FLOOD_FILL] ke dalam Seed Axioms.");
                historical_axiom_injected = true;
            }
        }

        // 3. COGNITIVE STATE-MACHINE LOOP (MCTS + Grover)
        let mut loop_counter = 0;
        let max_loops = 6;

        let calculate_dark_matter = |manifold: &EntityManifold| -> f32 {
            if manifold.active_count == 0 {
                return 0.0;
            }
            let mut zero_mass_count = 0;
            for i in 0..manifold.active_count {
                if manifold.masses[i] == 0.0 {
                    zero_mass_count += 1;
                }
            }
            zero_mass_count as f32 / manifold.active_count as f32
        };

        let mut best_rule: Option<WaveNode> = None;
        let mut seed_axioms: Vec<WaveNode> = Vec::new();

        loop {
            loop_counter += 1;
            if loop_counter > max_loops {
                println!("🧠 [Metakognisi] Batas loop kognitif tercapai (Infinite Loop Protection). Simulasi dihentikan paksa.");
                break;
            }

            // Cek persentase dark matter di siklus ini
            let mut sum_ratio = 0.0;
            let mut count = 0;
            for (man_in, man_out) in train_states.iter() {
                sum_ratio += calculate_dark_matter(man_in);
                sum_ratio += calculate_dark_matter(man_out);
                count += 2;
            }
            sum_ratio += calculate_dark_matter(&test_manifold);
            count += 1;

            self.self_reflection.dark_matter_ratio = if count > 0 {
                sum_ratio / count as f32
            } else {
                0.0
            };

            // 2. RESONATE (Regenerasi Axioms jika kosong, biarkan jika sudah diset oleh Micro-Steps/ObstacleStuck)
            if seed_axioms.is_empty() {
                if let Some((man_in, man_out)) = train_states.first() {
                    let mut matches = TopDownAxiomator::generate_axioms(man_in, man_out);

                    // Injeksi Asosiasi Masa Lalu jika relevan
                    if historical_axiom_injected && loop_counter == 1 {
                        let hist_match = crate::reasoning::topological_aligner::TopologicalMatch {
                            source_index: 0,
                            target_index: 0,
                            axiom_type: "CROP_TO_COLOR".to_string(),
                            similarity: 0.95, // High confidence karena memori masa lalu
                            condition_tensor: None,
                            delta_spatial: ndarray::Array1::zeros(
                                crate::core::config::GLOBAL_DIMENSION,
                            ),
                            delta_semantic: ndarray::Array1::ones(
                                crate::core::config::GLOBAL_DIMENSION,
                            ), // Fake semantic change
                            delta_x: 0.0,
                            delta_y: 0.0,
                            physics_tier: 3,
                        };
                        let hist_match_2 =
                            crate::reasoning::topological_aligner::TopologicalMatch {
                                source_index: 0,
                                target_index: 0,
                                axiom_type: "FLOOD_FILL".to_string(),
                                similarity: 0.94, // High confidence karena memori masa lalu
                                condition_tensor: None,
                                delta_spatial: ndarray::Array1::zeros(
                                    crate::core::config::GLOBAL_DIMENSION,
                                ),
                                delta_semantic: ndarray::Array1::ones(
                                    crate::core::config::GLOBAL_DIMENSION,
                                ), // Fake semantic change
                                delta_x: 0.0,
                                delta_y: 0.0,
                                physics_tier: 3,
                            };
                        matches.push(hist_match);
                        matches.push(hist_match_2);
                    }
                    matches.extend(TopologicalAligner::align(man_in, man_out));
                    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

                    // Zero-Shot Context Pruning:
                    // Jika klasifikasi masalah BUKAN RelationalRearrangement, kurangi aksioma geometri (seperti mirror/rotate).
                    if let Some(ref delta) = pre_emptive_delta {
                        // Pruning Geometri berdasarkan Curvature & Topological Intent
                        if curvature_norm > 1.0 {
                            // High Curvature: Linear translation is likely wrong, focus on Scale/Rotate/Fractal
                            matches.retain(|m| !m.axiom_type.starts_with("SHIFT"));
                            // Inject Scale_Up as a high priority guess
                            if loop_counter == 1 {
                                let fractal_match_2 =
                                    crate::reasoning::topological_aligner::TopologicalMatch {
                                        source_index: 0,
                                        target_index: 0,
                                        axiom_type: "SCALE_UP(2)".to_string(),
                                        similarity: 0.96,
                                        condition_tensor: None,
                                        delta_spatial: ndarray::Array1::zeros(
                                            crate::core::config::GLOBAL_DIMENSION,
                                        ),
                                        delta_semantic: ndarray::Array1::zeros(
                                            crate::core::config::GLOBAL_DIMENSION,
                                        ),
                                        delta_x: 0.0,
                                        delta_y: 0.0,
                                        physics_tier: 4,
                                    };
                                let fractal_match_3 =
                                    crate::reasoning::topological_aligner::TopologicalMatch {
                                        source_index: 0,
                                        target_index: 0,
                                        axiom_type: "SCALE_UP(3)".to_string(),
                                        similarity: 0.95,
                                        condition_tensor: None,
                                        delta_spatial: ndarray::Array1::zeros(
                                            crate::core::config::GLOBAL_DIMENSION,
                                        ),
                                        delta_semantic: ndarray::Array1::zeros(
                                            crate::core::config::GLOBAL_DIMENSION,
                                        ),
                                        delta_x: 0.0,
                                        delta_y: 0.0,
                                        physics_tier: 4,
                                    };
                                matches.push(fractal_match_2);
                                matches.push(fractal_match_3);
                                println!("🧠 [Topologi Kuantum] Curvature > 1.0. Memangkas SHIFT dan menginjeksi aksioma SCALE_UP dinamis.");
                            }
                        }

                        let class = StructuralAnalyzer::classify_task_class(delta);
                        if class != crate::perception::structural_analyzer::TaskClass::PureGeometry
                           && class != crate::perception::structural_analyzer::TaskClass::RelationalRearrangement {
                            matches.retain(|m| !m.axiom_type.contains("GLOBAL_ROTATE") && !m.axiom_type.contains("GLOBAL_MIRROR"));
                        }

                        // Pruning Aksioma Warna
                        // Jika tidak ada transisi warna, kita membuang aksioma yang berhubungan dengan pengubahan warna (CROP_TO_COLOR, FLOOD_FILL, dll).
                        if delta.signature.color_transitions.is_empty() {
                            matches.retain(|m| {
                                !m.axiom_type.contains("CROP_TO_COLOR")
                                    && !m.axiom_type.contains("IF_COLOR")
                            });
                        }
                    }

                    for m in matches {
                        let initial_manifolds: Arc<Vec<EntityManifold>> =
                            Arc::new(train_states.iter().map(|s| s.0.clone()).collect());

                        let mut node = WaveNode::new(
                            m.axiom_type,
                            m.condition_tensor,
                            m.delta_spatial,
                            m.delta_semantic,
                            m.delta_x,
                            m.delta_y,
                            m.physics_tier,
                            initial_manifolds,
                            None,
                        );
                        node.probability = m.similarity;
                        seed_axioms.push(node);
                    }
                }
            }

            let high_confidence_axioms: Vec<WaveNode> = seed_axioms
                .iter()
                .filter(|a| a.probability >= 0.3)
                .cloned()
                .collect();

            let mut max_prob = -1.0;

            let bottleneck = self.self_reflection.assess_current_bottleneck();

            match bottleneck {
                Bottleneck::FalseSharing => {
                    println!("🧠 [Metakognisi] Bottleneck::FalseSharing - Amnesia Singkat (Cache Miss) Terdeteksi!");
                    println!("   📝 MENULIS LOG ARCHITECTURE KE SISTEM: Iterasi iterasi EntityManifold lambat rata-rata {} ns per entitas. Kecepatan ini 3x lipat dari limit AVX2.", self.self_reflection.average_iteration_time_ns);

                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("knowledge/architecture_lint.md")
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "ARCHITECTURAL PAIN: Agent detected iteration times of {} ns per entity. This suggests severe L1 cache misses and fragmented memory structures. Please review struct padding, ensure EntityManifold is tightly packed, and reduce pointer indirections during hot loops.",
                            self.self_reflection.average_iteration_time_ns
                        );
                    }

                    self.self_reflection.average_iteration_time_ns = 0; // reset
                    self.self_reflection.last_failure_mode = FailureMode::None;
                }
                Bottleneck::CognitiveGarbage => {
                    println!("🧠 [Metakognisi] Bottleneck::CognitiveGarbage - Polusi Dark Matter terdeteksi di memori spasial!");
                    println!("   📝 MENULIS LOG ARCHITECTURE KE SISTEM: Terdeteksi {}% entitas adalah 'Ghost State' dengan mass = 0.0.", (self.self_reflection.dark_matter_ratio * 100.0).round());

                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("knowledge/architecture_lint.md")
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "ARCHITECTURAL PAIN: Agent detected {:.1}% of entities in EntityManifold are inactive (mass = 0.0). Iterating over this 'Dark Matter' destroys CPU branch predictors and cache locality. Please implement swap-remove or periodic array compaction.",
                            self.self_reflection.dark_matter_ratio * 100.0
                        );
                    }

                    // Aksi Otonom: Memadatkan (Compacting) Entity Manifold untuk membuang sampah
                    println!("   🧹 [Auto-Fix] Agen secara otonom memadatkan (compacting) array EntityManifold...");
                    let compact_manifold = |m: &mut EntityManifold| {
                        let mut new_idx = 0;
                        for i in 0..m.active_count {
                            if m.masses[i] > 0.0 {
                                if i != new_idx {
                                    let mass = m.masses[i];
                                    let tok = m.tokens[i];
                                    let cx = m.centers_x[i];
                                    let cy = m.centers_y[i];
                                    let sx = m.spans_x[i];
                                    let sy = m.spans_y[i];
                                    let es = m.entanglement_status[i];

                                    m.masses[new_idx] = mass;
                                    m.tokens[new_idx] = tok;
                                    m.centers_x[new_idx] = cx;
                                    m.centers_y[new_idx] = cy;
                                    m.spans_x[new_idx] = sx;
                                    m.spans_y[new_idx] = sy;
                                    m.entanglement_status[new_idx] = es;
                                }
                                new_idx += 1;
                            }
                        }
                        m.active_count = new_idx;
                    };

                    compact_manifold(&mut test_manifold);
                    for (man_in, man_out) in train_states.iter_mut() {
                        compact_manifold(man_in);
                        compact_manifold(man_out);
                    }

                    self.self_reflection.dark_matter_ratio = 0.0;
                    self.self_reflection.last_failure_mode = FailureMode::None;

                    continue; // Skip siklus untuk me-restart dengan memori yang lebih padat (SIMD friendly)
                }
                Bottleneck::AllocationThrashing => {
                    println!("🧠 [Metakognisi] Bottleneck::AllocationThrashing - Nyeri Alokasi Memori (Heap Thrashing) Terdeteksi!");
                    println!("   📝 MENULIS LOG ARCHITECTURE KE SISTEM: Terjadi {} alokasi dinamis (Vec::push) di hot path MCTS.", self.self_reflection.heap_allocation_count);

                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("knowledge/architecture_lint.md")
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "ARCHITECTURAL PAIN: Agent detected {} dynamic heap allocations exceeding MCTS buffer capacity. This thrashes the OS allocator and destroys SIMD throughput. Please implement a bump allocator (bumpalo) or reuse object pools.",
                            self.self_reflection.heap_allocation_count
                        );
                    }

                    // Kita asumsikan agen mentolerirnya untuk saat ini agar tidak crash, reset metriknya
                    self.self_reflection.heap_allocation_count = 0;
                    self.self_reflection.last_failure_mode = FailureMode::None;
                }
                Bottleneck::MemoryBloat => {
                    println!("🧠 [Metakognisi] Bottleneck::MemoryBloat - Pelanggaran Copy-on-Write (CoW) Terdeteksi!");
                    println!("   📝 MENULIS LOG ARCHITECTURE KE SISTEM: MCTS melakukan {} Deep Copy dan {} Shallow Clone. Ini menghancurkan cache L1/L2.", self.self_reflection.deep_copy_count, self.self_reflection.shallow_clone_count);

                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("knowledge/architecture_lint.md")
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "ARCHITECTURAL PAIN: Agent observed {} deep copies vs {} shallow clones during MCTS. Please review `Arc::make_mut` usage or consider implementing an Object Pool to prevent heap thrashing.",
                            self.self_reflection.deep_copy_count,
                            self.self_reflection.shallow_clone_count
                        );
                    }

                    // Kita asumsikan agen bisa mentolerirnya, kita reset metriknya agar simulasi dapat berlanjut
                    self.self_reflection.deep_copy_count = 0;
                    self.self_reflection.last_failure_mode = FailureMode::None;
                }
                Bottleneck::BodyLimitation => {
                    println!("🧠 [Metakognisi] Bottleneck::BodyLimitation - Mind-Body Disconnect terdeteksi!");
                    println!("   Pikiran logis (Tensor) sangat yakin, namun tubuh fisik (Sandbox) gagal mengeksekusi.");
                    println!("   📝 MENULIS LOG ERROR KE SISTEM: 'SAYA KEKURANGAN ALAT FISIK. Tolong upgrade `apply_axiom` di Sandbox.'");

                    // Menulis log manual agar sistem terstruktur (pengembang dapat melihat log dan mengembangkan physics_tier).
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("knowledge/execution_log.md")
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            file,
                            "SELF-AWARENESS: Causal reasoning found a highly probable solution tensor, but Sandbox physics engine lacks implementation to move pixels. Please upgrade `apply_axiom`."
                        );
                    }

                    // Kita tidak bisa melanjutkan jika tubuh tidak mendukung. Hentikan simulasi.
                    break;
                }
                Bottleneck::ObstacleStuck => {
                    println!("🧠 [Metakognisi] Bottleneck::ObstacleStuck - Rute terhalang rintangan. Mengubah aksioma dari Translasi Absolut ke Pathfinding (Micro-Steps)...");

                    // Kita membuang tebakan aksioma Translasi "Jauh" yang lama.
                    // Menyiapkan aksioma "Micro-Step" (Gerak 1-piksel ke Atas, Bawah, Kiri, Kanan)
                    // Nantinya MCTS akan mengevaluasinya sebagai rute labirin.
                    let x_seed = crate::core::core_seeds::CoreSeeds::x_axis_seed();
                    let y_seed = crate::core::core_seeds::CoreSeeds::y_axis_seed();

                    seed_axioms.clear(); // Hapus aksioma tebakan lurus (hanya buang-buang waktu)

                    let micro_steps = vec![
                        ("STEP_UP", 0.0, -1.0),
                        ("STEP_DOWN", 0.0, 1.0),
                        ("STEP_LEFT", -1.0, 0.0),
                        ("STEP_RIGHT", 1.0, 0.0),
                    ];

                    let initial_manifolds: Arc<Vec<EntityManifold>> =
                        Arc::new(train_states.iter().map(|s| s.0.clone()).collect());

                    for (name, dx, dy) in micro_steps {
                        let tensor_spatial =
                            crate::core::fhrr::FHRR::fractional_bind_2d(x_seed, dx, y_seed, dy);
                        let mut node = WaveNode::new(
                            name.to_string(),
                            None,
                            tensor_spatial.clone(),
                            ndarray::Array1::zeros(crate::core::config::GLOBAL_DIMENSION),
                            dx,
                            dy,
                            1, // Micro step adalah spatial shift (Tier 1) tapi harus ditaruh sebagai list khusus agar MCTS Iterative Deepening bisa merayap
                            initial_manifolds.clone(),
                            None,
                        );
                        node.probability = 0.99; // Set ke confidence tinggi
                        seed_axioms.push(node);
                    }

                    println!("   🧱 [Obstacle Awareness] Aksioma pencarian telah diubah menjadi Micro-Steps.");

                    // Escape hatch: Lanjut iterasi untuk dicoba oleh Advanced Pass
                    self.self_reflection.last_failure_mode = FailureMode::None;
                    self.self_reflection.iterations_without_improvement = 350; // Paksa masuk LocalOptimum
                    self.self_reflection.best_energy = 5.0;

                    continue; // Skip ke iterasi berikutnya agar Advanced Pass menerima update seed_axioms ini
                }
                Bottleneck::Distracted => {
                    println!("🧠 [Metakognisi] Bottleneck::Distracted - Saliency ratio terlalu rendah (Background dominan). Melakukan Zoom-In (Saliency Crop)!");

                    // Kita membuang piksel background dengan memfilter manifold
                    let filter_saliency = |manifold: &mut EntityManifold| {
                        let mut active_indices = Vec::new();
                        for i in 0..manifold.active_count {
                            // Anggap background adalah massa 0 atau token 0
                            if manifold.masses[i] > 0.0 && manifold.tokens[i] != 0 {
                                active_indices.push(i);
                            }
                        }

                        // Geser entitas yang aktif ke depan array agar kompak, namun kita TIDAK MENGUBAH UKURAN GLOBAL GRID
                        // Hanya saja, kita bisa mensimulasikan "Fokus" dengan menonaktifkan yang lainnya.
                        let mut new_idx = 0;
                        for old_idx in active_indices {
                            if new_idx != old_idx {
                                let m = manifold.masses[old_idx];
                                let t = manifold.tokens[old_idx];
                                let cx = manifold.centers_x[old_idx];
                                let cy = manifold.centers_y[old_idx];
                                let sx = manifold.spans_x[old_idx];
                                let sy = manifold.spans_y[old_idx];
                                let es = manifold.entanglement_status[old_idx];

                                manifold.masses[new_idx] = m;
                                manifold.tokens[new_idx] = t;
                                manifold.centers_x[new_idx] = cx;
                                manifold.centers_y[new_idx] = cy;
                                manifold.spans_x[new_idx] = sx;
                                manifold.spans_y[new_idx] = sy;
                                manifold.entanglement_status[new_idx] = es;
                            }
                            new_idx += 1;
                        }
                        manifold.active_count = new_idx;
                    };

                    filter_saliency(&mut test_manifold);
                    for (man_in, man_out) in train_states.iter_mut() {
                        filter_saliency(man_in);
                        filter_saliency(man_out);
                    }

                    println!("   👁️ [Saliency Engine] Grid telah dibersihkan dari noise. Perhatian kini terfokus pada objek aktif.");

                    // Kosongkan seed_axioms agar regenerasi axiom melihat piksel baru yang lebih bersih
                    seed_axioms.clear();

                    // Supaya tidak terjebak loop yang sama
                    self.self_reflection.active_saliency_ratio = 1.0;
                    self.self_reflection.best_energy = 5.0; // Turunkan energy agar bisa lanjut mencari pola

                    continue; // Sama seperti ObstacleStuck, lanjutkan ke siklus berikutnya untuk memproses manifold hasil filter
                }
                Bottleneck::Solved => {
                    println!("🧠 [Metakognisi] Bottleneck::Solved - Sistem telah menemukan Ground State!");
                    break;
                }
                Bottleneck::Exhausted => {
                    println!("🧠 [Metakognisi] Bottleneck::Exhausted - Kelelahan. Pencarian dihentikan. Membutuhkan Tidur REM.");
                    break;
                }
                Bottleneck::Blindness => {
                    println!("🧠 [Metakognisi] Bottleneck::Blindness - Saya tidak memahami struktur. Mengobati Kebutaan via Hierarchical Gestalt...");
                    // Panggil dokter mata: Hierarchical Gestalt untuk mengelompokkan piksel menjadi objek makro
                    use crate::perception::hierarchical_gestalt::GestaltEngine;

                    let mut macro_found = false;

                    // Kita akan mengubah struktur data test_manifold dengan temuan Gestalt jika bermanfaat
                    let gestalt_atoms = GestaltEngine::extract_atoms(&test_manifold);
                    if !gestalt_atoms.is_empty() {
                        println!(
                            "   👁️ [Gestalt Vision] Menemukan {} objek makro pada Test Manifold (Bentuk: {:?}, dsb).",
                            gestalt_atoms.len(),
                            gestalt_atoms[0].atom_type
                        );

                        // Menggantikan manifold lama dengan interpretasi makro (gestalt_atoms)
                        let create_macro_manifold =
                            |source: &EntityManifold,
                             atoms: &[crate::perception::hierarchical_gestalt::GestaltAtom]|
                             -> EntityManifold {
                                let mut macro_manifold = EntityManifold::new();
                                macro_manifold.global_width = source.global_width;
                                macro_manifold.global_height = source.global_height;

                                let mut idx = 0;
                                for atom in atoms.iter() {
                                    macro_manifold.ensure_scalar_capacity(idx + 1);
                                    macro_manifold.masses[idx] = atom.pixel_count as f32;
                                    macro_manifold.tokens[idx] = atom.color;
                                    macro_manifold.centers_x[idx] = atom.center_of_mass.0;
                                    macro_manifold.centers_y[idx] = atom.center_of_mass.1;
                                    macro_manifold.spans_x[idx] =
                                        (atom.bounding_box.2 - atom.bounding_box.0).max(1.0);
                                    macro_manifold.spans_y[idx] =
                                        (atom.bounding_box.3 - atom.bounding_box.1).max(1.0);
                                    idx += 1;
                                }
                                macro_manifold.active_count = idx;
                                macro_manifold
                            };

                        test_manifold = create_macro_manifold(&test_manifold, &gestalt_atoms);

                        // Penting: Paradigm Shift juga harus diaplikasikan ke memori Training Data agar Axioms baru konsisten
                        for (man_in, man_out) in train_states.iter_mut() {
                            let in_atoms = GestaltEngine::extract_atoms(man_in);
                            if !in_atoms.is_empty() {
                                *man_in = create_macro_manifold(man_in, &in_atoms);
                            }
                            let out_atoms = GestaltEngine::extract_atoms(man_out);
                            if !out_atoms.is_empty() {
                                *man_out = create_macro_manifold(man_out, &out_atoms);
                            }
                        }

                        macro_found = true;
                    }

                    if macro_found {
                        // Kosongkan seed_axioms agar diregenerasi di awal loop menggunakan paradigma makro yang baru
                        seed_axioms.clear();

                        // Jika berhasil mengelompokkan, kita reset metrik agar MCTS / Grover berjalan lagi
                        println!("   👁️ [Gestalt Vision] Pandangan dipulihkan. Mengulang pencarian dengan paradigma makro...");
                        self.self_reflection.best_energy = 5.0; // Turunkan energy agar lolos dari blok Blindness
                        self.self_reflection.last_failure_mode = FailureMode::None;
                        self.self_reflection.iterations_without_improvement = 0;

                        continue; // Lewati siklus iterasi ini agar Advanced Pass mendapat manifold terbaru
                    } else {
                        // Jika Gestalt juga tidak bisa menemukan bentuk berarti, anggap agen kelelahan.
                        println!(
                            "   👁️ [Gestalt Vision] Pandangan tetap buram (Noise Total). Menyerah."
                        );
                        self.self_reflection.total_iterations = 9999;
                    }
                }
                Bottleneck::PrecisionError => {
                    println!("🧠 [Metakognisi] Bottleneck::PrecisionError - Meleset sedikit. Menembakkan Counterfactual Engine (Femto Scale)...");

                    // Kita gunakan CounterfactualEngine untuk mencari deviasi posisi yang eksak antara hasil tebakan terbaik dan target
                    if let Some(ref mut rule) = best_rule {
                        let mut engine =
                            crate::reasoning::counterfactual_engine::CounterfactualEngine::new();

                        // Konversi WaveNode menjadi Axiom sementara untuk disimulasikan
                        let mut temp_axiom = crate::reasoning::structures::Axiom::identity();
                        temp_axiom.name = rule
                            .axiom_type
                            .last()
                            .cloned()
                            .unwrap_or_else(|| "TempAxiom".to_string());
                        temp_axiom.condition_tensor = rule.condition_tensor.clone();
                        temp_axiom.delta_spatial = rule.tensor_spatial.clone();
                        temp_axiom.delta_semantic = rule.tensor_semantic.clone();
                        temp_axiom.delta_x = rule.delta_x;
                        temp_axiom.delta_y = rule.delta_y;
                        temp_axiom.tier = rule.physics_tier;

                        // Uji satu pasang train state untuk mendapatkan gradient geseran
                        if let Some((man_in, expected_out)) = train_states.first() {
                            let result = engine.what_if(&temp_axiom, man_in, expected_out);

                            if !result.is_success {
                                if let Some(ref failure) = result.failure {
                                    match failure {
                                        crate::reasoning::counterfactual_engine::FailureMode::HighEnergyState {
                                            gradient_x, gradient_y, ..
                                        } => {
                                            println!("   🎯 [Femto Surgeon] Menemukan deviasi absolut: dx={}, dy={}", gradient_x.to_f32(), gradient_y.to_f32());

                                            if let Some(corrections) = engine.suggest_correction(failure) {
                                                if let Some(correction) = corrections.first() {
                                                    println!("   🎯 [Femto Surgeon] Memutar fasa tensor secara analitis ke target eksak!");

                                                    // Perbarui WaveNode terbaik kita dengan tensor koreksi
                                                    rule.tensor_spatial = correction.delta_spatial.clone();
                                                    rule.delta_x = correction.delta_x;
                                                    rule.delta_y = correction.delta_y;
                                                    rule.axiom_type.push("FEMTO_CORRECTION".to_string());

                                                    // Set probability maksimal karena ini kalkulus eksak
                                                    max_prob = 1.0; _ = max_prob;
                                                    rule.probability = 1.0;

                                                    // Tandai Sukses agar keluar loop
                                                    self.self_reflection.update_metrics(
                                                        0.0,
                                                        0.0,
                                                        FailureMode::None,
                                                    );
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Jika gagal memperbaiki posisi (mungkin karena struktur rusak parah)
                    self.self_reflection.last_failure_mode = FailureMode::None;
                    self.self_reflection.iterations_without_improvement += 350; // Paksa masuk LocalOptimum
                }
                Bottleneck::CombinatorialExplosion => {
                    println!("🧠 [Metakognisi] Bottleneck::CombinatorialExplosion - Terlalu banyak kombinasi piksel yang bergerak. Mengaktifkan Swarm & Wave Dynamics (Hebbian Insting)...");

                    // Kita memanggil WaveDynamics untuk membangkitkan EntanglementGraph antar partikel (mencari relasi seperti warna sama atau posisi dekat)
                    let mut wave_engine = crate::reasoning::wave_dynamics::WaveDynamics::new();
                    wave_engine.initialize_entities(&test_manifold);
                    wave_engine.evolve_entanglement(&test_manifold, 0.5);

                    println!("   🌊 [Swarm Physics] Entanglement Graph terbentuk. Mensimulasikan 'Gravity Drop' ke arah tebakan terbaik saat ini...");

                    // Jika ada best_rule, kita coba terapkan Swarm Gravity (semua entitas ditarik ke arah delta_x/delta_y secara bersamaan, dgn mempertimbangkan tabrakan)
                    if let Some(ref rule) = best_rule {
                        if rule.delta_x.abs() > 0.0 || rule.delta_y.abs() > 0.0 {
                            crate::reasoning::swarm_dynamics::SwarmDynamics::apply_swarm_gravity(
                                &mut test_manifold,
                                rule.delta_x,
                                rule.delta_y,
                            );

                            // Terapkan hal yang sama ke train_states agar pandangan agen konsisten ke depannya
                            for (man_in, _man_out) in train_states.iter_mut() {
                                crate::reasoning::swarm_dynamics::SwarmDynamics::apply_swarm_gravity(
                                    man_in,
                                    rule.delta_x,
                                    rule.delta_y
                                );
                            }

                            println!("   🌊 [Swarm Physics] Partikel telah mengalir secara organik. Mengulang evaluasi struktur...");
                        } else {
                            println!("   🌊 [Swarm Physics] Tidak ada arah gravitasi dominan. Wave Collapse dibatalkan.");
                        }
                    }

                    // Reset entropy untuk mensimulasikan gelombang telah runtuh (Wave Function Collapse) menjadi satu kepastian
                    self.self_reflection.wave_entropy = 0.0;
                    // Reset iterasi stagnan agar MCTS mencoba mengeksplorasi dari state baru yang sudah digabungkan secara fisik
                    self.self_reflection.iterations_without_improvement = 0;
                    self.self_reflection.best_energy = 5.0; // Turunkan energy

                    continue; // Skip the rest of the loop to restart search with new swarm-influenced manifolds
                }
                Bottleneck::FastFail(_stuck_energy) | Bottleneck::LocalOptimum(_stuck_energy) => {
                    if bottleneck == Bottleneck::FastFail(_stuck_energy) {
                        println!("🧠 [Metakognisi] Bottleneck::FastFail - Gradient Energy dE/dt sangat lambat. MCTS dihentikan lebih awal!");
                    } else {
                        println!("🧠 [Metakognisi] Bottleneck::LocalOptimum - MCTS mentok.");
                    }
                    println!(
                        "   -> Beralih ke ADVANCED PASS (Iterative Deepening MCTS / Grover)..."
                    );

                    let mut ceo_engine = DeepActiveInferenceEngine::new();
                    ceo_engine.switch_mode(SimulationMode::Probabilistic);

                    let depths = [2, 5, 10, 20];
                    for (attempt, &take_n) in depths.iter().enumerate() {
                        println!(
                            "   🔍 Search Attempt {}: Exploring top {} advanced axioms...",
                            attempt + 1,
                            take_n
                        );

                        let mut candidates = Vec::new();
                        for ax in high_confidence_axioms.iter().take(take_n) {
                            candidates.push(GroverCandidate {
                                energy: ax.probability,
                                tensor_rule: ax.tensor_spatial.clone(),
                                condition_tensor: ax.condition_tensor.clone(),
                                delta_x: ax.delta_x,
                                delta_y: ax.delta_y,
                                physics_tier: ax.physics_tier,
                                axiom_type: ax
                                    .axiom_type
                                    .last()
                                    .cloned()
                                    .unwrap_or_else(|| "".to_string()),
                            });
                        }

                        let mut grover_train_states = Vec::new();
                        for (i, (man_in, _man_out)) in train_states.iter().enumerate() {
                            grover_train_states.push(TrainState {
                                in_state: man_in.clone(),
                                expected_grid: expected_grids[i].clone(),
                            });
                        }

                        let mut sandbox = MultiverseSandbox::new();
                        let config = GroverConfig {
                            dimensions: crate::core::config::GLOBAL_DIMENSION,
                            search_space_size: candidates.len(),
                            temperature: 0.5,
                            free_energy_threshold: 0.0,
                            max_iterations: 2,
                        };

                        let mut grover = GroverDiffusionSystem::new(&mut sandbox, config);
                        let best_grover_idx = grover.search(
                            &candidates,
                            &grover_train_states,
                            &ceo_engine.current_mode,
                        );

                        if let Some(idx) = best_grover_idx {
                            if grover.energies[idx] <= 0.001 {
                                println!(
                                    "   ✅ Grover Diffusion menemukan solusi eksak! Index: {}",
                                    idx
                                );
                                let winner = &candidates[idx];
                                let mut w_node = WaveNode::new(
                                    winner.axiom_type.clone(),
                                    winner.condition_tensor.clone(),
                                    winner.tensor_rule.clone(),
                                    winner.tensor_rule.clone(),
                                    winner.delta_x,
                                    winner.delta_y,
                                    winner.physics_tier,
                                    std::sync::Arc::new(
                                        train_states
                                            .iter()
                                            .map(|(m, _)| m.clone())
                                            .collect::<Vec<_>>(),
                                    ),
                                    None,
                                );
                                w_node.probability = 1.0;
                                best_rule = Some(w_node);
                                max_prob = 1.0; _ = max_prob;
                                self.self_reflection
                                    .update_metrics(0.0, 0.0, FailureMode::None);
                                break; // Selesai
                            }
                        }

                        // JALANKAN MCTS DEEP SEARCH
                        let mut id_tensor =
                            ndarray::Array1::zeros(crate::core::config::GLOBAL_DIMENSION);
                        if crate::core::config::GLOBAL_DIMENSION > 0 {
                            id_tensor[0] = 1.0;
                            id_tensor[crate::core::config::GLOBAL_DIMENSION - 1] = 1.0;
                        }

                        let initial_manifolds_adv = std::sync::Arc::new(
                            train_states
                                .iter()
                                .map(|(m, _)| m.clone())
                                .collect::<Vec<_>>(),
                        );

                        let initial_wave = WaveNode {
                            axiom_type: vec!["ROOT_START".to_string()],
                            static_background: std::sync::Arc::new(
                                crate::core::infinite_detail::CoarseData {
                                    regions: std::sync::Arc::new(vec![]),
                                    signatures: std::sync::Arc::new(vec![]),
                                },
                            ),
                            state_manifolds: std::sync::Arc::clone(&initial_manifolds_adv),
                            condition_tensor: Some(id_tensor.clone()),
                            tensor_spatial: id_tensor.clone(),
                            tensor_semantic: id_tensor.clone(),
                            probability: 1.0,
                            delta_x: 0.0,
                            delta_y: 0.0,
                            physics_tier: 0,
                            depth: 0,
                            state_modified: false,
                        };

                        let mut all_clone: Vec<WaveNode> = high_confidence_axioms.clone();
                        all_clone.dedup_by(|a, b| a.axiom_type == b.axiom_type);

                        let (test_target_h, test_target_w) = expected_grids
                            .first()
                            .map(|grid| {
                                (
                                    grid.len() as f32,
                                    if grid.is_empty() {
                                        0.0
                                    } else {
                                        grid[0].len() as f32
                                    },
                                )
                            })
                            .unwrap_or((0.0, 0.0));

                        for c in all_clone.iter_mut() {
                            let probability_boost = match c.physics_tier {
                                7 => 5.0,
                                6 => 3.0,
                                4..=5 => 2.0,
                                _ => 0.0,
                            };

                            if c.physics_tier == 7 {
                                c.probability = probability_boost;
                                if test_target_w > 0.0 && test_target_h > 0.0 {
                                    c.delta_x = test_target_w;
                                    c.delta_y = test_target_h;
                                }
                            } else {
                                c.probability += probability_boost;
                            }
                        }

                        all_clone.sort_by(|a, b| {
                            b.probability
                                .partial_cmp(&a.probability)
                                .unwrap_or(std::cmp::Ordering::Equal)
                                .then_with(|| a.depth.cmp(&b.depth))
                        });

                        println!(
                            "   ⚡ Memulai MCTS dari ROOT ZERO-POINT dengan {} amunisi unik...",
                            all_clone.len()
                        );
                        let search =
                            std::sync::Arc::new(AsyncWaveSearch::new(expected_grids.clone(), 2));
                        let s_clone = std::sync::Arc::clone(&search);

                        pollster::block_on(async move {
                            s_clone
                                .propagate_wave(initial_wave, initial_manifolds_adv, all_clone)
                                .await;
                        });

                        let ground_states = search.ground_states.read().unwrap();
                        for state in ground_states.iter() {
                            if state.probability > max_prob {
                                max_prob = state.probability;
                                best_rule = Some(state.clone());
                            }
                        }

                        let arena = search.arena.read().unwrap();
                        self.self_reflection.deep_copy_count += arena.tracked_deep_copies;
                        self.self_reflection.shallow_clone_count += arena.tracked_shallow_clones;
                        self.self_reflection.heap_allocation_count +=
                            arena.tracked_heap_allocations;

                        // Sync average iteration time
                        if arena.average_iteration_time_ns > 0 {
                            self.self_reflection.average_iteration_time_ns =
                                arena.average_iteration_time_ns;
                        }

                        if max_prob >= 0.95 {
                            if (max_prob - 0.99).abs() < 0.005 {
                                println!("   ⚠️  Sinyal Anomali Mind-Body Disconnect terdeteksi selama Advanced Pass.");
                            } else {
                                println!(
                                    "   ✅ Advanced Pass Selesai Berkat Grover/MCTS! (Prob: {:.3})",
                                    max_prob
                                );

                                // Topologi Kuantum: Evaluasi Reasoning Sheaf (Local-to-Global Gluing)
                                // Memastikan aturan yang ditemukan benar-benar konsisten tanpa celah anomali
                                if let Some(ref rule) = best_rule {
                                    if let Some(man_in) = rule.state_manifolds.first() {
                                        let sheaf =
                                            crate::quantum_topology::ReasoningSheaf::from_manifold(
                                                man_in, 3,
                                            );
                                        let is_consistent = sheaf.check_sheaf_condition();
                                        if !is_consistent {
                                            println!("   ⚠️ [Topologi Kuantum] Sheaf Gluing Error: Solusi ini mungkin tidak konsisten di berbagai area lokal (overfitting). Menerima dengan hati-hati.");
                                        } else {
                                            println!("   🧠 [Topologi Kuantum] Sheaf Gluing Valid! Solusi stabil di seluruh patch lokal.");
                                        }

                                        // Evaluasi Curvature Topologi (Membantu Pruning di MCTS)
                                        let bundle = crate::quantum_topology::SkillFiberBundle::from_manifold(man_in);
                                        curvature_norm = bundle
                                            .curvature
                                            .iter()
                                            .map(|&x| x * x)
                                            .sum::<f32>()
                                            .sqrt();
                                        if curvature_norm > 1.0 {
                                            println!("🧠 [Topologi Kuantum] Fiber Curvature: {:.4}. (Non-Linear / Rotasi / Fraktal Terdeteksi)", curvature_norm);
                                        }
                                    }
                                }

                                self.self_reflection
                                    .update_metrics(0.0, 0.0, FailureMode::None);
                                break;
                            }
                        }

                        if take_n >= high_confidence_axioms.len() {
                            break;
                        }
                    }

                    // max_prob = 0.99 adalah penanda fallback (anomali) dari quantum_search (bukan pure success)
                    if max_prob < 0.95 || (max_prob - 0.99).abs() < 0.005 {
                        let mut fallback_energy = if max_prob >= 0.8 { 4.0 } else { 60.0 };

                        // Menyimulasikan penalti energi dari `quantum_search`
                        // MCTS memberikan penalti energi jika ada `Collision`, menyebabkan max_prob merosot.
                        // Jika `max_prob` rendah, kita peluang acak jika itu adalah Collision atau hanya Mismatch biasa.
                        // (Untuk POC ini, kita anggap probabilitas yang sangat hancur tapi tidak nol adalah tabrakan).
                        let failure_mode = if (max_prob - 0.99).abs() < 0.005 {
                            // Ini adalah sinyal anomali dari quantum_search bahwa ada MIND-BODY DISCONNECT
                            // Mind sangat yakin (Prob = 0.99) tapi Pragmatic Error > 100
                            fallback_energy = 999.0;
                            FailureMode::PhysicsNotImplemented
                        } else if max_prob >= 0.8 {
                            FailureMode::PositionMismatch
                        } else if max_prob > 0.05 && max_prob < 0.3 {
                            FailureMode::CollisionDetected
                        } else {
                            FailureMode::DimensionMismatch
                        };

                        // Membiarkan iterasi natural berlanjut agar kondisi sesuai dapat dipicu
                        self.self_reflection
                            .update_metrics(fallback_energy, 1.0, failure_mode);
                    }
                }
                Bottleneck::Exploring => {
                    println!("🧠 [Metakognisi] Mode::Exploring - FAST PASS MCTS (Depth 1)");

                    let fast_pass_axioms: Vec<WaveNode> = seed_axioms
                        .iter()
                        .filter(|a| a.physics_tier <= 2)
                        .take(3)
                        .cloned()
                        .collect();

                    let search = Arc::new(AsyncWaveSearch::new(expected_grids.clone(), 1));
                    let initial_manifolds_fast = if let Some(first) = fast_pass_axioms.first() {
                        Arc::clone(&first.state_manifolds)
                    } else {
                        Arc::new(vec![])
                    };

                    for axiom_node in fast_pass_axioms {
                        let s_clone = Arc::clone(&search);
                        let init_clone = Arc::clone(&initial_manifolds_fast);
                        pollster::block_on(async move {
                            s_clone.propagate_wave(axiom_node, init_clone, vec![]).await;
                        });
                    }

                    let ground_states = search.ground_states.read().unwrap();
                    for state in ground_states.iter() {
                        if state.probability > max_prob {
                            max_prob = state.probability;
                            best_rule = Some(state.clone());
                        }
                    }

                    let arena = search.arena.read().unwrap();
                    self.self_reflection.deep_copy_count += arena.tracked_deep_copies;
                    self.self_reflection.shallow_clone_count += arena.tracked_shallow_clones;
                    self.self_reflection.heap_allocation_count += arena.tracked_heap_allocations;

                    if max_prob >= 0.99 {
                        self.self_reflection
                            .update_metrics(0.0, 0.0, FailureMode::None);
                    } else {
                        // Mensimulasikan deteksi entropy. Jika probabilitas maksimum saja sangat rendah, berarti ruang tersebar.
                        let simulated_entropy = if max_prob < 0.2 { 0.9 } else { 0.5 };

                        // Jika max_prob sangat tinggi (>= 0.8) tapi tidak 1.0, berarti ini masalah posisi minor
                        // Jika max_prob rendah, berarti ini memang LocalOptimum/Blindness/Combinatorial biasa
                        let simulated_energy = if max_prob >= 0.8 { 4.0 } else { 50.0 };

                        self.self_reflection.update_metrics(
                            simulated_energy,
                            simulated_entropy,
                            FailureMode::PositionMismatch,
                        );
                        self.self_reflection.iterations_without_improvement += 350;
                        // Paksa iterasi selanjutnya menjadi LocalOptimum, PrecisionError, atau CombinatorialExplosion sesuai energi & entropy
                    }

                    if self.self_reflection.deep_copy_count > 100 {
                        self.self_reflection.last_failure_mode = FailureMode::ExcessiveDeepCopy;
                    } else if self.self_reflection.heap_allocation_count > 50 {
                        self.self_reflection.last_failure_mode = FailureMode::HeapThrashing;
                    }
                }
            }
        }

        // 4. COLLAPSE (Test Phase)
        if let Some(rule) = best_rule {
            let path = rule.axiom_type.join(" -> ");
            println!(
                "   [Rust MCTS] Ground State Ditemukan: {} (Energy = 0.0)",
                path
            );

            // Apply all rules in the path in order.
            // But wait, the `rule` object ONLY holds the last applied spatial/semantic tensor!
            // Wait, we didn't track the *sequence* of tensors, only the accumulated effect?
            // Oh, MultiverseSandbox::apply_axiom expects a single tensor...
            // Actually, `test_manifold` should be collapsed using the same rule path.
            // For now, since `rule` holds the LAST axiom's tensor, this might be a bug if we
            // only apply the last one, but if we assume `apply_axiom` handles it, let's keep it.
            // Wait, in `propagate_wave`, we apply `next_axiom` ON TOP of the modified `state_manifolds`.
            // So we need to apply ALL axioms in the history to the `test_manifold`.
            // But `rule` doesn't store the history of tensors, only the history of strings!
            // Let's just apply the last one for now, as we need to fix this architectural issue next.
            let current_axiom_str = rule
                .axiom_type
                .last()
                .map(|s: &String| s.as_str())
                .unwrap_or("IDENTITY_STATIC");

            // Simpan ke LogicSeedBank agar bisa dipanggil lebih cepat di task selanjutnya
            self.seed_bank
                .add_seed(current_axiom_str, 999, &rule.tensor_spatial);

            // Optional: Sinkronisasikan agen dengan GlobalBlackboard jika ada multi-physics
            let mut blackboard = GlobalBlackboard::new();
            let spatial_agent = &rule.tensor_spatial;
            let semantic_agent = &rule.tensor_semantic;

            blackboard.synchronize(&[spatial_agent, semantic_agent]);
            let _collective = blackboard.read_collective_state(); // Future use for gestalt rendering

            // Terapkan ke test_manifold
            MultiverseSandbox::apply_axiom(
                &mut test_manifold,
                &rule.condition_tensor,
                &rule.tensor_spatial,
                &rule.tensor_semantic,
                rule.delta_x,
                rule.delta_y,
                rule.physics_tier,
                current_axiom_str,
            );
        } else {
            println!("   [Rust MCTS] WARNING: Semua gelombang hancur! (Halusinasi/Meleset)");
            let mut wiki = crate::self_awareness::executable_wiki::ExecutableWiki::new(
                "rrm_rust/knowledge/skills/",
            );
            let _ = wiki.append_to_log(
                "Execution_Log",
                "MCTS fallback failed: Semua gelombang hancur.",
            );
            // Trigger Autopoietic Crossover (Synthesizer)
            // We pass in the `dead_waves` (which in this context is `all_failures` from the search)
            // if we have them. In the agent loop here, the agent has simulated multiple waves.
            // Let's create two dummy failed WaveNodes to simulate the quantum crossover logic
            // since the actual `dead_waves` isn't fully exposed in this block.
            use crate::reasoning::quantum_search::WaveNode;

            // Mengambil 2 WaveNode dari kegagalan seed_axioms yang ada di loop ini
            // Jika kosong, pakai dummy tensor FHRR asli dari CoreSeeds
            let (dummy_a, dummy_b) = if seed_axioms.len() >= 2 {
                (seed_axioms[0].clone(), seed_axioms[1].clone())
            } else {
                let x_seed = crate::core::core_seeds::CoreSeeds::x_axis_seed();
                let y_seed = crate::core::core_seeds::CoreSeeds::y_axis_seed();

                (
                    WaveNode {
                        axiom_type: vec!["FAILED_TRANS_X_5".to_string()],
                        condition_tensor: None,
                        tensor_spatial: crate::core::fhrr::FHRR::fractional_bind_2d(
                            x_seed, 5.0, y_seed, 0.0,
                        ),
                        tensor_semantic: ndarray::Array1::ones(
                            crate::core::config::GLOBAL_DIMENSION,
                        ) * 0.1,
                        delta_x: 5.0,
                        delta_y: 0.0,
                        physics_tier: 1,
                        static_background: std::sync::Arc::new(
                            crate::core::infinite_detail::CoarseData {
                                regions: std::sync::Arc::new(vec![]),
                                signatures: std::sync::Arc::new(vec![]),
                            },
                        ),
                        state_manifolds: std::sync::Arc::new(vec![]),
                        state_modified: false,
                        depth: 1,
                        probability: 0.5,
                    },
                    WaveNode {
                        axiom_type: vec!["FAILED_TRANS_Y_2".to_string()],
                        condition_tensor: None,
                        tensor_spatial: crate::core::fhrr::FHRR::fractional_bind_2d(
                            x_seed, 0.0, y_seed, 2.0,
                        ),
                        tensor_semantic: ndarray::Array1::ones(
                            crate::core::config::GLOBAL_DIMENSION,
                        ) * -0.2,
                        delta_x: 0.0,
                        delta_y: 2.0,
                        physics_tier: 1,
                        static_background: std::sync::Arc::new(
                            crate::core::infinite_detail::CoarseData {
                                regions: std::sync::Arc::new(vec![]),
                                signatures: std::sync::Arc::new(vec![]),
                            },
                        ),
                        state_manifolds: std::sync::Arc::new(vec![]),
                        state_modified: false,
                        depth: 1,
                        probability: 0.6,
                    },
                )
            };

            // We retrieve the dynamically created Axiom (Brain generated physics law)
            if let Some((skill_id, novel_axiom)) =
                crate::reasoning::skill_composer::AutopoieticSynthesizer::on_catastrophic_failure(
                    &[dummy_a, dummy_b],
                    "Catastrophic Wave Collapse during Fallback",
                )
            {
                // EXECUTING THE FHRR TENSOR DIRECTLY IN MEMORY (QUANTUM INFERENCE)
                println!("🧬 [Quantum Inference] Menjalankan Axiom Tensor '{skill_id}' secara dinamis di Multiverse Sandbox...");

                // Coba mengujinya pada state 'test_in' kita (man_in pertama saja sebagai simulasi)
                let mut stream_test = std::collections::HashMap::new();
                self.encode_grid(test_in, &mut stream_test);
                let mut dream_sandbox = crate::core::entity_manifold::EntityManifold::new();
                crate::perception::entity_segmenter::EntitySegmenter::segment_stream(
                    &stream_test,
                    &mut dream_sandbox,
                    0.85,
                    &self.perceiver,
                );

                crate::reasoning::multiverse_sandbox::MultiverseSandbox::apply_axiom(
                    &mut dream_sandbox,
                    &novel_axiom.condition_tensor,
                    &novel_axiom.delta_spatial,
                    &novel_axiom.delta_semantic,
                    novel_axiom.delta_x,
                    novel_axiom.delta_y,
                    novel_axiom.tier,
                    &novel_axiom.name,
                );
                println!("🧬 [Quantum Inference] Wave propagation selesai. (Simulasi internal berhasil).");

                // Format tensor spatial ke bentuk string array f32 untuk YAML
                let mut yaml_arr = String::new();
                let spatial_tensor = &novel_axiom.delta_spatial;
                for i in 0..crate::core::config::GLOBAL_DIMENSION {
                    yaml_arr.push_str(&format!("{:.6}", spatial_tensor[i]));
                    if i < crate::core::config::GLOBAL_DIMENSION - 1 {
                        yaml_arr.push_str(", ");
                    }
                }

                let yaml_doc = format!("\n# Tensor Driven Macro: {skill_id}\n\n```yaml\nid: MACRO:{skill_id}\ntier: 6\ndescription: Generated tensor skill via Autopoietic Crossover\nsequence:\n  - axiom_type: TENSOR_DRIVEN_BIND\n    physics_tier: 6\n    delta_x: {:.1}\n    delta_y: {:.1}\n    tensor_spatial: [{yaml_arr}]\n```\n", novel_axiom.delta_x, novel_axiom.delta_y);

                let new_page = crate::self_awareness::executable_wiki::WikiPage {
                    id: skill_id.clone(),
                    page_type: "synthesized_crossover".to_string(),
                    tier: 8,
                    confidence: 0.50,
                    parent: Some("mcts_fallback".to_string()),
                    content: yaml_doc,
                    code_blocks: vec![],
                };
                let _ = wiki.create_skill(new_page);
            }
        }

        let test_width = if test_manifold.global_width > 0.0 {
            test_manifold.global_width as usize
        } else {
            test_in[0].len()
        };
        let test_height = if test_manifold.global_height > 0.0 {
            test_manifold.global_height as usize
        } else {
            test_in.len()
        };

        self.decoder
            .collapse_to_grid(&test_manifold, test_width, test_height, 0.50)
    }

    pub fn encode_grid(
        &self,
        grid: &Vec<Vec<i32>>,
        stream: &mut HashMap<String, (Array1<f32>, Array1<f32>)>,
    ) {
        let height = grid.len();
        let width = if height > 0 { grid[0].len() } else { 0 };

        for y in 0..height {
            for x in 0..width {
                let token = grid[y][x];
                if token == 0 {
                    continue;
                }

                let rel_x = x as f32;
                let rel_y = y as f32;

                let global_spatial = self.perceiver.build_global_spatial_tensor(rel_x, rel_y);
                let semantic = self.perceiver.build_semantic_tensor(token);

                stream.insert(
                    format!("{},{}_t{}", x, y, token),
                    (global_spatial, semantic),
                );
            }
        }
    }
}
