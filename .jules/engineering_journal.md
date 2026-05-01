# Shared Engineering Journal (Nexus)

This journal serves as a collaborative knowledge base between the architectural mind (Carbo ⬡) and the performance optimizer (Bolt ⚡).

## Section 1: Performance & Memory Learnings (Bolt ⚡)

## 2026-04-22 - [⚡ Bolt] - Removing Arc<Vec<T>> overhead in EntityManifold
**Learning:** `Arc::make_mut` inside a hot inner loop (like MCTS simulation) causes severe heap thrashing and deep copies when the strong count > 1. This defeats the purpose of Copy-on-Write for small arrays and destroys L1 cache locality, drastically slowing down simulations.
**Action:** Changed `EntityManifold` internal arrays to use plain `Vec<T>`. Relied on top-level contiguous `m.clone()` for states which is easily optimized by `memcpy` and eliminates locking and branching overhead for thousands of internal tensor mutations.

## Section 2: Architectural Decisions (Carbo ⬡)

## Section 3: Future Ideas (Carbo ⬡)

## 2026-05-01 - [⬡ Carbo] - Secured AsyncSoulLog File Handling
**Context:** The `AsyncSoulLog` background thread was using `.expect()` when opening the log file, which could cause the entire process to panic or leave the logging thread in a broken state if I/O issues occurred.
**Decision:** Replaced `.expect()` with a graceful `match` block. If file opening fails, the error is reported to `stderr` and the thread exits cleanly. Also added error checking for log directory creation.
**Consequences:** Increased system resilience against I/O failures. Since logging happens in a dedicated thread, this fix preserves the zero-blocking performance characteristics of the main loop while ensuring safety.

## 2026-04-29 - [⬡ Carbo] - Hardened JSON Parsing in src/main.rs
**Context:** Unsafe `unwrap()` and `expect()` calls in JSON parsing logic were causing potential panics (DoS vulnerability) when processing malformed ARC-AGI task data.
**Decision:** Refactored the task processing loop and `parse_grid` closure to use safe Rust patterns (`match`, `Option`, `let-else`).
**Consequences:** Improved robustness against invalid input data. Minimal performance impact as these changes are outside the primary hot loops.

## 2026-04-30 - [⚡ Bolt] - Eliminating Static Array Clones in Crop Logic
**Learning:** Cloning `ndarray::Array1` (even if static) in a loop involving `fractional_bind_2d` adds unnecessary allocation overhead and reference counting increments/decrements. Since these seeds are `&'static Array1<f32>`, they can be passed by reference.
**Action:** Removed `.clone()` from `x_axis_seed()` and `y_axis_seed()` calls in `src/reasoning/multiverse_sandbox.rs`.
**Measured Improvement:** While absolute timings in micro-benchmarks showed noise (±2ms), the elimination of 4 global array clones per crop operation reduces heap pressure and ensures Zero-Cost Abstraction principles are maintained.
