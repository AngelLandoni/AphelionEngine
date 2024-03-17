use log::warn;
use shipyard::{Component, UniqueView, View};
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Operations, RenderPass,
    RenderPassDepthStencilAttachment, Texture, TextureView,
};

use crate::{
    graphics::{gpu::AbstractGpu, scene::Scene},
    scene::scene_state::SceneState,
    wgpu_graphics::{
        buffer::{WGPUBindGroup, WGPUTexture},
        gpu::Gpu,
        CommandQueue,
    },
};

pub(crate) const INTERNAL_MAIN_SCENE_ID: &str = "_INTERNAL_MAIN_SCENE_ID";

enum ForwardError {
    UnableToExtractTargetTexture,
    UnableToExtractTargetDepthTexture,

    MissingCameraBindGroup,

    IncorrectBindGroupType,
}

impl std::fmt::Display for ForwardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForwardError::UnableToExtractTargetTexture => {
                write!(f, "Unable to extract target texture")
            }

            ForwardError::UnableToExtractTargetDepthTexture => {
                write!(f, "Unable to extract depth texture")
            }

            ForwardError::MissingCameraBindGroup => {
                write!(f, "Unable to extract camera bind group")
            }

            ForwardError::IncorrectBindGroupType => {
                write!(f, "Incorrect bind group type")
            }
        }
    }
}

/// Component used to mark entities for forward rendering.
///
/// Entities with this component will be rendered using forward rendering.
#[derive(Component)]
pub struct ForwardRender;

/// Performs the forward rendering pass.
///
/// This system renders entities marked with the `ForwardRender` component.
///
/// # Arguments
///
/// * `gpu` - A unique view to the abstract GPU, used for rendering.
pub(crate) fn forward_pass_system(
    gpu: UniqueView<AbstractGpu>,
    queue: UniqueView<CommandQueue>,
    scenes: UniqueView<SceneState>,
) {
    let gpu = gpu.downcast_ref::<Gpu>().expect(
        "Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu",
    );

    let mut encoder =
        gpu.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Forward pass"),
            });

    // Need to keep the id in memory.
    let main_scene_temp_id = INTERNAL_MAIN_SCENE_ID.to_owned();

    // Chain the subscenes along side the main scene to make the code
    // easier and cleaner.
    let scenes = scenes
        .sub_scenes
        .iter()
        .chain(std::iter::once((&main_scene_temp_id, &scenes.main)));

    // Iterate over each scene and render the content.
    for (id, scene) in scenes {
        // Extract textures.
        let (target, depth) = match extract_scene_target_textures(&scene) {
            Ok(target) => target,
            Err(err) => {
                warn!("Forward pass: {}", err);
                continue;
            }
        };

        // Extract bind groups.
        let camera_bind_group = match extract_bind_groups(&scene) {
            Ok(bind_group) => bind_group,
            Err(err) => {
                warn!("Forward pass: {}", err);
                continue;
            }
        };

        // Init pass.
        let mut pass = start_pass(id, &mut encoder, &target.view, &depth.view);

        // Where I should get the pipeline, the material?.
        // pass.set_pipeline();
    }
}

/// Tries to extract the target and depth textures from the provided scene.
fn extract_scene_target_textures<'a>(
    scene: &'a Scene,
) -> Result<(&'a WGPUTexture, &'a WGPUTexture), ForwardError> {
    let target = scene
        .target_texture
        .downcast_ref()
        .ok_or(ForwardError::UnableToExtractTargetTexture)?;

    let depth = scene
        .depth_texture
        .downcast_ref()
        .ok_or(ForwardError::UnableToExtractTargetDepthTexture)?;

    Ok((target, depth))
}

/// Tries to extract the bind groups from the provided scene.
fn extract_bind_groups<'a>(
    scene: &'a Scene,
) -> Result<&WGPUBindGroup, ForwardError> {
    let camera_bind_group = scene
        .camera_bind_group
        .as_ref()
        .ok_or(ForwardError::MissingCameraBindGroup)?;

    let camera_bind_group = camera_bind_group
        .downcast_ref::<WGPUBindGroup>()
        .ok_or(ForwardError::IncorrectBindGroupType)?;

    Ok(camera_bind_group)
}

fn start_pass<'encoder>(
    id: &str,
    encoder: &'encoder mut CommandEncoder,
    target_texture: &'encoder TextureView,
    depth_texture: &'encoder TextureView,
) -> RenderPass<'encoder> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(format!("Forward pass scene {}", id).as_str()),
        color_attachments: &[
            // @location(0)
            Some(wgpu::RenderPassColorAttachment {
                view: target_texture,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            }),
        ],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            view: &depth_texture,
            depth_ops: Some(Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    })
}
