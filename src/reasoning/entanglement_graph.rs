pub struct EntanglementGraph {
    pub values: Vec<f32>,
    pub col_indices: Vec<usize>,
    pub row_ptr: Vec<usize>,
}

impl Default for EntanglementGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl EntanglementGraph {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            col_indices: Vec::new(),
            row_ptr: vec![0], // Minimal state
        }
    }

    pub fn reset_active(&mut self, active_count: usize) {
        self.values.clear();
        self.col_indices.clear();
        self.row_ptr.clear();
        self.row_ptr.resize(active_count + 1, 0);

        for i in 0..active_count {
            self.row_ptr[i] = i;
            self.values.push(1.0); // Self-loop
            self.col_indices.push(i);
        }

        self.row_ptr[active_count] = active_count;
    }

    #[inline(always)]
    pub fn iter_row(&self, row: usize) -> impl Iterator<Item = (usize, f32)> + '_ {
        let start = self.row_ptr[row];
        let end = self.row_ptr[row + 1];

        self.col_indices[start..end]
            .iter()
            .copied()
            .zip(self.values[start..end].iter().copied())
    }

    pub fn get_weight_csr(&self, row: usize, col: usize) -> f32 {
        let start = self.row_ptr[row];
        let end = self.row_ptr[row + 1];

        for i in start..end {
            if self.col_indices[i] == col {
                return self.values[i];
            } else if self.col_indices[i] > col {
                break;
            }
        }
        0.0
    }
}
