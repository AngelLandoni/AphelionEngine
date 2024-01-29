use wgpu::{VertexBufferLayout, VertexAttribute, BufferAddress};

/// Represents a `Vertex` that can be efficiently transferred to the GPU for 
/// rendering and serves as a fundamental building block for rendering geometry 
/// on the screen.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: [f32; 3],
    col: [f32; 3],
}

impl Vertex {
    /// Returns the descriptor used to map the data to the layout in the GPU.
    pub(crate) fn desc<'a>() -> VertexBufferLayout<'a> {
        use std::mem::size_of;
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position.
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },

                // Color.
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}