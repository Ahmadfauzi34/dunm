use crate::reasoning::counterfactual_engine::CounterfactualEngine;
use crate::reasoning::grover_diffusion_system::{
    GroverCandidate, GroverConfig, GroverDiffusionSystem, TrainState,
};
use crate::reasoning::hierarchical_inference::SimulationMode;
use crate::reasoning::multiverse_sandbox::MultiverseSandbox;
use crate::reasoning::skill_composer::{ComposedSkill, SkillComposer};
use crate::reasoning::structures::Axiom;
use crate::reasoning::structures::TopologyHint;
use crate::self_awareness::skill_ontology::{SkillOntology, TaskMemory};

pub struct MentalReplay {
    pub solved_tasks: Vec<TaskMemory>,
    pub dream_scenarios: Vec<CounterfactualScenario>,
    pub skill_composer: SkillComposer,
}

pub struct CounterfactualScenario {
    pub base_task: TaskMemory,
    pub variation: ScenarioVariation,
    pub difficulty_modifier: f32,
}

impl CounterfactualScenario {
    pub fn apply_variation(
        &self,
        state: &crate::core::entity_manifold::EntityManifold,
    ) -> crate::core::entity_manifold::EntityManifold {
        state.clone()
    }
}

pub enum ScenarioVariation {
    SizeScaling(f32),
    ColorPermutation(Vec<(u8, u8)>),
    NoiseInjection(f32),
    TopologyChange(TopologyHint),
}

impl Default for MentalReplay {
    fn default() -> Self {
        Self::new()
    }
}

impl MentalReplay {
    pub fn new() -> Self {
        Self {
            solved_tasks: Vec::new(),
            dream_scenarios: Vec::new(),
            skill_composer: SkillComposer::new(),
        }
    }

    pub fn generate_dreams(&mut self, count: usize) {
        for task in self.solved_tasks.iter() {
            for i in 0..count {
                let variation = match i % 4 {
                    0 => ScenarioVariation::SizeScaling(1.5 + (i as f32 * 0.5)),
                    1 => ScenarioVariation::ColorPermutation(vec![(1, 2)]),
                    2 => ScenarioVariation::NoiseInjection(0.1 * (i as f32)),
                    _ => ScenarioVariation::TopologyChange(TopologyHint::random()),
                };

                let dream = CounterfactualScenario {
                    base_task: crate::self_awareness::skill_ontology::TaskMemory::clone(task),

                    variation,
                    difficulty_modifier: 1.0 + (i as f32 * 0.2),
                };

                self.dream_scenarios.push(dream);
            }
        }
    }

    pub fn practice_in_dreams(
        &mut self,
        __engine: &mut CounterfactualEngine,
        __ontology: &SkillOntology,
    ) -> Vec<ComposedSkill> {
        let mut discovered_skills = Vec::new();

        for scenario in &self.dream_scenarios {
            let dream_state = scenario.base_task.initial_state.clone();
            let expected_dream_state = scenario.apply_variation(&scenario.base_task.expected_state);

            // Create a dummy sandbox and config for Grover
            let mut sandbox = MultiverseSandbox::new();
            let grover_config = GroverConfig {
                dimensions: crate::core::config::GLOBAL_DIMENSION,
                search_space_size: 5, // We'll test with 5 top axioms
                temperature: 0.1,
                free_energy_threshold: 0.05,
                max_iterations: 3,
            };

            let mut grover = GroverDiffusionSystem::new(&mut sandbox, grover_config);

            // Fetch base axioms
            let mut axioms = Vec::new();
            let x_seed = ndarray::Array1::<f32>::ones(crate::core::config::GLOBAL_DIMENSION);
            let y_seed = ndarray::Array1::<f32>::ones(crate::core::config::GLOBAL_DIMENSION);

            // Gradient Descent Mental Replay
            // Kita coba Axiom pertama dengan (dx=0, dy=0) lalu simulasi Counterfactual Engine
            let initial_axiom = Axiom::new(
                "DREAM_AXIOM_INIT",
                0,
                x_seed.clone(),
                y_seed.clone(),
                0.0,
                0.0,
            );

            let outcome = __engine.what_if(&initial_axiom, &dream_state, &expected_dream_state);

            let mut optimal_dx = 0.0;
            let mut optimal_dy = 0.0;

            if let Some(failure) = outcome.failure {
                let crate::reasoning::counterfactual_engine::FailureMode::HighEnergyState {
                    gradient_x,
                    gradient_y,
                    ..
                } = failure;
                if true {
                    // Kompas Ditemukan!
                    // Mental Replay tidak lagi melempar tebakan Noise Acak (DREAM_AXIOM_0 .. DREAM_AXIOM_5)
                    // Mental replay menggunakan Vektor Gradien untuk membuat tebakan yang pasti benar.
                    optimal_dx = gradient_x.to_f32().round();
                    optimal_dy = gradient_y.to_f32().round();
                }
            }

            for i in 0..5 {
                // Modifikasi gradient sedikit demi mengejar deterministik (Gradient Descent simulation)
                let d_x = optimal_dx + (i as f32 * 0.1);
                let d_y = optimal_dy;
                let t =
                    crate::reasoning::axiom_generator::AxiomGenerator::generate_translation_axiom(
                        d_x, d_y, &x_seed, &y_seed,
                    );
                let ax = Axiom::new(
                    &format!("DREAM_AXIOM_{}_{}_{}", i, d_x, d_y),
                    0,
                    t.clone(),
                    t,
                    d_x,
                    d_y,
                );
                axioms.push(ax);
            }
            let mut candidates = Vec::new();
            for axiom in axioms.iter().take(5) {
                candidates.push(GroverCandidate {
                    energy: 1.0,
                    tensor_rule: axiom.delta_spatial.clone(),
                    condition_tensor: axiom.condition_tensor.clone(),
                    delta_x: axiom.delta_x,
                    delta_y: axiom.delta_y,
                    physics_tier: axiom.tier,
                    axiom_type: axiom.name.clone(),
                });
            }

            let expected_grid = vec![
                vec![0; expected_dream_state.global_width as usize];
                expected_dream_state.global_height as usize
            ];
            let train_states = vec![TrainState {
                in_state: dream_state.clone(),
                expected_grid,
            }];

            let mode = SimulationMode::Counterfactual;

            if let Some(winner_idx) = grover.search(&candidates, &train_states, &mode) {
                // If grover found a universal axiom for this dream variation!
                println!("🌙 [Grover Dreamer] Menemukan Universal Axiom baru untuk skenario mimpi! Pemenang: {}", candidates[winner_idx].axiom_type);

                // Construct a ComposedSkill (dummy representation)
                let composed = ComposedSkill {
                    preconditions: vec![],
                    postconditions: vec![],
                    emergence_properties: vec![],
                    sequence: vec![],
                    usage_count: 1,
                    success_rate: 1.0,
                };
                discovered_skills.push(composed);
            }
        }

        discovered_skills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mental_replay_new() {
        let replay = MentalReplay::new();

        assert!(replay.solved_tasks.is_empty());
        assert!(replay.dream_scenarios.is_empty());

        // Ensure SkillComposer is also empty
        assert!(replay.skill_composer.primitives.is_empty());
        assert!(replay.skill_composer.composed.is_empty());
        assert!(replay.skill_composer.composition_history.is_empty());
    }

    #[test]
    fn test_mental_replay_default() {
        let replay = MentalReplay::default();

        assert!(replay.solved_tasks.is_empty());
        assert!(replay.dream_scenarios.is_empty());

        assert!(replay.skill_composer.primitives.is_empty());
        assert!(replay.skill_composer.composed.is_empty());
        assert!(replay.skill_composer.composition_history.is_empty());
    }
}
