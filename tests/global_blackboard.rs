use ndarray::Array1;
use rrm_rust::core::config::GLOBAL_DIMENSION;
use rrm_rust::reasoning::global_blackboard::GlobalBlackboard;

#[test]
fn test_synchronize_empty_agents() {
    let mut blackboard = GlobalBlackboard::new();
    let agents: Vec<&Array1<f32>> = vec![];

    blackboard.synchronize(&agents);

    let state = blackboard.read_collective_state();
    for i in 0..GLOBAL_DIMENSION {
        assert!(state[i].abs() < 1e-5, "Expected 0.0 but got {}", state[i]);
    }
}

#[test]
fn test_synchronize_single_agent() {
    let mut blackboard = GlobalBlackboard::new();
    let mut agent1 = Array1::zeros(GLOBAL_DIMENSION);
    // Set first two elements
    agent1[0] = 3.0;
    agent1[1] = 4.0;
    // Magnitude = sqrt(9 + 16) = 5
    // Normalized should be 3/5 = 0.6 and 4/5 = 0.8

    blackboard.synchronize(&[&agent1]);

    let state = blackboard.read_collective_state();
    assert!((state[0] - 0.6).abs() < 1e-5, "Expected 0.6 but got {}", state[0]);
    assert!((state[1] - 0.8).abs() < 1e-5, "Expected 0.8 but got {}", state[1]);

    // Check remaining elements are zero
    for i in 2..GLOBAL_DIMENSION {
        assert!(state[i].abs() < 1e-5, "Expected 0.0 but got {}", state[i]);
    }
}

#[test]
fn test_synchronize_multiple_agents_constructive() {
    let mut blackboard = GlobalBlackboard::new();

    let mut agent1 = Array1::zeros(GLOBAL_DIMENSION);
    agent1[0] = 1.0;
    agent1[1] = 1.0;

    let mut agent2 = Array1::zeros(GLOBAL_DIMENSION);
    agent2[0] = 2.0;
    agent2[1] = 2.0;

    // Sum is [3.0, 3.0, 0, ...]
    // Magnitude = sqrt(9 + 9) = sqrt(18) = 3 * sqrt(2) = 4.24264
    // Normalized is [3 / sqrt(18), 3 / sqrt(18), ...]
    // 3 / sqrt(18) = 1 / sqrt(2) = 0.707106

    blackboard.synchronize(&[&agent1, &agent2]);

    let state = blackboard.read_collective_state();
    let expected = std::f32::consts::FRAC_1_SQRT_2; // 1 / sqrt(2)

    assert!((state[0] - expected).abs() < 1e-5, "Expected {} but got {}", expected, state[0]);
    assert!((state[1] - expected).abs() < 1e-5, "Expected {} but got {}", expected, state[1]);

    for i in 2..GLOBAL_DIMENSION {
        assert!(state[i].abs() < 1e-5, "Expected 0.0 but got {}", state[i]);
    }
}

#[test]
fn test_synchronize_multiple_agents_destructive() {
    let mut blackboard = GlobalBlackboard::new();

    let mut agent1 = Array1::zeros(GLOBAL_DIMENSION);
    agent1[0] = 5.0;
    agent1[1] = -2.0;

    let mut agent2 = Array1::zeros(GLOBAL_DIMENSION);
    agent2[0] = -5.0;
    agent2[1] = 2.0;

    // Sum is [0.0, 0.0, 0, ...]
    // Destructive interference perfectly cancels out
    blackboard.synchronize(&[&agent1, &agent2]);

    let state = blackboard.read_collective_state();

    for i in 0..GLOBAL_DIMENSION {
        assert!(state[i].abs() < 1e-5, "Expected 0.0 but got {}", state[i]);
    }
}
