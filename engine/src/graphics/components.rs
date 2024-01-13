use shipyard::Unique;

use crate::graphics::{
    gpu::Gpu,
    CommandQueue,
};

/// Shipyard component responsible for storing all renderer-related resources.
#[derive(Unique)]
pub struct UniqueRenderer {
    pub(crate) gpu: Gpu,
}

#[derive(Unique)]
pub struct UniqueCommandQueue {
    pub(crate) queue: CommandQueue,
}