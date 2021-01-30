use crate::engine::cache::CachedEntities;
use crate::figure::PerVerexParams;
use crate::frame::system::FrameSystem;
use crate::scene::camera::CameraMatrices;
use crate::scene::lights::Light;
use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::SubpassContents;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sync::GpuFuture;

pub type ConcreteGraphicsPipeline = GraphicsPipeline<
    SingleBufferDefinition<PerVerexParams>,
    Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>,
    Arc<dyn RenderPassAbstract + Send + Sync + 'static>,
>;

/// Represents the active process of rendering a frame.
///
/// This struct mutably borrows the `FrameSystem`.
pub struct Frame<'a> {
    // The `FrameSystem`.
    pub system: &'a mut FrameSystem,

    // The active pass we are in. This keeps track of the step we are in.
    // - If `num_pass` is 0, then we haven't start anything yet.
    // - If `num_pass` is 1, then we have finished drawing all the objects of the scene.
    // - If `num_pass` is 2, then we have finished applying lighting.
    // - Otherwise the frame is finished.
    // In a more complex application you can have dozens of passes, in which case you probably
    // don't want to document them all here.
    pub num_pass: u8,

    // Future to wait upon before the main rendering.
    pub before_main_cb_future: Option<Box<dyn GpuFuture>>,
    // Framebuffer that was used when starting the render pass.
    #[allow(dead_code)]
    pub framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
    // The command buffer builder that will be built during the lifetime of this object.
    pub command_buffer_builder: Option<AutoCommandBufferBuilder>,

    pub matrices: CameraMatrices,
    pub lights: Vec<Light>,
    pub cached_scene: CachedEntities,
    pub dynamic_state: DynamicState,
}

impl<'a> Frame<'a> {
    /// Returns an enumeration containing the next pass of the rendering.
    pub fn next_pass<'f>(&'f mut self) -> Option<Pass<'f, 'a>> {
        // This function reads `num_pass` increments its value, and returns a struct corresponding
        // to that pass that the user will be able to manipulate in order to customize the pass.
        match {
            let current_pass = self.num_pass;
            self.num_pass += 1;
            current_pass
        } {
            0 => {
                // If we are in the pass 0 then we haven't start anything yet.
                // We already called `begin_render_pass` (in the `frame()` method), and that's the
                // state we are in.
                // We return an object that will allow the user to draw objects on the scene.
                Some(Pass::Deferred(DrawPass { frame: self }))
            }

            1 => {
                // If we are in pass 1 then we have finished drawing the objects on the scene.
                // Going to the next subpass.
                self.command_buffer_builder
                    .as_mut()
                    .unwrap()
                    .next_subpass(SubpassContents::SecondaryCommandBuffers)
                    .unwrap();

                // And returning an object that will allow the user to apply lighting to the scene.
                Some(Pass::Lighting(LightingPass { frame: self }))
            }

            2 => {
                // If we are in pass 2 then we have finished applying lighting.
                // We take the builder, call `end_render_pass()`, and then `build()` it to obtain
                // an actual command buffer.
                self.command_buffer_builder
                    .as_mut()
                    .unwrap()
                    .end_render_pass()
                    .unwrap();
                let command_buffer = self.command_buffer_builder.take().unwrap().build().unwrap();

                // Extract `before_main_cb_future` and append the command buffer execution to it.
                let after_main_cb = self
                    .before_main_cb_future
                    .take()
                    .unwrap()
                    .then_execute(self.system.gfx_queue.clone(), command_buffer)
                    .unwrap();
                // We obtain `after_main_cb`, which we give to the user.
                Some(Pass::Finished(Box::new(after_main_cb)))
            }

            // If the pass is over 2 then the frame is in the finished state and can't do anything
            // more.
            _ => None,
        }
    }
}

/// Struct provided to the user that allows them to customize or handle the pass.
pub enum Pass<'f, 's: 'f> {
    /// We are in the pass where we draw objects on the scene. The `DrawPass` allows the user to
    /// draw the objects.
    Deferred(DrawPass<'f, 's>),

    /// We are in the pass where we add lighting to the scene. The `LightingPass` allows the user
    /// to add light sources.
    Lighting(LightingPass<'f, 's>),

    /// The frame has been fully prepared, and here is the future that will perform the drawing
    /// on the image.
    Finished(Box<dyn GpuFuture>),
}

/// Allows the user to draw objects on the scene.
pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
    /// Appends a command that executes a secondary command buffer that performs drawing.
    #[inline]
    pub fn execute<C>(&mut self, command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        // Note that vulkano doesn't perform any safety check for now when executing secondary
        // command buffers, hence why it is unsafe. This operation will be safe in the future
        // however.
        // TODO: ^
        unsafe {
            self.frame
                .command_buffer_builder
                .as_mut()
                .unwrap()
                .execute_commands(command_buffer)
                .unwrap();
        }
    }
}

/// Allows the user to apply lighting on the scene.
pub struct LightingPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> LightingPass<'f, 's> {
    /// Applies an ambient lighting to the scene.
    ///
    /// All the objects will be colored with an intensity of `color`.

    /// Applies a spot lighting to the scene.
    ///
    /// All the objects will be colored with an intensity varying between `[0, 0, 0]` and `color`,
    /// depending on their distance with `position`. Objects that aren't facing `position` won't
    /// receive any light.
    pub fn light(&mut self, color_debug_level: i32) {
        // Note that vulkano doesn't perform any safety check for now when executing secondary
        // command buffers, hence why it is unsafe. This operation will be safe in the future
        // however.
        // TODO: ^
        unsafe {
            let command_buffer = {
                self.frame.system.lighting_system.draw(
                    self.frame.system.position_buffer.clone(),
                    self.frame.system.normals_buffer.clone(),
                    self.frame.system.albedo_buffer.clone(),
                    // self.frame.system.depth_buffer.clone(),
                    &self.frame.lights,
                    &self.frame.matrices,
                    &self.frame.cached_scene,
                    &self.frame.dynamic_state,
                    false,
                    color_debug_level,
                )
            };

            self.frame
                .command_buffer_builder
                .as_mut()
                .unwrap()
                .execute_commands(command_buffer)
                .unwrap();
        }
    }
}
