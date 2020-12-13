extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

pub mod camera;

use crate::figure::Figure;
use std::sync::Mutex;
use crate::scene::camera::ViewAndProject;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Scene<T: ViewAndProject + Sized> {
    pub camera: Arc<Mutex<T>>,
    pub figures: Vec<Figure>,
}

impl<T: ViewAndProject + Sized> Scene<T> {
    pub fn create(camera: Arc<Mutex<T>>, figures: Vec<Figure>) -> Self {
        Self { camera, figures }
    }
}