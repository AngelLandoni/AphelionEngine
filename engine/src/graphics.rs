use downcast_rs::{impl_downcast, Downcast};

pub mod mesh;
pub mod vertex;
pub mod buffer;
pub mod gpu;

pub trait VertexBuffer: Downcast {}
impl_downcast!(VertexBuffer);

pub trait IndexBuffer: Downcast {}
impl_downcast!(IndexBuffer);

pub trait BufferCreator {
    fn allocate_vertex_buffer(&self, label: &str, data: &[u8]) -> Box<dyn VertexBuffer>;
    fn allocate_index_buffer(&self, label: &str, data: &[u8]) -> Box<dyn IndexBuffer>;
}

// TODO(Angel): Implement this.
pub trait ShaderHandler {
    fn compile_program(&self);
}