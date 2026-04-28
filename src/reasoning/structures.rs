use crate::core::entity_manifold::EntityManifold;
use ndarray::Array1;
use std::marker::PhantomData;

/// ZST (Zero-Sized Type) for State Typing
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unverified;

/// ZST (Zero-Sized Type) for State Typing
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verified;

#[derive(Clone, Debug, PartialEq)]
pub struct Axiom<State = Unverified> {
    pub name: String,
    pub tier: u8,
    pub condition_tensor: Option<Array1<f32>>,
    pub delta_spatial: Array1<f32>,
    pub delta_semantic: Array1<f32>,
    pub delta_x: f32,
    pub delta_y: f32,
    pub _state: PhantomData<State>,
}

impl Axiom<Unverified> {
    pub fn new(
        name: &str,
        tier: u8,
        delta_spatial: Array1<f32>,
        delta_semantic: Array1<f32>,
        delta_x: f32,
        delta_y: f32,
    ) -> Self {
        Self {
            name: name.to_string(),
            tier,
            condition_tensor: None,
            delta_spatial,
            delta_semantic,
            delta_x,
            delta_y,
            _state: PhantomData,
        }
    }

    pub fn crop_to_content() -> Self {
        use crate::core::config::GLOBAL_DIMENSION;
        Self::new(
            "CROP_TO_COLOR",
            7,
            Array1::zeros(GLOBAL_DIMENSION),
            Array1::zeros(GLOBAL_DIMENSION),
            0.0,
            0.0,
        )
    }

    pub fn identity() -> Self {
        use crate::core::config::GLOBAL_DIMENSION;
        Self::new(
            "IDENTITY",
            0,
            Array1::zeros(GLOBAL_DIMENSION),
            Array1::zeros(GLOBAL_DIMENSION),
            0.0,
            0.0,
        )
    }

    /// Consume this Unverified Axiom and transition it to a Verified Axiom at compile time.
    /// This is a Zero-Cost Abstraction.
    pub fn verify(self) -> Axiom<Verified> {
        Axiom {
            name: self.name,
            tier: self.tier,
            condition_tensor: self.condition_tensor,
            delta_spatial: self.delta_spatial,
            delta_semantic: self.delta_semantic,
            delta_x: self.delta_x,
            delta_y: self.delta_y,
            _state: PhantomData,
        }
    }
}

impl<State> Axiom<State> {
    /// Transmutes state without data mutation, for generalized mapping if needed.
    pub fn into_unverified(self) -> Axiom<Unverified> {
        Axiom {
            name: self.name,
            tier: self.tier,
            condition_tensor: self.condition_tensor,
            delta_spatial: self.delta_spatial,
            delta_semantic: self.delta_semantic,
            delta_x: self.delta_x,
            delta_y: self.delta_y,
            _state: PhantomData,
        }
    }
}

/// 🌟 ZERO-COST ABSTRACTIONS FACADE 🌟
/// API ini menyembunyikan FHRR Math tingkat rendah ke dalam antarmuka semantik kognitif.
pub struct CognitiveAbstractions;

impl CognitiveAbstractions {
    /// Wrapper API berlevel tinggi untuk interference pattern.
    /// `#[inline(always)]` menjamin bahwa kompiler meratakan panggilan ini menjadi operasi SIMD murni,
    /// sehingga menghilangkan beban `call stack` namun menjaga kode tetap bersih (bebas Cognitive Debt).
    #[inline(always)]
    pub fn optimize_reasoning_paths(
        wave_a: &Array1<f32>,
        wave_b: &Array1<f32>,
        dominance_a: f32,
    ) -> Array1<f32> {
        let dominance_b = 1.0 - dominance_a;
        // Constructive Interference approximation
        let mut novel = wave_a * dominance_a + wave_b * dominance_b;

        // Fast L2 Normalization (L2 norm)
        let mut sq_sum = 0.0;
        for &v in novel.iter() {
            sq_sum += v * v;
        }
        let inv_mag = 1.0 / (sq_sum.sqrt() + 1e-15);
        for v in novel.iter_mut() {
            *v *= inv_mag;
        }

        novel
    }
}

#[derive(Clone, Debug)]
pub struct OldStructuralDelta {
    pub input_dim: (usize, usize),
    pub output_dim: (usize, usize),
}

impl OldStructuralDelta {
    pub fn analyze(input: &EntityManifold, output: &EntityManifold) -> Self {
        Self {
            input_dim: (input.global_width as usize, input.global_height as usize),
            output_dim: (output.global_width as usize, output.global_height as usize),
        }
    }

    pub fn consensus(deltas: &[Self]) -> Self {
        deltas.first().cloned().unwrap_or(Self {
            input_dim: (0, 0),
            output_dim: (0, 0),
        })
    }

    pub fn classify(&self) -> TaskClass {
        if self.input_dim != self.output_dim {
            TaskClass::StructuralTransform
        } else {
            TaskClass::PureGeometry
        }
    }

    pub fn to_signature(&self) -> StructuralSignature {
        StructuralSignature {
            dim_relation: DimensionRelation::Equal,
            object_delta: ObjectDelta::Same,
            color_mapping: None,
            topology_hint: TopologyHint::Grid,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskClass {
    PureGeometry,
    ObjectManipulation,
    StructuralTransform,
    Unknown,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct StructuralSignature {
    pub dim_relation: DimensionRelation,
    pub object_delta: ObjectDelta,
    pub color_mapping: Option<Vec<(u8, u8)>>,
    pub topology_hint: TopologyHint,
}

impl StructuralSignature {
    pub fn matches(&self, other: &Self) -> bool {
        self.dim_relation == other.dim_relation && self.topology_hint == other.topology_hint
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum DimensionRelation {
    Larger,
    Smaller,
    Equal,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ObjectDelta {
    Added,
    Removed,
    Same,
    Transformed,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum TopologyHint {
    Scatter,
    Grid,
    Linear,
    Nested,
}

impl TopologyHint {
    pub fn random() -> Self {
        Self::Grid // Placeholder
    }
}
