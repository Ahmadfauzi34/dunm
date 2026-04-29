use crate::reasoning::quantum_search::WaveNode;
use crate::reasoning::structures::Axiom;
use crate::self_awareness::skill_ontology::{
    Postcondition as PropertyGuarantee, Precondition as PropertyRequirement, SkillOntology,
};

pub struct SkillComposer {
    pub primitives: Vec<PrimitiveSkill>,
    pub composed: Vec<ComposedSkill>,
    pub composition_history: Vec<CompositionAttempt>,
}

#[derive(Clone)]
pub struct PrimitiveSkill {
    pub name: String,
    pub tier: u8,
    pub output_guarantees: Vec<PropertyGuarantee>,
}

impl PrimitiveSkill {
    pub fn to_axiom(&self) -> Axiom {
        use crate::core::config::GLOBAL_DIMENSION;
        use ndarray::Array1;
        Axiom::new(
            &self.name,
            self.tier,
            Array1::zeros(GLOBAL_DIMENSION),
            Array1::zeros(GLOBAL_DIMENSION),
            0.0,
            0.0,
        )
    }
}

pub struct ComposedSkill {
    pub sequence: Vec<PrimitiveSkill>,
    pub preconditions: Vec<PropertyRequirement>,
    pub postconditions: Vec<PropertyGuarantee>,
    pub emergence_properties: Vec<EmergenceProperty>,
    pub usage_count: usize,
    pub success_rate: f32,
}

pub struct CompositionAttempt;

pub enum EmergenceProperty {
    TemplateAwareRotation,
    ContextAwareFill,
}

impl Default for SkillComposer {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillComposer {
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
            composed: Vec::new(),
            composition_history: Vec::new(),
        }
    }

    pub fn compose_binary(&mut self, ontology: &SkillOntology) {
        let mut new_composed = Vec::new();
        for a in &self.primitives {
            for b in &self.primitives {
                if self.are_semantically_compatible(a, b, ontology) {
                    let composed = ComposedSkill {
                        sequence: vec![a.clone(), b.clone()],
                        preconditions: self.merge_preconditions(a, b),
                        postconditions: self.infer_postconditions(a, b),
                        emergence_properties: self.detect_emergence(a, b),
                        usage_count: 0,
                        success_rate: 0.0,
                    };
                    new_composed.push(composed);
                }
            }
        }
        self.composed.extend(new_composed);
    }

    fn are_semantically_compatible(
        &self,
        a: &PrimitiveSkill,
        b: &PrimitiveSkill,
        _ontology: &SkillOntology,
    ) -> bool {
        match (a.tier, b.tier) {
            (7, 4) => true,
            (4, 7) => a
                .output_guarantees
                .contains(&PropertyGuarantee::ObjectsPreserved),
            (6, 7) => true,
            (7, 6) => true,
            (4, 4) => self.are_commutative(a, b),
            _ => true,
        }
    }

    fn detect_emergence(&self, a: &PrimitiveSkill, b: &PrimitiveSkill) -> Vec<EmergenceProperty> {
        let mut emergent = Vec::new();
        if a.tier == 7 && b.tier == 4 {
            emergent.push(EmergenceProperty::TemplateAwareRotation);
        }
        if a.tier == 6 && b.tier == 7 {
            emergent.push(EmergenceProperty::ContextAwareFill);
        }
        emergent
    }

    fn are_commutative(&self, _a: &PrimitiveSkill, _b: &PrimitiveSkill) -> bool {
        true
    }

    fn merge_preconditions(
        &self,
        _a: &PrimitiveSkill,
        _b: &PrimitiveSkill,
    ) -> Vec<PropertyRequirement> {
        vec![]
    }

    fn infer_postconditions(
        &self,
        _a: &PrimitiveSkill,
        _b: &PrimitiveSkill,
    ) -> Vec<PropertyGuarantee> {
        vec![]
    }

    pub fn generate_novel_combinations(
        &self,
        _ontology: &SkillOntology,
        _base: &[Axiom],
    ) -> Vec<ComposedSkill> {
        vec![]
    }
}

/// Autopoietic Synthesizer: Menghasilkan kode Rust generatif dari Quantum Crossover
pub struct AutopoieticSynthesizer;

impl AutopoieticSynthesizer {
    pub fn on_catastrophic_failure(
        dead_waves: &[WaveNode],
        trigger_task: &str,
    ) -> Option<(String, Axiom)> {
        if dead_waves.len() < 2 {
            return None; // Butuh minimal 2 kegagalan untuk crossover
        }

        // 1. Quantum Crossover: Superposisi dari 2 ide terbaik yang gagal
        let node_a = &dead_waves[0];
        let node_b = &dead_waves[1];

        let mut novel_spatial = &node_a.tensor_spatial * 0.6 + &node_b.tensor_spatial * 0.4;
        let mut sq_sum: f32 = 0.0;
        for &v in novel_spatial.iter() {
            sq_sum += v * v;
        }
        let inv_mag = 1.0 / (sq_sum.sqrt() + 1e-15);
        for v in novel_spatial.iter_mut() {
            *v *= inv_mag;
        }

        let mut novel_semantic = &node_a.tensor_semantic * 0.5 + &node_b.tensor_semantic * 0.5;
        let mut sq_sum2: f32 = 0.0;
        for &v in novel_semantic.iter() {
            sq_sum2 += v * v;
        }
        let inv_mag2 = 1.0 / (sq_sum2.sqrt() + 1e-15);
        for v in novel_semantic.iter_mut() {
            *v *= inv_mag2;
        }

        let tensor_str = novel_spatial
            .iter()
            .take(5)
            .map(|v| format!("{v:.4}"))
            .collect::<Vec<_>>()
            .join(", ");

        let parent_a_name = node_a
            .axiom_type
            .last()
            .unwrap_or(&"UNKNOWN".to_string())
            .clone();
        let parent_b_name = node_b
            .axiom_type
            .last()
            .unwrap_or(&"UNKNOWN".to_string())
            .clone();

        let mut hasher = blake3::Hasher::new();
        hasher.update(trigger_task.as_bytes());
        hasher.update(parent_a_name.as_bytes());
        hasher.update(parent_b_name.as_bytes());
        let hash_hex = hasher.finalize().to_hex().to_string();
        let short_hash = &hash_hex[0..8];
        let skill_id = format!("synthesized_crossover_{}", short_hash);

        let md_content = format!(
            r#"---
id: {}
type: synthesized
confidence: 0.50
parent: mcts_fallback
---

## Origin
Generated from catastrophic failure.
- Trigger Task: {}
- Method: Quantum Crossover of failed WaveNodes.
- Parents: [{:?}, {:?}]

## Synthesis Tensor Approximation (First 5 dims)
[{}]

## Autopoietic Algorithm
```rust
use crate::core::entity_manifold::EntityManifold;
use ndarray::Array1;

pub fn execute_novel_skill(input: &mut EntityManifold) -> Result<(), String> {{
    // Apply Novel Spatial Tensor generated by Autopoietic Crossover
    // The actual tensor applies a non-linear continuous shift
    println!("🧬 Menerapkan aksioma yang disintesis sendiri: {}");
    Ok(())
}}
```
"#,
            skill_id, trigger_task, parent_a_name, parent_b_name, tensor_str, skill_id
        );

        let out_dir = std::path::PathBuf::from("knowledge/skills/auto");
        let _ = std::fs::create_dir_all(&out_dir);
        let out_path = out_dir.join(format!("{}.md", skill_id));

        if out_path.exists() {
            println!("🧬 [Autopoiesis] Kode genetik sudah ada di Wiki ({:?}). Menggunakan kembali tanpa menulis file baru.", out_path);
        } else {
            let _ = std::fs::write(&out_path, &md_content);
            println!(
                "🧬 [Autopoiesis] Kode genetik baru berhasil disintesis dan ditulis ke {:?}",
                out_path
            );
        }

        // Bypassing compiler: Create the executable Axiom purely in memory!
        let dynamic_axiom = Axiom {
            name: skill_id.clone(),
            tier: 1, // Primary tier
            condition_tensor: None,
            delta_spatial: novel_spatial,
            delta_semantic: novel_semantic,
            delta_x: node_a.delta_x * 0.6 + node_b.delta_x * 0.4,
            delta_y: node_a.delta_y * 0.6 + node_b.delta_y * 0.4,
            _state: std::marker::PhantomData,
        };

        Some((skill_id, dynamic_axiom))
    }
}
