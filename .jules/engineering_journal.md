# Shared Engineering Journal (Nexus)

This journal serves as a collaborative knowledge base between the architectural mind (Carbo ⬡) and the performance optimizer (Bolt ⚡).

## Section 1: Performance & Memory Learnings (Bolt ⚡)

## 2026-04-22 - [⚡ Bolt] - Removing Arc<Vec<T>> overhead in EntityManifold
**Learning:** `Arc::make_mut` inside a hot inner loop (like MCTS simulation) causes severe heap thrashing and deep copies when the strong count > 1. This defeats the purpose of Copy-on-Write for small arrays and destroys L1 cache locality, drastically slowing down simulations.
**Action:** Changed `EntityManifold` internal arrays to use plain `Vec<T>`. Relied on top-level contiguous `m.clone()` for states which is easily optimized by `memcpy` and eliminates locking and branching overhead for thousands of internal tensor mutations.

## 2024-05-15 - [⚡ Bolt] - Offloading Synchronous I/O in Cognitive Loops
**Learning:** Performing synchronous disk I/O (log file writing) deep within the agent's MCTS and reasoning loops introduces significant latency spikes, especially when the system is under pressure (e.g., during memory bloat). This blocks the reasoning iterations and destroys the efficiency of the cognitive state machine.
**Action:** Implemented `AsyncArchLogger` using a dedicated background thread and `mpsc` channel. Refactored `RrmAgent` to use this non-blocking logger for all architectural lints and execution logs.
**Consequences:** Reasoning iterations are no longer gated by disk I/O performance. The system maintains consistent throughput even when generating extensive diagnostic logs.

## Section 2: Architectural Decisions (Carbo ⬡)

## Section 3: Future Ideas (Carbo ⬡)


## 2026-04-29 - [⬡ Carbo] - Hardened JSON Parsing in src/main.rs
**Context:** Unsafe `unwrap()` and `expect()` calls in JSON parsing logic were causing potential panics (DoS vulnerability) when processing malformed ARC-AGI task data.
**Decision:** Refactored the task processing loop and `parse_grid` closure to use safe Rust patterns (`match`, `Option`, `let-else`).
**Consequences:** Improved robustness against invalid input data. Minimal performance impact as these changes are outside the primary hot loops.

## 2026-05-01 - [⬡ Carbo] - Secured AsyncSoulLog File Handling
**Context:** The `AsyncSoulLog` background thread was using `.expect()` when opening the log file, which could cause the entire process to panic or leave the logging thread in a broken state if I/O issues occurred.
**Decision:** Replaced `.expect()` with a graceful `match` block. If file opening fails, the error is reported to `stderr` and the thread exits cleanly. Also added error checking for log directory creation.
**Consequences:** Increased system resilience against I/O failures. Since logging happens in a dedicated thread, this fix preserves the zero-blocking performance characteristics of the main loop while ensuring safety.
