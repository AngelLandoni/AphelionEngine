use shipyard::{UniqueView, UniqueViewMut, World};
use wgpu::BufferUsages;

use crate::{
    app::App, wgpu_graphics::{
        gpu::Gpu,
        components::{
            ScreenTexture,
            ScreenFrame,
        },
        rendering::{
            acquire_screen_texture,
            present_screen_texture,
            submit_commands_in_order,
            reconfigure_surface_if_needed_system
        },
        CommandQueue,
        OrderCommandQueue,
        pipelines::traingle_test_pipeline::TriangleTestPipeline,
        passes::triangle_test_pass::triangle_test_pass_system,
        uniforms::{
            CameraUniform,
            sync_camera_perspective_uniform,
        },
        MAX_NUMBER_IF_COMMANDS_PER_FRAME,
    },
    host::window::Window,
    plugin::Pluggable, scene::{
        camera::Camera,
        perspective::Perspective
    }, schedule::Schedule
};

pub struct WgpuRendererPlugin;

impl Pluggable for WgpuRendererPlugin {
    fn configure(&self, app: &mut App) {
        let window_resource = app
            .world
            .borrow::<UniqueView<Window>>()
            .expect("Configure the window context before setting up the renderer"); 

        let gpu = futures_lite::future::block_on(
            Gpu::new(&window_resource.as_ref())
        );

        drop(window_resource);

        // Setup all the unique resources.
        {
            let world = &app.world;
            
            setup_screen_texture_and_queue(&world);
            setup_camera(&world, &gpu);
            setup_pipelines(&world, &gpu);

            world.add_unique(gpu);
        }
        
        // Setup scheludes.
        {
            app.schedule(Schedule::Start, |world| {
                world.run(reconfigure_surface_if_needed_system);
            });

            app.schedule(Schedule::InitFrame, |world| {
                world.run(acquire_screen_texture);
            });

            app.schedule(Schedule::Update, |world| {
                world.run(sync_camera_perspective_uniform_system);
            });

            app.schedule(Schedule::RequestRedraw, |world| {
                world.run(triangle_test_pass_system);
            });

            app.schedule(Schedule::QueueSubmit, |world| {
                world.run(submit_commands_in_order);
            });

            app.schedule(Schedule::EndFrame, |world| {
                world.run(present_screen_texture);
                world.run(|mut texture: UniqueViewMut<ScreenTexture>, mut s_frame: UniqueViewMut<ScreenFrame>| {
                    texture.0 = None;
                    s_frame.0 = None;
                })
            });
        }
    }
}

/// Setups the screen texture (the texture that will be presented over the 
/// screen), and the queue user to submit all the encoder commands.
fn setup_screen_texture_and_queue(world: &World) {
    world.add_unique(ScreenTexture(None));
    world.add_unique(ScreenFrame(None));
    
    world.add_unique(CommandQueue(
        OrderCommandQueue::new(MAX_NUMBER_IF_COMMANDS_PER_FRAME)
    ));
}

/// Allocates space in the gpu to handle the camera proj and submits the buffer
/// ref to the world. 
fn setup_camera(world: &World, gpu: &Gpu) {
    let camera = world
        .borrow::<UniqueView<Camera>>()
        .expect("Unable to acquire camera");

    let proj: [[f32; 4]; 4] = camera.view_matrix().into();

    let buffer = gpu.allocate_buffer_init(
        "Camera proj uniform",
        proj,
        BufferUsages::COPY_DST | BufferUsages::UNIFORM,
    );

    world.add_unique(CameraUniform(buffer));
}

/// Setups all the required pipelines.
fn setup_pipelines(world: &World, gpu: &Gpu) {
    let camera_uniform = world
        .borrow::<UniqueView<CameraUniform>>()
        .expect("Unable to acquire camera uniform");

    world.add_unique(TriangleTestPipeline::new(&gpu, &camera_uniform.0));
}

/// Calls the sync camera method.
fn sync_camera_perspective_uniform_system(
    gpu: UniqueView<Gpu>,
    camera: UniqueView<Camera>,
    perspective: UniqueView<Perspective>,
    c_uniform: UniqueView<CameraUniform>
) {
    sync_camera_perspective_uniform(
        &gpu,
        &camera,
        &perspective,
        &c_uniform.0
    );
}