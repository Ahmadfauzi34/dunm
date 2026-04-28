use rrm_rust::core::entity_manifold::EntityManifold;
use rrm_rust::quantum_topology::QuantumCellComplex;
use std::time::Instant;

#[test]
fn bench_from_manifold() {
    let n = 500;
    let mut manifold = EntityManifold::new();
    manifold.active_count = n;
    manifold.global_width = 100.0;
    manifold.global_height = 100.0;

    let cx = (0..n).map(|i| (i as f32 * 1.1) % 100.0).collect::<Vec<_>>();
    let cy = (0..n).map(|i| (i as f32 * 1.7) % 100.0).collect::<Vec<_>>();
    let masses = vec![1.0; n];
    let tokens = (0..n).map(|i| (i % 10) as i32).collect::<Vec<_>>();

    manifold.centers_x = cx;
    manifold.centers_y = cy;
    manifold.masses = masses;
    manifold.tokens = tokens;
    manifold.ensure_scalar_capacity(n);

    let epsilon = 5.0;

    // Warmup
    for _ in 0..5 {
        let _ = QuantumCellComplex::from_manifold(&manifold, epsilon);
    }

    let start = Instant::now();
    let iterations = 10;
    for _ in 0..iterations {
        let _ = QuantumCellComplex::from_manifold(&manifold, epsilon);
    }
    let duration = start.elapsed() / iterations as u32;

    println!("Average time for n={}: {:?}", n, duration);
}
