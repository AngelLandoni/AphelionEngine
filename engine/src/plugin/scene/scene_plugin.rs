use ahash::AHashMap;
use shipyard::{Unique, UniqueView, UniqueViewMut, World};

use crate::{
    app::App,
    graphics::{gpu::AbstractGpu, scene::{sync_main_scene_dynamic_entities_transform, Scene}},
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

    let mut sub_scenes_finished = AHashMap::new();

    let main = &descriptors.main;
    let sub_scenes = &descriptors.sub_scenes;

    for s_scene in sub_scenes {
        let camera: [[f32; 4]; 4] = s_scene.camera.view_matrix().into();
        let camera_buffer = gpu.allocate_uniform_buffer(
            format!("{} Camera Buffer", s_scene.label).as_str(),
            bytemuck::cast_slice(&[camera]),
        );

        // TODO(Angel): Determine how we are going to handle resoluton for sub
        // scenes.
        let target_texture = gpu.allocate_target_texture(
            format!("{} scene target texture", s_scene.label).as_ref(),
            s_scene
                .resolution
                .map(|s| s.width)
                .unwrap_or(gpu.surface_size().width),
            s_scene
                .resolution
                .map(|s| s.height)
                .unwrap_or(gpu.surface_size().height),
        );

        let depth_texture =
            gpu.allocate_depth_texture("Main scene depth texture");

        let scene = Scene {
            label: s_scene.label.clone(),
            camera: s_scene.camera,
            projection: s_scene.projection,
            camera_buffer,
            mesh_transform_buffers: AHashMap::new(),
            target_texture,
            depth_texture,
            should_sync_resolution_to_window: s_scene.resolution.is_none(),
        };

        sub_scenes_finished.insert(s_scene.id.clone(), scene);
    }

    let main_camera: [[f32; 4]; 4] = main.camera.view_matrix().into();
    let main_camera_buffer = gpu.allocate_uniform_buffer(
        format!("{} Camera Buffer", main.label).as_str(),
        bytemuck::cast_slice(&[main_camera]),
    );

    let target_texture = gpu.allocate_target_texture(
        "Main scene target texture",
        main.resolution
            .map(|s| s.width)
            .unwrap_or(gpu.surface_size().width),
        main.resolution
            .map(|s| s.height)
            .unwrap_or(gpu.surface_size().height),
    );

    let depth_texture = gpu.allocate_depth_texture("Main scene depth texture");

    let main_scene = Scene {
        label: main.label.clone(),
        camera: main.camera,
        projection: main.projection,
        camera_buffer: main_camera_buffer,
        mesh_transform_buffers: AHashMap::new(),
        target_texture,
        depth_texture,
        should_sync_resolution_to_window: main.resolution.is_none(),
    };

    world.add_unique(SceneState {
        main: main_scene,
        sub_scenes: sub_scenes_finished,
    });
}

/// If there was any change in any of the camera`Scene`s the GPU buffer must be
/// updated to refect the changes.
fn sync_scene_cameras_with_their_uniforms_system(
    gpu: UniqueView<AbstractGpu>,
    s_state: UniqueView<SceneState>,
) {
    let c_matrix: [[f32; 4]; 4] =
        s_state.main.calculate_camera_projection().into();

    gpu.write_uniform_buffer(
        &s_state.main.camera_buffer,
        0,
        bytemuck::cast_slice(&[c_matrix]),
    );

    for s in s_state.sub_scenes.values() {
        let c_matrix: [[f32; 4]; 4] = s.calculate_camera_projection().into();
        gpu.write_uniform_buffer(
            &s.camera_buffer,
            0,
            bytemuck::cast_slice(&[c_matrix]),
        );
    }
}
