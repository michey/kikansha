use crate::figure::FigureMutation;
use crate::figure::FigureSet;
use crate::figure::PerVerexParams;
use crate::figure::RenderableMesh;
use crate::scene::camera::ViewAndProject;
use crate::scene::Scene;
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;

#[derive(Debug, Clone)]
pub enum CachedEntity {
    Indexed(CachedIndexedEntity),
    Regular(CachedRegularEntity),
}

#[derive(Debug, Clone)]
pub struct CachedIndexedEntity {
    pub vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
    pub indices: Arc<CpuAccessibleBuffer<[u32]>>,
    pub mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
}

impl CachedIndexedEntity {
    pub fn new(
        vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
        indices: Arc<CpuAccessibleBuffer<[u32]>>,
        mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
    ) -> Self {
        CachedIndexedEntity {
            vert_params,
            indices,
            mutations,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedRegularEntity {
    pub vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
    pub mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
}

impl CachedRegularEntity {
    pub fn new(
        vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
        mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
    ) -> Self {
        CachedRegularEntity {
            vert_params,
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

                match figure_set.mesh.clone() {
                    RenderableMesh::Indexed(ind) => {
                        let per_vertex_params: Vec<PerVerexParams> =
                            ind.points.into_iter().map(|p| p.to_vert()).collect();

                        let ver_buff = CpuAccessibleBuffer::from_iter(
                            device.clone(),
                            BufferUsage::all(),
                            false,
                            per_vertex_params.into_iter(),
                        )
                        .unwrap();

                        let indices: Vec<u32> = ind.indices;

                        let indices_buff = CpuAccessibleBuffer::from_iter(
                            device.clone(),
                            BufferUsage::all(),
                            false,
                            indices.into_iter(),
                        )
                        .unwrap();

                        CachedEntity::Indexed(CachedIndexedEntity::new(
                            ver_buff,
                            indices_buff,
                            mutations,
                        ))
                    }
                    RenderableMesh::Regular(reg) => {
                        let per_vertex_params: Vec<PerVerexParams> =
                            reg.points.into_iter().map(|p| p.to_vert()).collect();

                        let ver_buff = CpuAccessibleBuffer::from_iter(
                            device.clone(),
                            BufferUsage::all(),
                            false,
                            per_vertex_params.into_iter(),
                        )
                        .unwrap();
                        CachedEntity::Regular(CachedRegularEntity::new(ver_buff, mutations))
                    }
                }
            })
            .collect();
        CachedEntities { entities }
    }
}
