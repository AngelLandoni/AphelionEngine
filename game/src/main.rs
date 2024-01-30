use egui_demo_lib::DemoWindows;

use engine::components::MeshComponent;
use engine::nalgebra::{Point3, Vector3};

use engine::plugin::scene::primitives_plugin::{PrimitivesPlugin, PENTAGON_MESH_RESOURCE_ID};
use engine::scene::asset_server::MeshResourceID;
use engine::scene::mouse::CursorDelta;
use engine::{
    app::App,
    plugin::{
        Pluggable,
        host::window::WinitWindowPlugin,
        graphics::wgpu_renderer::WgpuRendererPlugin,
        graphics::egui::{
            EguiPlugin,
            EguiContext
        },
        core::clock::{
            Clock,
            ClockPlugin
        }, 
        scene::scene_plugin::ScenePlugin,
    },
    scene::{
        camera::Camera,
        keyboard::{
            KeyCode,
            Keyboard
        },
        mouse::Cursor
    },
    schedule::Schedule,
    shipyard::{
        Component,
        UniqueView,
        Unique,
        UniqueViewMut,
    }
};

#[derive(Unique)]
struct Demo(DemoWindows);

unsafe impl Send for Demo {}
unsafe impl Sync for Demo {}

#[derive(Unique)]
pub struct FlyCamera {
    pub yaw: f64,
    pub pitch: f64,
    pub direction: Vector3<f32>,
    pub right_direction: engine::nalgebra::Vector3<f32>
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
    mut demo: UniqueViewMut<Demo>,
    clock: UniqueView<Clock>,
    mut camera: UniqueViewMut<Camera>,
    mouse_position: UniqueView<Cursor>
) {
    let delta: String = format!("{}", clock.delta_milliseconds()).chars().take(4).collect();
    
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
                    engine::egui::Grid::new("Info")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Delta: ");
                            ui.label(delta);
                            ui.end_row();

                            ui.label("Position: ");
                            ui.label(format!("x: {}, y: {}", mouse_position.x, mouse_position.y));
                            ui.end_row();                           
                        });
                });
            });
    });

    engine::egui::Window::new("Streamline CFD")
        // .vscroll(true)
        .default_open(true)
        //.max_width(1000.0)
        //.max_height(800.0)
        //.default_width(800.0)
        .resizable(true)
        //.anchor(Align2::LEFT_TOP, [0.0, 0.0])
        .show(&egui.0, |mut ui| {
            if ui.add(engine::egui::Button::new("Far")).clicked() {
                camera.add_translation(
                    engine::nalgebra::Vector3::new(0.0, 0.0, 1.0), 10.0 * clock.delta_seconds() as f32
                );
                camera.add_target_translation(
                    engine::nalgebra::Vector3::new(0.0, 0.0, 1.0), 10.0 * clock.delta_seconds() as f32
                );
            }

            if ui.add(engine::egui::Button::new("Near")).clicked() {
                camera.add_translation(
                    engine::nalgebra::Vector3::new(0.0, 0.0, -1.0),10.0 * clock.delta_seconds() as f32
                );
                camera.add_target_translation(
                    engine::nalgebra::Vector3::new(0.0, 0.0, -1.0),10.0 * clock.delta_seconds() as f32
                );
            }

            ui.label("Slider");
            // ui.add(egui::Slider::new(_, 0..=120).text("age"));
            ui.end_row();

            // proto_scene.egui(ui);
        });

    //demo.0.ui(&egui.0);
}

fn camera_system(
    keyboard: UniqueView<Keyboard>,
    mut camera: UniqueViewMut<Camera>,
    clock: UniqueView<Clock>,
    fly_camera: UniqueView<FlyCamera>,
) {
    if keyboard.is_key_down(&KeyCode::W) {
        camera.position += fly_camera.direction * 4.5 * clock.delta_seconds() as f32;
        camera.target += fly_camera.direction * 4.5 * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::S) {
        camera.position -= fly_camera.direction * 4.5 * clock.delta_seconds() as f32;
        camera.target -= fly_camera.direction * 4.5 * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::A) {
        camera.position -= fly_camera.right_direction * 4.5 * clock.delta_seconds() as f32;
        camera.target -= fly_camera.right_direction * 4.5 * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::D) {
        camera.position += fly_camera.right_direction * 4.5 * clock.delta_seconds() as f32;
        camera.target += fly_camera.right_direction * 4.5 * clock.delta_seconds() as f32;
    }
}

use std::f64::consts::PI;

fn grados_a_radianes(grados: f64) -> f64 {
    (PI / 180.0) * grados
}

fn fly_camera_system(
    keyboard: UniqueView<Keyboard>,
    mut camera: UniqueViewMut<Camera>,
    c_delta: UniqueView<CursorDelta>,
    clock: UniqueView<Clock>,
    mut fly_camera: UniqueViewMut<FlyCamera>,
) {
    if !keyboard.is_key_down(&KeyCode::W) &&
       !keyboard.is_key_down(&KeyCode::S) &&
       !keyboard.is_key_down(&KeyCode::A) &&
       !keyboard.is_key_down(&KeyCode::D) &&
       !keyboard.is_key_down(&KeyCode::G) {
        return;
    }

    // Update the yaw angle base on the delta.
    fly_camera.yaw += c_delta.x * clock.delta_seconds() * -3.0;
    fly_camera.pitch += c_delta.y * clock.delta_seconds() * -3.0;
    
    let rad_yaw = grados_a_radianes(fly_camera.yaw);
    let rad_pitch =  grados_a_radianes(fly_camera.pitch);

    let dir = Vector3::new(
        rad_yaw.sin() as f32 * rad_pitch.cos() as f32,
        rad_pitch.sin() as f32,
        rad_yaw.cos() as f32 * rad_pitch.cos() as f32,
    );

    camera.target = Point3::new(
        camera.position.x + dir.x,
        camera.position.y + dir.y,
        camera.position.z + dir.z,
    );

    let parallel_direction = Vector3::new(
        (rad_yaw - PI / 2.0).sin() as f32,
        0.0,
        (rad_yaw - PI / 2.0).cos() as f32
    );

    fly_camera.direction = dir;
    fly_camera.right_direction = parallel_direction;

}

struct PlayerPlugin;

impl Pluggable for PlayerPlugin {
    fn configure(&self, app: &mut App) {
        let demo = egui_demo_lib::DemoWindows::default();

        app.world.add_unique(Demo(demo));
        app.world.add_unique(FlyCamera::default());

        app.world.add_entity(MeshComponent(PENTAGON_MESH_RESOURCE_ID));

        app.schedule(Schedule::RequestRedraw, |world| {
            world.run(set_ui);
        });

        app.schedule(Schedule::CursorDelta, |world| {
        });

        app.schedule(Schedule::Update, |world| {
            world.run(fly_camera_system);
            world.run(camera_system);
        });
    }
}

pub fn main() {
    App::new()
        .add_plugin(WinitWindowPlugin::new(
            "My game",
            1024,
            800,
        ))
        .add_plugin(ScenePlugin)
        .add_plugin(WgpuRendererPlugin)
        .add_plugin(ClockPlugin::default())
        .add_plugin(PrimitivesPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}