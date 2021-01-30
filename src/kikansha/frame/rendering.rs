use std::sync::Arc;
use vulkano::device::Queue;
use vulkano::format::ClearValue;
use vulkano::format::Format;
use vulkano::framebuffer::RenderPass;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::{
    AttachmentDescription, LoadOp, PassDependencyDescription, PassDescription, RenderPassDesc,
    RenderPassDescClearValues, StoreOp,
};
use vulkano::image::ImageLayout;
use vulkano::sync::AccessFlagBits;
use vulkano::sync::PipelineStages;

struct DefaultRenderPassDesc {
    pub attachments: Vec<AttachmentDescription>,
    pub subpasses: Vec<PassDescription>,
    pub dependencies: Vec<PassDependencyDescription>,
}

unsafe impl RenderPassDesc for DefaultRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        self.attachments.len()
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<AttachmentDescription> {
        if id < self.attachments.len() {
            return Some(self.attachments[id].clone());
        }

        return None;
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        self.subpasses.len()
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<PassDescription> {
        if id < self.subpasses.len() {
            return Some(self.subpasses[id].clone());
        }
        return None;
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        self.dependencies.len()
    }

    #[inline]
    fn dependency_desc(&self, id: usize) -> Option<PassDependencyDescription> {
        if id < self.dependencies.len() {
            return Some(self.dependencies[id].clone());
        }
        return None;
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for DefaultRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        // FIXME: safety checks
        Box::new(values.into_iter())
    }
}

pub fn build_render_pass(
    gfx_queue: &Arc<Queue>,
    final_output_format: Format,
) -> Arc<dyn RenderPassAbstract + Send + Sync> {
    let render_pass_description = {
        let mut attachments = Vec::new();

        // 0: Position
        attachments.push(AttachmentDescription {
            format: Format::R16G16B16A16Sfloat,
            samples: 1,
            load: LoadOp::Clear,
            store: StoreOp::DontCare,
            stencil_load: LoadOp::DontCare,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        // 1: Normal
        attachments.push(AttachmentDescription {
            format: Format::R16G16B16A16Sfloat,
            samples: 1,
            load: LoadOp::Clear,
            store: StoreOp::DontCare,
            stencil_load: LoadOp::DontCare,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        // 2: Albedo
        attachments.push(AttachmentDescription {
            format: Format::R8G8B8A8Unorm,
            samples: 1,
            load: LoadOp::Clear,
            store: StoreOp::DontCare,
            stencil_load: LoadOp::DontCare,
            stencil_store: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::ColorAttachmentOptimal,
        });

        // 3: Final Color
        attachments.push(AttachmentDescription {
            format: final_output_format,
            samples: 1,
            load: LoadOp::Clear,
            store: StoreOp::Store,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::PresentSrc,
        });

        // 4: Depth
        attachments.push(AttachmentDescription {
            format: Format::D16Unorm,
            samples: 1,
            load: LoadOp::Clear,
            store: StoreOp::Store,
            stencil_load: LoadOp::Clear,
            stencil_store: StoreOp::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::DepthStencilAttachmentOptimal,
        });

        let mut subpasses = Vec::new();
        // Deferred rendering to  GBuffer
        subpasses.push(PassDescription {
            color_attachments: vec![
                (0, ImageLayout::ColorAttachmentOptimal), // Position
                (1, ImageLayout::ColorAttachmentOptimal), // Normal
                (2, ImageLayout::ColorAttachmentOptimal), // Albedo
            ],
            depth_stencil: Some((4, ImageLayout::DepthStencilAttachmentOptimal)),
            input_attachments: Vec::new(),
            resolve_attachments: Vec::new(),
            preserve_attachments: Vec::new(),
        });

        // Composition
        subpasses.push(PassDescription {
            color_attachments: vec![(3, ImageLayout::ColorAttachmentOptimal)],
            depth_stencil: None,
            input_attachments: vec![
                (0, ImageLayout::ShaderReadOnlyOptimal), // Position
                (1, ImageLayout::ShaderReadOnlyOptimal), // Normal
                (2, ImageLayout::ShaderReadOnlyOptimal), // Albedo
            ],
            resolve_attachments: Vec::new(),
            preserve_attachments: Vec::new(),
        });

        let mut dependencies = Vec::new();

        // ? -> deferred
        dependencies.push(PassDependencyDescription {
            source_subpass: vk::SUBPASS_EXTERNAL as usize,
            destination_subpass: 0,
            source_stages: PipelineStages {
                bottom_of_pipe: true,
                ..PipelineStages::none()
            },
            destination_stages: PipelineStages {
                color_attachment_output: true,
                ..PipelineStages::none()
            },
            source_access: AccessFlagBits {
                memory_read: true,
                ..AccessFlagBits::none()
            },
            destination_access: AccessFlagBits {
                color_attachment_read: true,
                color_attachment_write: true,
                ..AccessFlagBits::none()
            },
            by_region: true,
        });

        // Deferred -> composition
        dependencies.push(PassDependencyDescription {
            source_subpass: 0,
            destination_subpass: 1,
            source_stages: PipelineStages {
                color_attachment_output: true,
                ..PipelineStages::none()
            },
            destination_stages: PipelineStages {
                fragment_shader: true,
                ..PipelineStages::none()
            },
            source_access: AccessFlagBits {
                color_attachment_write: true,
                ..AccessFlagBits::none()
            },
            destination_access: AccessFlagBits {
                shader_read: true,
                ..AccessFlagBits::none()
            },
            by_region: true,
        });

        dependencies.push(PassDependencyDescription {
            source_subpass: 0,
            destination_subpass: vk::SUBPASS_EXTERNAL as usize,
            source_stages: PipelineStages {
                color_attachment_output: true,
                ..PipelineStages::none()
            },
            destination_stages: PipelineStages {
                bottom_of_pipe: true,
                ..PipelineStages::none()
            },
            source_access: AccessFlagBits {
                color_attachment_read: true,
                color_attachment_write: true,
                ..AccessFlagBits::none()
            },
            destination_access: AccessFlagBits {
                memory_read: true,
                ..AccessFlagBits::none()
            },
            by_region: true,
        });

        DefaultRenderPassDesc {
            attachments,
            subpasses,
            dependencies,
        }
    };

    Arc::new(RenderPass::new(gfx_queue.device().clone(), render_pass_description).unwrap())
}
