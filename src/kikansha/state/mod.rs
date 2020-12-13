mod q;

use crate::debug::fps::Counter;
use crate::figure::PerIndicesParams;
use crate::figure::PerVerexParams;
use crate::scene::camera::ViewAndProject;
use crate::scene::Scene;
use crate::state::q::QueueFamilyIndices;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Mutex;
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;

use std::collections::HashSet;
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::AttachmentImage;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::{
    debug::DebugCallback, debug::MessageSeverity, debug::MessageType, layers_list, ApplicationInfo,
    Instance, InstanceExtensions, PhysicalDevice, Version,
};
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::{viewport::Viewport, GraphicsPipeline};
use vulkano::swapchain;
use vulkano::swapchain::{AcquireError, SwapchainCreationError};
use vulkano::swapchain::{
    Capabilities, ColorSpace, FullscreenExclusive, PresentMode, SupportedPresentModes, Surface,
    Swapchain,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture, SharingMode};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::platform::desktop::EventLoopExtDesktop;
use winit::window::Window;
use winit::window::WindowBuilder;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/ressha/triangle.vert"
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/ressha/triangle.frag"
    }
}

/// Required device extensions
fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        khr_storage_buffer_storage_class: true,
        ..DeviceExtensions::none()
    }
}

type ConcreteGraphicsPipeline = GraphicsPipeline<
    SingleBufferDefinition<PerVerexParams>,
    Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>,
    Arc<dyn RenderPassAbstract + Send + Sync + 'static>,
>;

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

const VALIDATION_LAYERS: &[&str] = &[
    "VK_LAYER_KHRONOS_validation",
    // "VK_LAYER_LUNARG_api_dump"
];

pub struct State {
    instance: Arc<Instance>,
    debug_callback: Option<DebugCallback>,
    physical_device_index: usize,
    pub device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
    pub surface: Arc<Surface<Window>>,
    pub swap_chain: Arc<Swapchain<Window>>,
    pub swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<ConcreteGraphicsPipeline>,
    swap_chain_framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    command_buffers: Vec<Arc<AutoCommandBuffer>>,
    pub dynamic_state: Mutex<DynamicState>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swap_chain: bool,
}

impl State {
    fn init<T: ViewAndProject + Sized>(
        scene: &Scene<T>,
        surface: Arc<Surface<Window>>,
        instance: Arc<Instance>,
    ) -> Self {
        let debug_callback = Self::setup_debug_callback(&instance);

        let physical_device_index = Self::pick_physical_device(&instance, &surface);
        let (device, graphics_queue, present_queue) =
            Self::create_logical_device(physical_device_index, &instance, &surface);
        let (swap_chain, swap_chain_images) = Self::create_swap_chain(
            &instance,
            &surface,
            physical_device_index,
            &device,
            &graphics_queue,
            &present_queue,
            None,
        );

        let render_pass = Self::create_render_pass(&device, swap_chain.format());

        let dynamic_state_raw = DynamicState::none();

        let dynamic_state = Mutex::new(dynamic_state_raw);

        let graphics_pipeline = Self::create_graphic_pipeline(
            &device,
            swap_chain.dimensions(),
            &render_pass,
            &dynamic_state,
        );

        let swap_chain_framebuffers =
            Self::create_framebuffers(&swap_chain_images, &render_pass, &device);
        // let instance = Some(instance_unb);

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        let mut app = State {
            instance,
            debug_callback,
            physical_device_index,
            device,
            graphics_queue,
            present_queue,
            surface,
            swap_chain,
            swap_chain_images,
            render_pass,
            graphics_pipeline,
            swap_chain_framebuffers,
            command_buffers: vec![],
            dynamic_state,
            previous_frame_end,
            recreate_swap_chain: false,
        };
        app.create_command_buffers(scene);
        app
    }

    pub fn update_swapchain(&mut self) {
        let dimensions: [u32; 2] = self.surface.window().inner_size().into();

        let (new_swapchain, _new_images) =
            match self.swap_chain.recreate_with_dimensions(dimensions) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        self.swap_chain = new_swapchain;
    }

    fn init_loop(instance: &Arc<Instance>) -> (EventLoop<()>, Arc<Surface<Window>>) {
        let events_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_title("Vulkan")
            .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
            .build_vk_surface(&events_loop, instance.clone())
            .expect("Failed to create window surface");
        (events_loop, surface)
    }

    fn create_instance() -> Arc<Instance> {
        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support() {
            println!("Validation layers requested, but not available!")
        }
        let supported_extensions = InstanceExtensions::supported_by_core()
            .expect("failed to retrieve supported extensions");
        let layers: Vec<_> = layers_list()
            .unwrap()
            .map(|l| l.name().to_owned())
            .collect();

        println!("Supported core extensions: {:?}", supported_extensions);
        println!("Supported extensions: {:?}", layers);
        let app_info = ApplicationInfo {
            application_name: Some("Hello Triangle".into()),
            application_version: Some(Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
            engine_name: Some("No Engine".into()),
            engine_version: Some(Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
        };
        let required_extensions = Self::get_required_extensions();
        if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            Instance::new(
                Some(&app_info),
                &required_extensions,
                VALIDATION_LAYERS.iter().cloned(),
            )
            .expect("failed to create Vulkan instance")
        } else {
            Instance::new(Some(&app_info), &required_extensions, None)
                .expect("failed to create Vulkan instance")
        }
    }

    fn pick_physical_device(instance: &Arc<Instance>, surface: &Arc<Surface<Window>>) -> usize {
        PhysicalDevice::enumerate(&instance)
            .position(|device| Self::is_device_suitable(&device, surface))
            .expect("failed to find a suitable GPU!")
    }

    fn create_logical_device(
        physical_device_idx: usize,
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
    ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        let physical_device = PhysicalDevice::from_index(instance, physical_device_idx).unwrap();
        let indices = Self::find_queue_families(surface, &physical_device);

        let families = [indices.graphics_family, indices.present_family];

        use std::iter::FromIterator;

        let uniquer_queue_family: HashSet<&i32> = HashSet::from_iter(families.iter());
        let queue_priority = 1.0;

        let queue_families = uniquer_queue_family.iter().map(|i| {
            (
                physical_device.queue_families().nth(**i as usize).unwrap(),
                queue_priority,
            )
        });

        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &device_extensions(),
            queue_families,
        )
        .expect("Failed to create logical device");
        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
        (device, graphics_queue, present_queue)
    }

    fn check_device_support_extension(device: &PhysicalDevice) -> bool {
        let available_extensions = DeviceExtensions::supported_by_device(*device);
        let device_extensions = device_extensions();
        available_extensions.intersection(&device_extensions) == device_extensions
    }

    fn is_device_suitable(device: &PhysicalDevice, surface: &Arc<Surface<Window>>) -> bool {
        let indices = Self::find_queue_families(surface, device);
        let extension_supported = Self::check_device_support_extension(device);
        let swap_chain_adequate = if extension_supported {
            let capabilities = surface
                .capabilities(*device)
                .expect("Failes to get surface capabilities");
            !capabilities.supported_formats.is_empty()
                && capabilities.present_modes.iter().next().is_some()
        } else {
            false
        };
        indices.is_complete() && extension_supported && swap_chain_adequate
    }

    fn find_queue_families(
        surface: &Arc<Surface<Window>>,
        device: &PhysicalDevice,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::new();
        // TODO: replace index with id to simplify?
        for (i, queue_family) in device.queue_families().enumerate() {
            if queue_family.supports_graphics() {
                indices.graphics_family = i as i32;
            }

            if surface.is_supported(queue_family).unwrap() {
                indices.present_family = i as i32;
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    fn create_swap_chain(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_index: usize,
        device: &Arc<Device>,
        graphics_queue: &Arc<Queue>,
        present_queue: &Arc<Queue>,
        old_swapchain: Option<Arc<Swapchain<Window>>>,
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let physical_device = PhysicalDevice::from_index(&instance, physical_device_index).unwrap();
        let capabilities = surface
            .capabilities(physical_device)
            .expect("Failed to get surface capabilities");

        let surface_format = Self::choose_swap_surface_format(&capabilities.supported_formats);
        let present_mode = Self::choose_swap_present_mode(capabilities.present_modes);
        let extent = Self::choose_swap_extent(&capabilities);

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count.is_some()
            && image_count > capabilities.max_image_count.unwrap()
        {
            image_count = capabilities.max_image_count.unwrap();
        }

        let alpha = capabilities
            .supported_composite_alpha
            .iter()
            .next()
            .unwrap();

        let image_usage = ImageUsage {
            color_attachment: true,
            ..ImageUsage::none()
        };

        let indicies = Self::find_queue_families(&surface, &physical_device);

        let sharing: SharingMode = if indicies.graphics_family != indicies.present_family {
            vec![graphics_queue, present_queue].as_slice().into()
        } else {
            graphics_queue.into()
        };

        match old_swapchain {
            Some(old) => Swapchain::with_old_swapchain(
                device.clone(),
                surface.clone(),
                image_count,
                surface_format.0,
                extent,
                1,
                image_usage,
                sharing,
                capabilities.current_transform,
                alpha,
                present_mode,
                FullscreenExclusive::Default,
                true,
                ColorSpace::SrgbNonLinear,
                old,
            )
            .expect("Failed to create swap chain"),
            None => Swapchain::new(
                device.clone(),
                surface.clone(),
                image_count,
                surface_format.0,
                extent,
                1,
                image_usage,
                sharing,
                capabilities.current_transform,
                alpha,
                present_mode,
                FullscreenExclusive::Default,
                true,
                ColorSpace::SrgbNonLinear,
            )
            .expect("Failed to create swap chain"),
        }
    }

    fn choose_swap_surface_format(
        available_formats: &[(Format, ColorSpace)],
    ) -> (Format, ColorSpace) {
        // NOTE: the 'preferred format' mentioned in the tutorial doesn't seem to be
        // queryable in Vulkano (no VK_FORMAT_UNDEFINED enum)
        *available_formats
            .iter()
            .find(|(format, color_space)| {
                *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
            })
            .unwrap_or_else(|| &available_formats[0])
    }

    fn choose_swap_present_mode(available_present_modes: SupportedPresentModes) -> PresentMode {
        if available_present_modes.mailbox {
            PresentMode::Mailbox
        } else if available_present_modes.immediate {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo
        }
    }

    fn choose_swap_extent(capabilities: &Capabilities) -> [u32; 2] {
        if let Some(current_extent) = capabilities.current_extent {
            return current_extent;
        } else {
            let mut actual_extent = [WIDTH, HEIGHT];
            actual_extent[0] = capabilities.min_image_extent[0]
                .max(capabilities.max_image_extent[0].min(actual_extent[0]));
            actual_extent[1] = capabilities.min_image_extent[1]
                .max(capabilities.max_image_extent[1].min(actual_extent[1]));
            actual_extent
        }
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = layers_list()
            .unwrap()
            .map(|l| l.name().to_owned())
            .collect();
        VALIDATION_LAYERS
            .iter()
            .all(|layer_name| layers.contains(&layer_name.to_string()))
    }
    fn get_required_extensions() -> InstanceExtensions {
        let mut extensions = vulkano_win::required_extensions();
        if ENABLE_VALIDATION_LAYERS {
            // TODO!: this should be ext_debug_utils (_report is deprecated), but that doesn't exist yet in vulkano
            extensions.ext_debug_utils = true;
        }
        extensions
    }
    fn setup_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }
        let msg_types = MessageType::all();

        let msg_severity = MessageSeverity {
            error: true,
            warning: true,
            information: true,
            verbose: true,
        };
        DebugCallback::new(&instance, msg_severity, msg_types, |msg| {
            println!("validation layer: {:?}", msg.description);
        })
        .ok()
    }

    fn create_render_pass(
        device: &Arc<Device>,
        color_format: Format,
    ) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        Arc::new(
            vulkano::single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: color_format,
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            )
            .unwrap(),
        )
    }

    fn create_graphic_pipeline(
        device: &Arc<Device>,
        swap_chain_extent: [u32; 2],
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &Mutex<DynamicState>,
    ) -> Arc<ConcreteGraphicsPipeline> {
        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("failed to create vertex shader module!");
        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("failed to create fragment shader module!");

        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0..1.0,
        };
        {
            dynamic_state.lock().unwrap().viewports = Some(vec![viewport]);
        };

        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer()
                .vertex_shader(vert_shader_module.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(frag_shader_module.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        )
    }

    fn create_framebuffers(
        swap_chain_images: &[Arc<SwapchainImage<Window>>],
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
        device: &Arc<Device>,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = swap_chain_images[0].dimensions();

        let depth_buffer =
            AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

        swap_chain_images
            .iter()
            .map(|image| {
                let fba: Arc<dyn FramebufferAbstract + Send + Sync> = Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(depth_buffer.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                );
                fba
            })
            .collect::<Vec<_>>()
    }

    pub fn create_command_buffers<T: ViewAndProject + Sized>(&mut self, scene: &Scene<T>) {
        let queue_family = self.graphics_queue.family();

        let matrices = {
            let locked_camera = scene.camera.lock().unwrap();
            locked_camera.get_matrices().clone()
        };
        self.command_buffers = self
            .swap_chain_framebuffers
            .iter()
            .map(|framebuffer| {
                let layout = self
                    .graphics_pipeline
                    .layout()
                    .descriptor_set_layout(0)
                    .unwrap();

                let matrices_buff = CpuAccessibleBuffer::from_data(
                    self.device.clone(),
                    BufferUsage::all(),
                    false,
                    matrices,
                )
                .unwrap();

                let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
                    self.device.clone(),
                    queue_family,
                )
                .unwrap();

                let dynamic_state = { self.dynamic_state.lock().unwrap().clone() };
                let passes = builder
                    .begin_render_pass(
                        framebuffer.clone(),
                        false,
                        vec![[1.0, 1.0, 1.0, 1.0].into(), 1f32.into()],
                    )
                    .unwrap();

                let pep = scene
                    .figures
                    .clone()
                    .into_iter()
                    .fold(passes, |b, figure_set| {
                        let per_vertex_data: Vec<PerVerexParams> = figure_set
                            .figure
                            .vertices
                            .clone()
                            .into_iter()
                            .map(|v| PerVerexParams {
                                position: v.position,
                                color: figure_set.figure.base_color,
                            })
                            .collect();

                        let ver_buff = CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            false,
                            per_vertex_data.into_iter(),
                        )
                        .unwrap();
                        let indices_buff = CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            false,
                            figure_set.figure.indices.into_iter(),
                        )
                        .unwrap();

                        figure_set.mutations.into_iter().fold(b, |acc, mutation| {
                            let mutation_buff = CpuAccessibleBuffer::from_data(
                                self.device.clone(),
                                BufferUsage::all(),
                                false,
                                mutation,
                            )
                            .unwrap();

                            let set = PersistentDescriptorSet::start(layout.clone())
                                .add_buffer(matrices_buff.clone())
                                .unwrap()
                                .add_buffer(mutation_buff.clone())
                                .unwrap()
                                .build()
                                .unwrap();

                            acc.draw_indexed(
                                self.graphics_pipeline.clone(),
                                &dynamic_state,
                                ver_buff.clone(),
                                indices_buff.clone(),
                                set,
                                (),
                            )
                            .unwrap()
                        })
                    });

                pep.end_render_pass().unwrap();

                Arc::new(builder.build().unwrap())
            })
            .collect();
    }

    pub fn update_size_dependent(&mut self) {
        let images = self.swap_chain_images.clone();
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };

        {
            let mut guard = self.dynamic_state.lock().unwrap();
            guard.viewports = Some(vec![viewport]);
        }

        let depth_buffer =
            AttachmentImage::transient(self.device.clone(), dimensions, Format::D16Unorm).unwrap();
        let swap_chain_framebuffers = images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(self.render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(depth_buffer.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>();
        self.swap_chain_framebuffers = swap_chain_framebuffers;

        let new_pipeline = Self::create_graphic_pipeline(
            &self.device,
            dimensions,
            &self.render_pass,
            &self.dynamic_state,
        );
        self.graphics_pipeline = new_pipeline;
    }

    fn recreate_swap_chain<T: ViewAndProject + Sized>(&mut self, scene: &Scene<T>) {
        let (swap_chain, images) = Self::create_swap_chain(
            &self.instance,
            &self.surface,
            self.physical_device_index,
            &self.device,
            &self.graphics_queue,
            &self.present_queue,
            Some(self.swap_chain.clone()),
        );
        self.swap_chain = swap_chain;
        self.swap_chain_images = images;

        self.render_pass = Self::create_render_pass(&self.device, self.swap_chain.format());
        self.graphics_pipeline = Self::create_graphic_pipeline(
            &self.device,
            self.swap_chain.dimensions(),
            &self.render_pass,
            &self.dynamic_state,
        );
        self.swap_chain_framebuffers =
            Self::create_framebuffers(&self.swap_chain_images, &self.render_pass, &self.device);
        self.create_command_buffers(scene);
    }

    pub fn run_loop<T: ViewAndProject + Sized>(
        scene: &Scene<T>,
        _event_send: SyncSender<f32>,
        quit_recv: Receiver<bool>,
    ) {
        let instance_unb = Self::create_instance();
        let (mut event_loop, surface) = Self::init_loop(&instance_unb);
        let mut state = Self::init(scene, surface, instance_unb);

        let mut counter = Counter::new(1);

        {
            let dimensions = state.swap_chain_images[0].dimensions();
            let mut locked_camera = scene.camera.lock().unwrap();
            locked_camera.update_ar(dimensions[0] as f32 / dimensions[1] as f32);
        }

        event_loop.run_return(|event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                state.recreate_swap_chain = true;
            }
            Event::RedrawEventsCleared => {
                match quit_recv.try_recv() {
                    Ok(flag) => {
                        if flag {
                            *control_flow = ControlFlow::Exit
                        }
                    }
                    _ => (),
                }

                state
                    .previous_frame_end
                    .as_mut()
                    .unwrap()
                    .cleanup_finished();

                if state.recreate_swap_chain {
                    {
                        let mut locked_camera = scene.camera.lock().unwrap();
                        let dimensions: [u32; 2] = state.surface.window().inner_size().into();
                        locked_camera.update_ar(dimensions[0] as f32 / dimensions[1] as f32);
                    }
                    // state.update_size_dependent();
                    state.recreate_swap_chain(scene);
                    state.recreate_swap_chain = false;
                }

                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(state.swap_chain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            state.recreate_swap_chain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };
                if suboptimal {
                    state.recreate_swap_chain = true;
                }

                state.create_command_buffers(&scene);

                let command_buffer = state.command_buffers[image_num].clone();
                let future = state
                    .previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(state.graphics_queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        state.graphics_queue.clone(),
                        state.swap_chain.clone(),
                        image_num,
                    )
                    .then_signal_fence_and_flush();

                match counter.tick() {
                    Some(v) => println!("{:?} fps", v),
                    None => (),
                };

                match future {
                    Ok(future) => {
                        state.previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        state.recreate_swap_chain = true;
                        state.previous_frame_end = Some(sync::now(state.device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        state.previous_frame_end = Some(sync::now(state.device.clone()).boxed());
                    }
                }
            }
            _ => (),
        });
    }
}
