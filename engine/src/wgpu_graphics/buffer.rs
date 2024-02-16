use wgpu::Buffer;

use crate::{graphics::{IndexBuffer, Texture, UniformBuffer, VertexBuffer}, types::Size};

pub struct WgpuVertexBuffer(pub(crate) Buffer);

impl VertexBuffer for WgpuVertexBuffer {}

pub struct WgpuIndexBuffer(pub(crate) Buffer);

impl IndexBuffer for WgpuIndexBuffer {}

pub struct WgpuUniformBuffer(pub(crate) Buffer);

impl UniformBuffer for WgpuUniformBuffer {}

pub struct WGPUTexture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: Option<wgpu::Sampler>,
}

impl Texture for WGPUTexture {
    fn size(&self) -> Size<u32> {
        Size::new(self.texture.size().width, self.texture.size().width)
    }
}
