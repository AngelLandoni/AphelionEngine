use downcast_rs::{impl_downcast, Downcast};
use shipyard::Unique;
use std::ops::{Deref, DerefMut};

use crate::graphics::{BufferCreator, ShaderHandler};

/// Rust does not allow Trait composition / additional traits on the fly threfore
/// we need to create a trait which use them as supertraits.
pub trait GpuAbstractor: Downcast + BufferCreator + ShaderHandler + Send + Sync {}
impl_downcast!(GpuAbstractor);

#[derive(Unique)]
pub struct AbstractGpu(pub(crate) Box<dyn GpuAbstractor>);

impl Deref for AbstractGpu {
    type Target = Box<dyn GpuAbstractor>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AbstractGpu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
