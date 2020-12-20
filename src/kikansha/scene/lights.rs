use nalgebra::Vector3;

#[derive(Debug, Clone)]
pub enum Light {
    Ambient([f32; 3]),
    Directional(Vector3<f32>, [f32; 3]),
    Point(Vector3<f32>, [f32; 3]),
}
