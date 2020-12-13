extern crate vulkano;


pub mod scene;
pub mod state;
pub mod debug;
pub mod figure;



// extern crate vulkano;
// extern crate vulkano_win;
// extern crate winit;

// use crate::vulkano::sync::GpuFuture;
// use crate::winit::platform::desktop::EventLoopExtDesktop;
// use std::collections::HashSet;
// use std::sync::Arc;
// use vulkano::command_buffer::AutoCommandBuffer;
// use vulkano::command_buffer::AutoCommandBufferBuilder;
// use vulkano::command_buffer::DynamicState;
// use vulkano::descriptor::PipelineLayoutAbstract;
// use vulkano::framebuffer::Framebuffer;
// use vulkano::framebuffer::FramebufferAbstract;
// use vulkano::framebuffer::Subpass;
// use vulkano::pipeline::vertex::BufferlessVertices;
// use vulkano::swapchain::acquire_next_image;

// use vulkano_win::VkSurfaceBuild;
// use winit::dpi::LogicalSize;

// use winit::event::{Event, WindowEvent};
// use winit::event_loop::EventLoop;
// use winit::window::Window;
// use winit::window::WindowBuilder;

// use winit::error::OsError;
// use winit::event_loop::ControlFlow;

// use vulkano::device::{Device, DeviceExtensions, Features, Queue};
// use vulkano::format::Format;
// use vulkano::image::{swapchain::SwapchainImage, ImageUsage};
// use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
// use vulkano::instance::{
//     layers_list, ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice, Version,
// };
// use vulkano::swapchain::{
//     Capabilities, ColorSpace, CompositeAlpha, FullscreenExclusive, PresentMode,
//     SupportedPresentModes, Surface, Swapchain,
// };

// use vulkano::pipeline::{vertex::BufferlessDefinition, viewport::Viewport, GraphicsPipeline};

// use vulkano::sync::SharingMode;

// use vulkano::framebuffer::RenderPassAbstract;

// const WIDTH: u32 = 800;
// const HEIGHT: u32 = 600;

// struct QueueFamilyIndices {
//     graphics_family: i32,
//     present_family: i32,
// }

// impl QueueFamilyIndices {
//     fn new() -> Self {
//         Self {
//             graphics_family: -1,
//             present_family: -1,
//         }
//     }

//     fn is_complete(&self) -> bool {
//         self.graphics_family >= 0 && self.present_family >= 0
//     }
// }

// /// Required device extensions
// fn device_extensions() -> DeviceExtensions {
//     DeviceExtensions {
//         khr_swapchain: true,
//         ..vulkano::device::DeviceExtensions::none()
//     }
// }

// type ConcreteGraphicsPipeline = GraphicsPipeline<
//     BufferlessDefinition,
//     Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>,
//     Arc<dyn RenderPassAbstract + Send + Sync + 'static>,
// >;

// #[cfg(debug_assertions)]
// const ENABLE_VALIDATION_LAYERS: bool = true;
// #[cfg(not(debug_assertions))]
// const ENABLE_VALIDATION_LAYERS: bool = false;

// const VALIDATION_LAYERS: &[&str] = &[
//     "VK_LAYER_KHRONOS_validation",
//     // "VK_LAYER_LUNARG_api_dump"
// ];

// struct ResshaState {
//     event_loop: EventLoop<()>,
//     instance: Option<Arc<Instance>>,
//     debug_callback: Option<DebugCallback>,
//     physical_device_index: usize,
//     device: Arc<Device>,
//     graphics_queue: Arc<Queue>,
//     present_queue: Arc<Queue>,
//     surface: Arc<Surface<Window>>,
//     swap_chain: Arc<Swapchain<Window>>,
//     swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
//     render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
//     graphics_pipeline: Arc<ConcreteGraphicsPipeline>,
//     swap_chain_framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
//     command_buffers: Vec<Arc<AutoCommandBuffer>>,
// }

// impl ResshaState {
//     pub fn init() -> Self {
//         let instance_unb = Self::create_instance();
//         let (event_loop, surface) = Self::init_loop(&instance_unb);
//         let debug_callback = Self::setup_debug_callback(&instance_unb);

//         let physical_device_index = Self::pick_physical_device(&instance_unb, &surface);
//         let (device, graphics_queue, present_queue) =
//             Self::create_logical_device(physical_device_index, &instance_unb, &surface);
//         let (swap_chain, swap_chain_images) = Self::create_swap_chain(
//             &instance_unb,
//             &surface,
//             physical_device_index,
//             &device,
//             &graphics_queue,
//             &present_queue,
//         );

//         let render_pass = Self::create_render_pass(&device, swap_chain.format());

//         let graphics_pipeline =
//             Self::create_graphic_pipeline(&device, swap_chain.dimensions(), &render_pass);
//         let swap_chain_framebuffers = Self::create_framebuffers(&swap_chain_images, &render_pass);
//         let instance = Some(instance_unb);

//         let mut app = Self {
//             event_loop,
//             instance,
//             debug_callback,
//             physical_device_index,
//             device,
//             graphics_queue,
//             present_queue,
//             surface,
//             swap_chain,
//             swap_chain_images,
//             render_pass,
//             graphics_pipeline,
//             swap_chain_framebuffers,
//             command_buffers: vec![],
//         };

//         app.create_command_buffers();
//         app
//     }

//     fn init_window(events_loop: &EventLoop<()>) -> Result<Window, OsError> {
//         WindowBuilder::new()
//             .with_title("Vulkan")
//             .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
//             .build(&events_loop)
//     }
//     fn init_loop(instance: &Arc<Instance>) -> (EventLoop<()>, Arc<Surface<Window>>) {
//         let events_loop = EventLoop::new();
//         let surface = WindowBuilder::new()
//             .with_title("Vulkan")
//             .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
//             .build_vk_surface(&events_loop, instance.clone())
//             .expect("Failed to create window surface");
//         (events_loop, surface)
//     }

//     fn create_instance() -> Arc<Instance> {
//         if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support() {
//             println!("Validation layers requested, but not available!")
//         }
//         let supported_extensions = InstanceExtensions::supported_by_core()
//             .expect("failed to retrieve supported extensions");
//         let layers: Vec<_> = layers_list()
//             .unwrap()
//             .map(|l| l.name().to_owned())
//             .collect();

//         println!("Supported core extensions: {:?}", supported_extensions);
//         println!("Supported extensions: {:?}", layers);
//         let app_info = ApplicationInfo {
//             application_name: Some("Hello Triangle".into()),
//             application_version: Some(Version {
//                 major: 1,
//                 minor: 0,
//                 patch: 0,
//             }),
//             engine_name: Some("No Engine".into()),
//             engine_version: Some(Version {
//                 major: 1,
//                 minor: 0,
//                 patch: 0,
//             }),
//         };
//         let required_extensions = Self::get_required_extensions();
//         if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
//             Instance::new(
//                 Some(&app_info),
//                 &required_extensions,
//                 VALIDATION_LAYERS.iter().cloned(),
//             )
//             .expect("failed to create Vulkan instance")
//         } else {
//             Instance::new(Some(&app_info), &required_extensions, None)
//                 .expect("failed to create Vulkan instance")
//         }
//     }

//     fn pick_physical_device(instance: &Arc<Instance>, surface: &Arc<Surface<Window>>) -> usize {
//         PhysicalDevice::enumerate(&instance)
//             .position(|device| Self::is_device_suitable(&device, surface))
//             .expect("failed to find a suitable GPU!")
//     }

//     fn create_logical_device(
//         physical_device_idx: usize,
//         instance: &Arc<Instance>,
//         surface: &Arc<Surface<Window>>,
//     ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
//         let physical_device = PhysicalDevice::from_index(instance, physical_device_idx).unwrap();
//         let indices = Self::find_queue_families(surface, &physical_device);

//         let families = [indices.graphics_family, indices.present_family];

//         use std::iter::FromIterator;

//         let uniquer_queue_family: HashSet<&i32> = HashSet::from_iter(families.iter());
//         let queue_priority = 1.0;

//         let queue_families = uniquer_queue_family.iter().map(|i| {
//             (
//                 physical_device.queue_families().nth(**i as usize).unwrap(),
//                 queue_priority,
//             )
//         });

//         let (device, mut queues) = Device::new(
//             physical_device,
//             &Features::none(),
//             &device_extensions(),
//             queue_families,
//         )
//         .expect("Failed to create logical device");
//         let graphics_queue = queues.next().unwrap();
//         let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
//         (device, graphics_queue, present_queue)
//     }

//     fn check_device_support_extension(device: &PhysicalDevice) -> bool {
//         let available_extensions = DeviceExtensions::supported_by_device(*device);
//         let device_extensions = device_extensions();
//         available_extensions.intersection(&device_extensions) == device_extensions
//     }

//     fn is_device_suitable(device: &PhysicalDevice, surface: &Arc<Surface<Window>>) -> bool {
//         let indices = Self::find_queue_families(surface, device);
//         let extension_supported = Self::check_device_support_extension(device);
//         let swap_chain_adequate = if extension_supported {
//             let capabilities = surface
//                 .capabilities(*device)
//                 .expect("Failes to get surface capabilities");
//             !capabilities.supported_formats.is_empty()
//                 && capabilities.present_modes.iter().next().is_some()
//         } else {
//             false
//         };
//         indices.is_complete() && extension_supported && swap_chain_adequate
//     }

//     fn find_queue_families(
//         surface: &Arc<Surface<Window>>,
//         device: &PhysicalDevice,
//     ) -> QueueFamilyIndices {
//         let mut indices = QueueFamilyIndices::new();
//         // TODO: replace index with id to simplify?
//         for (i, queue_family) in device.queue_families().enumerate() {
//             if queue_family.supports_graphics() {
//                 indices.graphics_family = i as i32;
//             }

//             if surface.is_supported(queue_family).unwrap() {
//                 indices.present_family = i as i32;
//             }

//             if indices.is_complete() {
//                 break;
//             }
//         }

//         indices
//     }

//     fn create_swap_chain(
//         instance: &Arc<Instance>,
//         surface: &Arc<Surface<Window>>,
//         physical_device_index: usize,
//         device: &Arc<Device>,
//         graphics_queue: &Arc<Queue>,
//         present_queue: &Arc<Queue>,
//     ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
//         let physical_device = PhysicalDevice::from_index(&instance, physical_device_index).unwrap();
//         let capabilities = surface
//             .capabilities(physical_device)
//             .expect("Failed to get surface capabilities");

//         let surface_format = Self::choose_swap_surface_format(&capabilities.supported_formats);
//         let present_mode = Self::choose_swap_present_mode(capabilities.present_modes);
//         let extent = Self::choose_swap_extent(&capabilities);

//         let mut image_count = capabilities.min_image_count + 1;
//         if capabilities.max_image_count.is_some()
//             && image_count > capabilities.max_image_count.unwrap()
//         {
//             image_count = capabilities.max_image_count.unwrap();
//         }

//         let image_usage = ImageUsage {
//             color_attachment: true,
//             ..ImageUsage::none()
//         };

//         let indicies = Self::find_queue_families(&surface, &physical_device);

//         let sharing: SharingMode = if indicies.graphics_family != indicies.present_family {
//             vec![graphics_queue, present_queue].as_slice().into()
//         } else {
//             graphics_queue.into()
//         };

//         let (swap_chain, images) = Swapchain::new(
//             device.clone(),
//             surface.clone(),
//             image_count,
//             surface_format.0,
//             extent,
//             1,
//             image_usage,
//             sharing,
//             capabilities.current_transform,
//             CompositeAlpha::Opaque,
//             present_mode,
//             FullscreenExclusive::Default,
//             true,
//             ColorSpace::SrgbNonLinear,
//         )
//         .expect("Failed to create swap chain");
//         (swap_chain, images)
//     }

//     fn choose_swap_surface_format(
//         available_formats: &[(Format, ColorSpace)],
//     ) -> (Format, ColorSpace) {
//         // NOTE: the 'preferred format' mentioned in the tutorial doesn't seem to be
//         // queryable in Vulkano (no VK_FORMAT_UNDEFINED enum)
//         *available_formats
//             .iter()
//             .find(|(format, color_space)| {
//                 *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
//             })
//             .unwrap_or_else(|| &available_formats[0])
//     }

//     fn choose_swap_present_mode(available_present_modes: SupportedPresentModes) -> PresentMode {
//         if available_present_modes.mailbox {
//             PresentMode::Mailbox
//         } else if available_present_modes.immediate {
//             PresentMode::Immediate
//         } else {
//             PresentMode::Fifo
//         }
//     }

//     fn choose_swap_extent(capabilities: &Capabilities) -> [u32; 2] {
//         if let Some(current_extent) = capabilities.current_extent {
//             return current_extent;
//         } else {
//             let mut actual_extent = [WIDTH, HEIGHT];
//             actual_extent[0] = capabilities.min_image_extent[0]
//                 .max(capabilities.max_image_extent[0].min(actual_extent[0]));
//             actual_extent[1] = capabilities.min_image_extent[1]
//                 .max(capabilities.max_image_extent[1].min(actual_extent[1]));
//             actual_extent
//         }
//     }

//     fn check_validation_layer_support() -> bool {
//         let layers: Vec<_> = layers_list()
//             .unwrap()
//             .map(|l| l.name().to_owned())
//             .collect();
//         VALIDATION_LAYERS
//             .iter()
//             .all(|layer_name| layers.contains(&layer_name.to_string()))
//     }
//     fn get_required_extensions() -> InstanceExtensions {
//         let mut extensions = vulkano_win::required_extensions();
//         if ENABLE_VALIDATION_LAYERS {
//             // TODO!: this should be ext_debug_utils (_report is deprecated), but that doesn't exist yet in vulkano
//             extensions.ext_debug_utils = true;
//         }
//         extensions
//     }
//     fn setup_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
//         if !ENABLE_VALIDATION_LAYERS {
//             return None;
//         }
//         let msg_types = MessageType::all();

//         let msg_severity = MessageSeverity {
//             error: true,
//             warning: true,
//             information: true,
//             verbose: true,
//         };
//         DebugCallback::new(&instance, msg_severity, msg_types, |msg| {
//             println!("validation layer: {:?}", msg.description);
//         })
//         .ok()
//     }

//     fn create_render_pass(
//         device: &Arc<Device>,
//         color_format: Format,
//     ) -> Arc<dyn RenderPassAbstract + Send + Sync> {
//         Arc::new(
//             vulkano::single_pass_renderpass!(device.clone(),
//                 attachments: {
//                     color: {
//                         load: Clear,
//                         store: Store,
//                         format: color_format,
//                         samples: 1,
//                     }
//                 },
//                 pass: {
//                     color: [color],
//                     depth_stencil: {}
//                 }
//             )
//             .unwrap(),
//         )
//     }

//     fn create_graphic_pipeline(
//         device: &Arc<Device>,
//         swap_chain_extent: [u32; 2],
//         render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
//     ) -> Arc<ConcreteGraphicsPipeline> {
//         mod vertex_shader {
//             vulkano_shaders::shader! {
//                 ty: "vertex",
//                 path: "src/ressha/triangle.vert"
//             }
//         }

//         mod fragment_shader {
//             vulkano_shaders::shader! {
//                 ty: "fragment",
//                 path: "src/ressha/triangle.frag"
//             }
//         }

//         let vert_shader_module = vertex_shader::Shader::load(device.clone())
//             .expect("failed to create vertex shader module!");
//         let frag_shader_module = fragment_shader::Shader::load(device.clone())
//             .expect("failed to create fragment shader module!");

//         let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
//         let viewport = Viewport {
//             origin: [0.0, 0.0],
//             dimensions,
//             depth_range: 0.0..1.0,
//         };

//         Arc::new(
//             GraphicsPipeline::start()
//                 .vertex_input(BufferlessDefinition {})
//                 .vertex_shader(vert_shader_module.main_entry_point(), ())
//                 .triangle_list()
//                 .primitive_restart(false)
//                 .viewports(vec![viewport]) // NOTE: also sets scissor to cover whole viewport
//                 .fragment_shader(frag_shader_module.main_entry_point(), ())
//                 .depth_clamp(false)
//                 // NOTE: there's an outcommented .rasterizer_discard() in Vulkano...
//                 .polygon_mode_fill() // = default
//                 .line_width(1.0) // = default
//                 .cull_mode_back()
//                 .front_face_clockwise()
//                 // NOTE: no depth_bias here, but on pipeline::raster::Rasterization
//                 .blend_pass_through() // = default
//                 .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
//                 .build(device.clone())
//                 .unwrap(),
//         )
//     }

//     fn create_framebuffers(
//         swap_chain_images: &[Arc<SwapchainImage<Window>>],
//         render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
//     ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
//         swap_chain_images
//             .iter()
//             .map(|image| {
//                 let fba: Arc<dyn FramebufferAbstract + Send + Sync> = Arc::new(
//                     Framebuffer::start(render_pass.clone())
//                         .add(image.clone())
//                         .unwrap()
//                         .build()
//                         .unwrap(),
//                 );
//                 fba
//             })
//             .collect::<Vec<_>>()
//     }

//     fn create_command_buffers(&mut self) {
//         // let queue_family = self.graphics_queue.family();
//         // self.command_buffers = self
//         //     .swap_chain_framebuffers
//         //     .iter()
//         //     .map(|framebuffer| {
//         //         let vertices = BufferlessVertices {
//         //             vertices: 3,
//         //             instances: 1,
//         //         };

//         //         AutoCommandBufferBuilder::primary_simultaneous_use(
//         //             self.device.clone(),
//         //             queue_family,
//         //         )
//         //         .unwrap()
//         //         .begin_render_pass(
//         //             framebuffer.clone(),
//         //             false,
//         //             vec![[0.0, 0.0, 0.0, 1.0].into()],
//         //         )
//         //         .unwrap()
//         //         .draw(
//         //             self.graphics_pipeline.clone(),
//         //             &DynamicState::none(),
//         //             vertices,
//         //             (),
//         //             (),
//         //         )
//         //         .unwrap()
//         //         .end_render_pass()
//         //         .unwrap()
//         //         .build()
//         //         .unwrap()
//         //     })
//         //     .collect();
//     }

//     fn draw_frame(&mut self) {
//         let (image_index, _, acquire_future) =
//             acquire_next_image(self.swap_chain.clone(), None).unwrap();

//         let command_buffer = self.command_buffers[image_index].clone();

//         let future = acquire_future
//             .then_execute(self.graphics_queue.clone(), command_buffer)
//             .unwrap()
//             .then_swapchain_present(
//                 self.present_queue.clone(),
//                 self.swap_chain.clone(),
//                 image_index,
//             )
//             .then_signal_fence_and_flush()
//             .unwrap();

//         future.wait(None).unwrap();
//     }

//     pub fn main_loop(&mut self) {
//         let desired_window_id = self.surface.window().id();

//         let mut quit = false;

//         while !quit {
//             self.event_loop.run_return(|event, _, control_flow| {
//                 *control_flow = ControlFlow::Wait;

//                 if let Event::WindowEvent { event, .. } = &event {
//                     // Print only Window events to reduce noise
//                     println!("{:?}", event);
//                 }

//                 match event {
//                     Event::WindowEvent {
//                         event: WindowEvent::CloseRequested,
//                         ..
//                     } => {
//                         quit = true;
//                     }
//                     Event::MainEventsCleared => {
//                         *control_flow = ControlFlow::Exit;
//                     }
//                     _ => (),
//                 }
//             });
//             self.draw_frame();
//         }
//     }
// }