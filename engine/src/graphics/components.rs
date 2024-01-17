use shipyard::Unique;

use wgpu::{
    SurfaceTexture,
    TextureView
};

use crate::graphics::gpu::Gpu;

/// Shipyard component responsible for storing all renderer-related resources.
#[derive(Unique)]
pub struct UniqueRenderer {
    pub(crate) gpu: Gpu,
}

#[derive(Unique)]
pub struct ScreenFrame(pub(crate) Option<SurfaceTexture>);

#[derive(Unique)]
pub struct ScreenTexture(pub(crate) Option<TextureView>);