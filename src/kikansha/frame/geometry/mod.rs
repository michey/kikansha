use crate::engine::cache::{CachedEntities, CachedEntity};
use crate::frame::ConcreteGraphicsPipeline;
use crate::scene::camera::CameraMatrices;
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
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

pub struct TriangleDrawSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
    default_sampler: Arc<Sampler>,
}

impl TriangleDrawSystem {
    /// Initializes a triangle drawing system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
    ) -> TriangleDrawSystem {
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
                    .cull_mode_back()
                    .front_face_clockwise()
                    .depth_write(true)
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs.main_entry_point(), ())
                    .depth_stencil_simple_depth()
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

        TriangleDrawSystem {
            gfx_queue,
            pipeline,
            default_sampler,
        }
    }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn draw(
        &self,
        matrices_buff: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
    ) -> AutoCommandBuffer {
        let push_constants = vs::ty::UBO {
            projection: matrices_buff.alligned_projection_matrix(),
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            view: matrices_buff.alligned_view_matrix(),
            instancePos: [0.0, 0.0, 0.0, 0.0],
            // [-4.0, 0.0, -4.0, 0.0],
            // [4.0, 0.0, -4.0, 0.0],
        };

        let buff = CpuAccessibleBuffer::from_data(
            self.pipeline.device().clone(),
            BufferUsage::all(),
            false,
            push_constants,
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        for cached_entity in cached_scene.entities.clone() {
            match cached_entity {
                CachedEntity::Regular(r) => {
                    for _mutation in r.mutations {
                        let layout = self.pipeline.layout().descriptor_set_layout(0).unwrap();

                        let set = PersistentDescriptorSet::start(layout.clone())
                            .add_buffer(buff.clone())
                            .unwrap()
                            .add_sampled_image(r.color_texture.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_sampled_image(r.normal_texture.clone(), self.default_sampler.clone())
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
                        let layout = self.pipeline.layout().descriptor_set_layout(0).unwrap();

                        let set = PersistentDescriptorSet::start(layout.clone())
                            .add_buffer(buff.clone())
                            .unwrap()
                            .add_sampled_image(i.color_texture.clone(), self.default_sampler.clone())
                            .unwrap()
                            .add_sampled_image(i.normal_texture.clone(), self.default_sampler.clone())
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

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/kikansha/frame/shaders/geomerty.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/kikansha/frame/shaders/geomerty.frag"
    }
}
