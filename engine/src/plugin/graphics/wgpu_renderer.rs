use std::collections::HashMap;

use shipyard::{IntoIter, UniqueView, UniqueViewMut, View, World};
use wgpu::{Buffer, BufferUsages};

use crate::{
    app::App,
    components::MeshComponent,
    graphics::gpu::AbstractGpu,
    host::window::Window,
    plugin::Pluggable,
    scene::{
        asset_server::MeshResourceID, camera::Camera, components::Transform,
        perspective::Perspective,
    },
    schedule::Schedule,
    wgpu_graphics::{
        components::{ScreenFrame, ScreenTexture},
        gpu::Gpu,
        passes::triangle_test_pass::triangle_test_pass_system,
        pipelines::traingle_test_pipeline::TriangleTestPipeline,
        rendering::{
            acquire_screen_texture, present_screen_texture, reconfigure_surface_if_needed_system,
            submit_commands_in_order,
        },
        uniforms::{sync_camera_perspective_uniform, CameraUniform},
        CommandQueue, OrderCommandQueue, MAX_NUMBER_IF_COMMANDS_PER_FRAME,
    },
};

pub struct WgpuRendererPlugin;

impl Pluggable for WgpuRendererPlugin {
    fn configure(&self, app: &mut App) {
        let window_resource = app
            .world
            .borrow::<UniqueView<Window>>()
            .expect("Configure the window context before setting up the renderer");

        let gpu = futures_lite::future::block_on(Gpu::new(window_resource.as_ref()));

        drop(window_resource);

        // Setup all the unique resources.
        {
            let world = &app.world;

            setup_screen_texture_and_queue(world);
            setup_camera(world, &gpu);
            setup_pipelines(world, &gpu);

            world.add_unique(AbstractGpu(Box::new(gpu)));
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
                world.run(sync_dynamic_entities_position);
            });

            app.schedule(Schedule::RequestRedraw, |world| {
                world.run(triangle_test_pass_system);
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

    world.add_unique(TriangleTestPipeline::new(gpu, &camera_uniform.0));
}

/// Calls the sync camera method.
fn sync_camera_perspective_uniform_system(
    gpu: UniqueView<AbstractGpu>,
    camera: UniqueView<Camera>,
    perspective: UniqueView<Perspective>,
    c_uniform: UniqueView<CameraUniform>,
) {
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu");

    sync_camera_perspective_uniform(gpu, &camera, &perspective, &c_uniform.0);
}

/// Recreates the transformation buffer and pass the transform information.
fn sync_dynamic_entities_position(
    gpu: UniqueView<AbstractGpu>,
    mut pipeline: UniqueViewMut<TriangleTestPipeline>,
    transforms: View<Transform>,
    meshes: View<MeshComponent>,
) {
    let gpu = gpu
        .downcast_ref::<Gpu>()
        .expect("Incorrect Gpu abstractor provided, it was expecting a Wgpu Gpu");

    // Agrupate transforms based on types.
    let mut grouped_transforms: HashMap<MeshResourceID, Vec<[[f32; 4]; 4]>> = HashMap::new();

    for (mesh, transform) in (&meshes, &transforms).iter() {
        grouped_transforms
            .entry(mesh.0)
            .or_insert(Vec::new())
            .push(transform.as_matrix_array());
    }

    let mut buffers: HashMap<MeshResourceID, (Buffer, u32)> = HashMap::new();

    for (mesh, transforms) in grouped_transforms {
        let buffer = gpu.raw_allocate_buffer_init(
            &format!("Mesh({}) transform", mesh.0),
            bytemuck::cast_slice(transforms.as_slice()),
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
        );

        buffers.insert(mesh, (buffer, transforms.len() as u32));
    }

    pipeline.mesh_transform_buffers = buffers;
}
