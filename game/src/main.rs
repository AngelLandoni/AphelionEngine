use egui_demo_lib::DemoWindows;

use engine::graphics::components::MeshComponent;
use engine::nalgebra::{Point3, Unit, UnitQuaternion, Vector3};

use engine::plugin::graphics::egui::EguiSceneSelector;
use engine::plugin::scene::primitives_plugin::{
    PrimitivesPlugin, CUBE_MESH_RESOURCE_ID,
};

use engine::scene::components::Transform;
use engine::scene::mouse::CursorDelta;
use engine::scene::projection::Projection;
use engine::scene::scene::SceneDescriptor;
use engine::scene::scene_state::SceneState;
use engine::{
    app::App,
    plugin::{
        core::clock::{Clock, ClockPlugin},
        graphics::egui::{EguiContext, EguiPlugin},
        graphics::wgpu_renderer::WgpuRendererPlugin,
        host::window::WinitWindowPlugin,
        scene::scene_plugin::ScenePlugin,
        Pluggable,
    },
    scene::{
        camera::Camera,
        keyboard::{KeyCode, Keyboard},
        mouse::Cursor,
    },
    schedule::Schedule,
    shipyard::{Component, Unique, UniqueView, UniqueViewMut},
};
use shipyard::{View, ViewMut};

#[derive(Unique)]
struct Demo(DemoWindows);

unsafe impl Send for Demo {}
unsafe impl Sync for Demo {}

#[derive(Unique)]
pub struct RotCubeAngle(f32);

#[derive(Unique)]
pub struct FlyCamera {
    pub yaw: f64,
    pub pitch: f64,
    pub direction: Vector3<f32>,
    pub right_direction: engine::nalgebra::Vector3<f32>,
}

impl Default for FlyCamera {
    /// Creates and returns a new `FlyCamera`.
    fn default() -> Self {
        Self {
            yaw: 180.0,
            pitch: 0.0,
            direction: Vector3::new(0.0, 0.0, -1.0),
            right_direction: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Component, Debug)]
struct Pos(f32, f32);

fn set_ui(
    egui: UniqueView<EguiContext>,
    _demo: UniqueViewMut<Demo>,
    clock: UniqueView<Clock>,
    mouse_position: UniqueView<Cursor>,
) {
    let delta: String = format!("{}", clock.delta_milliseconds())
        .chars()
        .take(4)
        .collect();

    let window = engine::egui::Window::new("Delta time")
        .id(engine::egui::Id::new("delta_window"))
        .resizable(false);

    // Add a menu bar at the top.
    engine::egui::TopBottomPanel::top("wrap_app_top_bar").show(&egui.0, |ui| {
        engine::egui::menu::bar(ui, |ui| {
            // Create 'File' menu.
            engine::egui::menu::menu_button(ui, "File", |ui| {
                ui.separator();

                if ui.add(engine::egui::Button::new("❌ Exit")).clicked() {
                    std::process::exit(0);
                }
            });
        });
    });

    window.show(&egui.0, |ui| {
        engine::egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Info");
                });

                engine::egui::ScrollArea::vertical().show(ui, |ui| {
                    engine::egui::Grid::new("Info").num_columns(2).show(
                        ui,
                        |ui| {
                            ui.label("Delta: ");
                            ui.label(delta);
                            ui.end_row();

                            ui.label("Position: ");
                            ui.label(format!(
                                "x: {}, y: {}",
                                mouse_position.x, mouse_position.y
                            ));
                            ui.end_row();
                        },
                    );
                });
            });
    });

    //demo.0.ui(&egui.0);
}

fn camera_system(
    keyboard: UniqueView<Keyboard>,
    mut scenes: UniqueViewMut<SceneState>,
    clock: UniqueView<Clock>,
    fly_camera: UniqueView<FlyCamera>,
) {
    if keyboard.is_key_down(&KeyCode::W) {
        scenes.main.camera.position +=
            fly_camera.direction * 8.5 * clock.delta_seconds() as f32;
        scenes.main.camera.target +=
            fly_camera.direction * 8.5 * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::S) {
        scenes.main.camera.position -=
            fly_camera.direction * 8.5 * clock.delta_seconds() as f32;
        scenes.main.camera.target -=
            fly_camera.direction * 8.5 * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::A) {
        scenes.main.camera.position -=
            fly_camera.right_direction * 8.5 * clock.delta_seconds() as f32;
        scenes.main.camera.target -=
            fly_camera.right_direction * 8.5 * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::D) {
        scenes.main.camera.position +=
            fly_camera.right_direction * 8.5 * clock.delta_seconds() as f32;
        scenes.main.camera.target +=
            fly_camera.right_direction * 8.5 * clock.delta_seconds() as f32;
    }
}

use std::f64::consts::PI;

fn grados_a_radianes(grados: f64) -> f64 {
    (PI / 180.0) * grados
}

fn fly_camera_system(
    keyboard: UniqueView<Keyboard>,
    mut scenes: UniqueViewMut<SceneState>,
    c_delta: UniqueView<CursorDelta>,
    clock: UniqueView<Clock>,
    mut fly_camera: UniqueViewMut<FlyCamera>,
) {
    if !keyboard.is_key_down(&KeyCode::W)
        && !keyboard.is_key_down(&KeyCode::S)
        && !keyboard.is_key_down(&KeyCode::A)
        && !keyboard.is_key_down(&KeyCode::D)
        && !keyboard.is_key_down(&KeyCode::G)
    {
        return;
    }

    // Update the yaw angle base on the delta.
    fly_camera.yaw += c_delta.x * clock.delta_seconds() * -3.0;
    fly_camera.pitch += c_delta.y * clock.delta_seconds() * -3.0;

    let rad_yaw = grados_a_radianes(fly_camera.yaw);
    let rad_pitch = grados_a_radianes(fly_camera.pitch);

    let dir = Vector3::new(
        rad_yaw.sin() as f32 * rad_pitch.cos() as f32,
        rad_pitch.sin() as f32,
        rad_yaw.cos() as f32 * rad_pitch.cos() as f32,
    );

    scenes.main.camera.target = Point3::new(
        scenes.main.camera.position.x + dir.x,
        scenes.main.camera.position.y + dir.y,
        scenes.main.camera.position.z + dir.z,
    );

    let parallel_direction = Vector3::new(
        (rad_yaw - PI / 2.0).sin() as f32,
        0.0,
        (rad_yaw - PI / 2.0).cos() as f32,
    );

    fly_camera.direction = dir;
    fly_camera.right_direction = parallel_direction;
}

fn rotate_cubes_system(
    _meshes: View<MeshComponent>,
    _transforms: ViewMut<Transform>,
    _angl: UniqueViewMut<RotCubeAngle>,
    _clock: UniqueView<Clock>,
) {
    let _axis = Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0));

    /*for (index, (_, t)) in (&meshes, &mut transforms).iter().enumerate() {
        let rot = UnitQuaternion::from_axis_angle(&axis, angl.0 + index as f32);
        t.rotation = rot;
    }*/

    // angl.0 += 0.5 * clock.delta_seconds() as f32;
}

struct PlayerPlugin;

impl Pluggable for PlayerPlugin {
    fn configure(&self, app: &mut App) {
        let demo = egui_demo_lib::DemoWindows::default();

        app.world.add_unique(Demo(demo));
        app.world.add_unique(FlyCamera::default());
        app.world.add_unique(RotCubeAngle(0.0));

        let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
        let _rot = UnitQuaternion::from_axis_angle(&axis, 1.78);

        /*app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: rot,
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
        ));*/

        let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
        let _rot = UnitQuaternion::from_axis_angle(&axis, 0.0);

        /*         app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(5.0, 0.0, 0.0),
                rotation: rot,
                scale: Vector3::new(2.0, 1.0, 1.0),
            },
        ));*/

        let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
        let rot = UnitQuaternion::from_axis_angle(&axis, 0.0);

        /* app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: rot,
                scale: Vector3::new(6.0, 2.0, 2.0),
            },
        ));


        app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: rot,
                scale: Vector3::new(2.0, 12.0, 2.0),
            },
        ));

        app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: rot,
                scale: Vector3::new(2.0, 2.0, 24.0),
            },
        ));

        app.world.add_entity((
            MeshComponent(CUBE_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(10.0, 0.0, 0.0),
                rotation: rot,
                scale: Vector3::new(2.0, 2.0, 24.0),
            },
        ));*/

        for i in 0..58 {
            for j in 0..58 {
                for k in 0..58 {
                    app.world.add_entity((
                        MeshComponent(CUBE_MESH_RESOURCE_ID),
                        Transform {
                            position: Vector3::new(
                                10.0 + i as f32 * 5.0,
                                k as f32 * 5.0,
                                j as f32 * 5.0,
                            ),
                            rotation: rot,
                            scale: Vector3::new(1.0, 1.0, 1.0),
                        },
                    ));
                }
            }
        }

        /*app.world.add_entity((
            MeshComponent(PENTAGON_MESH_RESOURCE_ID),
            Transform {
                position: Vector3::new(8.0, 0.0, 0.0),
                rotation: rot,
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
        ));*/

        app.schedule(Schedule::RequestRedraw, |world| {
            world.run(set_ui);
        });

        app.schedule(Schedule::CursorDelta, |_world| {});

        app.schedule(Schedule::Update, |world| {
            world.run(fly_camera_system);
            world.run(camera_system);
            world.run(rotate_cubes_system);
        });
    }
}

pub fn main() {
    App::new()
        .add_plugin(WinitWindowPlugin::new("My game", 1024, 800))
        .add_plugin(WgpuRendererPlugin)
        .add_plugin(ScenePlugin {
            main: SceneDescriptor {
                label: "Main Scene".to_owned(),
                id: "MainScene".to_owned(),
                camera: Camera::default(),
                projection: Projection::default(),
                resolution: None,
            },
            sub_scenes: Vec::new(),
        })
        .add_plugin(ClockPlugin)
        .add_plugin(PrimitivesPlugin)
        .add_plugin(EguiPlugin {
            scene: EguiSceneSelector::Main,
        })
        .add_plugin(PlayerPlugin)
        .run();
}
