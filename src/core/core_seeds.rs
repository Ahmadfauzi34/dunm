use crate::core::fhrr::FHRR;
use ndarray::Array1;
use std::sync::OnceLock;

pub struct CoreSeeds;

impl CoreSeeds {
    pub fn x_axis_seed() -> &'static Array1<f32> {
        static X_AXIS: OnceLock<Array1<f32>> = OnceLock::new();
        X_AXIS.get_or_init(|| FHRR::create(Some(100)))
    }

    pub fn y_axis_seed() -> &'static Array1<f32> {
        static Y_AXIS: OnceLock<Array1<f32>> = OnceLock::new();
        Y_AXIS.get_or_init(|| FHRR::create(Some(200)))
    }

    pub fn color_seed() -> &'static Array1<f32> {
        static COLOR: OnceLock<Array1<f32>> = OnceLock::new();
        COLOR.get_or_init(|| FHRR::create(Some(300)))
    }

    pub fn time_seed() -> &'static Array1<f32> {
        static TIME: OnceLock<Array1<f32>> = OnceLock::new();
        TIME.get_or_init(|| FHRR::create(Some(400)))
    }
}
