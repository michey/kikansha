extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

use kikansha::figure::Figure;
use kikansha::figure::FigureMutation;
use kikansha::scene::camera::StickyRotatingCamera;
use kikansha::scene::Scene;
use kikansha::state::State;
use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

struct QuitOnScopeExit<'a> {
    quit_send: &'a std::sync::mpsc::Sender<bool>,
}

impl Drop for QuitOnScopeExit<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            println!("Panicking");
        }

        let _ = self.quit_send.send(true);
    }
}

fn main() {
    let yaw = 0.0;
    let pitch = PI / 4.0;
    let yaw_loop = Duration::from_secs(10 as u64);
    let yaw_step = (PI * 2.0) / yaw_loop.as_millis() as f32;
    let mut init_ts = SystemTime::now();
    let p_camera = StickyRotatingCamera::new(1.5, yaw, pitch);
    let camera = Arc::new(Mutex::new(p_camera));

    let figure = Figure::unit_cube(FigureMutation::unit());
    // let mut scene = Arc::new(Scene::create(camera, vec![figure]));
    let scene = Scene::create(camera.clone(), vec![figure]);

    let sleep = Duration::from_millis(10);

    let (event_send, _event_recv) = std::sync::mpsc::sync_channel(1);
    let (quit_send, quit_recv) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let _scoped_quit = QuitOnScopeExit {
            quit_send: &quit_send,
        };

        println!("Thread created");

        // let mut input_processor = core::input::InputProvider::default();

        // let mut swapchain: Option<Arc<dyn Swapchain>> = None;

        loop {
            // let event = if let Ok(event) = event_recv.recv() {
            //     event
            // } else {
            //     // error getting the event, most likely reason is channel closed,
            //     // which means we can terminate.
            //     println!("`event_recv` failed to receive event, starting process shutdown");
            //     return;
            // };

            let current_ts = SystemTime::now();
            let elapsed = current_ts.duration_since(init_ts).unwrap();
            if elapsed >= yaw_loop {
                init_ts = SystemTime::now();
            }

            let new_yaw = elapsed.as_millis() as f32 * yaw_step;
            {
                camera.lock().unwrap().set_yaw(new_yaw);
            }

            std::thread::sleep(sleep);
        }
    });

    State::run_loop(&scene, event_send, quit_recv);
}
