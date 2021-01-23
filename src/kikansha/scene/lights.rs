use nalgebra_glm::Vec3;
use nalgebra_glm::Vec4;

#[derive(Debug, Clone)]
pub enum Light {
    Point(PointLight),
}

#[derive(Debug, Clone)]
pub struct PointLight {
    pub position: Vec4,
    pub color: Vec3,
    pub radius: f32,
}

impl PointLight {
    pub fn new(position: Vec4, color: Vec3, radius: f32) -> Self {
        PointLight {
            position,
            color,
            radius,
        }
    }

    pub fn default_lights() -> Vec<Light> {
        let a_light = PointLight::new(
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            15.0 * 0.25,
        );
        // let b_light = PointLight::new(
        //     Vec4::new(-2.0, 0.0, 0.0, 0.0),
        //     Vec3::new(1.0, 0.0, 0.0),
        //     15.0,
        // );

        // let c_light = PointLight::new(
        //     Vec4::new(2.0, -1.0, 0.0, 0.0),
        //     Vec3::new(0.0, 0.0, 2.5),
        //     5.0,
        // );

        // let d_light = PointLight::new(
        //     Vec4::new(0.0, -0.9, 0.5, 0.0),
        //     Vec3::new(1.0, 1.0, 0.0),
        //     2.0,
        // );

        // let e_light = PointLight::new(
        //     Vec4::new(0.0, -0.5, 0.0, 0.0),
        //     Vec3::new(0.0, 1.0, 0.2),
        //     5.0,
        // );

        // let f_light = PointLight::new(
        //     Vec4::new(0.0, -1.0, 0.0, 0.0),
        //     Vec3::new(1.0, 0.7, 0.3),
        //     25.0,
        // );

        [a_light,
        // b_light, c_light, d_light, e_light, f_light
        ]
            .iter()
            .map(|pl| Light::Point(pl.clone()))
            .collect()
    }
}
