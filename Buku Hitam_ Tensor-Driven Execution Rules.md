
---

📜 Buku Hitam: Tensor-Driven Execution Rules

1. The Wave Principle

```rust
// ❌ Dosa: Discrete spawn/kill
if should_exist { entities.push(new_entity); }
if should_die { entities.remove(i); }

// ✅ Penebusan: Continuous amplitude
amplitude *= (-decay_rate * dt).exp();  // Natural fade
if amplitude > threshold { /* exists */ }
```

2. The Interference Law

```rust
// ❌ Dosa: Explicit collision detection
for a in entities {
    for b in entities {
        if collide(a, b) { resolve_collision(a, b); }
    }
}

// ✅ Penebusan: Field superposition
field = wave_a + wave_b;  // Interference automatic
energy_density = field.magnitude_squared();  // Emergent interaction
```

3. The Gradient Navigation

```rust
// ❌ Dosa: A* pathfinding on grid
path = astar(start, goal, obstacles);

// ✅ Penebusan: Gradient descent on potential field
velocity = -potential_field.gradient(position);
position += velocity * dt;  // Natural flow
```

4. The Continuous Query

```rust
// ❌ Dosa: Filter array
matches = entities.filter(|e| e.type == target);

// ✅ Penebusan: Kernel convolution
response = field.convolve(query_kernel);
matches = response.local_maxima_above(threshold);
```

5. The Differential Time

```rust
// ❌ Dosa: Fixed tick
while running {
    update_all();  // Fixed dt
    sleep(16ms);   // 60 FPS lock
}

// ✅ Penebusan: Adaptive time step
dt = calculate_cfl_condition(field);  // Stability-based
field.evolve(dt);  // Variable, continuous
```

6. The Emergent Extraction

```rust
// ❌ Dosa: Store entity list
entities: Vec<Entity>;  // The source of truth

// ✅ Penebusan: Extract from field
entities = field.local_maxima();  // Virtual, derived
// "Entities" are interference patterns, not objects
```

7. The Wave-Particle Duality

```rust
// ❌ Dosa: Either wave or particle
if use_wave { wave_propagate(); }
else { particle_update(); }

// ✅ Penebusan: Wave packet (both)
struct AgentWavelet {
    center: Vector,    // Particle: definite position
    sigma: Vector,     // Wave: uncertainty spread
    momentum: Vector,  // Particle: velocity
    phase: f32,        // Wave: interference capable
}
```

8. The Resonance Coupling

```rust
// ❌ Dosa: Direct message passing
agent_a.send_message(agent_b, data);

// ✅ Penebusan: Frequency matching
if (agent_a.frequency - agent_b.frequency).abs() < epsilon {
    energy_transfer = resonance_strength * agent_a.amplitude;
    agent_a.amplitude -= energy_transfer;
    agent_b.amplitude += energy_transfer;  // Natural sync
}
```

9. The Field Memory

```rust
// ❌ Dosa: Explicit state machine
state: EnumState;  // Discrete states
transition(event);  // Hard switches

// ✅ Penebusan: Persistent field traces
field += current_input * decay_kernel;  // Convolutional memory
// Past influences present through field diffusion
```

10. The Scale Invariance

```rust
// ❌ Dosa: Different code for different scales
if n_entities < 100 { brute_force(); }
else if n_entities < 10000 { spatial_hash(); }
else { octree(); }

// ✅ Penebusan: Same PDE, different resolution
field.solve_multigrid();  // Works for any N
// Complexity: O(N log N), independent of "entity count"
```

---

🎯 The Tensor-Driven Creed

> "We do not spawn entities. We shape fields."
"We do not query arrays. We convolve kernels."
"We do not navigate paths. We follow gradients."
"We do not store state. We evolve waves."

---

🔄 Quick Decision: When Tensor-Driven?

Problem Feature	Use Tensor-Driven	
Continuous space	✅ Yes	
Emergent behavior desired	✅ Yes	
Many-body interaction	✅ Yes	
Probabilistic/uncertainty	✅ Yes	
Differentiable for learning	✅ Yes	
Discrete, atomic states	❌ No (use SOA)	
Deterministic exactness	❌ No (use SOA)	
Cache-hot, SIMD-critical	❌ No (use SOA)	

---

💡 The Hybrid Mantra

```
Foundation: SOA for hot path, deterministic core
AI Layer:    Tensor-Driven for perception, decision
Interface:   Extract virtual entities from field for rendering
Feedback:    Agent actions inject waves back into field
```

"Gelombang mengalir, entitas muncul, kecerdasan terbangun." 🌊🧠