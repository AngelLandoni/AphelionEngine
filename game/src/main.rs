use egui_demo_lib::DemoWindows;
use engine::{
    plugin::{
        Pluggable,
        host::window::WinitWindowPlugin,
        graphics::renderer::WgpuRendererPlugin,
        graphics::egui::{EguiPlugin, EguiContext}, core::clock::{Clock, ClockPluggin},
    },
    schedule::Schedule,
    app::App,
};

use engine::shipyard::{
    Component,
    EntitiesViewMut,
    ViewMut,
    Workload,
    IntoWorkload
};
use shipyard::{UniqueView, Unique, UniqueViewMut};

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

fn set_ui(egui: UniqueView<EguiContext>, mut demo: UniqueViewMut<Demo>, clock: UniqueView<Clock>) {
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
        .add_plugin(WgpuRendererPlugin)
        .add_plugin(ClockPluggin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}