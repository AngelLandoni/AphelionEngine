mod camera;
mod gui;

use engine::{
    app::App,
    plugin::{
        core::clock::ClockPlugin,
        graphics::{egui::EguiPlugin, wgpu_renderer::WgpuRendererPlugin},
        host::window::WinitWindowPlugin,
        scene::{
            primitives_plugin::PrimitivesPlugin, scene_plugin::ScenePlugin,
        },
    },
};

use camera::CameraPlugin;
use gui::GuiPlugin;

pub fn main() {
    App::new()
        .add_plugin(WinitWindowPlugin::new("My game", 1024, 800))
        .add_plugin(ScenePlugin)
        .add_plugin(WgpuRendererPlugin)
        .add_plugin(ClockPlugin)
        .add_plugin(PrimitivesPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(GuiPlugin)
        .run();
}
