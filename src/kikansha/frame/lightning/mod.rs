use crate::engine::cache::{CachedEntities, CachedEntity};
use crate::frame::frame::ConcreteGraphicsPipeline;
use crate::scene::camera::CameraMatrices;
use crate::scene::lights::Light;
use crate::scene::lights::PointLight;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSetBuf;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSetImg;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSetSampler;
use vulkano::image::AttachmentImage;
use vulkano::image::SwapchainImage;
use winit::window::Window;

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

// type FullImage = ImageViewAccess + Send + Sync + Clone + 'static;

type PDS = PersistentDescriptorSet<(
    (
        (
            (
                (
                    (
                        ((), PersistentDescriptorSetImg<Arc<AttachmentImage>>),
                        PersistentDescriptorSetSampler,
                    ),
                    PersistentDescriptorSetImg<Arc<AttachmentImage>>,
                ),
                PersistentDescriptorSetSampler,
            ),
            PersistentDescriptorSetImg<Arc<AttachmentImage>>,
        ),
        PersistentDescriptorSetSampler,
    ),
    PersistentDescriptorSetBuf<Arc<CpuAccessibleBuffer<fs::ty::UBO>>>,
)>;

/// Allows applying a directional light source to a scene.
pub struct LightingSystem {
    gfx_queue: Arc<Queue>,
    pipeline: Arc<ConcreteGraphicsPipeline>,
    default_sampler: Arc<Sampler>,
    buff: Arc<CpuAccessibleBuffer<fs::ty::UBO>>,
    set: Arc<PDS>,
}

impl LightingSystem {
    /// Initializes the directional lighting system.
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass<Arc<dyn RenderPassAbstract + Send + Sync + 'static>>,
        position_buffer: Arc<AttachmentImage>,
        normals_input: Arc<AttachmentImage>,
        albedo_input: Arc<AttachmentImage>,
        color_debug_level: i32,
    ) -> LightingSystem {
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

        let default_lights = PointLight::default_lights();
        let mut packed_lights = Vec::new();

        for l in default_lights {
            match l {
                Light::Point(pl) => packed_lights.push(fs::ty::Light {
                    position: pl.position.into(),
                    color: pl.color.into(),
                    radius: pl.radius,
                }),
            };
        }

        let push_constants = fs::ty::UBO {
            lights: packed_lights.try_into().unwrap(),
            viewPos: [1.0, 1.0, 1.0, 0.0],
            displayDebugTarget: color_debug_level,
        };

        let buff = CpuAccessibleBuffer::from_data(
            pipeline.device().clone(),
            BufferUsage::uniform_buffer(),
            false,
            push_constants,
        )
        .unwrap();

        let layout = pipeline.layout().descriptor_set_layout(0).unwrap();

        let set = Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_empty()
                .unwrap()
                .add_sampled_image(position_buffer, default_sampler.clone())
                .unwrap()
                .add_sampled_image(normals_input, default_sampler.clone())
                .unwrap()
                .add_sampled_image(albedo_input, default_sampler.clone())
                .unwrap()
                .add_buffer(buff.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        LightingSystem {
            gfx_queue,
            pipeline,
            default_sampler,
            buff,
            set,
        }
    }

    pub fn draw(
        &self,
        albedo_input: Arc<AttachmentImage>,
        normals_input: Arc<AttachmentImage>,
        depth_input: Arc<AttachmentImage>,
        lights: &[Light],
        matrices_buff: &CameraMatrices,
        cached_scene: &CachedEntities,
        dynamic_state: &DynamicState,
        update_images_pds: bool,
        color_debug_level: i32,
    ) -> AutoCommandBuffer {
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
            let mut content = self.buff.write().unwrap();
            content.lights = packed_lights.try_into().unwrap();
            content.viewPos = view_pos;
        }

        let mut builder = AutoCommandBufferBuilder::secondary_graphics_one_time_submit(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        let cached_entity = &cached_scene.entities[0];
        // for cached_entity in cached_scene.entities.clone() {
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
        // }

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
