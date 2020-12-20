use crate::frame::CachedEntities;
use crate::frame::CameraMatrices;
use crate::frame::ConcreteGraphicsPipeline;
use nalgebra::Vector3;
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

pub struct PointLightingSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
}

impl PointLightingSystem {
    /// Initializes the point lighting system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
    ) -> PointLightingSystem {
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
                        alpha_op: BlendOp::Add,
                        alpha_source: BlendFactor::Zero,
                        alpha_destination: BlendFactor::DstColor,
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

        PointLightingSystem {
            gfx_queue,
            pipeline,
        }
    }

    /// Builds a secondary command buffer that applies a point lighting.
    ///
    /// This secondary command buffer will read `depth_input` and rebuild the world position of the
    /// pixel currently being processed (modulo rounding errors). It will then compare this
    /// position with `position`, and process the lighting based on the distance and orientation
    /// (similar to the directional lighting system).
    ///
    /// It then writes the output to the current framebuffer with additive blending (in other words
    /// the value will be added to the existing value in the framebuffer, and not replace the
    /// existing value).
    ///
    /// Note that in a real-world application, you probably want to pass additional parameters
    /// such as some way to indicate the distance at which the lighting decrease. In this example
    /// this value is hardcoded in the shader.
    ///
    /// - `viewport_dimensions` contains the dimensions of the current framebuffer.
    /// - `color_input` is an image containing the albedo of each object of the scene. It is the
    ///   result of the deferred pass.
    /// - `normals_input` is an image containing the normals of each object of the scene. It is the
    ///   result of the deferred pass.
    /// - `depth_input` is an image containing the depth value of each pixel of the scene. It is
    ///   the result of the deferred pass.
    /// - `screen_to_world` is a matrix that turns coordinates from framebuffer space into world
    ///   space. This matrix is used alongside with `depth_input` to determine the world
    ///   coordinates of each pixel being processed.
    /// - `position` is the position of the spot light in world coordinates.
    /// - `color` is the color of the light.
    ///
    pub fn draw<C, N, D>(
        &self,
        color_input: C,
        normals_input: N,
        depth_input: D,
        position: Vector3<f32>,
        color: [f32; 3],
        matrices: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
    ) -> AutoCommandBuffer
    where
        C: ImageViewAccess + Send + Sync + Clone + 'static,
        N: ImageViewAccess + Send + Sync + Clone + 'static,
        D: ImageViewAccess + Send + Sync + Clone + 'static,
    {
        let push_constants = fs::ty::PushConstants {
            color: [color[0], color[1], color[2], 1.0],
            position: [position[0], position[1], position[2], 0.0],
            projection_matrix: matrices.alligned_projection_matrix(),
            view_matrix: matrices.alligned_view_matrix(),
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

                let descriptor_set = PersistentDescriptorSet::start(color_layout.clone())
                    .add_image(color_input.clone())
                    .unwrap()
                    .add_image(normals_input.clone())
                    .unwrap()
                    .add_image(depth_input.clone())
                    .unwrap()
                    .build()
                    .unwrap();

                builder
                    .draw_indexed(
                        self.pipeline.clone(),
                        dynamic_state,
                        cached_entity.vert_params.clone(),
                        cached_entity.indices_params.clone(),
                        (vertex_desc_set, descriptor_set),
                        push_constants,
                    )
                    .unwrap();
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
        path: "src/kikansha/frame/shaders/pointing.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/kikansha/frame/shaders/pointing.frag"
    }
}
