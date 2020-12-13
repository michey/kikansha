extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

pub mod camera;

use crate::figure::FigureSet;
use crate::scene::camera::ViewAndProject;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Scene<T: ViewAndProject + Sized> {
    pub camera: Arc<Mutex<T>>,
    pub figures: Vec<FigureSet>,
    pub global_scene_id: u32,
}

impl<T: ViewAndProject + Sized> Scene<T> {
    pub fn create(camera: Arc<Mutex<T>>, figures: Vec<FigureSet>) -> Self {
        Self {
            camera,
            figures,
            global_scene_id: 1,
        }
    }
}
