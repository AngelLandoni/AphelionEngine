use shipyard::{UniqueView, UniqueViewMut, World};

use crate::{
    app::App,
    graphics::{components::DepthTexture, gpu::AbstractGpu, BufferCreator},
    host::window::Window,
    plugin::Pluggable,
    schedule::Schedule,
    wgpu_graphics::{
        components::{ScreenFrame, ScreenTexture},
        gpu::Gpu,
        passes::{
            dynamic_mesh_pass::dynamic_mesh_pass_system,
            frame_composition_pass_system::frame_composition_pass_system,
            infinite_grid_pass::infinite_grid_pass_system,
        },
        pipelines::{
            create_camera_bind_group_layout,
            dynamic_mesh_pipeline::DynamicMeshPipeline,
            frame_composition_pipeline::{
                setup_frame_composition_pipelines_uniforms_system,
                FrameCompositionPipeline,
            },
            infinite_grid_pipeline::InfiniteGridPipeline,
            setup_scenes_uniforms_system,
            sky_pipeline::{
                clear_sky_updater, sync_sky_pipeline_uniforms, SkyPipeline,
            },
            GlobalBindGroupLayouts,
        },
        rendering::{
            acquire_screen_texture, present_screen_texture,
            reconfigure_main_textures_if_needed_system,
            submit_commands_in_order,
        },
        CommandQueue, OrderCommandQueue, MAX_NUMBER_IF_COMMANDS_PER_FRAME,
    },
};

pub struct WgpuRendererPlugin;

impl Pluggable for WgpuRendererPlugin {
    fn configure(&self, app: &mut App) {
        let window_resource = app.world.borrow::<UniqueView<Window>>().expect(
            "Configure the window context before setting up the renderer",
        );

        let gpu =
            futures_lite::future::block_on(Gpu::new(window_resource.as_ref()));

        drop(window_resource);

        // Setup all the unique resources.
        {
            let world = &app.world;

            setup_screen_texture_and_queue(world);
            setup_depth_texture(world, &gpu);
        }

        {
            let world = &app.world;
            world.add_unique(AbstractGpu(Box::new(gpu)));
        }

        // Setup scheludes.
        {
            app.schedule(Schedule::PipelineConfiguration, |world| {
                setup_pipelines(world);
            });

            app.schedule(Schedule::PipelineUniformsSetup, |world| {
                world.run(setup_frame_composition_pipelines_uniforms_system);
                world.run(setup_scenes_uniforms_system);
            });

            app.schedule(Schedule::Start, |world| {
                world.run(reconfigure_main_textures_if_needed_system);
            });

            app.schedule(Schedule::InitFrame, |world| {
                world.run(acquire_screen_texture);
            });

            app.schedule(Schedule::Update, |world| {
                sync_sky_pipeline_uniforms(world);
                clear_sky_updater(world);
            });

            app.schedule(Schedule::RequestRedraw, |world| {
                world.run(dynamic_mesh_pass_system);
                world.run(frame_composition_pass_system);
                world.run(infinite_grid_pass_system);
            });

            app.schedule(Schedule::QueueSubmit, |world| {
                world.run(submit_commands_in_order);
            });

            app.schedule(Schedule::EndFrame, |world| {
                world.run(present_screen_texture);
                world.run(
                    |mut texture: UniqueViewMut<ScreenTexture>,
                     mut s_frame: UniqueViewMut<ScreenFrame>| {
                        texture.0 = None;
                        s_frame.0 = None;
                    },
                )
            });
        }
    }
}

/// Setups the screen texture (the texture that will be presented over the
/// screen), and the queue user to submit all the encoder commands.
fn setup_screen_texture_and_queue(world: &World) {
    world.add_unique(ScreenTexture(None));
    world.add_unique(ScreenFrame(None));

    world.add_unique(CommandQueue(OrderCommandQueue::new(
        MAX_NUMBER_IF_COMMANDS_PER_FRAME,
    )));
}

/// Setups the global depth texture.
fn setup_depth_texture(world: &World, gpu: &Gpu) {
    let d_texture = gpu.allocate_depth_texture(
        "Global depth texture",
        gpu.surface_config.width,
        gpu.surface_config.height,
    );
    world.add_unique(DepthTexture(d_texture));
}

/// Setups all the required pipelines.
fn setup_pipelines(world: &World) {
    let a_gpu = world
        .borrow::<UniqueView<AbstractGpu>>()
        .expect("Unable to acquire AbtractGpu");

    let gpu = a_gpu
        .downcast_ref::<Gpu>()
        .expect("Unable to acquire Wgpu GPU");

    // Creates the commond camera bindgroup layout used in all the
    // pipelines.
    let camera_bind_group_layout = create_camera_bind_group_layout(gpu);

    let dynamic_mesh = DynamicMeshPipeline::new(gpu, &camera_bind_group_layout);
    let frame_composition = FrameCompositionPipeline::new(gpu);
    let infinite_grid =
        InfiniteGridPipeline::new(gpu, &camera_bind_group_layout);
    let sky = SkyPipeline::new(gpu, &camera_bind_group_layout);

    world.add_unique(dynamic_mesh);
    world.add_unique(frame_composition);
    world.add_unique(infinite_grid);
    world.add_unique(sky);

    world.add_unique(GlobalBindGroupLayouts {
        camera: camera_bind_group_layout,
    });
}
