pub mod cache;
mod queue;

use crate::debug::fps::Counter;
use crate::engine::cache::SceneCache;
use crate::engine::queue::QueueFamilyIndices;
use crate::frame::geometry::TriangleDrawSystem;
use crate::frame::system::FrameSystem;
use crate::frame::Pass;
use crate::scene::camera::ViewAndProject;
use crate::scene::Scene;
use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::sync::Mutex;
use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::{
    debug::DebugCallback, debug::MessageSeverity, debug::MessageType, layers_list, ApplicationInfo,
    Instance, InstanceExtensions, PhysicalDevice, Version,
};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
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
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;
use winit::window::WindowBuilder;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

/// Required device extensions
fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        khr_storage_buffer_storage_class: true,
        // ext_debug_utils: true,
        ..DeviceExtensions::none()
    }
}

#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

const VALIDATION_LAYERS: &[&str] = &[
    // "VK_LAYER_RENDERDOC_Capture",
    "VK_LAYER_NV_optimus",
    // "VK_LAYER_LUNARG_monitor",
    // "VK_LAYER_LUNARG_screenshot",
    // "VK_LAYER_LUNARG_device_simulation",
    // "VK_LAYER_LUNARG_api_dump",
    "VK_LAYER_KHRONOS_validation",
];

pub struct State {
    instance: Arc<Instance>,
    #[allow(dead_code)]
    debug_callback: Option<DebugCallback>,
    physical_device_index: usize,
    pub device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
    pub surface: Arc<Surface<Window>>,
    pub swap_chain: Arc<Swapchain<Window>>,
    pub swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
    pub dynamic_state: Mutex<DynamicState>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swap_chain: bool,
    scene_cache: SceneCache,
    pub frame_system: FrameSystem,
    pub triangle_draw_system: TriangleDrawSystem,
}

impl State {
    fn init(surface: Arc<Surface<Window>>, instance: Arc<Instance>) -> Self {
        let debug_callback = Self::setup_debug_callback(&instance);

        let physical_device_index = Self::pick_physical_device(&instance, &surface);
        let (device, graphics_queue, present_queue) =
            Self::create_logical_device(physical_device_index, &instance, &surface);
        let dynamic_state_raw = DynamicState::none();

        let dynamic_state = Mutex::new(dynamic_state_raw);

        let (swap_chain, swap_chain_images) = Self::create_swap_chain(
            &instance,
            &surface,
            physical_device_index,
            &device,
            &graphics_queue,
            &present_queue,
            None,
            &dynamic_state,
        );

        let dimensions = swap_chain_images[0].dimensions();
        let frame_system =
            FrameSystem::new(graphics_queue.clone(), swap_chain.format(), dimensions);

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        let triangle_draw_system =
            TriangleDrawSystem::new(graphics_queue.clone(), frame_system.deferred_subpass());

        State {
            instance,
            debug_callback,
            physical_device_index,
            device,
            graphics_queue,
            present_queue,
            surface,
            swap_chain,
            swap_chain_images,
            dynamic_state,
            previous_frame_end,
            recreate_swap_chain: false,
            scene_cache: SceneCache::default(),
            frame_system,
            triangle_draw_system,
        }
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
            log::error!("Validation layers requested, but not available!")
        }
        let supported_extensions = InstanceExtensions::supported_by_core()
            .expect("failed to retrieve supported extensions");
        let layers: Vec<_> = layers_list()
            .unwrap()
            .map(|l| l.name().to_owned())
            .collect();

        log::info!("Supported core extensions: {:?}", supported_extensions);
        log::info!("Supported extensions: {:?}", layers);
        let app_info = ApplicationInfo {
            application_name: Some("Kikansha".into()),
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

        let uniquer_queue_family: HashSet<&i32> = families.iter().collect();
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
        dynamic_state: &Mutex<DynamicState>,
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

        let (swap_chain, images) = match old_swapchain {
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
        };

        let swap_chain_extent = swap_chain.dimensions();
        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0..1.0,
        };
        {
            dynamic_state.lock().unwrap().viewports = Some(vec![viewport]);
        };

        (swap_chain, images)
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
                // false
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
            current_extent
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

        // extensions.khr_swapchain = true;
        // extensions.khr_storage_buffer_storage_class = true;
        // extensions.ext_debug_utils = true;
        if ENABLE_VALIDATION_LAYERS {
            extensions.ext_debug_utils = true;
            extensions.khr_wayland_surface = false;
            extensions.khr_android_surface = false;
            extensions.khr_win32_surface = false;
            extensions.mvk_ios_surface = false;
            extensions.mvk_macos_surface = false;
            extensions.nn_vi_surface = false;
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
            // log::info!(
            //     "Debug CB: LP:{:?}, msg: {:?}",
            //     msg.layer_prefix,
            //     msg.description,
            // );
        })
        .ok()
    }
    fn recreate_swap_chain(&mut self) {
        let (swap_chain, images) = Self::create_swap_chain(
            &self.instance,
            &self.surface,
            self.physical_device_index,
            &self.device,
            &self.graphics_queue,
            &self.present_queue,
            Some(self.swap_chain.clone()),
            &self.dynamic_state,
        );

        let dimensions = images[0].dimensions();

        self.frame_system
            .recreate_render_pass(swap_chain.format(), dimensions);
        self.swap_chain = swap_chain;
        self.swap_chain_images = images;
    }

    pub fn run_loop<T: ViewAndProject + Sized>(
        scene: &Scene<T>,
        _event_send: SyncSender<f32>,
        quit_recv: Receiver<bool>,
    ) {
        let instance_unb = Self::create_instance();
        let (mut event_loop, surface) = Self::init_loop(&instance_unb);
        let mut state = Self::init(surface, instance_unb);

        let mut counter = Counter::new(10);

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
                if let Ok(flag) = quit_recv.try_recv() {
                    if flag {
                        *control_flow = ControlFlow::Exit
                    }
                }

                state
                    .previous_frame_end
                    .as_mut()
                    .unwrap()
                    .cleanup_finished();

                if state.recreate_swap_chain {
                    log::trace!("recreate_swap_chain");
                    {
                        let mut locked_camera = scene.camera.lock().unwrap();
                        let dimensions: [u32; 2] = state.surface.window().inner_size().into();
                        locked_camera.update_ar(dimensions[0] as f32 / dimensions[1] as f32);
                    }
                    state.recreate_swap_chain();
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

                let future = state
                    .previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future);

                let mut after_future = None;

                let matrices = {
                    let locked_camera = scene.camera.lock().unwrap();
                    locked_camera.get_matrices()
                };

                let dynamic_state = { state.dynamic_state.lock().unwrap().clone() };
                let cached_scene = state.scene_cache.get_cache(
                    scene,
                    state.device.clone(),
                    state.graphics_queue.clone(),
                );

                let mut frame = state.frame_system.frame(
                    future,
                    state.swap_chain_images[image_num].clone(),
                    scene.lights.clone(),
                    matrices,
                    cached_scene.clone(),
                    dynamic_state.clone(),
                );

                while let Some(pass) = frame.next_pass() {
                    match pass {
                        Pass::Deferred(mut draw_pass) => {
                            let cb = state.triangle_draw_system.draw(
                                &matrices,
                                &cached_scene,
                                &dynamic_state,
                            );
                            draw_pass.execute(cb);
                        }
                        Pass::Lighting(mut lighting) => {
                            lighting.light();
                        }
                        Pass::Finished(af) => {
                            after_future = Some(af);
                        }
                    }
                }
                let future = after_future
                    .unwrap()
                    .then_swapchain_present(
                        state.graphics_queue.clone(),
                        state.swap_chain.clone(),
                        image_num,
                    )
                    .then_signal_fence_and_flush();

                if let Some(v) = counter.tick() {
                    log::info!("{:?} fps", v)
                }
                match future {
                    Ok(future) => {
                        state.previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        state.recreate_swap_chain = true;
                        state.previous_frame_end = Some(sync::now(state.device.clone()).boxed());
                    }
                    Err(e) => {
                        log::info!("Failed to flush future: {:?}", e);
                        state.previous_frame_end = Some(sync::now(state.device.clone()).boxed());
                    }
                }
            }
            _ => (),
        });
    }
}
