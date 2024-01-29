use shipyard::Unique;
use wgpu::Buffer;

use crate::{
    wgpu_graphics::gpu::Gpu,
    scene::{
        camera::Camera,
        perspective::Perspective
    },
};

#[derive(Unique)]
pub struct CameraUniform(pub(crate) Buffer);

/// Copies the projection matrix of the Camera to the GPU uniform designated for
/// storing projection information.
pub(crate) fn sync_camera_perspective_uniform(
    gpu: &Gpu,
    camera: &Camera,
    perspective: &Perspective,
    c_uniform: &Buffer
) {
    let proj = perspective.matrix() * camera.view_matrix();
    // TODO(Angel): Copy to the buffer only if the camera changed
    // otherwise avoid the update.
    let data: [[f32; 4]; 4] = proj.into();
    gpu
        .queue
        .write_buffer(
            &c_uniform,
            0,
            bytemuck::cast_slice(&[data])
       );
}