use crate::figure::FigureMutation;
use crate::figure::FigureSet;
use crate::figure::PerVerexParams;
use vulkano::buffer::BufferUsage;
use crate::scene::Scene;
use crate::scene::camera::ViewAndProject;
use std::sync::Arc;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;

#[derive(Debug, Clone)]
pub struct CachedEntity {
    pub vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
    pub indices_params: Arc<CpuAccessibleBuffer<[u32]>>,
    pub mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
}

impl CachedEntity {
    pub fn new(
        vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
        indices_params: Arc<CpuAccessibleBuffer<[u32]>>,
        mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
    ) -> Self {
        CachedEntity {
            vert_params,
            indices_params,
            mutations,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct CachedEntities {
    pub entities: Vec<CachedEntity>,
}

pub struct SceneCache {
    cache_id: u32,
    state: Option<CachedEntities>,
}

impl SceneCache {
    pub fn default() -> Self {
        SceneCache {
            cache_id: 0,
            state: None,
        }
    }

    pub fn get_cache<T: ViewAndProject + Sized>(
        &mut self,
        scene: &Scene<T>,
        device: Arc<Device>,
    ) -> CachedEntities {
        if scene.global_scene_id == self.cache_id {
            match &self.state {
                Some(cached) => cached.clone(),
                None => {
                    let new_cache = Self::prepare_cache(&scene.figures, device);
                    self.state = Some(new_cache.clone());
                    new_cache
                }
            }
        } else {
            let new_cache = Self::prepare_cache(&scene.figures, device);
            self.state = Some(new_cache.clone());
            self.cache_id = scene.global_scene_id;
            new_cache
        }
    }

    fn prepare_cache(figures: &[FigureSet], device: Arc<Device>) -> CachedEntities {
        let entities = figures
            .iter()
            .map(|figure_set| {
                let per_vertex_data: Vec<PerVerexParams> = figure_set
                    .figure
                    .vertices
                    .clone()
                    .into_iter()
                    .map(|v| PerVerexParams {
                        position: v.position,
                        color: figure_set.figure.base_color,
                    })
                    .collect();
                let ver_buff = CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    false,
                    per_vertex_data.into_iter(),
                )
                .unwrap();
                let indices_buff = CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    false,
                    figure_set.figure.indices.clone().into_iter(),
                )
                .unwrap();
                let mutations = figure_set
                    .mutations
                    .clone()
                    .into_iter()
                    .map(|mutation| {
                        CpuAccessibleBuffer::from_data(
                            device.clone(),
                            BufferUsage::all(),
                            false,
                            mutation,
                        )
                        .unwrap()
                    })
                    .collect();
                CachedEntity::new(ver_buff, indices_buff, mutations)
            })
            .collect();
        CachedEntities { entities }
    }
}
