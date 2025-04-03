use crate::my_camera::MyCamera;

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(camera: &MyCamera, aspect: f32, translate: bool) -> Self {
        let mut view = camera.build_view_matrix();
        if !translate{
            view[3][0] = 0.0;
            view[3][1] = 0.0;
            view[3][2] = 0.0;
        }
        Self {
            view: view.into(),
            proj: camera.build_projection_matrix(aspect).into(),
        }
    }
}
