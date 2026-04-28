use rrm_rust::core::fhrr::FHRR;
use rrm_rust::core::config::GLOBAL_DIMENSION;
use rrm_rust::core::core_seeds::CoreSeeds;
use ndarray::Array1;
use std::fs;

fn main() {
    println!("--- TENSOR DISTILLATION STARTED ---");

    let mut in_agg_spatial = Array1::<f32>::zeros(GLOBAL_DIMENSION);
    let mut in_agg_semantic = Array1::<f32>::zeros(GLOBAL_DIMENSION);

    let x_seed = CoreSeeds::x_axis_seed();
    let y_seed = CoreSeeds::y_axis_seed();
    let sem_seeds = CoreSeeds::semantic_seeds();

    for y in 0..5 {
        for x in 0..5 {
            let color = if x >= 1 && x <= 2 && y >= 1 && y <= 2 { 2 } else { 1 };
            let spatial = FHRR::fractional_bind_2d(&x_seed, x as f32, &y_seed, y as f32);
            in_agg_spatial = &in_agg_spatial + &spatial;
            let semantic = sem_seeds[color as usize % 10].clone();
            in_agg_semantic = &in_agg_semantic + &semantic;
        }
    }
    in_agg_spatial = FHRR::normalize(&in_agg_spatial);
    in_agg_semantic = FHRR::normalize(&in_agg_semantic);

    let mut out_agg_spatial = Array1::<f32>::zeros(GLOBAL_DIMENSION);
    let mut out_agg_semantic = Array1::<f32>::zeros(GLOBAL_DIMENSION);

    for y in 0..2 {
        for x in 0..2 {
            let color = 2;
            let spatial = FHRR::fractional_bind_2d(&x_seed, x as f32, &y_seed, y as f32);
            out_agg_spatial = &out_agg_spatial + &spatial;
            let semantic = sem_seeds[color as usize % 10].clone();
            out_agg_semantic = &out_agg_semantic + &semantic;
        }
    }
    out_agg_spatial = FHRR::normalize(&out_agg_spatial);
    out_agg_semantic = FHRR::normalize(&out_agg_semantic);

    let tensor_spatial = FHRR::bind(&out_agg_spatial, &FHRR::inverse(&in_agg_spatial));
    let tensor_semantic = FHRR::bind(&out_agg_semantic, &FHRR::inverse(&in_agg_semantic));

    let mut spatial_str = String::from("[");
    for i in 0..GLOBAL_DIMENSION {
        spatial_str.push_str(&format!("{:.4}", tensor_spatial[i]));
        if i < GLOBAL_DIMENSION - 1 {
            spatial_str.push_str(", ");
        }
    }
    spatial_str.push_str("]");

    fs::write("anomaly_spatial.json", spatial_str).unwrap();

    let mut semantic_str = String::from("[");
    for i in 0..GLOBAL_DIMENSION {
        semantic_str.push_str(&format!("{:.4}", tensor_semantic[i]));
        if i < GLOBAL_DIMENSION - 1 {
            semantic_str.push_str(", ");
        }
    }
    semantic_str.push_str("]");

    fs::write("anomaly_semantic.json", semantic_str).unwrap();

    println!("--- TENSOR DISTILLATION COMPLETED ---");
}
