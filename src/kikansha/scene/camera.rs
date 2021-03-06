extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::Vector3;

#[derive(Default, Debug, Clone, Copy)]
pub struct CameraMatrices {
    pub projection_matrix: [f32; 16],
    pub view_matrix: [f32; 16],
    pub camera_position: [f32; 3],
}

impl CameraMatrices {
    fn allign(m: [f32; 16]) -> [[f32; 4]; 4] {
        [
            [m[0], m[1], m[2], m[3]],
            [m[4], m[5], m[6], m[7]],
            [m[8], m[9], m[10], m[11]],
            [m[12], m[13], m[14], m[15]],
        ]
    }

    pub fn alligned_projection_matrix(self) -> [[f32; 4]; 4] {
        Self::allign(self.projection_matrix)
    }

    pub fn alligned_view_matrix(self) -> [[f32; 4]; 4] {
        Self::allign(self.view_matrix)
    }

    pub fn emmpty() -> [[f32; 4]; 4] {
        [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ]
    }
}

pub trait ViewAndProject {
    fn view_m(&self) -> Matrix4<f32>;

    fn proj_m(&self) -> Matrix4<f32>;

    fn camera_p(&self) -> Point3<f32>;

    fn update_ar(&mut self, aspect_ratio: f32);

    fn update_fov(&mut self, fov: f32);

    fn get_matrices(&self) -> CameraMatrices {
        let p = self.proj_m();
        let v = self.view_m();
        let c = self.camera_p();
        CameraMatrices {
            projection_matrix: [
                p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8], p[9], p[10], p[11], p[12],
                p[13], p[14], p[15],
            ],
            view_matrix: [
                v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11], v[12],
                v[13], v[14], v[15],
            ],
            camera_position: [c[0], c[1], c[2]],
        }
    }
}

fn calcullate_view_m(eye: Point3<f32>, dest: Point3<f32>) -> Matrix4<f32> {
    let up: Vector3<f32> = Vector3::new(0.0, -1.0, 0.0);

    let center = Vector3::new(dest[0], dest[1], dest[2]);
    let eye_v = Vector3::new(eye[0], eye[1], eye[2]);
    glm::look_at(&eye_v, &center, &up)
}

fn calcullate_proj_m(
    fov: f32,
    aspect_ratio: f32,
    near_plane_dist: f32,
    far_plane_dist: f32,
) -> Matrix4<f32> {
    glm::perspective(aspect_ratio, fov, near_plane_dist, far_plane_dist)
}

#[derive(Debug, Copy, Clone)]
pub struct StickyRotatingCamera {
    view_m: Matrix4<f32>,
    proj_m: Matrix4<f32>,
    yaw: f32,
    pitch: f32,
    distance: f32,
    fov: f32,
    aspect_ratio: f32,
    near_plane_dist: f32,
    far_plane_dist: f32,
    eye: Point3<f32>,
}

impl StickyRotatingCamera {
    fn calculate_eye(distance: f32, yaw: f32, pitch: f32) -> Point3<f32> {
        let x = yaw.cos() * pitch.cos();
        let y = -pitch.sin();
        let z = yaw.sin() * pitch.cos();

        Point3::new(x * distance, y * distance, z * distance)
    }

    pub fn new(distance: f32, yaw: f32, pitch: f32) -> Self {
        log::trace!("insance of {}", std::any::type_name::<Self>());
        let eye = Self::calculate_eye(distance, yaw, pitch);
        let fov: f32 = 45.0;
        let aspect_ratio: f32 = 16.0 / 9.0;
        let near_plane_dist: f32 = 0.1;
        let far_plane_dist: f32 = 1000.0;

        let view_m: Matrix4<f32> = calcullate_view_m(eye, Point3::new(0.0, 0.0, 0.0));
        let proj_m: Matrix4<f32> =
            calcullate_proj_m(fov, aspect_ratio, near_plane_dist, far_plane_dist);
        StickyRotatingCamera {
            view_m,
            proj_m,
            yaw,
            pitch,
            distance,
            fov,
            aspect_ratio,
            near_plane_dist,
            far_plane_dist,
            eye,
        }
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        let eye = Self::calculate_eye(self.distance, yaw, self.pitch);
        let view_m: Matrix4<f32> = calcullate_view_m(eye, Point3::new(0.0, 0.0, 0.0));
        self.view_m = view_m;
        self.yaw = yaw;
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        let eye = Self::calculate_eye(self.distance, self.yaw, pitch);
        let view_m: Matrix4<f32> = calcullate_view_m(eye, Point3::new(0.0, 0.0, 0.0));
        self.view_m = view_m;
        self.pitch = pitch;
    }
}

impl ViewAndProject for StickyRotatingCamera {
    fn view_m(&self) -> Matrix4<f32> {
        self.view_m
    }

    fn proj_m(&self) -> Matrix4<f32> {
        self.proj_m
    }

    fn update_ar(&mut self, aspect_ratio: f32) {
        let proj_m: Matrix4<f32> = calcullate_proj_m(
            self.fov,
            aspect_ratio,
            self.near_plane_dist,
            self.far_plane_dist,
        );
        self.aspect_ratio = aspect_ratio;
        self.proj_m = proj_m;
    }

    fn update_fov(&mut self, fov: f32) {
        let proj_m: Matrix4<f32> = calcullate_proj_m(
            fov,
            self.aspect_ratio,
            self.near_plane_dist,
            self.far_plane_dist,
        );
        self.fov = fov;
        self.proj_m = proj_m;
    }

    fn camera_p(&self) -> Point3<f32> {
        self.eye
    }
}
