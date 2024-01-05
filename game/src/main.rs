use engine::{
    plugin::window::winit_window_plugin::WinitWindowPlugin,
    app::App, plugin::Pluggable
};

pub fn main() {
    println!("Game running");
    App::new()
        .add_plugin(WinitWindowPlugin)
        .run();
}