pub(crate) mod gpu;
pub(crate) mod passes;
pub(crate) mod components;

use wgpu::CommandBuffer;
use crossbeam_queue::ArrayQueue;

/// Specifies the rendering order of different passes; a higher value indicates 
/// that the pass should be drawn later in the rendering pipeline.
pub(crate) enum PassSubmitOrder {
    DebugGui,    
}

/// Wraps a `wgpu` `CommandBuffer` and includes information about the position 
/// at which the command should be executed.
pub(crate) struct OrderCommandBuffer {
    label: Option<String>,
    order: usize,
    command: CommandBuffer,
}

/// Defines the max number of commands that can be performaned per frame, this
/// is becase the `ArrayQueu` needs an start size.
pub const MAX_NUMBER_IF_COMMANDS_PER_FRAME: usize = 20;

/// A queue uses to store all the commands to be submited per frame.
pub(crate) type CommandQueue = ArrayQueue<OrderCommandBuffer>;