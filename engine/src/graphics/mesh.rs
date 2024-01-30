use crate::graphics::{IndexBuffer, VertexBuffer};

pub struct Mesh {
    vertex_buffer: Box<dyn VertexBuffer>,
    index_buffer: Box<dyn IndexBuffer>,
}

impl Mesh {
    /// Creates and returns a new `Mesh` instance which uses the provided 
    /// buffers.
    pub fn new(
        vertex_buffer: Box<dyn VertexBuffer>,
        index_buffer: Box<dyn IndexBuffer>
    ) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}