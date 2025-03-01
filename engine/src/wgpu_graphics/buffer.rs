use wgpu::{Buffer, BufferUsages};

use crate::{
    graphics::{
        BindGroup, BufferUsage, IndexBuffer, Texture, UniformBuffer,
        VertexBuffer,
    },
    types::Size,
};

pub struct WgpuVertexBuffer(pub(crate) Buffer);

impl VertexBuffer for WgpuVertexBuffer {}

pub struct WgpuIndexBuffer(pub(crate) Buffer);

impl IndexBuffer for WgpuIndexBuffer {}

pub struct WgpuUniformBuffer(pub(crate) Buffer);

impl UniformBuffer for WgpuUniformBuffer {}

pub struct WGPUBindGroup(pub(crate) wgpu::BindGroup);
impl BindGroup for WGPUBindGroup {}

pub struct WGPUTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub(crate) sampler: Option<wgpu::Sampler>,
}

impl Texture for WGPUTexture {
    fn size(&self) -> Size<u32> {
        Size::new(self.texture.size().width, self.texture.size().width)
    }
}

// TODO(Angel): Impl `into` instead.
pub(crate) fn map_usages(usage: BufferUsage) -> BufferUsages {
    match usage {
        BufferUsage::COPY_DST => BufferUsages::COPY_DST,
    }
}
