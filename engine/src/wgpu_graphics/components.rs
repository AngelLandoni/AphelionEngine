use shipyard::Unique;

use wgpu::{
    SurfaceTexture,
    TextureView
};

#[derive(Unique)]
pub struct ScreenFrame(pub(crate) Option<SurfaceTexture>);

#[derive(Unique)]
pub struct ScreenTexture(pub(crate) Option<TextureView>);