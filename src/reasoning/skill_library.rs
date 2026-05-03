use crate::reasoning::quantum_search::WaveNode;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MacroSkill {
    pub id: String,
    pub sequence: Vec<WaveNode>,
    pub utility_score: f32,
}

pub struct SkillLibrary {
    pub macros: HashMap<String, MacroSkill>,
}

impl Default for SkillLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillLibrary {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    ///
    /// # Panics
    /// Panics if `winning_path` is somehow empty but bypasses the `is_empty` check.
    pub fn register_chunk(&mut self, winning_path: &[WaveNode]) {
        if winning_path.len() <= 1 {
            return;
        }

        // We can't map h.description directly since WaveNode doesn't have it.
        // Instead, we use axiom_type (the sequence of strings) of the FINAL winning node!
        // A single WaveNode that won already stores its full history inside `axiom_type: Vec<String>`.
        // So `winning_path` might just be a single node, let's look at its axiom_type path.
        let Some(final_node) = winning_path.last() else { return; };

        // Filter out ROOT_START
        let path: Vec<String> = final_node
            .axiom_type
            .iter()
            .filter(|s| *s != "ROOT_START")
            .cloned()
            .collect();

        if path.len() <= 1 {
            return;
        } // Need at least 2 steps to form a macro

        let id = path.join("|");

        let entry = self.macros.entry(id.clone()).or_insert(MacroSkill {
            id: format!("MACRO:{}", id),
            sequence: winning_path.to_vec(), // Original node reference
            utility_score: 0.0,
        });

        entry.utility_score += 1.0; // Hebbian Learning: neurons that fire together, wire together
    }

    pub fn load_grammar_from_wiki(&mut self, wiki_dir: &str) {
        if let Ok(entries) = std::fs::read_dir(wiki_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "md" {
                                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                    self.parse_markdown_grammar(&content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn parse_markdown_grammar(&mut self, markdown: &str) {
        let mut content_str = "";

        if let Some(start) = markdown.find("```yaml") {
            if let Some(end) = markdown[start + 7..].find("```") {
                content_str = markdown[start + 7..start + 7 + end].trim();
            }
        } else if let Some(start) = markdown.find("```json") {
            if let Some(end) = markdown[start + 7..].find("```") {
                content_str = markdown[start + 7..start + 7 + end].trim();
            }
        }

        if !content_str.is_empty() {
            if let Ok(parsed) = serde_yaml::from_str::<serde_yaml::Value>(content_str) {
                if let Some(id) = parsed["id"].as_str() {
                    let mut sequence = Vec::new();
                    let base_tensor =
                        ndarray::Array1::<f32>::zeros(crate::core::config::GLOBAL_DIMENSION);

                    if let Some(seq_array) = parsed["sequence"].as_sequence() {
                        for step in seq_array {
                            let axiom_type =
                                step["axiom_type"].as_str().unwrap_or("UNKNOWN").to_string();
                            let dx = step["delta_x"].as_f64().unwrap_or(0.0) as f32;
                            let dy = step["delta_y"].as_f64().unwrap_or(0.0) as f32;
                            let tier = step["physics_tier"].as_u64().unwrap_or(5) as u8;

                            let mut t_spatial = base_tensor.clone();
                            if let Some(arr) = step["tensor_spatial"].as_sequence() {
                                for (idx, val) in arr.iter().enumerate() {
                                    if idx < crate::core::config::GLOBAL_DIMENSION {
                                        t_spatial[idx] = val.as_f64().unwrap_or(0.0) as f32;
                                    }
                                }
                            }

                            let mut t_semantic = base_tensor.clone();
                            if let Some(arr) = step["tensor_semantic"].as_sequence() {
                                for (idx, val) in arr.iter().enumerate() {
                                    if idx < crate::core::config::GLOBAL_DIMENSION {
                                        t_semantic[idx] = val.as_f64().unwrap_or(0.0) as f32;
                                    }
                                }
                            }

                            let node = WaveNode {
                                axiom_type: vec![axiom_type],
                                condition_tensor: None,
                                tensor_spatial: t_spatial,
                                tensor_semantic: t_semantic,
                                delta_x: dx,
                                delta_y: dy,
                                physics_tier: tier,
                                static_background: std::sync::Arc::new(
                                    crate::core::infinite_detail::CoarseData {
                                        regions: std::sync::Arc::new(vec![]),
                                        signatures: std::sync::Arc::new(vec![]),
                                    },
                                ),
                                state_manifolds: std::sync::Arc::new(Vec::new()),
                                state_modified: false,
                                probability: 10.0,
                                depth: 0,
                            };
                            sequence.push(node);
                        }
                    }

                    if !sequence.is_empty() {
                        self.macros.insert(
                            id.to_string(),
                            MacroSkill {
                                id: id.to_string(),
                                sequence,
                                utility_score: 50.0, // High starting utility
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn inject_macros_as_hypotheses(&self) -> Vec<WaveNode> {
        self.macros
            .values()
            .map(|macro_skill| {
                let first_axiom = &macro_skill.sequence[0];

                WaveNode {
                    condition_tensor: first_axiom.condition_tensor.clone(),
                    tensor_spatial: first_axiom.tensor_spatial.clone(),
                    tensor_semantic: first_axiom.tensor_semantic.clone(),
                    probability: 10.0, // VIP Pass: Sangat memprioritaskan prosedur yang sudah terbukti
                    axiom_type: vec![macro_skill.id.clone()],
                    delta_x: 0.0,
                    delta_y: 0.0,
                    depth: 0,
                    physics_tier: 8, // Tier Makro
                    static_background: std::sync::Arc::new(
                        crate::core::infinite_detail::CoarseData {
                            regions: std::sync::Arc::new(vec![]),
                            signatures: std::sync::Arc::new(vec![]),
                        },
                    ),
                    state_manifolds: first_axiom.state_manifolds.clone(),
                    state_modified: false,
                }
            })
            .collect()
    }
}
