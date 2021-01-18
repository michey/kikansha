use crate::engine::cache::{CachedEntities, CachedEntity};
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

/// Allows applying a directional light source to a scene.
pub struct DirectionalLightingSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
}

impl DirectionalLightingSystem {
    /// Initializes the directional lighting system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
    ) -> DirectionalLightingSystem {
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

        DirectionalLightingSystem {
            gfx_queue,
            pipeline,
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
    pub fn draw<C, N>(
        &self,
        color_input: C,
        normals_input: N,
        direction: Vector3<f32>,
        color: [f32; 3],
        matrices: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
    ) -> AutoCommandBuffer
    where
        C: ImageViewAccess + Send + Sync + Clone + 'static,
        N: ImageViewAccess + Send + Sync + Clone + 'static,
    {
        let push_constants = fs::ty::PushConstants {
            color: [color[0], color[1], color[2], 1.0],
            direction: [direction[0], direction[1], direction[2], 0.0],
            projection_matrix: matrices.alligned_projection_matrix(),
            view_matrix: matrices.alligned_view_matrix(),
        };

        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        let color_layout = self.pipeline.layout().descriptor_set_layout(1).unwrap();

        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(color_layout.clone())
                .add_image(color_input.clone())
                .unwrap()
                .add_image(normals_input.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        for cached_entity in cached_scene.entities.clone() {
            match cached_entity {
                CachedEntity::Regular(r) => {
                    for mutation in r.mutations {
                        let layout = self.pipeline.layout().descriptor_set_layout(0).unwrap();

                        let set = PersistentDescriptorSet::start(layout.clone())
                            .add_buffer(mutation)
                            .unwrap()
                            .build()
                            .unwrap();

                        builder
                            .draw(
                                self.pipeline.clone(),
                                dynamic_state,
                                r.vert_params.clone(),
                                (set, descriptor_set.clone()),
                                push_constants,
                            )
                            .unwrap();
                    }
                }
                CachedEntity::Indexed(i) => {
                    for mutation in i.mutations {
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
                                i.vert_params.clone(),
                                i.indices.clone(),
                                (set, descriptor_set.clone()),
                                push_constants,
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
        path: "src/kikansha/frame/shaders/directional.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/kikansha/frame/shaders/directional.frag"
    }
}
