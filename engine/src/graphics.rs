use downcast_rs::{impl_downcast, Downcast};

pub mod buffer;
pub mod components;
pub mod gpu;
pub mod mesh;
pub mod vertex;

pub trait VertexBuffer: Downcast {}
impl_downcast!(VertexBuffer);

pub trait IndexBuffer: Downcast {}
impl_downcast!(IndexBuffer);

pub trait Texture: Downcast + Send + Sync {}
impl_downcast!(Texture);

pub trait BufferCreator {
    fn allocate_vertex_buffer(&self, label: &str, data: &[u8]) -> Box<dyn VertexBuffer>;
    fn allocate_index_buffer(&self, label: &str, data: &[u8]) -> Box<dyn IndexBuffer>;
    fn allocate_depth_texture(&self, label: &str) -> Box<dyn Texture>;
}

// TODO(Angel): Implement this.
pub trait ShaderHandler {
    fn compile_program(&self);
}
