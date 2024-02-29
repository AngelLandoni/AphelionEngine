use shipyard::Unique;
use wgpu::RenderPipeline;

use crate::wgpu_graphics::gpu::Gpu;

#[derive(Unique)]
pub(crate) struct SkyPipeline {
    pipeline: RenderPipeline,
}

impl SkyPipeline {
    /*
    pub(crate) fn new(gpu: &Gpu) -> SkyPipeline {

    }*/
}
