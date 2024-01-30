pub mod mesh;
pub mod vertex;
pub mod buffer;
pub mod gpu;

pub trait VertexBuffer {}

pub trait IndexBuffer {}

pub trait BufferCreator {
    fn create_vertex_buffer(&self, label: &str, data: &[u8]) -> Box<dyn VertexBuffer>;
    fn create_index_buffer(&self, label: &str, data: &[u8]) -> Box<dyn IndexBuffer>;
}

// TODO(Angel): Implement this.
pub trait ShaderHandler {
    fn compile_program(&self);
}