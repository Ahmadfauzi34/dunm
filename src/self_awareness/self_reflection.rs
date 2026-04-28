use crate::core::entity_manifold::EntityManifold;
use crate::perception::structural_analyzer::*;
use crate::self_awareness::skill_ontology::*;
use std::time::Instant;

/// Kategori Kegagalan yang dialami agen (untuk membantu routing)
#[derive(Debug, Clone, PartialEq)]
pub enum FailureMode {
    None,
    DimensionMismatch,
    ColorMismatch,
    TopologyMismatch,
    PositionMismatch,
    CollisionDetected, // Terjadi tabrakan dengan rintangan saat mencoba bergerak
    PhysicsNotImplemented, // Pikiran (Tensor) sangat yakin, tapi Tubuh (Grid) tidak bisa mengeksekusi
    ExcessiveDeepCopy,     // Deteksi Memory Bloat karena CoW Violation
    HeapThrashing,         // Terlalu banyak alokasi dinamis (Vec::push) di Hot Path MCTS
    GhostStateDetected, // Banyaknya entitas bermassa 0.0 (Dark Matter) di dalam array EntityManifold
}

/// Status Bottleneck Kognitif RRM
#[derive(Debug, Clone, PartialEq)]
pub enum Bottleneck {
    Distracted, // Agen menyadari 90% CPU terbuang untuk memproses background mati
    Blindness,
    LocalOptimum(f32),
    CombinatorialExplosion,
    PrecisionError,
    ObstacleStuck,       // Agen menyadari pergerakannya terhalang rintangan
    BodyLimitation, // Agen sadar ia kekurangan kapabilitas fisik (membutuhkan upgrade kode manusia)
    MemoryBloat,    // Mengeluh karena Deep Copy Arrays berlebihan
    AllocationThrashing, // Agen merasakan nyeri karena "alloc/malloc" dinamis (Heap) berlebihan
    CognitiveGarbage, // RRM mengeluh Array penuh sampah/Ghost State yang memperlambat iterasi SIMD
    FalseSharing,   // Amnesia Singkat (Cache Misses parah)
    FastFail(f32),  // Kesabaran dinamis, dE/dt terlalu lambat di fase awal
    Solved,
    Exhausted,
    Exploring,
}

pub struct SelfReflection {
    ontology: SkillOntology,
    current_context: Option<StructuralSignature>,

    // Metrik Pemantauan Diri (Cognitive Metrics)
    pub start_time: Instant,
    pub time_spent_ms: u64,
    pub average_iteration_time_ns: u64, // Waktu rata-rata iterasi entity manifold (Cache miss estimation)
    pub best_energy: f32,
    pub iterations_without_improvement: usize,
    pub wave_entropy: f32, // Semakin tinggi = probabilitas menyebar (tebakan buta)
    pub active_saliency_ratio: f32, // Rasio area grid yang benar-benar berubah (0.0 - 1.0)
    pub dark_matter_ratio: f32, // Rasio entitas dengan massa 0 (sampah / ghost state) vs total active_count (0.0 - 1.0)
    pub initial_energy: Option<f32>, // Energi saat awal pencarian, untuk mengukur Gradient of Energy
    pub deep_copy_count: usize,      // Pelacakan rasa sakit memori (CoW violations)
    pub shallow_clone_count: usize,
    pub heap_allocation_count: usize, // Pelacakan alokasi heap (push dinamis melebihi kapasitas awal)
    pub last_failure_mode: FailureMode,
    pub total_iterations: usize,
}

pub struct IntrospectionReport {
    pub situation_assessment: String,
    pub available_skills: Vec<SkillExplanation>,
    pub recommended_strategy: String,
    pub confidence_explanation: String,
    pub alternative_approaches: Vec<String>,
}

pub struct SkillExplanation {
    pub name: String,
    pub why_applicable: String,
    pub expected_outcome: String,
    pub risks: Vec<String>,
    pub historical_performance: f32,
}

pub struct ConsequencePrediction {
    pub guaranteed_effects: Vec<Postcondition>,
    pub likely_side_effects: Vec<SideEffect>,
    pub possible_risks: Vec<String>,
    pub estimated_success_probability: f32,
}

impl SelfReflection {
    pub fn new(ontology: SkillOntology) -> Self {
        Self {
            ontology,
            current_context: None,
            start_time: Instant::now(),
            time_spent_ms: 0,
            average_iteration_time_ns: 0,
            best_energy: f32::MAX,
            iterations_without_improvement: 0,
            wave_entropy: 0.0,
            active_saliency_ratio: 1.0,
            dark_matter_ratio: 0.0,
            deep_copy_count: 0,
            shallow_clone_count: 0,
            heap_allocation_count: 0,
            last_failure_mode: FailureMode::None,
            total_iterations: 0,
            initial_energy: None,
        }
    }

    /// Mereset kondisi metrik kognitif setiap kali soal baru dimulai
    pub fn reset_metrics(&mut self) {
        self.start_time = Instant::now();
        self.time_spent_ms = 0;
        self.average_iteration_time_ns = 0;
        self.best_energy = f32::MAX;
        self.iterations_without_improvement = 0;
        self.wave_entropy = 0.0;
        self.active_saliency_ratio = 1.0; // Default: seluruh area penting
        self.dark_matter_ratio = 0.0;
        self.deep_copy_count = 0;
        self.shallow_clone_count = 0;
        self.heap_allocation_count = 0;
        self.last_failure_mode = FailureMode::None;
        self.total_iterations = 0;
        self.initial_energy = None;
    }

    /// Mengupdate metrik kognitif berdasarkan hasil komputasi terkini
    pub fn update_metrics(&mut self, current_energy: f32, entropy: f32, failure: FailureMode) {
        self.time_spent_ms = self.start_time.elapsed().as_millis() as u64;
        self.wave_entropy = entropy;
        self.last_failure_mode = failure;
        self.total_iterations += 1;

        if self.initial_energy.is_none() {
            self.initial_energy = Some(current_energy);
        }

        if current_energy < self.best_energy {
            self.best_energy = current_energy;
            self.iterations_without_improvement = 0;
        } else {
            self.iterations_without_improvement += 1;
        }
    }

    /// Menganalisis kondisi pikiran (bottleneck) berdasarkan metrik saat ini
    pub fn assess_current_bottleneck(&mut self) -> Bottleneck {
        self.time_spent_ms = self.start_time.elapsed().as_millis() as u64;

        if self.best_energy <= 0.001 {
            return Bottleneck::Solved;
        }

        if self.time_spent_ms > 30_000 || self.total_iterations > 5000 {
            // Jika sudah telalu lama, menyerah
            return Bottleneck::Exhausted;
        }

        if self.last_failure_mode == FailureMode::GhostStateDetected
            || self.dark_matter_ratio >= 0.5
        {
            // Jika lebih dari 50% dari iterasi array agent (active_count) hanyalah entitas bermassa 0.0 ("Dark Matter"),
            // berarti SIMD dan Cache L1 terbuang percuma untuk branching 'if mass == 0.0 { continue }'
            return Bottleneck::CognitiveGarbage;
        }

        if self.last_failure_mode == FailureMode::HeapThrashing {
            // Nyeri akibat Heap Memory Allocation
            return Bottleneck::AllocationThrashing;
        }

        if self.last_failure_mode == FailureMode::ExcessiveDeepCopy {
            // Agen merasakan nyeri alokasi memori berlebih
            return Bottleneck::MemoryBloat;
        }

        if self.average_iteration_time_ns > 500
            && self.last_failure_mode != FailureMode::GhostStateDetected
        {
            // Jika iterasi per piksel memakan waktu > 500 ns (ini sangat lambat untuk standar CPU 3GHz yang butuh ~0.3ns per instruksi)
            // Sistem mendeteksi adanya "Amnesia Singkat" (Cache Misses yang parah).
            return Bottleneck::FalseSharing;
        }

        if self.active_saliency_ratio < 0.2 && self.total_iterations <= 1 {
            // Evaluasi Saliency di awal pencarian:
            // Jika area yang benar-benar berubah (aktif) sangat kecil dibanding total grid,
            // agen sadar bahwa ia sedang "Distracted" oleh background yang membuang CPU.
            return Bottleneck::Distracted;
        }

        if self.last_failure_mode == FailureMode::PhysicsNotImplemented {
            // Agen menyadari ada diskoneksi antara pikiran logis (Tensor) dan kemampuan fisik (Sandbox)
            return Bottleneck::BodyLimitation;
        }

        if self.last_failure_mode == FailureMode::CollisionDetected {
            // Agen mencoba bergeser tapi menabrak dinding / rintangan. Harus mencari celah.
            return Bottleneck::ObstacleStuck;
        }

        if self.last_failure_mode == FailureMode::DimensionMismatch && self.best_energy >= 50.0 {
            // Agen tidak melihat gambaran besarnya (mungkin noise / segmentasi salah)
            return Bottleneck::Blindness;
        }

        // Fast-Fail MCTS: Gradient of Energy Check
        // Jika agen tidak membuat banyak kemajuan di 10 iterasi awal, jangan buang waktu. Langsung gagal-cepat.
        if self.total_iterations >= 10 && self.total_iterations < 20 {
            if let Some(init_e) = self.initial_energy {
                let energy_delta = init_e - self.best_energy;
                // Jika energinya turun kurang dari 5% dari initial energy, berarti mentok
                if energy_delta < init_e * 0.05 {
                    return Bottleneck::FastFail(self.best_energy);
                }
            }
        }

        if self.iterations_without_improvement > 300 {
            // Mentok. Tidak ada kemajuan setelah sekian iterasi.
            return Bottleneck::LocalOptimum(self.best_energy);
        }

        if self.wave_entropy > 0.8 && self.total_iterations > 1 {
            // Tebakan terlalu acak, probabilitas sangat tersebar pada awal pencarian.
            // Kita butuh Wave/Swarm Dynamics untuk meruntuhkannya secara organik.
            return Bottleneck::CombinatorialExplosion;
        }

        if self.best_energy > 0.0
            && self.best_energy < 5.0
            && self.last_failure_mode == FailureMode::PositionMismatch
        {
            // Sudah sangat dekat (energy kecil), tapi ada piksel meleset.
            // Gunakan kalkulus eksak (Counterfactual Femto)
            return Bottleneck::PrecisionError;
        }

        Bottleneck::Exploring
    }

    pub fn assess_situation(&mut self, delta: &StructuralDelta) -> IntrospectionReport {
        self.current_context = Some(delta.signature.clone());

        let class = StructuralAnalyzer::classify_task_class(delta);
        let available = self.ontology.introspect(&delta.signature);
        let strategy = self.ontology.can_solve(delta);

        IntrospectionReport {
            situation_assessment: self.describe_situation(delta, &class),
            available_skills: available
                .iter()
                .map(|cap| self.explain_skill(cap, delta))
                .collect(),
            recommended_strategy: self.describe_strategy(&strategy),
            confidence_explanation: self.explain_confidence(&strategy, &available),
            alternative_approaches: self.suggest_alternatives(delta, &strategy),
        }
    }

    pub fn explain_decision(&self, chosen_skill: u8, rejected: &[u8]) -> String {
        let chosen = self
            .ontology
            .capabilities
            .get(&chosen_skill)
            .expect("Invalid skill ID");

        let mut explanation = format!("Saya memilih {} karena:\n", chosen.name);

        explanation.push_str("- Situasi cocok dengan kondisi aktivasi:\n");
        for trigger in &chosen.activation_triggers {
            explanation.push_str(&format!("  • {}\n", self.describe_trigger(trigger)));
        }

        if !rejected.is_empty() {
            explanation.push_str("\nAlternatif yang saya pertimbangkan tapi tolak:\n");
            for &rej_id in rejected {
                if let Some(rej) = self.ontology.capabilities.get(&rej_id) {
                    let reason = self.explain_rejection(rej, chosen);
                    explanation.push_str(&format!("- {}: {}\n", rej.name, reason));
                }
            }
        }

        explanation.push_str(&format!(
            "\nPerforma historis skill ini: {:.0}% sukses",
            chosen.historical_success_rate * 100.0
        ));

        explanation
    }

    pub fn predict_consequences(
        &self,
        skill_id: u8,
        current: &EntityManifold,
    ) -> ConsequencePrediction {
        let skill = self
            .ontology
            .capabilities
            .get(&skill_id)
            .expect("Invalid skill");

        ConsequencePrediction {
            guaranteed_effects: skill.postconditions.clone(),
            likely_side_effects: skill.side_effects.clone(),
            possible_risks: self.identify_risks(skill, current),
            estimated_success_probability: skill.historical_success_rate,
        }
    }

    fn describe_situation(&self, delta: &StructuralDelta, class: &TaskClass) -> String {
        let sig = &delta.signature;

        format!(
            "Task ini menunjukkan: {}. \
             Dimensi {} ({} → {}), \
             {} objek ({} → {}), \
             topologi berubah dari {:?} ke {:?}. \
             {} template frame.",
            self.class_name(class),
            self.describe_dim_change(&sig.dim_relation),
            delta.input_stats.bounding_box.0,
            delta.output_stats.bounding_box.0,
            self.describe_object_change(&sig.object_delta),
            delta.input_stats.count,
            delta.output_stats.count,
            sig.topology_in,
            sig.topology_out,
            if sig.has_template_frame {
                "Ada"
            } else {
                "Tidak ada"
            }
        )
    }

    fn explain_skill(&self, cap: &TierCapability, _delta: &StructuralDelta) -> SkillExplanation {
        SkillExplanation {
            name: cap.name.clone(),
            why_applicable: self.match_triggers(&cap.activation_triggers, _delta),
            expected_outcome: self.describe_postconditions(&cap.postconditions),
            risks: cap
                .side_effects
                .iter()
                .map(|se| self.describe_side_effect(se))
                .collect(),
            historical_performance: cap.historical_success_rate,
        }
    }

    fn identify_risks(&self, skill: &TierCapability, current: &EntityManifold) -> Vec<String> {
        let mut risks = Vec::new();

        for side_effect in &skill.side_effects {
            match side_effect {
                SideEffect::BackgroundRemoved => {
                    if current.active_count > 10 {
                        risks.push(
                            "Mungkin menghapus objek penting sebagai 'background'".to_string(),
                        );
                    }
                }
                SideEffect::TemplateMarkerLost => {
                    risks.push(
                        "Frame/template akan hilang, tidak bisa digunakan untuk alignment"
                            .to_string(),
                    );
                }
                SideEffect::PositionReset => {
                    risks.push(
                        "Koordinat akan berubah, relational positioning mungkin gagal".to_string(),
                    );
                }
                _ => {}
            }
        }

        risks
    }

    fn class_name(&self, class: &TaskClass) -> String {
        format!("{:?}", class)
    }

    fn describe_dim_change(&self, change: &DimensionRelation) -> String {
        format!("{:?}", change)
    }

    fn describe_object_change(&self, change: &ObjectDelta) -> String {
        format!("{:?}", change)
    }

    fn match_triggers(&self, _triggers: &[ActivationTrigger], _delta: &StructuralDelta) -> String {
        "Matched structural attributes".to_string()
    }

    fn describe_postconditions(&self, _posts: &[Postcondition]) -> String {
        "Expected changes".to_string()
    }

    fn describe_side_effect(&self, effect: &SideEffect) -> String {
        match effect {
            SideEffect::BackgroundRemoved => "Background removed".to_string(),
            SideEffect::TemplateMarkerLost => "Template marker lost".to_string(),
            SideEffect::PositionReset => "Position reset".to_string(),
            SideEffect::BoundingBoxChanged => "Bounding box changed".to_string(),
        }
    }

    fn describe_strategy(&self, strategy: &Option<SolutionStrategy>) -> String {
        if strategy.is_some() {
            "Available".to_string()
        } else {
            "None".to_string()
        }
    }

    fn explain_confidence(
        &self,
        _strategy: &Option<SolutionStrategy>,
        _available: &[&TierCapability],
    ) -> String {
        "Estimated via heuristic".to_string()
    }

    fn suggest_alternatives(
        &self,
        _delta: &StructuralDelta,
        _strategy: &Option<SolutionStrategy>,
    ) -> Vec<String> {
        vec![]
    }

    fn describe_trigger(&self, _trigger: &ActivationTrigger) -> String {
        "Trigger".to_string()
    }

    fn explain_rejection(&self, _rej: &TierCapability, _chosen: &TierCapability) -> String {
        "Lower score".to_string()
    }
}
