use crate::perception::structural_analyzer::{
    DimensionRelation, ObjectDelta, StructuralAnalyzer, StructuralDelta, StructuralSignature,
    TaskClass, TopologyHint,
};
use crate::reasoning::structures::Axiom;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SkillOntology {
    pub capabilities: HashMap<u8, TierCapability>,
    pub applicability_index: HashMap<StructuralSignature, Vec<u8>>,
    pub usage_history: Vec<SkillUsage>,
    pub transition_rules: Vec<TransitionRule>,
}

#[derive(Clone)]
pub struct TierCapability {
    pub tier_id: u8,
    pub name: String,
    pub description: String,
    pub activation_triggers: Vec<ActivationTrigger>,
    pub preconditions: Vec<Precondition>,
    pub postconditions: Vec<Postcondition>,
    pub side_effects: Vec<SideEffect>,
    pub cost: f32,
    pub historical_success_rate: f32,
    pub typical_signatures: Vec<StructuralSignature>,
}

impl TierCapability {
    pub fn to_axiom(&self) -> Axiom {
        use crate::core::config::GLOBAL_DIMENSION;
        use ndarray::Array1;
        Axiom::new(
            &self.name,
            self.tier_id,
            Array1::zeros(GLOBAL_DIMENSION),
            Array1::zeros(GLOBAL_DIMENSION),
            0.0,
            0.0,
        )
    }
}

#[derive(Clone)]
pub enum ActivationTrigger {
    Dimension(DimensionRelation),
    ObjectCount(ObjectDelta),
    TopologyChange(TopologyHint, TopologyHint),
    HasTemplateFrame,
    ColorMismatch,
    SymmetryIssue,
}

#[derive(Clone)]
pub enum Precondition {
    MinObjects(usize),
    MaxObjects(usize),
    HasColor(u8),
    DimensionAtLeast(u8, u8),
    TopologyIs(TopologyHint),
    ObjectsAre(TopologyHint),
}

#[derive(Clone, PartialEq)]
pub enum Postcondition {
    ObjectsPreserved,
    DimensionChanged,
    ObjectsAdded(usize),
    ObjectsRemoved(usize),
    ColorsMapped(Vec<(u8, u8)>),
    TopologyBecomes(TopologyHint),
    SymmetryRestored,
}

#[derive(Clone, PartialEq)]
pub enum SideEffect {
    BackgroundRemoved,
    TemplateMarkerLost,
    PositionReset,
    BoundingBoxChanged,
}

#[derive(Clone)]
pub struct SkillUsage {
    pub task_id: String,
    pub tier_used: u8,
    pub context_signature: StructuralSignature,
    pub success: bool,
    pub execution_time_ms: u64,
}

#[derive(Clone)]
pub struct TransitionRule {
    pub from_tier: u8,
    pub to_tier: u8,
    pub compatibility: TransitionCompatibility,
    pub reason: String,
}

#[derive(Clone)]
pub enum TransitionCompatibility {
    Always,
    Conditional,
    Risky,
    Never,
}

pub enum CompositionAdvice {
    Recommended,
    TestInCounterfactual,
    ProceedWithCaution,
    Forbidden,
}

pub enum SolutionStrategy {
    DirectExecution {
        primary_skill: u8,
    },
    TemplateDriven {
        structural_skill: u8,
        refinement_skills: Vec<u8>,
    },
    ObjectCentricSearch {
        detection_skill: u8,
        transformation_skills: Vec<u8>,
        max_depth: usize,
    },
    HierarchicalPlanning {
        subgoals: Vec<u8>,
    },
    ProgramSynthesis {
        max_program_depth: usize,
        primitive_set: Vec<u8>,
    },
    IterativeDeepening {
        skills_to_try: Vec<u8>,
        max_iterations: usize,
    },
}

impl SkillOntology {
    pub fn initialize() -> Self {
        let mut ontology = Self {
            capabilities: HashMap::new(),
            applicability_index: HashMap::new(),
            usage_history: Vec::new(),
            transition_rules: Vec::new(),
        };

        ontology.register_tier_0_translation();
        ontology.register_tier_7_crop();

        // Coba load dinamis dari Executable Wiki
        let mut wiki = crate::self_awareness::executable_wiki::ExecutableWiki::new(
            "rrm_rust/knowledge/skills/",
        );
        if let Ok(count) = wiki.load_all() {
            if count > 0 {
                println!(
                    "📚 Berhasil meload {} skill dari Executable Wiki (.md)",
                    count
                );
            }
            for (_, page) in wiki.knowledge_base.iter() {
                let cap = TierCapability {
                    tier_id: page.tier,
                    name: page.id.clone(),
                    description: format!("(WIKI) {}", page.page_type),
                    activation_triggers: vec![],
                    preconditions: vec![],
                    postconditions: vec![],
                    side_effects: vec![],
                    cost: 1.0,
                    historical_success_rate: page.confidence,
                    typical_signatures: vec![],
                };
                ontology.capabilities.insert(page.tier, cap);
            }
        }

        ontology.build_transition_rules();
        ontology.index_capabilities();

        ontology
    }

    pub fn introspect(&self, signature: &StructuralSignature) -> Vec<&TierCapability> {
        if let Some(tiers) = self.applicability_index.get(signature) {
            return tiers
                .iter()
                .filter_map(|id| self.capabilities.get(id))
                .collect();
        }
        self.fuzzy_match_capabilities(signature)
    }

    pub fn can_solve(&self, delta: &StructuralDelta) -> Option<SolutionStrategy> {
        let class = StructuralAnalyzer::classify_task_class(delta);
        let available = self.introspect(&delta.signature);

        match class {
            TaskClass::PureGeometry => {
                let geom_skills: Vec<_> = available.iter().filter(|c| c.tier_id == 4).collect();
                if !geom_skills.is_empty() {
                    Some(SolutionStrategy::DirectExecution {
                        primary_skill: geom_skills[0].tier_id,
                    })
                } else {
                    Some(SolutionStrategy::DirectExecution { primary_skill: 4 })
                }
            }
            TaskClass::StructuralTransform => {
                let structural: Vec<_> = available.iter().filter(|c| c.tier_id == 7).collect();
                if !structural.is_empty() {
                    Some(SolutionStrategy::TemplateDriven {
                        structural_skill: structural[0].tier_id,
                        refinement_skills: self.find_compatible_refinements(7, &available),
                    })
                } else {
                    Some(SolutionStrategy::TemplateDriven {
                        structural_skill: 7,
                        refinement_skills: vec![],
                    })
                }
            }
            TaskClass::ObjectManipulation => {
                let manipulators: Vec<_> = available
                    .iter()
                    .filter(|c| c.tier_id <= 6 && c.tier_id >= 3)
                    .collect();
                if manipulators.len() >= 2 {
                    Some(SolutionStrategy::ObjectCentricSearch {
                        detection_skill: 0,
                        transformation_skills: manipulators.iter().map(|c| c.tier_id).collect(),
                        max_depth: 3,
                    })
                } else {
                    Some(SolutionStrategy::ObjectCentricSearch {
                        detection_skill: 0,
                        transformation_skills: vec![],
                        max_depth: 3,
                    })
                }
            }
            TaskClass::RelationalRearrangement => Some(SolutionStrategy::HierarchicalPlanning {
                subgoals: vec![3, 0],
            }),
            TaskClass::AlgorithmicPattern => Some(SolutionStrategy::ProgramSynthesis {
                max_program_depth: 4,
                primitive_set: available.iter().map(|c| c.tier_id).collect(),
            }),
            _ => Some(SolutionStrategy::IterativeDeepening {
                skills_to_try: available.iter().map(|c| c.tier_id).collect(),
                max_iterations: 20,
            }),
        }
    }

    pub fn can_compose(&self, first: u8, second: u8) -> CompositionAdvice {
        for rule in &self.transition_rules {
            if rule.from_tier == first && rule.to_tier == second {
                return match rule.compatibility {
                    TransitionCompatibility::Always => CompositionAdvice::Recommended,
                    TransitionCompatibility::Conditional => CompositionAdvice::TestInCounterfactual,
                    TransitionCompatibility::Risky => CompositionAdvice::ProceedWithCaution,
                    TransitionCompatibility::Never => CompositionAdvice::Forbidden,
                };
            }
        }
        CompositionAdvice::TestInCounterfactual
    }

    pub fn learn_from_usage(&mut self, usage: SkillUsage) {
        let tier_id = usage.tier_used;
        if let Some(cap) = self.capabilities.get_mut(&tier_id) {
            let n = cap.historical_success_rate;
            cap.historical_success_rate = (n + if usage.success { 1.0 } else { 0.0 }) / 2.0;
            if usage.success {
                cap.typical_signatures.push(usage.context_signature.clone());
                if cap.typical_signatures.len() > 100 {
                    cap.typical_signatures.remove(0);
                }
            }
        }
        self.usage_history.push(usage);
    }

    fn register_tier_0_translation(&mut self) {
        self.capabilities.insert(
            0,
            TierCapability {
                tier_id: 0,
                name: "ConditionalTranslation".to_string(),
                description: "Move objects".to_string(),
                activation_triggers: vec![ActivationTrigger::ObjectCount(ObjectDelta::SameCount)],
                preconditions: vec![Precondition::MinObjects(1)],
                postconditions: vec![Postcondition::ObjectsAdded(0)],
                side_effects: vec![],
                cost: 1.0,
                historical_success_rate: 0.5,
                typical_signatures: vec![],
            },
        );
    }

    fn register_tier_7_crop(&mut self) {
        self.capabilities.insert(
            7,
            TierCapability {
                tier_id: 7,
                name: "CropToContent".to_string(),
                description: "Remove background".to_string(),
                activation_triggers: vec![
                    ActivationTrigger::Dimension(DimensionRelation::Smaller),
                    ActivationTrigger::HasTemplateFrame,
                ],
                preconditions: vec![
                    Precondition::MinObjects(1),
                    Precondition::TopologyIs(TopologyHint::Framed),
                ],
                postconditions: vec![Postcondition::DimensionChanged],
                side_effects: vec![
                    SideEffect::BackgroundRemoved,
                    SideEffect::TemplateMarkerLost,
                ],
                cost: 2.0,
                historical_success_rate: 0.7,
                typical_signatures: vec![],
            },
        );
    }

    fn build_transition_rules(&mut self) {
        use TransitionCompatibility::*;
        let rules = vec![
            (7, 4, Always, "Crop then Geometry"),
            (0, 0, Always, "Multiple translations"),
            (6, 7, Always, "Spawn then Crop"),
            (7, 6, Always, "Crop then Spawn"),
            (4, 7, Risky, "Geometry then Crop"),
            (4, 4, Conditional, "Geometry+Geometry"),
            (5, 6, Conditional, "Erase then Spawn"),
            (7, 7, Never, "Double Crop"),
        ];

        for (from, to, compat, reason) in rules {
            self.transition_rules.push(TransitionRule {
                from_tier: from,
                to_tier: to,
                compatibility: compat,
                reason: reason.to_string(),
            });
        }
    }

    fn index_capabilities(&mut self) {}

    fn fuzzy_match_capabilities(&self, signature: &StructuralSignature) -> Vec<&TierCapability> {
        let mut scored: Vec<(f32, &TierCapability)> = self
            .capabilities
            .values()
            .map(|cap| {
                let score = cap
                    .typical_signatures
                    .iter()
                    .map(|sig| signature.similarity_score(sig))
                    .fold(0.0f32, |a: f32, b: f32| a.max(b));
                (score, cap)
            })
            .filter(|(s, _)| *s > 0.5)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().map(|(_, cap)| cap).collect()
    }

    pub fn find_compatible_refinements(
        &self,
        _primary: u8,
        _available: &[&TierCapability],
    ) -> Vec<u8> {
        vec![]
    }

    pub fn get_capabilities_for(
        &self,
        _subgoal: &crate::reasoning::hierarchical_planner::SubgoalType,
    ) -> Vec<&TierCapability> {
        self.capabilities.values().collect()
    }
}

impl StructuralSignature {
    pub fn similarity_score(&self, other: &StructuralSignature) -> f32 {
        let mut score = 0.0;
        let mut weights = 0.0;

        if self.dim_relation == other.dim_relation {
            score += 3.0;
        }
        weights += 3.0;

        if self.object_delta == other.object_delta {
            score += 2.0;
        }
        weights += 2.0;

        if self.topology_in == other.topology_in && self.topology_out == other.topology_out {
            score += 2.0;
        }
        weights += 2.0;

        if self.has_template_frame == other.has_template_frame {
            score += 1.0;
        }
        weights += 1.0;

        let color_matches = self
            .color_transitions
            .iter()
            .filter(|ct| other.color_transitions.contains(ct))
            .count() as f32;
        score += color_matches.min(2.0);
        weights += 2.0;

        score / weights
    }
}

// Ensure TaskMemory builds
#[derive(Clone)]
pub struct TaskMemory {
    pub initial_state: crate::core::entity_manifold::EntityManifold,
    pub expected_state: crate::core::entity_manifold::EntityManifold,
    pub solution_path: Vec<Axiom>,
}

impl TaskMemory {
    pub fn random_color_swap(&self) -> Vec<(u8, u8)> {
        vec![(1, 2)]
    }
}
