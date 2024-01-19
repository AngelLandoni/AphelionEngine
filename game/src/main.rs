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
struct UIState {
    name: String,
    age: u32,
}

#[derive(Component, Debug)]
struct Pos(f32, f32);

fn create_ints(mut _entities: EntitiesViewMut, mut _vm_vel: ViewMut<Pos>) {
}

fn delete_ints(mut _vm_vel: ViewMut<Pos>) {
}

fn set_ui(egui: UniqueView<EguiContext>, mut state: UniqueViewMut<UIState>) {
    engine::egui::CentralPanel::default().show(&egui.0, |ui| {
        ui.heading("My egui Application");
        ui.horizontal(|ui| {
            let name_label = ui.label("Your name: ");
            ui.text_edit_singleline(&mut state.name)
                .labelled_by(name_label.id);
        });
        ui.add(engine::egui::Slider::new(&mut state.age, 0..=120).text("age"));
        if ui.button("Click each year").clicked() {
            state.age += 1;
        }
        ui.label(format!("Hello '{}', age {}", state.name, state.age));
    });
}

fn int_cycle() -> Workload {
    (create_ints, delete_ints).into_workload()
}

struct PlayerPlugin;

impl Pluggable for PlayerPlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_workload(int_cycle); 
        app.world.add_unique(UIState {
            name: "Angel".to_string(),
            age: 28,
        });

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