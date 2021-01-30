extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

use clap::App;
use clap::Arg;
use kikansha::engine::State;
use kikansha::figure::FigureMutation;
use kikansha::figure::FigureSet;
use kikansha::figure::RenderableMesh;
use kikansha::scene::camera::StickyRotatingCamera;
use kikansha::scene::gltf::load_figures;
use kikansha::scene::gltf::LoadingError;
use kikansha::scene::lights::PointLight;
use kikansha::scene::Scene;
use std::f32::consts::PI;
use std::process::exit;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

struct QuitOnScopeExit<'a> {
    quit_send: &'a std::sync::mpsc::Sender<bool>,
}

impl Drop for QuitOnScopeExit<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            log::error!("Panicking");
        }

        let _ = self.quit_send.send(true);
    }
}

fn main() {
    log4rs::init_file(
        "/home/michey/Projects/hello_vulkan/config/log4rs.yaml",
        Default::default(),
    )
    .unwrap();

    let matches = App::new("kikansha")
        .version("1.0")
        .author("")
        .about("")
        .arg(
            Arg::with_name("debugger")
                .short("d")
                .long("debugger")
                .help("Wait for debugger"),
        )
        .arg(
            Arg::with_name("validation")
                .short("v")
                .long("validation")
                .help("Run with validation layer"),
        )
        .arg(
            Arg::with_name("color_l")
                .short("c")
                .long("color_l")
                .takes_value(true)
                .value_name("level")
                .help("Set debug level for deferred shader"),
        )
        .get_matches();

    if matches.is_present("debugger") {
        let url = format!(
            "vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}",
            std::process::id()
        );
        log::info!("{}", &url);
        std::process::Command::new("code")
            .arg("--open-url")
            .arg(url)
            .output()
            .unwrap();
        std::thread::sleep_ms(10000); // Wait for debugger to attach
    }

    let color_debug_level: i32 = matches
        .value_of("color_l")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let run_with_validation = matches.is_present("validation");


    let mut yaw = PI / 4.0;
    let mut pitch = -PI / 4.0;
    let yaw_loop = Duration::from_secs(6_u64);
    let mut yaw_step = (PI * 2.0) / yaw_loop.as_millis() as f32;

    let pitch_loop = Duration::from_secs(10_u64);
    let mut pitch_step = PI / pitch_loop.as_millis() as f32;

    let mut init_ts = SystemTime::now();
    let p_camera = StickyRotatingCamera::new(5.5, yaw, pitch);
    let camera = Arc::new(Mutex::new(p_camera));

    let mut scene_sets: Vec<FigureSet> = Vec::new();

    let teapot_scale = 1.0;
    let teapot_mutations = vec![FigureMutation::new([0.0, 0.0, 0.0], teapot_scale)];

    let sce2: Result<Vec<RenderableMesh>, LoadingError> =
        // load_scene_from_file("/home/michey/Projects/hello_vulkan/data/models/teapot.gltf");
        load_figures("/home/michey/Projects/hello_vulkan/data/models/teapot.gltf");

    match sce2 {
        Ok(meshes) => match meshes.first() {
            Some(mesh) => {
                let teapot_set = FigureSet::new(
                    mesh.clone(),
                    teapot_mutations,
                    "/home/michey/Projects/hello_vulkan/src/kikansha/frame/resources/tex.png"
                        .to_string(),
                    "/home/michey/Projects/hello_vulkan/src/kikansha/frame/resources/tex.png"
                        .to_string(),
                );
                scene_sets.push(teapot_set);
            }
            _ => {}
        },
        _ => {}
    }

    let scene = Scene::create(camera.clone(), scene_sets, PointLight::default_lights());

    let sleep = Duration::from_millis(100);

    let (event_send, _event_recv) = std::sync::mpsc::sync_channel(1);
    let (quit_send, quit_recv) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let _scoped_quit = QuitOnScopeExit {
            quit_send: &quit_send,
        };

        log::info!("Thread created");

        loop {
            let current_ts = SystemTime::now();
            let elapsed = current_ts.duration_since(init_ts).unwrap();
            init_ts = current_ts;

            let new_yaw = yaw + (elapsed.as_millis() as f32 * yaw_step);
            yaw = new_yaw;
            if new_yaw >= (PI * 2.0) {
                yaw = PI * 2.0;
                yaw_step = -yaw_step;
            }
            if new_yaw <= 0.0 {
                yaw = 0.0;
                yaw_step = -yaw_step;
            }

            let new_pitch = pitch + (elapsed.as_millis() as f32 * pitch_step);
            pitch = new_pitch;
            if new_pitch >= (PI / 2.0) {
                pitch = PI / 2.0 - pitch_step.abs();
                pitch_step = -pitch_step
            }

            if new_pitch <= -(PI / 2.0) {
                pitch = -(PI / 2.0) + pitch_step.abs();
                pitch_step = -pitch_step
            }
            {
                camera.lock().unwrap().set_yaw(yaw);
            }
            // {
            //     camera.lock().unwrap().set_pitch(pitch);
            // }

            std::thread::sleep(sleep);
        }
    });

    State::run_loop(
        &scene,
        event_send,
        quit_recv,
        run_with_validation,
        color_debug_level,
    );
}
