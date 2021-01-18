use crate::frame::lightning::ambient::AmbientLightingSystem;
use crate::frame::lightning::directional::DirectionalLightingSystem;
use crate::frame::lightning::pointing::PointLightingSystem;
use crate::frame::CachedEntities;
use crate::frame::CameraMatrices;
use crate::frame::Frame;
use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::SubpassContents;
use vulkano::device::Queue;
use vulkano::format::ClearValue;
use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::AttachmentImage;
use vulkano::image::ImageAccess;
use vulkano::image::ImageUsage;
use vulkano::image::ImageViewAccess;
use vulkano::sync::GpuFuture;

/// System that contains the necessary facilities for rendering a single frame.
pub struct FrameSystem {
    // Queue to use to render everything.
    pub gfx_queue: Arc<Queue>,

    // Render pass used for the drawing. See the `new` method for the actual render pass content.
    // We need to keep it in `FrameSystem` because we may want to recreate the intermediate buffers
    // in of a change in the dimensions.
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,

    // Intermediate render target that will contain the albedo of each pixel of the scene.
    pub diffuse_buffer: Arc<AttachmentImage>,
    // Intermediate render target that will contain the normal vector in world coordinates of each
    // pixel of the scene.
    // The normal vector is the vector perpendicular to the surface of the object at this point.
    pub normals_buffer: Arc<AttachmentImage>,
    // Intermediate render target that will contain the depth of each pixel of the scene.
    // This is a traditional depth buffer. `0.0` means "near", and `1.0` means "far".
    pub depth_buffer: Arc<AttachmentImage>,

    // Will allow us to add an ambient lighting to a scene during the second subpass.
    pub ambient_lighting_system: AmbientLightingSystem,
    // Will allow us to add a directional light to a scene during the second subpass.
    pub directional_lighting_system: DirectionalLightingSystem,
    // Will allow us to add a spot light source to a scene during the second subpass.
    pub point_lighting_system: PointLightingSystem,
}

impl FrameSystem {
    fn create_everything(
        gfx_queue: &Arc<Queue>,
        final_output_format: Format,
        dimensions: [u32; 2],
    ) -> (
        Arc<(dyn RenderPassAbstract + Send + Sync + 'static)>,
        Arc<AttachmentImage>,
        Arc<AttachmentImage>,
        Arc<AttachmentImage>,
        AmbientLightingSystem,
        DirectionalLightingSystem,
        PointLightingSystem,
    ) {
        let render_pass: Arc<dyn RenderPassAbstract + Send + Sync + 'static> = Arc::new(
            vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
                attachments: {
                    // The image that will contain the final rendering (in this example the swapchain
                    // image, but it could be another image).
                    final_color: {
                        load: Clear,
                        store: Store,
                        format: final_output_format,
                        samples: 1,
                    },
                    // Will be bound to `self.diffuse_buffer`.
                    diffuse: {
                        load: Clear,
                        store: DontCare,
                        format: Format::A2B10G10R10UnormPack32,
                        samples: 1,
                    },
                    // Will be bound to `self.normals_buffer`.
                    normals: {
                        load: Clear,
                        store: DontCare,
                        format: Format::R16G16B16A16Sfloat,
                        samples: 1,
                    },
                    // Will be bound to `self.depth_buffer`.
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                passes: [
                    // Write to the diffuse, normals and depth attachments.
                    {
                        color: [diffuse, normals],
                        depth_stencil: {depth},
                        input: []
                    },
                    // Apply lighting by reading these three attachments and writing to `final_color`.
                    {
                        color: [final_color],
                        depth_stencil: {},
                        input: [diffuse, normals, depth]
                    }
                ]
            )
            .unwrap(),
        );

        // For now we create three temporary images with a dimension of 1 by 1 pixel.
        // These images will be replaced the first time we call `frame()`.
        // TODO: use shortcut provided in vulkano 0.6
        let atch_usage = ImageUsage {
            transient_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };
        let diffuse_buffer = AttachmentImage::with_usage(
            gfx_queue.device().clone(),
            dimensions,
            Format::A2B10G10R10UnormPack32,
            atch_usage,
        )
        .unwrap();
        let normals_buffer = AttachmentImage::with_usage(
            gfx_queue.device().clone(),
            dimensions,
            Format::R16G16B16A16Sfloat,
            atch_usage,
        )
        .unwrap();
        let depth_buffer = AttachmentImage::with_usage(
            gfx_queue.device().clone(),
            dimensions,
            Format::D16Unorm,
            atch_usage,
        )
        .unwrap();

        // Initialize the three lighting systems.
        // Note that we need to pass to them the subpass where they will be executed.
        let lighting_subpass = Subpass::from(render_pass.clone(), 1).unwrap();
        let ambient_lighting_system =
            AmbientLightingSystem::new(gfx_queue.clone(), lighting_subpass.clone());
        let directional_lighting_system =
            DirectionalLightingSystem::new(gfx_queue.clone(), lighting_subpass.clone());
        let point_lighting_system = PointLightingSystem::new(gfx_queue.clone(), lighting_subpass);

        (
            render_pass,
            diffuse_buffer,
            normals_buffer,
            depth_buffer,
            ambient_lighting_system,
            directional_lighting_system,
            point_lighting_system,
        )
    }

    pub fn new(
        gfx_queue: Arc<Queue>,
        final_output_format: Format,
        dimensions: [u32; 2],
    ) -> FrameSystem {
        let (
            render_pass,
            diffuse_buffer,
            normals_buffer,
            depth_buffer,
            ambient_lighting_system,
            directional_lighting_system,
            point_lighting_system,
        ) = Self::create_everything(&gfx_queue, final_output_format, dimensions);

        FrameSystem {
            gfx_queue,
            render_pass,
            diffuse_buffer,
            normals_buffer,
            depth_buffer,
            ambient_lighting_system,
            directional_lighting_system,
            point_lighting_system,
        }
    }

    pub fn recreate_render_pass(&mut self, final_output_format: Format, dimensions: [u32; 2]) {
        let (
            render_pass,
            diffuse_buffer,
            normals_buffer,
            depth_buffer,
            ambient_lighting_system,
            directional_lighting_system,
            point_lighting_system,
        ) = Self::create_everything(&self.gfx_queue, final_output_format, dimensions);

        self.render_pass = render_pass;
        self.diffuse_buffer = diffuse_buffer;
        self.normals_buffer = normals_buffer;
        self.depth_buffer = depth_buffer;
        self.ambient_lighting_system = ambient_lighting_system;
        self.directional_lighting_system = directional_lighting_system;
        self.point_lighting_system = point_lighting_system;
    }

    /// Returns the subpass of the render pass where the rendering should write info to gbuffers.
    ///
    /// Has two outputs: the diffuse color (3 components) and the normals in world coordinates
    /// (3 components). Also has a depth attachment.
    ///
    /// This method is necessary in order to initialize the pipelines that will draw the objects
    /// of the scene.
    #[inline]
    pub fn deferred_subpass(&self) -> Subpass<Arc<dyn RenderPassAbstract + Send + Sync>> {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    /// Starts drawing a new frame.
    ///
    /// - `before_future` is the future after which the main rendering should be executed.
    /// - `final_image` is the image we are going to draw to.
    /// - `world_to_framebuffer` is the matrix that will be used to convert from 3D coordinates in
    ///   the world into 2D coordinates on the framebuffer.
    ///
    pub fn frame<F, I>(
        &mut self,
        before_future: F,
        final_image: I,
        matrices: CameraMatrices,
        cached_scene: CachedEntities,
        dynamic_state: DynamicState,
    ) -> Frame
    where
        F: GpuFuture + 'static,
        I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static,
    {
        // First of all we recreate `self.diffuse_buffer`, `self.normals_buffer` and
        // `self.depth_buffer` if their dimensions doesn't match the dimensions of the final image.

        let img_dims = ImageAccess::dimensions(&final_image).width_height();
        if ImageAccess::dimensions(&self.diffuse_buffer).width_height() != img_dims {
            // TODO: use shortcut provided in vulkano 0.6
            let atch_usage = ImageUsage {
                transient_attachment: true,
                input_attachment: true,
                ..ImageUsage::none()
            };

            // Note that we create "transient" images here. This means that the content of the
            // image is only defined when within a render pass. In other words you can draw to
            // them in a subpass then read them in another subpass, but as soon as you leave the
            // render pass their content becomes undefined.
            self.diffuse_buffer = AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                img_dims,
                Format::A2B10G10R10UnormPack32,
                atch_usage,
            )
            .unwrap();
            self.normals_buffer = AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                img_dims,
                Format::R16G16B16A16Sfloat,
                atch_usage,
            )
            .unwrap();
            self.depth_buffer = AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                img_dims,
                Format::D16Unorm,
                atch_usage,
            )
            .unwrap();
        }

        // Build the framebuffer. The image must be attached in the same order as they were defined
        // with the `ordered_passes_renderpass!` macro.
        let framebuffer = Arc::new(
            Framebuffer::start(self.render_pass.clone())
                .add(final_image)
                .unwrap()
                .add(self.diffuse_buffer.clone())
                .unwrap()
                .add(self.normals_buffer.clone())
                .unwrap()
                .add(self.depth_buffer.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        // Start the command buffer builder that will be filled throughout the frame handling.
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
        )
        .unwrap();
        command_buffer_builder
            .begin_render_pass(
                framebuffer.clone(),
                SubpassContents::SecondaryCommandBuffers,
                vec![
                    ClearValue::Float([0.0, 0.0, 0.0, 0.0]),
                    ClearValue::Float([0.0, 0.0, 0.0, 0.0]),
                    ClearValue::Float([0.0, 0.0, 0.0, 0.0]),
                    ClearValue::Depth(1.0),
                ],
            )
            .unwrap();

        Frame {
            system: self,
            before_main_cb_future: Some(Box::new(before_future)),
            framebuffer,
            num_pass: 0,
            command_buffer_builder: Some(command_buffer_builder),
            matrices,
            cached_scene,
            dynamic_state,
        }
    }
}
