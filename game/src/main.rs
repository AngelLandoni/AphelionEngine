use std::sync::{Arc, Mutex};

use egui_demo_lib::DemoWindows;
use engine::{
    plugin::{
        Pluggable,
        window::WinitWindowPlugin,
        renderer::WgpuRendererPlugin,
        egui::{EguiPlugin, EguiContext},
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

fn set_ui(egui: UniqueView<EguiContext>, mut demo: UniqueViewMut<Demo>) {
    demo.0.ui(&egui.0);
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
        .add_plugin(EguiPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}