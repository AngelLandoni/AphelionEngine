use engine::{
    plugin::{window::WinitWindowPlugin, Pluggable, renderer::WgpuRendererPlugin},
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
    println!("Creating stuff!");
}

fn delete_ints(mut _vm_vel: ViewMut<Pos>) {
    println!("Deleting stuff!");
}

fn int_cycle() -> Workload {
    (create_ints, delete_ints).into_workload()
}

struct PlayerPlugin;

impl Pluggable for PlayerPlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_workload(int_cycle); 

        app.schedule(Schedule::Update, |world| {
            println!("Running workload from userland!");
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
        .add_plugin(PlayerPlugin)
        .run();
}