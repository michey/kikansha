use crate::figure::FigureMutation;
use crate::figure::FigureSet;
use crate::figure::PerVerexParams;
use crate::figure::RenderableMesh;
use crate::scene::camera::ViewAndProject;
use crate::scene::Scene;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage, MipmapsCount};

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
    pub color_texture: Arc<ImmutableImage<Format>>,
    pub normal_texture: Arc<ImmutableImage<Format>>,
}

impl CachedIndexedEntity {
    pub fn new(
        vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
        indices: Arc<CpuAccessibleBuffer<[u32]>>,
        mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
        color_texture: Arc<ImmutableImage<Format>>,
        normal_texture: Arc<ImmutableImage<Format>>,
    ) -> Self {
        log::trace!("insance of {}",  std::any::type_name::<Self>());
        CachedIndexedEntity {
            vert_params,
            indices,
            mutations,
            color_texture,
            normal_texture,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedRegularEntity {
    pub vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
    pub mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
    pub color_texture: Arc<ImmutableImage<Format>>,
    pub normal_texture: Arc<ImmutableImage<Format>>,
}

impl CachedRegularEntity {
    pub fn new(
        vert_params: Arc<CpuAccessibleBuffer<[PerVerexParams]>>,
        mutations: Vec<Arc<CpuAccessibleBuffer<FigureMutation>>>,
        color_texture: Arc<ImmutableImage<Format>>,
        normal_texture: Arc<ImmutableImage<Format>>,
    ) -> Self {
        log::trace!("insance of {}",  std::any::type_name::<Self>());
        CachedRegularEntity {
            vert_params,
            mutations,
            color_texture,
            normal_texture,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedEntities {
    pub entities: Vec<CachedEntity>,
}

#[derive(Debug, Clone)]
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
        queue: Arc<Queue>,
    ) -> CachedEntities {
        if scene.global_scene_id == self.cache_id {
            match &self.state {
                Some(cached) => cached.clone(),
                None => {
                    let new_cache = Self::prepare_cache(&scene.figures, device, queue);
                    self.state = Some(new_cache.clone());
                    new_cache
                }
            }
        } else {
            let new_cache = Self::prepare_cache(&scene.figures, device, queue);
            self.state = Some(new_cache.clone());
            self.cache_id = scene.global_scene_id;
            new_cache
        }
    }

    fn prepare_cache(
        figures: &[FigureSet],
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> CachedEntities {
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

                let color_texture = load_texture_by_path(
                    figure_set.color_texture_path.clone(),
                    Format::R8G8B8A8Srgb,
                    queue.clone(),
                )
                .unwrap();
                let normal_texture = load_texture_by_path(
                    figure_set.color_texture_path.clone(),
                    Format::R8G8B8A8Srgb,
                    queue.clone(),
                )
                .unwrap();

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
                            color_texture,
                            normal_texture,
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
                        CachedEntity::Regular(CachedRegularEntity::new(
                            ver_buff,
                            mutations,
                            color_texture,
                            normal_texture,
                        ))
                    }
                }
            })
            .collect();
        CachedEntities { entities }
    }
}

fn load_texture_by_path(
    path: String,
    format: Format,
    queue: Arc<Queue>,
) -> io::Result<Arc<ImmutableImage<Format>>> {
    let (texture, _tex_future) = {
        let f = File::open(path)?;
        let b = BufReader::new(f);

        let decoder = png::Decoder::new(b);
        let (info, mut reader) = decoder.read_info().unwrap();
        let dimensions = Dimensions::Dim2d {
            width: info.width,
            height: info.height,
        };
        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            dimensions,
            MipmapsCount::One,
            format,
            queue,
        )
        .unwrap()
    };
    io::Result::Ok(texture)
}
