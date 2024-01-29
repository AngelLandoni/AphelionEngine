use shipyard::Unique;

use wgpu::{
    SurfaceTexture,
    TextureView
};

use crate::wgpu_graphics::gpu::Gpu;

#[derive(Unique)]
pub struct ScreenFrame(pub(crate) Option<SurfaceTexture>);

#[derive(Unique)]
pub struct ScreenTexture(pub(crate) Option<TextureView>);