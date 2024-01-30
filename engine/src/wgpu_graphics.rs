pub(crate) mod gpu;
pub(crate) mod passes;
pub(crate) mod components;
pub(crate) mod rendering;
pub(crate) mod pipelines;
pub(crate) mod uniforms;
pub(crate) mod buffer;

use shipyard::Unique;
use wgpu::CommandBuffer;
use crossbeam_queue::ArrayQueue;

#[derive(Unique)]
pub struct CommandQueue(pub(crate) OrderCommandQueue);

/// Specifies the rendering order of different passes; a higher value indicates 
/// that the pass should be drawn later in the rendering pipeline.
#[derive(Copy, Clone)]
pub(crate) enum CommandSubmitOrder {
    DebugGui,
    TriangleTest,
}

impl CommandSubmitOrder {
    fn as_index(&self) -> usize {
        *self as usize
    }
}

/// Wraps a `wgpu` `CommandBuffer` and includes information about the position 
/// at which the command should be executed.
pub(crate) struct OrderCommandBuffer {
    pub(crate) label: Option<String>,
    pub(crate) order: usize,
    pub(crate) command: CommandBuffer,
}

impl OrderCommandBuffer {
    /// Creates and returns a new `OrderCommandBuffer`.
    pub(crate) fn new(label: Option<String>,
                      order: CommandSubmitOrder,
                      command: CommandBuffer) -> OrderCommandBuffer {
        OrderCommandBuffer {
            label,
            order: order.as_index(),
            command
        }
    }
}

/// Defines the max number of commands that can be performaned per frame, this
/// is becase the `ArrayQueu` needs an start size.
pub const MAX_NUMBER_IF_COMMANDS_PER_FRAME: usize = 20;

/// A queue uses to store all the commands to be submited per frame.
pub(crate) type OrderCommandQueue = ArrayQueue<OrderCommandBuffer>;
