#[global_allocator]
static ALLOCATOR: rrm_rust::memory::allocator::TrackingAllocator = rrm_rust::memory::allocator::TrackingAllocator::new();

use rrm_rust::core::entity_manifold::EntityManifold;
use rrm_rust::quantum_topology::QuantumCellComplex;
use std::time::Instant;

fn main() {
    let n = 500;
    let mut manifold = EntityManifold::new();
    manifold.active_count = n;
    manifold.global_width = 100.0;
    manifold.global_height = 100.0;

    let mut cx = vec![0.0; n];
    let mut cy = vec![0.0; n];
    let masses = vec![1.0; n];
    let tokens = vec![1; n];

    for i in 0..n {
        cx[i] = (i as f32 * 1.1) % 100.0;
        cy[i] = (i as f32 * 1.7) % 100.0;
    }

    manifold.centers_x = cx;
    manifold.centers_y = cy;
    manifold.masses = masses;
    manifold.tokens = tokens;
    manifold.ensure_scalar_capacity(n);

    let epsilon = 5.0;

    for _ in 0..5 {
        let _ = QuantumCellComplex::from_manifold(&manifold, epsilon);
    }

    let start = Instant::now();
    let iterations = 20;
    for _ in 0..iterations {
        let _ = QuantumCellComplex::from_manifold(&manifold, epsilon);
    }
    let duration = start.elapsed() / iterations as u32;

    println!("Average time for n={}: {:?}", n, duration);
}
