/// Represents a `Vertex` that can be efficiently transferred to the GPU for
/// rendering and serves as a fundamental building block for rendering geometry
/// on the screen.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub col: [f32; 3],
    pub uv: [f32; 2],
}
