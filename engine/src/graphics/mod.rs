use downcast_rs::{impl_downcast, Downcast};

use crate::types::Size;

pub mod buffer;
pub mod camera;
pub mod components;
pub mod gpu;
pub mod material;
pub mod mesh;
pub mod passes;
pub mod pipeline;
pub mod rendering;
pub mod scene;
pub mod vertex;

pub trait VertexBuffer: Downcast + Send + Sync {}
impl_downcast!(VertexBuffer);

pub trait IndexBuffer: Downcast {}
impl_downcast!(IndexBuffer);

pub trait UniformBuffer: Downcast + Send + Sync {}
impl_downcast!(UniformBuffer);

pub trait Texture: Downcast + Send + Sync {
    fn size(&self) -> Size<u32>;
}
impl_downcast!(Texture);

pub trait BindGroup: Downcast + Send + Sync {}
impl_downcast!(BindGroup);

pub trait SurfaceHandler {
    fn surface_size(&self) -> Size<u32>;
}

pub enum BufferUsage {
    COPY_DST,
}

pub trait BufferCreator {
    fn allocate_vertex_buffer(
        &self,
        label: &str,
        data: &[u8],
    ) -> Box<dyn VertexBuffer>;

    fn allocate_index_buffer(
        &self,
        label: &str,
        data: &[u8],
    ) -> Box<dyn IndexBuffer>;

    fn allocate_depth_texture(
        &self,
        label: &str,
        width: u32,
        height: u32,
    ) -> Box<dyn Texture>;

    fn allocate_target_texture(
        &self,
        label: &str,
        width: u32,
        height: u32,
    ) -> Box<dyn Texture>;

    fn allocate_cubemap_texture(
        &self,
        label: &str,
        size: u32,
    ) -> Box<dyn Texture>;

    fn allocate_uniform_buffer(
        &self,
        label: &str,
        data: &[u8],
    ) -> Box<dyn UniformBuffer>;

    fn allocate_aligned_zero_vertex_buffer(
        &self,
        label: &str,
        size: u64,
        uses: BufferUsage,
    ) -> Box<dyn VertexBuffer>;
}

pub trait BufferHandler {
    fn write_uniform_buffer(
        &self,
        buffer: &Box<dyn UniformBuffer>,
        offset: u64,
        data: &[u8],
    );

    fn write_vertex_buffer(
        &self,
        buffer: &Box<dyn VertexBuffer>,
        offset: u64,
        data: &[u8],
    );
}

// TODO(Angel): Implement this.
pub trait ShaderHandler {
    fn compile_program(&self);
}

use crossbeam_queue::ArrayQueue;
use shipyard::Unique;
use wgpu::CommandBuffer;

#[derive(Unique)]
pub struct CommandQueue(pub(crate) OrderCommandQueue);

/// Specifies the rendering order of different passes; a higher value indicates
/// that the pass should be drawn later in the rendering pipeline.
#[derive(Copy, Clone)]
pub(crate) enum CommandSubmitOrder {
    FrameComposition,
    DebugGui,
    DebugGrid,
    DynamicMeshes,
    ForwardPass,
    Sky,
}

impl CommandSubmitOrder {
    fn as_index(&self) -> usize {
        *self as usize
    }
}

/// Wraps a `wgpu` `CommandBuffer` and includes information about the position
/// at which the command should be executed.
pub(crate) struct OrderCommandBuffer {
    #[allow(dead_code)]
    pub(crate) label: Option<String>,
    pub(crate) order: usize,
    pub(crate) command: CommandBuffer,
}

impl OrderCommandBuffer {
    /// Creates and returns a new `OrderCommandBuffer`.
    pub(crate) fn new(
        label: Option<String>,
        order: CommandSubmitOrder,
        command: CommandBuffer,
    ) -> OrderCommandBuffer {
        OrderCommandBuffer {
            label,
            order: order.as_index(),
            command,
        }
    }
}

/// Defines the max number of commands that can be performaned per frame, this
/// is becase the `ArrayQueu` needs an start size.
pub const MAX_NUMBER_IF_COMMANDS_PER_FRAME: usize = 20;

/// A queue uses to store all the commands to be submited per frame.
pub(crate) type OrderCommandQueue = ArrayQueue<OrderCommandBuffer>;
