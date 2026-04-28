// ============================================================================
// QUANTUM TOPOLOGY MODULE
// ============================================================================

use crate::core::config::GLOBAL_DIMENSION;
use crate::core::entity_manifold::EntityManifold;
use ndarray::{Array1, Array2};

/// Quantum Cell Complex: Simplicial complex dengan amplitude kuantum
#[derive(Clone, Debug)]
pub struct QuantumCellComplex {
    pub max_dimension: usize,
    pub boundary_operators: Vec<Array2<f32>>,
    pub laplacians: Vec<Array2<f32>>,
    pub betti_numbers: Vec<usize>,
    pub amplitudes: Vec<Array1<f32>>,
    pub persistence_barcode: Vec<(f32, f32)>,
}

impl QuantumCellComplex {
    pub fn from_manifold(manifold: &EntityManifold, epsilon: f32) -> Self {
        let n = manifold.active_count;
        let mut complex = QuantumCellComplex {
            max_dimension: 2,
            boundary_operators: Vec::new(),
            laplacians: Vec::new(),
            betti_numbers: Vec::new(),
            amplitudes: Vec::new(),
            persistence_barcode: Vec::new(),
        };

        if n == 0 {
            return complex;
        }

        let mut dist_matrix = Array2::<f32>::zeros((n, n));
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = manifold.centers_x[i] - manifold.centers_x[j];
                let dy = manifold.centers_y[i] - manifold.centers_y[j];
                let dist_sq = dx * dx + dy * dy;
                dist_matrix[[i, j]] = dist_sq;
                dist_matrix[[j, i]] = dist_sq;
            }
        }

        let epsilon_sq = epsilon * epsilon;
        let mut edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                if dist_matrix[[i, j]] <= epsilon_sq {
                    edges.push((i, j));
                }
            }
        }

        let mut triangles = Vec::new();
        for &(i, j) in &edges {
            for k in (j + 1)..n {
                if dist_matrix[[i, k]] <= epsilon_sq && dist_matrix[[j, k]] <= epsilon_sq {
                    triangles.push((i, j, k));
                }
            }
        }

        let mut d1 = Array2::<f32>::zeros((n, edges.len().max(1)));
        if !edges.is_empty() {
            for (idx, &(i, j)) in edges.iter().enumerate() {
                d1[[i, idx]] = 1.0;
                d1[[j, idx]] = -1.0;
            }
            complex.boundary_operators.push(d1);
        }

        if !triangles.is_empty() && !edges.is_empty() {
            let mut d2 = Array2::<f32>::zeros((edges.len(), triangles.len()));
            for (t_idx, &(i, j, k)) in triangles.iter().enumerate() {
                for (e_idx, &(a, b)) in edges.iter().enumerate() {
                    if (a == i && b == j) || (a == j && b == i) {
                        d2[[e_idx, t_idx]] = if a == i { 1.0 } else { -1.0 };
                    }
                    if (a == i && b == k) || (a == k && b == i) {
                        d2[[e_idx, t_idx]] = if a == k { 1.0 } else { -1.0 };
                    }
                    if (a == j && b == k) || (a == k && b == j) {
                        d2[[e_idx, t_idx]] = if a == j { 1.0 } else { -1.0 };
                    }
                }
            }
            complex.boundary_operators.push(d2);
        }

        complex.compute_laplacians_and_betti();
        complex.compute_persistence(&dist_matrix, &edges, &triangles);
        complex
    }

    fn compute_laplacians_and_betti(&mut self) {
        if self.boundary_operators.is_empty() {
            self.betti_numbers.push(0);
            return;
        }

        let d1 = &self.boundary_operators[0];
        let l0 = d1.dot(&d1.t());
        self.laplacians.push(l0);

        if self.boundary_operators.len() >= 2 {
            let d2 = &self.boundary_operators[1];
            let l1 = d1.t().dot(d1) + d2.dot(&d2.t());
            self.laplacians.push(l1);
        }

        for laplacian in &self.laplacians {
            let eigenvalues = self.estimate_eigenvalues(laplacian);
            let zero_count = eigenvalues.iter().filter(|&&x| x.abs() < 1e-4).count();
            self.betti_numbers.push(zero_count);
        }
    }

    fn estimate_eigenvalues(&self, matrix: &Array2<f32>) -> Vec<f32> {
        if matrix.is_empty() {
            return vec![0.0];
        }
        let n = matrix.shape()[0];
        let mut v = Array1::<f32>::ones(n);
        let norm = v.dot(&v).sqrt();
        if norm > 0.0 {
            v /= norm;
        } else {
            return vec![0.0];
        }

        for _ in 0..20 {
            let v_new = matrix.dot(&v);
            let norm = v_new.dot(&v_new).sqrt();
            if norm > 1e-6 {
                v = v_new / norm;
            }
        }
        let lambda = v.dot(&(matrix.dot(&v)));
        vec![lambda]
    }

    fn compute_persistence(
        &mut self,
        dist_matrix: &Array2<f32>,
        edges: &[(usize, usize)],
        _triangles: &[(usize, usize, usize)],
    ) {
        // Here dist_matrix holds squared distances, we take the sqrt when pushing to barcode
        let mut edge_filtration: Vec<(f32, usize)> = edges
            .iter()
            .enumerate()
            .map(|(idx, &(i, j))| (dist_matrix[[i, j]].sqrt(), idx))
            .collect();
        edge_filtration.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut uf = UnionFind::new(dist_matrix.shape()[0]);

        for (dist, e_idx) in edge_filtration {
            let (i, j) = edges[e_idx];
            let root_i = uf.find(i);
            let root_j = uf.find(j);

            if root_i != root_j {
                uf.union(root_i, root_j);
            } else {
                self.persistence_barcode.push((0.0, dist));
            }
        }
    }
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }
    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x != root_y {
            match self.rank[root_x].cmp(&self.rank[root_y]) {
                std::cmp::Ordering::Less => self.parent[root_x] = root_y,
                std::cmp::Ordering::Greater => self.parent[root_y] = root_x,
                std::cmp::Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.rank[root_x] += 1;
                }
            }
        }
    }
}

/// Skill Fiber Bundle: Representasi skill sebagai section dari bundle
/// Base = Task manifold, Fiber = FHRR tensor space
#[derive(Clone)]
pub struct SkillFiberBundle {
    pub base: EntityManifold,
    pub fibers: Vec<Array1<f32>>, // Section values
    pub connection: Array2<f32>,  // Parallel transport coefficients
    pub curvature: Array2<f32>,   // [∇_i, ∇_j] - ∇_[i,j]
}

impl SkillFiberBundle {
    pub fn from_manifold(manifold: &EntityManifold) -> Self {
        let n = manifold.active_count;
        let mut fibers = Vec::with_capacity(n);

        // Inisialisasi fiber: FHRR encoding dari local features
        for i in 0..n {
            let mut fiber = Array1::<f32>::zeros(GLOBAL_DIMENSION);
            let px = manifold.centers_x[i] / manifold.global_width.max(1.0);
            let py = manifold.centers_y[i] / manifold.global_height.max(1.0);
            let token_val = manifold.tokens[i] as f32 / 10.0;

            // Positional encoding via sinusoidal interference (wave field)
            for d in 0..GLOBAL_DIMENSION {
                let phase = (d as f32) * 0.017_453_3; // ~1 degree in radians
                fiber[d] = (px * (d as f32)).sin() * 0.3
                    + (py * (d as f32)).cos() * 0.3
                    + (token_val * phase).sin() * 0.4;
            }

            // L2 Normalisasi (stabil, anti phase explosion)
            let norm = fiber.dot(&fiber).sqrt();
            if norm > 1e-6 {
                fiber /= norm;
            }
            fibers.push(fiber);
        }

        // Connection: local trivialization dari adjacency
        let mut connection = Array2::<f32>::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    let dx = manifold.centers_x[i] - manifold.centers_x[j];
                    let dy = manifold.centers_y[i] - manifold.centers_y[j];
                    let dist_sq = dx * dx + dy * dy;
                    // Gaussian kernel: local connection
                    let weight = (-dist_sq / 2.0).exp();
                    connection[[i, j]] = weight * (weight > 0.01) as i32 as f32;
                }
            }
        }

        // Curvature: obstruction to flatness
        let mut curvature = Array2::<f32>::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += connection[[i, k]] * connection[[k, j]]
                        - connection[[j, k]] * connection[[k, i]];
                }
                curvature[[i, j]] = sum;
            }
        }

        SkillFiberBundle {
            base: manifold.clone(),
            fibers,
            connection,
            curvature,
        }
    }

    /// Parallel transport sepanjang geodesic di base manifold
    pub fn parallel_transport(
        &self,
        start: usize,
        end: usize,
        section: &Array1<f32>,
    ) -> Array1<f32> {
        let mut result = section.clone();
        let mut current = start;

        // Greedy geodesic following (gradient navigation)
        while current != end {
            let mut best_next = current;
            let mut best_dist_sq = f32::INFINITY;

            for next in 0..self.base.active_count {
                if next != current && self.connection[[current, next]] > 0.0 {
                    let dx = self.base.centers_x[end] - self.base.centers_x[next];
                    let dy = self.base.centers_y[end] - self.base.centers_y[next];
                    let dist_sq = dx * dx + dy * dy;
                    // Branchless min selection
                    let is_better = (dist_sq < best_dist_sq) as i32 as f32;
                    best_dist_sq = best_dist_sq * (1.0 - is_better) + dist_sq * is_better;
                    best_next = ((best_next as f32) * (1.0 - is_better) + (next as f32) * is_better)
                        as usize;
                }
            }

            if best_next == current {
                break;
            }

            // Apply connection: rotate fiber (fractional binding style)
            let transport_weight = self.connection[[current, best_next]];
            result = &result * transport_weight;
            current = best_next;
        }

        result
    }

    /// Holonomy: transport around closed loop (topological invariant)
    pub fn compute_holonomy(&self, loop_indices: &[usize]) -> f32 {
        if loop_indices.len() < 2 {
            return 0.0;
        }

        let mut section = self.fibers[loop_indices[0]].clone();
        let original = section.clone();

        for i in 0..loop_indices.len() - 1 {
            section = self.parallel_transport(loop_indices[i], loop_indices[i + 1], &section);
        }
        section = self.parallel_transport(
            loop_indices[loop_indices.len() - 1],
            loop_indices[0],
            &section,
        );

        // Cosine similarity: measure of holonomy
        let dot = original.dot(&section);
        let norm_orig = original.dot(&original).sqrt();
        let norm_final = section.dot(&section).sqrt();
        if norm_orig * norm_final > 1e-10 {
            dot / (norm_orig * norm_final)
        } else {
            0.0
        }
    }
}

/// Reasoning Sheaf: Local-to-global consistency untuk reasoning
#[derive(Clone, Debug)]
pub struct ReasoningSheaf {
    pub cover: Vec<Vec<usize>>,                // Open cover (Voronoi cells)
    pub restrictions: Vec<Array2<f32>>,        // Restriction maps F(U)→F(V)
    pub local_sections: Vec<Array1<f32>>,      // Local solutions
    pub gluing_data: Vec<(usize, usize, f32)>, // (i, j, compatibility)
}

impl ReasoningSheaf {
    /// Konstruksi sheaf dari manifold dengan k-means++ style cover
    pub fn from_manifold(manifold: &EntityManifold, num_centers: usize) -> Self {
        let n = manifold.active_count;
        if n == 0 {
            return ReasoningSheaf {
                cover: vec![],
                restrictions: vec![],
                local_sections: vec![],
                gluing_data: vec![],
            };
        }

        // Farthest-first traversal untuk cover centers
        let mut centers: Vec<usize> = Vec::with_capacity(num_centers);
        centers.push(0); // Start dari index 0

        for _ in 1..num_centers {
            let mut max_dist_sq = -1.0f32;
            let mut max_idx = 0;
            for i in 0..n {
                let mut min_dist_sq = f32::INFINITY;
                for &c in &centers {
                    let dx = manifold.centers_x[i] - manifold.centers_x[c];
                    let dy = manifold.centers_y[i] - manifold.centers_y[c];
                    let dist_sq = dx * dx + dy * dy;
                    min_dist_sq = min_dist_sq.min(dist_sq);
                }
                if min_dist_sq > max_dist_sq {
                    max_dist_sq = min_dist_sq;
                    max_idx = i;
                }
            }
            centers.push(max_idx);
        }

        // Voronoi decomposition sebagai open cover
        let mut cover: Vec<Vec<usize>> = vec![Vec::new(); num_centers];
        for i in 0..n {
            let mut min_dist_sq = f32::INFINITY;
            let mut nearest = 0;
            for (c_idx, &c) in centers.iter().enumerate() {
                let dx = manifold.centers_x[i] - manifold.centers_x[c];
                let dy = manifold.centers_y[i] - manifold.centers_y[c];
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    nearest = c_idx;
                }
            }
            cover[nearest].push(i);
        }

        let mut restrictions = Vec::new();
        let mut local_sections = Vec::new();
        let mut gluing_data = Vec::new();

        // Build local sections dan restriction maps
        for (i, set_i) in cover.iter().enumerate() {
            let mut section = Array1::<f32>::zeros(GLOBAL_DIMENSION);
            for &idx in set_i {
                let mut fiber = Array1::<f32>::zeros(GLOBAL_DIMENSION);
                let px = manifold.centers_x[idx] / manifold.global_width.max(1.0);
                let py = manifold.centers_y[idx] / manifold.global_height.max(1.0);
                for d in 0..GLOBAL_DIMENSION {
                    fiber[d] = (px * (d as f32)).sin() + (py * (d as f32)).cos();
                }
                section += &fiber;
            }
            if !set_i.is_empty() {
                section /= set_i.len() as f32;
            }
            local_sections.push(section);

            // Overlap dengan set lain
            for (j, set_j) in cover.iter().enumerate().skip(i + 1) {
                let overlap: Vec<usize> = set_i
                    .iter()
                    .filter(|x| set_j.contains(x))
                    .cloned()
                    .collect();

                if !overlap.is_empty() {
                    let compat = local_sections[i].dot(&local_sections[j]);
                    gluing_data.push((i, j, compat));

                    // Restriction: projection ke overlap
                    let mut restriction = Array2::<f32>::zeros((overlap.len(), set_i.len()));
                    for (o_idx, &orig_idx) in overlap.iter().enumerate() {
                        if let Some(pos) = set_i.iter().position(|&x| x == orig_idx) {
                            restriction[[o_idx, pos]] = 1.0;
                        }
                    }
                    restrictions.push(restriction);
                }
            }
        }

        ReasoningSheaf {
            cover,
            restrictions,
            local_sections,
            gluing_data,
        }
    }

    /// Sheaf condition: compatibility pada overlap
    pub fn check_sheaf_condition(&self) -> bool {
        // Cek gluing condition: restrictions cocok pada overlap
        for &(_i, _j, compat) in &self.gluing_data {
            if compat < 0.7 {
                // Threshold compatibility
                return false;
            }
        }
        true
    }

    /// Global section via partition of unity
    pub fn compute_global_section(&self) -> Option<Array1<f32>> {
        if !self.check_sheaf_condition() {
            return None; // Tidak bisa diglue
        }

        let mut global = Array1::<f32>::zeros(GLOBAL_DIMENSION);
        let mut total_weight = 0.0;

        for (i, section) in self.local_sections.iter().enumerate() {
            let weight = self.cover[i].len() as f32;
            global += &(section * weight);
            total_weight += weight;
        }

        if total_weight > 0.0 {
            global /= total_weight;
            Some(global)
        } else {
            None
        }
    }
}

/// Spectral Embedding: Laplacian Eigenmaps untuk manifold learning
#[derive(Clone, Debug)]
pub struct SpectralEmbedding {
    pub embedding: Array2<f32>, // N x k
    pub eigenvalues: Vec<f32>,
    pub eigenvectors: Array2<f32>,
}

impl SpectralEmbedding {
    pub fn from_manifold(manifold: &EntityManifold, k: usize) -> Self {
        let n = manifold.active_count;
        if n == 0 {
            return SpectralEmbedding {
                embedding: Array2::<f32>::zeros((0, 0)),
                eigenvalues: vec![],
                eigenvectors: Array2::<f32>::zeros((0, 0)),
            };
        }

        // Similarity matrix: heat kernel (continuous query)
        let mut w = Array2::<f32>::zeros((n, n));
        let mut d = Array1::<f32>::zeros(n);

        for i in 0..n {
            for j in (i + 1)..n {
                let dx = manifold.centers_x[i] - manifold.centers_x[j];
                let dy = manifold.centers_y[i] - manifold.centers_y[j];
                let dist_sq = dx * dx + dy * dy;
                let sim = (-dist_sq / 2.0).exp();
                w[[i, j]] = sim;
                w[[j, i]] = sim;
                d[i] += sim;
                d[j] += sim;
            }
        }

        // Normalized Laplacian: L = I - D^{-1/2} W D^{-1/2}
        let mut d_inv_sqrt = vec![0.0; n];
        for i in 0..n {
            d_inv_sqrt[i] = 1.0 / d[i].sqrt().max(1e-10);
        }

        let mut laplacian = Array2::<f32>::zeros((n, n));
        for i in 0..n {
            let di = d_inv_sqrt[i];
            for j in 0..n {
                let dj = d_inv_sqrt[j];
                if i == j {
                    laplacian[[i, j]] = 1.0 - w[[i, j]] * di * dj;
                } else {
                    laplacian[[i, j]] = -w[[i, j]] * di * dj;
                }
            }
        }

        // Compute first k eigenvectors (power iteration + Gram-Schmidt)
        let mut embedding = Array2::<f32>::zeros((n, k));
        let mut eigenvalues = vec![0.0; k];

        for dim in 0..k {
            let mut v = Array1::<f32>::from_vec(
                (0..n).map(|i| ((i * 7919) as f32).sin()).collect(), // Deterministic init
            );
            let v_norm = v.dot(&v).sqrt();
            if v_norm > 0.0 {
                v /= v_norm;
            }

            // Gram-Schmidt orthogonalization
            for prev in 0..dim {
                let u = embedding.column(prev).to_owned();
                let proj = v.dot(&u);
                v -= &(u * proj);
            }

            // Power iteration
            for _ in 0..20 {
                let v_new = laplacian.dot(&v);
                let norm = v_new.dot(&v_new).sqrt();
                if norm > 1e-6 {
                    v = v_new / norm;
                }
            }

            let lambda = v.dot(&(laplacian.dot(&v)));
            eigenvalues[dim] = lambda;
            for i in 0..n {
                embedding[[i, dim]] = v[i];
            }
        }

        SpectralEmbedding {
            embedding: embedding.clone(),
            eigenvalues,
            eigenvectors: embedding,
        }
    }
}

/// Quantum Tensor Network: MERA untuk hierarchical representation
#[derive(Clone, Debug)]
pub struct QuantumTensorNetwork {
    pub layers: Vec<TensorLayer>,
    pub top_tensor: Array1<f32>,
}

#[derive(Clone, Debug)]
pub struct TensorLayer {
    pub isometries: Vec<Array2<f32>>,    // Coarse-graining unitaries
    pub disentanglers: Vec<Array2<f32>>, // Remove local entanglement
    pub scale: usize,
}

impl QuantumTensorNetwork {
    pub fn from_manifold(manifold: &EntityManifold) -> Self {
        let mut layers = Vec::new();
        let mut current_n = manifold.active_count;

        if current_n == 0 {
            return QuantumTensorNetwork {
                layers: vec![],
                top_tensor: Array1::<f32>::zeros(GLOBAL_DIMENSION),
            };
        }

        // Initialize features dari SOA
        let mut current_features: Vec<Array1<f32>> = (0..current_n)
            .map(|i| {
                let mut f = Array1::<f32>::zeros(GLOBAL_DIMENSION);
                f[0] = manifold.centers_x[i] / manifold.global_width.max(1.0);
                f[1] = manifold.centers_y[i] / manifold.global_height.max(1.0);
                f[2] = manifold.tokens[i] as f32 / 10.0;
                f
            })
            .collect();

        let mut scale = 1usize;
        while current_n > 1 {
            let mut isometries = Vec::new();
            let mut disentanglers = Vec::new();
            let mut next_features = Vec::new();

            // Block renormalization: group by 2
            for i in (0..current_n).step_by(2) {
                if i + 1 < current_n {
                    // Disentangler: identity (simplified)
                    let mut dis = Array2::<f32>::zeros((GLOBAL_DIMENSION, GLOBAL_DIMENSION));
                    for d in 0..GLOBAL_DIMENSION {
                        dis[[d, d]] = 1.0;
                    }
                    disentanglers.push(dis);

                    // Isometry: merge 2 sites -> 1 site (Haar wavelet style)
                    let mut iso = Array2::<f32>::zeros((GLOBAL_DIMENSION, GLOBAL_DIMENSION));
                    for d in 0..GLOBAL_DIMENSION {
                        iso[[d, d]] = 0.5_f32.sqrt();
                    }
                    isometries.push(iso);

                    // Coarse-grain: average dengan phase alignment
                    let merged = &current_features[i] * 0.5_f32.sqrt()
                        + &current_features[i + 1] * 0.5_f32.sqrt();
                    next_features.push(merged);
                } else {
                    next_features.push(current_features[i].clone());
                }
            }

            layers.push(TensorLayer {
                isometries,
                disentanglers,
                scale,
            });

            current_n = next_features.len();
            current_features = next_features;
            scale *= 2;
        }

        let top_tensor = current_features
            .first()
            .cloned()
            .unwrap_or_else(|| Array1::<f32>::zeros(GLOBAL_DIMENSION));

        QuantumTensorNetwork { layers, top_tensor }
    }

    /// Decode: expand top tensor ke spatial resolution
    pub fn decode(&self) -> Vec<Array1<f32>> {
        let mut current = vec![self.top_tensor.clone()];

        for layer in self.layers.iter().rev() {
            let mut next = Vec::with_capacity(current.len() * 2);

            for (i, iso) in layer.isometries.iter().enumerate() {
                if i < current.len() {
                    // Apply isometry transpose (upsampling)
                    let up = iso.t().dot(&current[i]);

                    // Split ke 2 features (simplified)
                    let half = GLOBAL_DIMENSION / 2;
                    let mut left = Array1::<f32>::zeros(GLOBAL_DIMENSION);
                    let mut right = Array1::<f32>::zeros(GLOBAL_DIMENSION);

                    for d in 0..half.min(GLOBAL_DIMENSION) {
                        left[d] = up[d];
                        if d + half < GLOBAL_DIMENSION {
                            right[d] = up[d + half];
                        }
                    }
                    next.push(left);
                    next.push(right);
                }
            }
            current = next;
        }

        current
    }
}

// ============================================================================
// PHASE 1: HARMONIC ANALYSIS (FOURIER NEURAL OPERATOR)
// ============================================================================

use rustfft::{num_complex::Complex, FftPlanner};

pub struct FourierSkillOperator {
    pub modes: usize,
}

impl FourierSkillOperator {
    pub fn new(modes: usize) -> Self {
        Self { modes }
    }

    /// Transformasi Grid Piksel ke Domain Frekuensi (2D FFT)
    pub fn transform(&self, grid: &Vec<Vec<i32>>) -> Vec<Vec<Complex<f32>>> {
        let height = grid.len();
        if height == 0 {
            return vec![];
        }
        let width = grid[0].len();
        if width == 0 {
            return vec![];
        }

        let mut planner = FftPlanner::<f32>::new();
        let fft_row = planner.plan_fft_forward(width);
        let fft_col = planner.plan_fft_forward(height);

        // Konversi ke format flat complex row-major
        let mut buffer: Vec<Complex<f32>> = Vec::with_capacity(width * height);
        for row in grid {
            for &val in row {
                buffer.push(Complex {
                    re: val as f32,
                    im: 0.0,
                });
            }
        }

        // 1. FFT per Baris (Rows)
        for y in 0..height {
            let row_start = y * width;
            let row_slice = &mut buffer[row_start..row_start + width];
            fft_row.process(row_slice);
        }

        // 2. Transpose, lalu FFT per Kolom (Cols)
        let mut transposed_buffer: Vec<Complex<f32>> =
            vec![Complex { re: 0.0, im: 0.0 }; width * height];
        for y in 0..height {
            for x in 0..width {
                transposed_buffer[x * height + y] = buffer[y * width + x];
            }
        }

        for x in 0..width {
            let col_start = x * height;
            let col_slice = &mut transposed_buffer[col_start..col_start + height];
            fft_col.process(col_slice);
        }

        // 3. Transpose kembali ke urutan semula
        let mut final_spectral = vec![vec![Complex { re: 0.0, im: 0.0 }; width]; height];
        for x in 0..width {
            for y in 0..height {
                final_spectral[y][x] = transposed_buffer[x * height + y];
            }
        }

        // 4. Pruning frekuensi tinggi (Low-Pass Filter) -> simpan hanya 'modes' terendah
        let mode_limit = self.modes;
        for y in 0..height {
            for x in 0..width {
                if x >= mode_limit && y >= mode_limit {
                    final_spectral[y][x] = Complex { re: 0.0, im: 0.0 };
                }
            }
        }

        final_spectral
    }

    /// Transformasi Domain Frekuensi kembali ke Grid Spasial (2D IFFT)
    pub fn inverse_transform(&self, spectral: &Vec<Vec<Complex<f32>>>) -> Vec<Vec<i32>> {
        let height = spectral.len();
        if height == 0 {
            return vec![];
        }
        let width = spectral[0].len();
        if width == 0 {
            return vec![];
        }

        let mut planner = FftPlanner::<f32>::new();
        let ifft_row = planner.plan_fft_inverse(width);
        let ifft_col = planner.plan_fft_inverse(height);

        let mut buffer: Vec<Complex<f32>> = Vec::with_capacity(width * height);
        for row in spectral {
            for &val in row {
                buffer.push(val);
            }
        }

        // 1. IFFT per Baris
        for y in 0..height {
            let row_start = y * width;
            let row_slice = &mut buffer[row_start..row_start + width];
            ifft_row.process(row_slice);
        }

        // 2. Transpose, lalu IFFT per Kolom
        let mut transposed_buffer: Vec<Complex<f32>> =
            vec![Complex { re: 0.0, im: 0.0 }; width * height];
        for y in 0..height {
            for x in 0..width {
                transposed_buffer[x * height + y] = buffer[y * width + x];
            }
        }

        for x in 0..width {
            let col_start = x * height;
            let col_slice = &mut transposed_buffer[col_start..col_start + height];
            ifft_col.process(col_slice);
        }

        // 3. Normalisasi hasil IFFT & Transpose kembali
        let norm_factor = 1.0 / ((width * height) as f32);
        let mut final_grid = vec![vec![0; width]; height];
        for x in 0..width {
            for y in 0..height {
                let val = transposed_buffer[x * height + y].re * norm_factor;
                // Pembulatan ke token terdekat (ARC range 0-9)
                let mut token = val.round() as i32;
                if token < 0 {
                    token = 0;
                }
                if token > 9 {
                    token = 9;
                }
                final_grid[y][x] = token;
            }
        }

        final_grid
    }
}
