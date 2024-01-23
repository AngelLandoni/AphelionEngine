use egui_demo_lib::DemoWindows;

use engine::{
    plugin::{
        Pluggable,
        host::window::WinitWindowPlugin,
        graphics::renderer::WgpuRendererPlugin,
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
    schedule::Schedule,
    app::App,
    shipyard::{
        Component,
        EntitiesViewMut,
        ViewMut,
        Workload,
        IntoWorkload,
        UniqueView,
        Unique,
        UniqueViewMut,
    }, scene::camera::Camera
};

#[derive(Unique)]
struct Demo(DemoWindows);

unsafe impl Send for Demo {}
unsafe impl Sync for Demo {}

#[derive(Component, Debug)]
struct Pos(f32, f32);

fn create_ints(mut _entities: EntitiesViewMut, mut _vm_vel: ViewMut<Pos>) {
}

fn delete_ints(mut _vm_vel: ViewMut<Pos>) {
}

fn set_ui(egui: UniqueView<EguiContext>, mut demo: UniqueViewMut<Demo>, clock: UniqueView<Clock>, mut camera: UniqueViewMut<Camera>) {
    let delta: String = format!("{}", clock.delta_milliseconds()).chars().take(4).collect();
    
    let window = engine::egui::Window::new("Delta time")
        .id(engine::egui::Id::new("delta_window"))
        .resizable(false);

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

fn int_cycle() -> Workload {
    (create_ints, delete_ints).into_workload()
}

struct PlayerPlugin;

impl Pluggable for PlayerPlugin {
    fn configure(&self, app: &mut App) {
        let demo = egui_demo_lib::DemoWindows::default();

        app.world.add_workload(int_cycle); 
        app.world.add_unique(Demo(demo));

        app.schedule(Schedule::RequestRedraw, |world| {
            world.run(set_ui);
        });

        app.schedule(Schedule::Update, |world| {
            world.run_workload(int_cycle).unwrap();
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
        .add_plugin(EguiPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}