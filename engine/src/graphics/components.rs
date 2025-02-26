use shipyard::{Component, Unique};
use std::ops::{Deref, DerefMut};

use crate::{graphics::Texture, scene::assets::AssetResourceID};

use wgpu::{SurfaceTexture, TextureView};

#[derive(Unique)]
pub struct ScreenFrame(pub(crate) Option<SurfaceTexture>);

#[derive(Unique)]
pub struct ScreenTexture(pub(crate) Option<TextureView>);

#[derive(Component)]
pub struct MeshComponent(pub AssetResourceID);

impl Deref for MeshComponent {
    type Target = AssetResourceID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Unique)]
pub struct DepthTexture(pub(crate) Box<dyn Texture>);

impl Deref for DepthTexture {
    type Target = Box<dyn Texture>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DepthTexture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
