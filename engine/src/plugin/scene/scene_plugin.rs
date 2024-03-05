use ahash::AHashMap;

use shipyard::{Unique, UniqueView, UniqueViewMut, World};

use crate::{
    app::App,
    graphics::{
        camera::CameraUniform,
        gpu::AbstractGpu,
        scene::{sync_main_scene_dynamic_entities_transform, Scene}, Texture, UniformBuffer,
    },
    host::window::Window,
    plugin::Pluggable,
    scene::{
        asset_server::AssetServer,
        keyboard::Keyboard,
        mouse::{Cursor, CursorDelta},
        scene::SceneDescriptor,
        scene_state::SceneState,
    },
    schedule::Schedule,
};

#[derive(Unique)]
struct TempSceneDescriptors {
    pub main: SceneDescriptor,
    pub sub_scenes: Vec<SceneDescriptor>,
}

pub struct ScenePlugin {
    pub main: SceneDescriptor,
    pub sub_scenes: Vec<SceneDescriptor>,
}

impl Pluggable for ScenePlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_unique(Keyboard::default());
        app.world.add_unique(Cursor::default());
        app.world.add_unique(CursorDelta::default());
        app.world.add_unique(AssetServer::default());

        app.world.add_unique(TempSceneDescriptors {
            main: self.main.clone(),
            sub_scenes: self.sub_scenes.clone(),
        });

        app.schedule(Schedule::SceneConfiguration, |world| {
            allocate_scenes(world);
        });

        // Update aspect ratio when window is resized only for the main `Scene`.
        app.schedule(Schedule::WindowResize, |world| {
            world.run(
                |w: UniqueView<Window>, mut s_s: UniqueViewMut<SceneState>| {
                    s_s.main.projection.update_aspect_ratio(
                        w.size.width as f32 / w.size.height as f32,
                    );
                },
            );
        });

        app.schedule(Schedule::Update, |world| {
            world.run(sync_scene_cameras_with_their_uniforms_system);
            world.run(sync_main_scene_dynamic_entities_transform);
        });
    }
}

/// Takes all the descriptors provided by the user and transform them in actual
/// scenes.
fn allocate_scenes(world: &World) {
    let descriptors = world
        .borrow::<UniqueView<TempSceneDescriptors>>()
        .expect("Unable to acquire TempSceneDescriptors");

    let gpu = world.borrow::<UniqueView<AbstractGpu>>().expect(
        "Unable to acquire AbstractGpu, the scenes cannot be allocated",
    );

    let mut scenes_finished = AHashMap::new();

    let main = &descriptors.main;
    let sub_scenes = &descriptors.sub_scenes;

    for scene_d in sub_scenes.iter().chain(std::iter::once(main)) {
        let (camera_buffer, target_texture, depth_texture) =
            allocate_scene_main_resources(&gpu, scene_d);

        let sky_texture = if scene_d.should_render_sky {
            Some(allocate_sky_resources(&gpu))
        } else {
            //Some(allocate_sky_resources(&gpu))
            None
        };

        let scene = Scene {
            label: scene_d.label.clone(),
            camera: scene_d.camera,
            projection: scene_d.projection,
            camera_buffer,
            mesh_transform_buffers: AHashMap::new(),
            target_texture,
            depth_texture,
            should_sync_resolution_to_window: scene_d.resolution.is_none(),
            camera_bind_group: None,
            sky_texture,
            sky_env_bind_group: None,
        };

        scenes_finished.insert(scene_d.id.clone(), scene);
    }

    // The main scene must be in the list of scenes so we can just unwrap.
    let main_scene = scenes_finished.remove(&main.id).unwrap();

    world.add_unique(SceneState {
        main: main_scene,
        sub_scenes: scenes_finished,
    });
}

/// If there was any change in any of the camera`Scene`s the GPU buffer must be
/// updated to refect the changes.
fn sync_scene_cameras_with_their_uniforms_system(
    gpu: UniqueView<AbstractGpu>,
    s_state: UniqueView<SceneState>,
) {
    let uniform = CameraUniform::view_proj(
        &s_state.main.camera,
        &s_state.main.projection,
    );

    gpu.write_uniform_buffer(
        &s_state.main.camera_buffer,
        0,
        bytemuck::cast_slice(&[uniform]),
    );

    for s in s_state.sub_scenes.values() {
        let uniform = CameraUniform::view_proj(&s.camera, &s.projection);
        gpu.write_uniform_buffer(
            &s.camera_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );
    }
}

/// Allocates all the resoures needed to correctly setup the main resources (
/// camera and scene targets).
fn allocate_scene_main_resources(
    gpu: &AbstractGpu,
    scene: &SceneDescriptor,
) -> (Box<dyn UniformBuffer>, Box<dyn Texture>, Box<dyn Texture>) {
    let uniform = CameraUniform::view_proj(&scene.camera, &scene.projection);

    let camera_buffer = gpu.allocate_uniform_buffer(
        format!("{} Camera Buffer", scene.label).as_str(),
        bytemuck::cast_slice(&[uniform]),
    );

    // TODO(Angel): Determine how we are going to handle resolution for sub
    // scenes.
    let target_texture = gpu.allocate_target_texture(
        format!("{} scene target texture", scene.label).as_ref(),
        scene
            .resolution
            .map(|s| s.width)
            .unwrap_or(gpu.surface_size().width),
        scene
            .resolution
            .map(|s| s.height)
            .unwrap_or(gpu.surface_size().height),
    );

    let depth_texture = gpu.allocate_depth_texture(
        format!("{} scene depth texture", scene.id).as_ref(),
        scene
            .resolution
            .map(|s| s.width)
            .unwrap_or(gpu.surface_size().width),
        scene
            .resolution
            .map(|s| s.height)
            .unwrap_or(gpu.surface_size().height),
    );

    (camera_buffer, target_texture, depth_texture)
}

/// Allocate the required resources to render the sky.
fn allocate_sky_resources(gpu: &AbstractGpu) -> Box<dyn Texture> {
    // TODO(Angel): Change the 1080 resolution.
    gpu.allocate_cubemap_texture("Sky cubemap", 1080)
}
