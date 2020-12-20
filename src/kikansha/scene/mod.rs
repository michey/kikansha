extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

pub mod camera;
pub mod lights;

use crate::figure::FigureSet;
use crate::scene::camera::ViewAndProject;
use crate::scene::lights::Light;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Scene<T: ViewAndProject + Sized> {
    pub camera: Arc<Mutex<T>>,
    pub figures: Vec<FigureSet>,
    pub global_scene_id: u32,
    pub lights: Vec<Light>,
}

impl<T: ViewAndProject + Sized> Scene<T> {
    pub fn create(camera: Arc<Mutex<T>>, figures: Vec<FigureSet>, lights: Vec<Light>) -> Self {
        Self {
            camera,
            figures,
            global_scene_id: 1,
            lights,
        }
    }
}
