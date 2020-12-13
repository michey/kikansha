extern crate vulkano;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

use kikansha::figure::Figure;
use kikansha::figure::FigureMutation;
use kikansha::figure::FigureSet;
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
    let p_camera = StickyRotatingCamera::new(3.0, yaw, pitch);
    let camera = Arc::new(Mutex::new(p_camera));

    let cube_mutations = vec![
        FigureMutation::unit(),
        FigureMutation::new([0.75, 0.0, 0.0], 1.0),
        FigureMutation::new([-0.75, 0.0, 0.0], 1.0),
        FigureMutation::new([0.0, 0.0, 0.75], 1.0),
        FigureMutation::new([0.0, 0.0, -0.75], 1.0),
        FigureMutation::new([0.0, 0.75, 0.0], 1.0),
        FigureMutation::new([0.0, -0.75, 0.0], 1.0),
    ];

    let cubes_set = FigureSet::new(Figure::unit_cube(), cube_mutations);

    let tetra_mutations = vec![
        FigureMutation::new([0.75, 0.75, 0.75], 1.0),
        FigureMutation::new([0.75, 0.75, -0.75], 1.0),
        FigureMutation::new([0.75, -0.75, 0.75], 1.0),
        FigureMutation::new([0.75, -0.75, -0.75], 1.0),
        FigureMutation::new([-0.75, 0.75, 0.75], 1.0),
        FigureMutation::new([-0.75, 0.75, -0.75], 1.0),
        FigureMutation::new([-0.75, -0.75, 0.75], 1.0),
        FigureMutation::new([-0.75, -0.75, -0.75], 1.0),
    ];

    let tetra_set = FigureSet::new(Figure::unit_tetrahedron(), tetra_mutations);

    let scene = Scene::create(camera.clone(), vec![cubes_set, tetra_set]);

    let sleep = Duration::from_millis(10);

    let (event_send, _event_recv) = std::sync::mpsc::sync_channel(1);
    let (quit_send, quit_recv) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let _scoped_quit = QuitOnScopeExit {
            quit_send: &quit_send,
        };

        println!("Thread created");

        loop {
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
