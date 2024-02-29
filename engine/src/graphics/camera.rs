use crate::scene::{camera::Camera, projection::Projection};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    /// Contains the position of the camera.
    view_position: [f32; 4],
    /// Contains the proj * camera view (a matrix of where the camera is lookit at).
    view_proj: [[f32; 4]; 4],
    /// Conatins camera view (a matrix of where the camera is lookit at).
    view: [[f32; 4]; 4],
    /// Conatins the inversion of the projection.
    inv_proj: [[f32; 4]; 4],
    /// Conatins the inversion of the view.
    inv_view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn view_proj(camera: &Camera, projection: &Projection) -> Self {
        let proj = projection.matrix();
        let view = camera.view_matrix();

        Self {
            view_position: camera.position.to_homogeneous().into(),
            view_proj: (proj * view).into(),
            view: view.into(),
            inv_proj: proj.try_inverse().unwrap().into(),
            inv_view: view.transpose().into(),
        }
    }
}
