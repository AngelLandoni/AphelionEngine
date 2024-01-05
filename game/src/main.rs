use engine::{
    plugin::{window::winit_window_plugin::WinitWindowPlugin, Pluggable},
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

#[derive(Component, Debug)]
struct Pos(f32, f32);

fn create_ints(mut _entities: EntitiesViewMut, mut _vm_vel: ViewMut<Pos>) {
}

fn delete_ints(mut _vm_vel: ViewMut<Pos>) {
}


fn int_cycle() -> Workload {
    (create_ints, delete_ints).into_workload()
}

struct PlayerPlugin;

impl Pluggable for PlayerPlugin {
    fn configure(&self, app: &mut App) {
        // Configure the workload.
        app.schedule(Schedule::Startup, |world| {
            world.add_workload(int_cycle);
        });

        // Trigger the workload.
        app.schedule(Schedule::Update, |world| {
            world.run_workload(int_cycle).unwrap();
        });
    }
}

pub fn main() {
    println!("Game running");
    App::new()
        .add_plugin(WinitWindowPlugin::new(
            "My game",
            1024.0,
            800.0
        ))
        .add_plugin(PlayerPlugin)
        .run();
}