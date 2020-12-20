use crate::engine::cache::CachedEntities;
use crate::frame::ConcreteGraphicsPipeline;
use crate::scene::camera::CameraMatrices;
use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

pub struct TriangleDrawSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
}

impl TriangleDrawSystem {
    /// Initializes a triangle drawing system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
    ) -> TriangleDrawSystem {
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
                    .depth_stencil_simple_depth()
                    .render_pass(subpass)
                    .build(gfx_queue.device().clone())
                    .unwrap(),
            )
        };

        TriangleDrawSystem {
            gfx_queue,
            pipeline,
        }
    }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn draw(
        &self,
        matrices_buff: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
    ) -> AutoCommandBuffer {
        let push_constants = vs::ty::PushConstants {
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
                let layout = self.pipeline.layout().descriptor_set_layout(0).unwrap();

                let set = PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(mutation)
                    .unwrap()
                    .build()
                    .unwrap();

                builder
                    .draw_indexed(
                        self.pipeline.clone(),
                        dynamic_state,
                        cached_entity.vert_params.clone(),
                        cached_entity.indices_params.clone(),
                        set,
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
        path: "src/kikansha/frame/shaders/geomerty.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/kikansha/frame/shaders/geomerty.frag"
    }
}
