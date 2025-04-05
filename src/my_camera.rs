use cgmath::{InnerSpace, Zero};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct MyCamera {
    pub pos: cgmath::Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    // speed
    pub max_speed: f32,
    pub acceleration: f32,
    pub damp_factor: f32,
    pub curr_local_speed: cgmath::Vector3<f32>,
    pub sensitivity: f32,
}

impl MyCamera {
    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        let forward = cgmath::Vector3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize();

        // World up vector
        let world_up = cgmath::Vector3::unit_y();

        // Calculate the right vector (perpendicular to forward and world up)
        let right = forward.cross(world_up).normalize();

        // Calculate the actual up vector (perpendicular to forward and right)
        let up = right.cross(forward).normalize();
        let target = self.pos + forward;
        let view = cgmath::Matrix4::look_at_rh(self.pos, target, up);
        view
    }
    pub fn build_projection_matrix(&self, aspect: f32) -> cgmath::Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fovy), aspect, self.znear, self.zfar)
    }
}

impl Default for MyCamera {
    fn default() -> Self {
        MyCamera {
            pos: cgmath::Point3::new(0.0, 0.0, 10.0),
            yaw: -90.0,
            pitch: 0.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            max_speed: 2.5,
            acceleration: 10.0,
            damp_factor: 5.0,
            curr_local_speed: cgmath::Vector3::zero(),
            sensitivity: 0.25,
        }
    }
}
