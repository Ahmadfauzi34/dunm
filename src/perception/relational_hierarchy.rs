use crate::perception::hierarchical_gestalt::{AtomType, GestaltAtom};

/// Level 2: Relational structures antara atoms
#[derive(Clone, Debug)]
pub struct RelationalStructure {
    pub structure_type: StructureType,
    pub atoms: Vec<usize>, // Indices ke list GestaltAtom
    pub relations: Vec<SpatialRelation>,
    pub bounding_box: (f32, f32, f32, f32),
    pub symmetry_group: Option<SymmetryGroup>,
}

#[derive(Clone, Debug)]
pub enum StructureType {
    ContainerWithContent, // Hollow rect dengan sesuatu di dalam
    NestedBoxes,          // Kotak dalam kotak
    GridPattern,          // Array teratur (NxM)
    LinearSequence,       // Baris/kolom berurutan
    MirrorPair,           // Dua objek simetris
    FrameAndFill,         // Border + interior berbeda
    ScatteredGroup,       // Kumpulan tidak teratur
}

#[derive(Clone, Debug)]
pub enum SpatialRelation {
    Inside,     // A di dalam B
    Contains,   // A berisi B
    AdjacentTo, // A menempel B
    Above,
    Below,
    Left,
    Right,       // Relasi directional
    AlignedWith, // Sejajar
    SymmetricTo, // Simetris terhadap
}

#[derive(Clone, Debug)]
pub struct SymmetryGroup {
    pub axis: SymmetryAxis,
    pub members: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum SymmetryAxis {
    Vertical(f32),            // x = constant
    Horizontal(f32),          // y = constant
    Diagonal,                 // y = x or y = -x
    Rotational(f32, f32, u8), // (cx, cy, order)
}

pub struct RelationalEngine;

impl RelationalEngine {
    /// Build relational hierarchy dari atoms
    /// O(n^2) tapi n = jumlah atoms (kecil, << pixel count)
    pub fn analyze_relations(atoms: &[GestaltAtom]) -> Vec<RelationalStructure> {
        let mut structures = Vec::new();
        let n = atoms.len();

        if n == 0 {
            return structures;
        }

        // 1. Detect containers (hollow rects dengan content)
        for (i, atom) in atoms.iter().enumerate() {
            if let AtomType::HollowRectangle = atom.atom_type {
                let mut contents = Vec::new();

                for (j, other) in atoms.iter().enumerate() {
                    if i == j {
                        continue;
                    }

                    if Self::is_inside(other, atom) {
                        contents.push(j);
                    }
                }

                if !contents.is_empty() {
                    structures.push(RelationalStructure {
                        structure_type: StructureType::ContainerWithContent,
                        atoms: std::iter::once(i).chain(contents.iter().copied()).collect(),
                        relations: vec![SpatialRelation::Contains; contents.len() + 1],
                        bounding_box: atom.bounding_box,
                        symmetry_group: None,
                    });
                }
            }
        }

        // 2. Detect nested structures
        for i in 0..n {
            for j in (i + 1)..n {
                if Self::is_nested(&atoms[i], &atoms[j]) {
                    structures.push(RelationalStructure {
                        structure_type: StructureType::NestedBoxes,
                        atoms: vec![i, j],
                        relations: vec![SpatialRelation::Contains],
                        bounding_box: Self::union_bbox(
                            &atoms[i].bounding_box,
                            &atoms[j].bounding_box,
                        ),
                        symmetry_group: None,
                    });
                }
            }
        }

        // 3. Detect grid patterns
        if let Some(grid) = Self::detect_grid(atoms) {
            structures.push(grid);
        }

        // 4. Detect mirror symmetry antara objects
        if let Some(mirror) = Self::detect_mirror_pair(atoms) {
            structures.push(mirror);
        }

        structures
    }

    fn is_inside(inner: &GestaltAtom, outer: &GestaltAtom) -> bool {
        let (omin_x, omin_y, omax_x, omax_y) = outer.bounding_box;
        let (imin_x, imin_y, imax_x, imax_y) = inner.bounding_box;

        imin_x > omin_x && imin_y > omin_y && imax_x < omax_x && imax_y < omax_y
    }

    fn is_nested(outer: &GestaltAtom, inner: &GestaltAtom) -> bool {
        matches!(
            outer.atom_type,
            AtomType::HollowRectangle | AtomType::SolidRectangle
        ) && matches!(
            inner.atom_type,
            AtomType::HollowRectangle | AtomType::SolidRectangle
        ) && Self::is_inside(inner, outer)
    }

    fn detect_grid(atoms: &[GestaltAtom]) -> Option<RelationalStructure> {
        if atoms.len() < 4 {
            return None;
        }

        // Group by similar size dan spacing
        let mut x_positions: Vec<f32> = atoms.iter().map(|a| a.center_of_mass.0).collect();
        let mut y_positions: Vec<f32> = atoms.iter().map(|a| a.center_of_mass.1).collect();

        x_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        y_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Check uniform spacing
        let x_spacings: Vec<f32> = x_positions.windows(2).map(|w| w[1] - w[0]).collect();
        let y_spacings: Vec<f32> = y_positions.windows(2).map(|w| w[1] - w[0]).collect();

        let x_uniform = Self::is_uniform(&x_spacings);
        let y_uniform = Self::is_uniform(&y_spacings);

        if x_uniform && y_uniform && atoms.len() >= 4 {
            Some(RelationalStructure {
                structure_type: StructureType::GridPattern,
                atoms: (0..atoms.len()).collect(),
                relations: vec![SpatialRelation::AlignedWith; atoms.len()],
                bounding_box: Self::bbox_of_all(atoms),
                symmetry_group: None,
            })
        } else {
            None
        }
    }

    fn detect_mirror_pair(atoms: &[GestaltAtom]) -> Option<RelationalStructure> {
        for (i, a) in atoms.iter().enumerate() {
            for (j, b) in atoms.iter().enumerate().skip(i + 1) {
                if Self::are_mirrors(a, b) {
                    return Some(RelationalStructure {
                        structure_type: StructureType::MirrorPair,
                        atoms: vec![i, j],
                        relations: vec![SpatialRelation::SymmetricTo],
                        bounding_box: Self::union_bbox(&a.bounding_box, &b.bounding_box),
                        symmetry_group: Some(SymmetryGroup {
                            axis: SymmetryAxis::Vertical(
                                (a.center_of_mass.0 + b.center_of_mass.0) / 2.0,
                            ),
                            members: vec![i, j],
                        }),
                    });
                }
            }
        }
        None
    }

    fn are_mirrors(a: &GestaltAtom, b: &GestaltAtom) -> bool {
        // Same type, similar size, symmetric positions
        a.atom_type == b.atom_type
            && (a.pixel_count as f32 / b.pixel_count as f32 - 1.0).abs() < 0.1
            && (a.aspect_ratio - b.aspect_ratio).abs() < 0.1
    }

    fn is_uniform(spacings: &[f32]) -> bool {
        if spacings.len() < 2 {
            return true;
        }
        let mean = spacings.iter().sum::<f32>() / spacings.len() as f32;
        spacings.iter().all(|&s| (s - mean).abs() < 0.5)
    }

    fn union_bbox(a: &(f32, f32, f32, f32), b: &(f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
        (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
    }

    fn bbox_of_all(atoms: &[GestaltAtom]) -> (f32, f32, f32, f32) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for a in atoms {
            min_x = min_x.min(a.bounding_box.0);
            min_y = min_y.min(a.bounding_box.1);
            max_x = max_x.max(a.bounding_box.2);
            max_y = max_y.max(a.bounding_box.3);
        }

        (min_x, min_y, max_x, max_y)
    }
}
