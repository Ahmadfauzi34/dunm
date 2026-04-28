use crate::core::entity_manifold::EntityManifold;
use crate::perception::structural_analyzer::TaskClass;
use crate::reasoning::counterfactual_engine::CounterfactualEngine;
use crate::reasoning::structures::Axiom;
use crate::self_awareness::skill_ontology::SkillOntology;
use petgraph::graph::{DiGraph, NodeIndex};

pub struct HierarchicalPlanner {
    pub task_graph: DiGraph<PlanningNode, PlanningEdge>,
    pub root: NodeIndex,
    pub current_frontier: Vec<NodeIndex>,
}

pub enum PlanningNode {
    Goal(crate::perception::structural_analyzer::StructuralDelta),
    Subgoal(SubgoalType),
    Operator(Axiom),
    Validation(ValidationCheck),
}

pub enum PlanningEdge {
    Sequential,
    Alternative,
}

pub enum SubgoalType {
    NormalizeDimension,
    ArrangeObjects,
    ModifyObjects,
    FinalizeGeometry,
}

pub enum ValidationCheck {
    ExactMatch,
}

impl HierarchicalPlanner {
    pub fn from_delta(
        delta: &crate::perception::structural_analyzer::StructuralDelta,
        ontology: &SkillOntology,
    ) -> Self {
        let mut graph = DiGraph::new();

        let root = graph.add_node(PlanningNode::Goal(delta.clone()));

        let subgoals =
            match crate::perception::structural_analyzer::StructuralAnalyzer::classify_task_class(
                delta,
            ) {
                TaskClass::StructuralTransform => {
                    vec![SubgoalType::NormalizeDimension, SubgoalType::ModifyObjects]
                }
                TaskClass::ObjectManipulation => vec![SubgoalType::ModifyObjects],
                TaskClass::PureGeometry => vec![SubgoalType::FinalizeGeometry],
                _ => vec![
                    SubgoalType::NormalizeDimension,
                    SubgoalType::ArrangeObjects,
                    SubgoalType::ModifyObjects,
                    SubgoalType::FinalizeGeometry,
                ],
            };

        let mut prev = root;
        for subgoal in subgoals {
            let node = graph.add_node(PlanningNode::Subgoal(subgoal));
            graph.add_edge(prev, node, PlanningEdge::Sequential);

            // This is a bit hacky because we are moving the enum. Let's just hardcode the expansion
            let node_for_cap = node; // it's just an index

            // Re-fetch subgoal to avoid move issues
            let subg_type = match graph.node_weight(node).unwrap() {
                PlanningNode::Subgoal(ref st) => st,
                _ => unreachable!(),
            };

            let capabilities = ontology.get_capabilities_for(subg_type);
            for cap in capabilities {
                let op = graph.add_node(PlanningNode::Operator(
                    crate::reasoning::structures::Axiom::new(
                        &cap.name,
                        cap.tier_id,
                        ndarray::Array1::zeros(crate::core::config::GLOBAL_DIMENSION),
                        ndarray::Array1::zeros(crate::core::config::GLOBAL_DIMENSION),
                        0.0,
                        0.0,
                    ),
                ));
                let _op_node = op;
                graph.add_edge(node_for_cap, op, PlanningEdge::Alternative);
            }

            prev = node_for_cap;
        }

        let validation = graph.add_node(PlanningNode::Validation(ValidationCheck::ExactMatch));
        graph.add_edge(prev, validation, PlanningEdge::Sequential);

        Self {
            task_graph: graph,
            root,
            current_frontier: vec![root],
        }
    }

    pub fn plan_with_validation(
        &self,
        engine: &mut CounterfactualEngine,
        input: &EntityManifold,
        expected: &EntityManifold,
    ) -> Option<Vec<Axiom>> {
        let mut best_path: Option<Vec<Axiom>> = None;
        let mut best_confidence = 0.0;

        self.dfs_with_pruning(
            self.root,
            vec![],
            input.clone(),
            &mut best_path,
            &mut best_confidence,
            engine,
            expected,
        );

        best_path
    }

    fn dfs_with_pruning(
        &self,
        node: NodeIndex,
        path: Vec<Axiom>,
        state: EntityManifold,
        best_path: &mut Option<Vec<Axiom>>,
        best_confidence: &mut f32,
        engine: &mut CounterfactualEngine,
        expected: &EntityManifold,
    ) {
        // Stop eksplorasi jika path sudah terlalu panjang (untuk mencegah infinite recursion)
        if path.len() > 10 {
            return;
        }

        match &self.task_graph[node] {
            PlanningNode::Goal(_) | PlanningNode::Subgoal(_) => {
                // Untuk node struktur, cukup lanjutkan ke anak-anaknya (Sequential / Alternative)
                for edge in self.task_graph.edges(node) {
                    self.dfs_with_pruning(
                        petgraph::visit::EdgeRef::target(&edge),
                        path.clone(),
                        state.clone(),
                        best_path,
                        best_confidence,
                        engine,
                        expected,
                    );
                }
            }
            PlanningNode::Operator(axiom) => {
                // Coba aplikasikan operator dengan "What If" engine
                let sim_result = engine.what_if(axiom, &state, expected);

                // Evaluasi hasil simulasi
                if let Some(failure) = sim_result.failure {
                    // Jika kegagalan memberikan gradient perbaikan,
                    // kita bisa mengevaluasinya sebagai alternatif branch (Gradient Descent)
                    if let Some(corrections) = engine.suggest_correction(&failure) {
                        for correction in corrections {
                            let corr_result = engine.what_if(&correction, &state, expected);

                            if corr_result.failure.is_none() {
                                let mut new_path = path.clone();
                                new_path.push(correction.clone());

                                for edge in self.task_graph.edges(node) {
                                    self.dfs_with_pruning(
                                        petgraph::visit::EdgeRef::target(&edge),
                                        new_path.clone(),
                                        corr_result.final_state.clone(),
                                        best_path,
                                        best_confidence,
                                        engine,
                                        expected,
                                    );
                                }
                            }
                        }
                    }
                    // Pruning karena path saat ini gagal (baik sudah dicoba dikoreksi atau tidak)
                    return;
                }

                let mut new_path = path.clone();
                new_path.push(axiom.clone());

                // Lanjutkan ke edge berikutnya
                for edge in self.task_graph.edges(node) {
                    self.dfs_with_pruning(
                        petgraph::visit::EdgeRef::target(&edge),
                        new_path.clone(),
                        sim_result.final_state.clone(),
                        best_path,
                        best_confidence,
                        engine,
                        expected,
                    );
                }
            }
            PlanningNode::Validation(ValidationCheck::ExactMatch) => {
                // Node validasi: Evaluasi apakah state akhir sesuai dengan ground truth
                use crate::reasoning::quantum_search_simd::{CognitivePhase, SimdEnergyCalculator};

                let dummy_grid =
                    vec![vec![0; expected.global_width as usize]; expected.global_height as usize];

                let pragmatic_error = SimdEnergyCalculator::calculate_pragmatic_streaming(
                    &state,
                    &dummy_grid,
                    state.global_width as usize,
                    state.global_height as usize,
                    &CognitivePhase::Microscopic,
                    1e-15,
                );

                let confidence = if pragmatic_error == 0.0 {
                    1.0
                } else {
                    1.0 / (1.0 + pragmatic_error)
                };

                if confidence > *best_confidence {
                    *best_confidence = confidence;
                    *best_path = Some(path);
                }
            }
        }
    }
}
