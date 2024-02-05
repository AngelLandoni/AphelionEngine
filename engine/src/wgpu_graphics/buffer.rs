use wgpu::Buffer;

use crate::graphics::{IndexBuffer, Texture, VertexBuffer};

pub struct WgpuVertexBuffer(pub(crate) Buffer);

impl VertexBuffer for WgpuVertexBuffer {}

pub struct WgpuIndexBuffer(pub(crate) Buffer);

impl IndexBuffer for WgpuIndexBuffer {}


pub struct WGPUTexture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
}

impl Texture for WGPUTexture {}
