use crate::core::config::GLOBAL_DIMENSION;
use crate::core::fhrr::FHRR;
use ndarray::Array1;

pub struct AxiomGenerator;

impl AxiomGenerator {
    pub fn generate_translation_axiom(
        delta_x: f32,
        delta_y: f32,
        x_seed: &Array1<f32>,
        y_seed: &Array1<f32>,
    ) -> Array1<f32> {
        FHRR::fractional_bind_2d(x_seed, delta_x, y_seed, delta_y)
    }

    pub fn generate_geometric_axiom(
        name: &str,
        delta_x: f32,
        delta_y: f32,
        x_seed: &Array1<f32>,
        y_seed: &Array1<f32>,
    ) -> Array1<f32> {
        let trans = Self::generate_translation_axiom(delta_x, delta_y, x_seed, y_seed);
        let geom_mod = match name {
            "MIRROR_X" => FHRR::fractional_bind(x_seed, -1.0),
            "MIRROR_Y" => FHRR::fractional_bind(y_seed, -1.0),
            "MIRROR_XY" => FHRR::fractional_bind_2d(x_seed, -1.0, y_seed, -1.0),
            _ => {
                let mut identity = Array1::zeros(GLOBAL_DIMENSION);
                identity[0] = 1.0;
                identity[GLOBAL_DIMENSION - 1] = 1.0;
                identity
            }
        };

        FHRR::bind(&geom_mod, &trans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_geometric_axiom() {
        let x_seed = FHRR::create(Some(42));
        let y_seed = FHRR::create(Some(43));

        // Test MIRROR_X
        let mirror_x = AxiomGenerator::generate_geometric_axiom("MIRROR_X", 1.0, 2.0, &x_seed, &y_seed);
        assert_eq!(mirror_x.len(), GLOBAL_DIMENSION);

        // Test MIRROR_Y
        let mirror_y = AxiomGenerator::generate_geometric_axiom("MIRROR_Y", 1.0, 2.0, &x_seed, &y_seed);
        assert_eq!(mirror_y.len(), GLOBAL_DIMENSION);

        // Test MIRROR_XY
        let mirror_xy = AxiomGenerator::generate_geometric_axiom("MIRROR_XY", 1.0, 2.0, &x_seed, &y_seed);
        assert_eq!(mirror_xy.len(), GLOBAL_DIMENSION);

        // Test default case
        let default_case = AxiomGenerator::generate_geometric_axiom("UNKNOWN", 1.0, 2.0, &x_seed, &y_seed);
        assert_eq!(default_case.len(), GLOBAL_DIMENSION);
    }
}
