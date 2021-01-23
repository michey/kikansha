use crate::engine::cache::{CachedEntities, CachedEntity};
use crate::frame::CameraMatrices;
use crate::frame::ConcreteGraphicsPipeline;
use crate::frame::Light;

use std::convert::TryInto;
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::ImageViewAccess;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

/// Allows applying a directional light source to a scene.
pub struct LightingSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
    default_sampler: Arc<Sampler>,
}

impl LightingSystem {
    /// Initializes the directional lighting system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
    ) -> LightingSystem {


        log::trace!("insance of {}",  std::any::type_name::<Self>());
        let pipeline = {
            let vs = vs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");
            let fs = fs::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader module");

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer()
                    .vertex_shader(vs.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs.main_entry_point(), ())
                    .render_pass(subpass)
                    .build(gfx_queue.device().clone())
                    .unwrap(),
            )
        };

        let default_sampler = Sampler::new(
            pipeline.device().clone(),
            Filter::Nearest,
            Filter::Nearest,
            MipmapMode::Linear,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0,
            1.0,
            0.0,
            1.0,
        )
        .unwrap();


        LightingSystem {
            gfx_queue,
            pipeline,
            default_sampler
        }
    }

    /// Builds a secondary command buffer that applies directional lighting.
    ///
    /// This secondary command buffer will read `color_input` and `normals_input`, and multiply the
    /// color with `color` and the dot product of the `direction` with the normal.
    /// It then writes the output to the current framebuffer with additive blending (in other words
    /// the value will be added to the existing value in the framebuffer, and not replace the
    /// existing value).
    ///
    /// Since `normals_input` contains normals in world coordinates, `direction` should also be in
    /// world coordinates.
    ///
    /// - `viewport_dimensions` contains the dimensions of the current framebuffer.
    /// - `color_input` is an image containing the albedo of each object of the scene. It is the
    ///   result of the deferred pass.
    /// - `normals_input` is an image containing the normals of each object of the scene. It is the
    ///   result of the deferred pass.
    /// - `direction` is the direction of the light in world coordinates.
    /// - `color` is the color to apply.
    ///
    pub fn draw<D, C, N>(
        &self,
        albedo_input: D,
        color_input: C,
        normals_input: N,
        lights: &[Light],
        matrices_buff: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
    ) -> AutoCommandBuffer
    where
        D: ImageViewAccess + Send + Sync + Clone + 'static,
        C: ImageViewAccess + Send + Sync + Clone + 'static,
        N: ImageViewAccess + Send + Sync + Clone + 'static,
    {

        let eye = matrices_buff.camera_position;
        let view_pos = [eye[0] * -1.0, eye[1] * -1.0, eye[2] * -1.0, 0.0];


        let mut packed_lights = Vec::new();

        for l in lights {
            match l {
                Light::Point(pl) => packed_lights.push(fs::ty::Light {
                    position: pl.position.into(),
                    color: pl.color.into(),
                    radius: pl.radius,
                }),
            }
        }


        let push_constants = fs::ty::UBO {
            lights: packed_lights.try_into().unwrap(),
            viewPos: view_pos,
            displayDebugTarget: 0,
        };

        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        let buff = CpuAccessibleBuffer::from_data(
            self.pipeline.device().clone(),
            BufferUsage::uniform_buffer(),
            false,
            push_constants,
        )
        .unwrap();

        let layout = self.pipeline.layout().descriptor_set_layout(0).unwrap();



        for cached_entity in cached_scene.entities.clone() {
            match cached_entity {
                CachedEntity::Regular(r) => {
                    for _mutation in r.mutations {
                        let set = PersistentDescriptorSet::start(layout.clone())
                            .add_empty()
                            .unwrap()
                            .add_sampled_image(color_input.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_sampled_image(normals_input.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_sampled_image(albedo_input.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_buffer(buff.clone())
                            .unwrap()
                            .build()
                            .unwrap();

                        builder
                            .draw(
                                self.pipeline.clone(),
                                dynamic_state,
                                r.vert_params.clone(),
                                set,
                                (),
                            )
                            .unwrap();
                    }
                }
                CachedEntity::Indexed(i) => {
                    for _mutation in i.mutations {
                        let set = PersistentDescriptorSet::start(layout.clone())
                            .add_empty()
                            .unwrap()
                            .add_sampled_image(color_input.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_sampled_image(normals_input.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_sampled_image(albedo_input.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_buffer(buff.clone())
                            .unwrap()
                            .build()
                            .unwrap();

                        builder
                            .draw_indexed(
                                self.pipeline.clone(),
                                dynamic_state,
                                i.vert_params.clone(),
                                i.indices.clone(),
                                set,
                                (),
                            )
                            .unwrap();
                    }
                }
            }
        }

        builder.build().unwrap()
    }
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/kikansha/frame/shaders/deferred.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        types_meta: {
            #[derive(Clone, Debug, Copy)]
        },
        path: "src/kikansha/frame/shaders/deferred.frag"
    }
}
