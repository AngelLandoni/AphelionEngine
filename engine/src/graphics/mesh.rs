use crate::graphics::{IndexBuffer, VertexBuffer};

pub struct Mesh {
    /// Contains a refernece to the GPU RAM allocated vertex buffer.
    pub vertex_buffer: Box<dyn VertexBuffer>,
    /// Contains a refernece to the GPU RAM allocated index buffer.
    pub index_buffer: Box<dyn IndexBuffer>,
    /// Contains the number of indices in the index buffer.
    pub index_count: u32,
}

impl Mesh {
    /// Creates and returns a new `Mesh` instance which uses the provided 
    /// buffers.
    pub fn new(
        vertex_buffer: Box<dyn VertexBuffer>,
        index_buffer: Box<dyn IndexBuffer>,
        index_count: u32,
    ) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}