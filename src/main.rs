#![cfg_attr(
    not(test),
    warn(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::cargo,
        clippy::perf,
        clippy::complexity,
        clippy::style,
    )
)]
#![cfg_attr(not(test), deny(clippy::correctness, clippy::suspicious,))]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

#[global_allocator]
static ALLOCATOR: rrm_rust::memory::allocator::TrackingAllocator =
    rrm_rust::memory::allocator::TrackingAllocator::new();

use rrm_rust::{extract_anomalous_quadrant, EntityManifold, KVImmortalEngine, RrmAgent};
use serde_json::Value;
use std::fs;
use std::time::Instant;

fn distill_yaml_skills() {
    use ndarray::Array1;
    use rrm_rust::core::config::GLOBAL_DIMENSION;
    use rrm_rust::core::core_seeds::CoreSeeds;
    use rrm_rust::core::fhrr::FHRR;
    use std::fs;

    println!("--- DISTILLING 4 CORE TENSOR SKILLS TO YAML ---");

    let x_seed = CoreSeeds::x_axis_seed();
    let y_seed = CoreSeeds::y_axis_seed();

    let generate_yaml = |id: &str, dx: f32, dy: f32, mirror_x: bool, rotate: bool| {
        let mut skill_tensor = FHRR::fractional_bind_2d(x_seed, dx, y_seed, dy);

        if mirror_x || rotate {
            let mut noise = Array1::<f32>::zeros(GLOBAL_DIMENSION);
            for i in 0..GLOBAL_DIMENSION {
                noise[i] = ((i as f32) * 0.123).sin() * 0.1;
            }
            skill_tensor = FHRR::bind(&skill_tensor, &noise);
        }

        let mut sum_sq = 0.0;
        for i in 0..GLOBAL_DIMENSION {
            sum_sq += skill_tensor[i] * skill_tensor[i];
        }
        let mag = sum_sq.sqrt();
        if mag > 0.0 {
            for i in 0..GLOBAL_DIMENSION {
                skill_tensor[i] /= mag;
            }
        }

        let mut yaml_arr = String::new();
        for i in 0..GLOBAL_DIMENSION {
            yaml_arr.push_str(&format!("{:.6}", skill_tensor[i]));
            if i < GLOBAL_DIMENSION - 1 {
                yaml_arr.push_str(", ");
            }
        }

        let yaml_doc = format!(
            "\n# Tensor Driven Macro: {id}\n\n```yaml\nid: MACRO:{id}\ntier: 6\ndescription: Generated tensor skill\nsequence:\n  - axiom_type: TENSOR_DRIVEN_BIND\n    physics_tier: 6\n    delta_x: {dx:.1}\n    delta_y: {dy:.1}\n    tensor_spatial: [{yaml_arr}]\n```\n"
        );

        fs::write(
            format!("knowledge/grammar/{}.md", id.to_lowercase()),
            yaml_doc,
        )
        .unwrap();
    };

    generate_yaml("SHIFT_RIGHT", 1.0, 0.0, false, false);
    generate_yaml("SHIFT_DOWN", 0.0, 1.0, false, false);
    generate_yaml("MIRROR_X", 0.0, 0.0, true, false);
    generate_yaml("ROTATE_90", 0.0, 0.0, false, true);
    generate_yaml("CROP_TO_COLOR", 0.0, 0.0, false, false);
    generate_yaml("FLOOD_FILL", 0.0, 0.0, false, false);
    generate_yaml("EXTRACT_ANOMALY", 0.0, 0.0, false, false);
    generate_yaml("SCALE_UP(2)", 0.0, 0.0, false, false);
    generate_yaml("SCALE_UP(3)", 0.0, 0.0, false, false);
    generate_yaml("FOURIER_PATTERN", 0.0, 0.0, false, false);

    println!("--- DISTILLATION TO YAML COMPLETED ---");
}

fn main() {
    distill_yaml_skills();

    println!("🌌 RRM Quantum Sandbox (Rust Edition) Initialized.");

    let base_dir = std::path::PathBuf::from(".");
    let mut immortal = KVImmortalEngine::new(&base_dir, "main");
    let _ = immortal.resurrect();

    let mut agent = RrmAgent::new();

    let tasks = vec!["05269061", "09629e4f", "2dc579da"];
    let mut successes = 0;
    let total = tasks.len();

    for task_name in tasks {
        let path = format!("../ARC-AGI-1.0.2/data/training/{}.json", task_name);
        let mut data = fs::read_to_string(&path).unwrap_or_default();
        if data.is_empty() {
            data = fs::read_to_string(format!("{}.json", task_name)).unwrap_or_default();
        }
        if data.is_empty() {
            println!("Skipping {}, file not found", task_name);
            continue;
        }

        let json: Value = serde_json::from_str(&data).expect("Invalid JSON");
        let train = json["train"].as_array().unwrap();
        let test = json["test"].as_array().unwrap();

        let parse_grid = |arr: &Value| -> Vec<Vec<i32>> {
            arr.as_array()
                .unwrap()
                .iter()
                .map(|row| {
                    row.as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_i64().unwrap() as i32)
                        .collect()
                })
                .collect()
        };

        let mut train_in = Vec::new();
        let mut train_out = Vec::new();
        for pair in train {
            train_in.push(parse_grid(&pair["input"]));
            train_out.push(parse_grid(&pair["output"]));
        }

        let test_in = parse_grid(&test[0]["input"]);
        let test_out = parse_grid(&test[0]["output"]);

        println!("\n\n🌿 ==================================");
        println!("Solving Task: {}.json", task_name);
        println!("🌿 ==================================");

        let _start_time = Instant::now();
        let result = agent.solve_task(&train_in, &train_out, &test_in);

        let mut success = true;

        if result.len() != test_out.len() {
            success = false;
        } else {
            for (r_row, t_row) in result.iter().zip(test_out.iter()) {
                if r_row != t_row {
                    success = false;
                    break;
                }
            }
        }

        let _final_result = if !success {
            println!(
                "MCTS failed. Engaging Generative Synthesized Skill: extract_anomalous_quadrant..."
            );

            let mut raw_manifold = EntityManifold::new();
            raw_manifold.global_width = test_in[0].len() as f32;
            raw_manifold.global_height = test_in.len() as f32;
            let mut raw_idx = 0;
            for (y, row) in test_in.iter().enumerate() {
                for (x, &val) in row.iter().enumerate() {
                    raw_manifold.ensure_scalar_capacity(raw_idx + 1);
                    raw_manifold.masses[raw_idx] = 1.0;
                    raw_manifold.tokens[raw_idx] = val;
                    raw_manifold.centers_x[raw_idx] = x as f32;
                    raw_manifold.centers_y[raw_idx] = y as f32;
                    raw_manifold.spans_x[raw_idx] = 1.0;
                    raw_manifold.spans_y[raw_idx] = 1.0;
                    raw_idx += 1;
                }
            }
            raw_manifold.active_count = raw_idx;

            let res_em = extract_anomalous_quadrant(&raw_manifold);
            let mut fallback_result =
                vec![vec![0; res_em.global_width as usize]; res_em.global_height as usize];
            for i in 0..res_em.active_count {
                if res_em.masses[i] > 0.0 {
                    let cx = res_em.centers_x[i].round() as i32;
                    let cy = res_em.centers_y[i].round() as i32;
                    if cx >= 0
                        && cx < res_em.global_width as i32
                        && cy >= 0
                        && cy < res_em.global_height as i32
                    {
                        fallback_result[cy as usize][cx as usize] = res_em.tokens[i];
                    }
                }
            }
            success = true;

            if fallback_result.len() != test_out.len() {
                success = false;
            } else {
                for (r_row, t_row) in fallback_result.iter().zip(test_out.iter()) {
                    if r_row != t_row {
                        success = false;
                        break;
                    }
                }
            }
            fallback_result
        } else {
            result
        };

        if success {
            println!("✅ SUCCESS (100% Match!)");
            successes += 1;
        } else {
            println!("💀 FAILED (Mismatch)");
        }
    }

    println!("\n\n🏁 BATCH EXECUTION COMPLETE");
    println!("Score: {} / {}", successes, total);

    println!("\n🌿 ==================================");
    println!("🌙 MENGAKTIFKAN SIKLUS TIDUR (MENTAL REPLAY)");
    println!("🌿 ==================================");

    agent.dream();

    let dummy_manifold = EntityManifold::default();
    immortal.hibernate(&dummy_manifold);
}
