use crate::engine::cache::empty_texture;
use crate::engine::cache::{CachedEntities, CachedEntity};
use crate::frame::frame::ConcreteGraphicsPipeline;
use crate::scene::camera::CameraMatrices;
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSetBuf;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSetImg;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSetSampler;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::ImmutableImage;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

type PDS = PersistentDescriptorSet<(
    (
        (
            (
                (
                    (),
                    PersistentDescriptorSetBuf<Arc<CpuAccessibleBuffer<vs::ty::UBO>>>,
                ),
                PersistentDescriptorSetImg<Arc<ImmutableImage<Format>>>,
            ),
            PersistentDescriptorSetSampler,
        ),
        PersistentDescriptorSetImg<Arc<ImmutableImage<Format>>>,
    ),
    PersistentDescriptorSetSampler,
)>;

pub struct TriangleDrawSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
    default_sampler: Arc<Sampler>,
    buff: Arc<CpuAccessibleBuffer<vs::ty::UBO>>,
    set: Arc<PDS>,
}

impl TriangleDrawSystem {
    /// Initializes a triangle drawing system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
        format: Format,
    ) -> TriangleDrawSystem {
        log::trace!("insance of {}", std::any::type_name::<Self>());
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

        let push_constants = vs::ty::UBO {
            projection: CameraMatrices::emmpty(),
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            view: CameraMatrices::emmpty(),
            instancePos: [0.0, 0.0, 0.0, 0.0],
            // [-4.0, 0.0, -4.0, 0.0],
            // [4.0, 0.0, -4.0, 0.0],
        };

        let buff = CpuAccessibleBuffer::from_data(
            pipeline.device().clone(),
            BufferUsage::all(),
            false,
            push_constants,
        )
        .unwrap();

        let layout = pipeline.layout().descriptor_set_layout(0).unwrap();
        let texture = empty_texture(format, gfx_queue.clone());
        let set = Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_buffer(buff.clone())
                .unwrap()
                .add_sampled_image(texture.clone(), default_sampler.clone())
                .unwrap()
                .add_sampled_image(texture.clone(), default_sampler.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        TriangleDrawSystem {
            gfx_queue,
            pipeline,
            default_sampler,
            buff,
            set,
        }
    }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn draw(
        &self,
        matrices_buff: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
    ) -> AutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        {
            let mut content = self.buff.write().unwrap();
            content.projection = matrices_buff.alligned_projection_matrix();
            content.view = matrices_buff.alligned_view_matrix();
        }

        for cached_entity in cached_scene.entities.clone() {
            match cached_entity {
                CachedEntity::Regular(r) => {
                    // for _mutation in r.mutations {

                    builder
                        .draw(
                            self.pipeline.clone(),
                            dynamic_state,
                            r.vert_params.clone(),
                            self.set.clone(),
                            (),
                        )
                        .unwrap();
                    // }
                }
                CachedEntity::Indexed(i) => {
                    // for _mutation in i.mutations {

                    builder
                        .draw_indexed(
                            self.pipeline.clone(),
                            dynamic_state,
                            i.vert_params.clone(),
                            i.indices.clone(),
                            self.set.clone(),
                            (),
                        )
                        .unwrap();
                    // }
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
