use crate::engine::cache::CachedEntities;
use crate::frame::CameraMatrices;
use crate::frame::ConcreteGraphicsPipeline;
use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::ImageViewAccess;
use vulkano::pipeline::blend::AttachmentBlend;
use vulkano::pipeline::blend::BlendFactor;
use vulkano::pipeline::blend::BlendOp;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

/// Allows applying an ambient lighting to a scene.
pub struct AmbientLightingSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
}

impl AmbientLightingSystem {
    /// Initializes the ambient lighting system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
    ) -> AmbientLightingSystem {
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
                    .blend_collective(AttachmentBlend {
                        enabled: true,
                        color_op: BlendOp::Add,
                        color_source: BlendFactor::One,
                        color_destination: BlendFactor::One,
                        alpha_op: BlendOp::Max,
                        alpha_source: BlendFactor::One,
                        alpha_destination: BlendFactor::One,
                        mask_red: true,
                        mask_green: true,
                        mask_blue: true,
                        mask_alpha: true,
                    })
                    .render_pass(subpass)
                    .build(gfx_queue.device().clone())
                    .unwrap(),
            )
        };

        AmbientLightingSystem {
            gfx_queue,
            pipeline,
        }
    }

    pub fn draw<C>(
        &self,
        color_input: C,
        ambient_color: [f32; 3],
        matrices_buff: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
    ) -> AutoCommandBuffer
    where
        C: ImageViewAccess + Send + Sync + Clone + 'static,
    {
        let push_constants = fs::ty::PushConstants {
            color: [ambient_color[0], ambient_color[1], ambient_color[2], 1.0],
            projection_matrix: matrices_buff.alligned_projection_matrix(),
            view_matrix: matrices_buff.alligned_view_matrix(),
        };

        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        for cached_entity in cached_scene.entities.clone() {
            for mutation in cached_entity.mutations {
                let vertex_layout = self.pipeline.layout().descriptor_set_layout(0).unwrap();
                let color_layout = self.pipeline.layout().descriptor_set_layout(1).unwrap();
        
                let vertex_desc_set = Arc::new(
                    PersistentDescriptorSet::start(vertex_layout.clone())        
                        .add_buffer(mutation)
                        .unwrap()
                        .build()
                        .unwrap(),
                );

                let color_desc_set = Arc::new(
                    PersistentDescriptorSet::start(color_layout.clone())
                        .add_image(color_input.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                );

                builder
                    .draw_indexed(
                        self.pipeline.clone(),
                        dynamic_state,
                        cached_entity.vert_params.clone(),
                        cached_entity.indices_params.clone(),
                        (vertex_desc_set, color_desc_set),
                        push_constants,
                    )
                    .unwrap();
            }
        }

        builder.build().unwrap()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/kikansha/frame/shaders/ambient.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/kikansha/frame/shaders/ambient.frag"
    }
}
