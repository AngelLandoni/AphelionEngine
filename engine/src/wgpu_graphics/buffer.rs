use wgpu::Buffer;

use crate::graphics::{VertexBuffer, IndexBuffer};

pub struct WgpuVertexBuffer(pub(crate) Buffer);

impl VertexBuffer for WgpuVertexBuffer {}

pub struct WgpuIndexBuffer(pub(crate) Buffer);

impl IndexBuffer for WgpuIndexBuffer {}