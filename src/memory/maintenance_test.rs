#[cfg(test)]
mod tests {
    use crate::core::config::GLOBAL_DIMENSION;
    use crate::memory::maintenance_engine::MaintenanceEngine;
    use ndarray::Array1;

    #[test]
    fn test_maintenance_engine_anneal() {
        let engine = MaintenanceEngine::new();

        let mut tensors = Vec::new();
        let mut t1 = Array1::<f32>::zeros(GLOBAL_DIMENSION);
        let mut t2 = Array1::<f32>::zeros(GLOBAL_DIMENSION);

        // T1 and T2 are not completely orthogonal
        for i in 0..100 {
            t1[i] = 1.0 / (100.0f32).sqrt();
            t2[i] = 0.5 / (100.0f32).sqrt();
        }
        for i in 100..200 {
            t2[i] = 0.866 / (100.0f32).sqrt();
        }

        tensors.push(t1);
        tensors.push(t2);

        let (noise_before, noise_after) = engine.anneal_memory(&mut tensors, 0.5, 30);

        assert!(noise_after < noise_before);
    }
}
