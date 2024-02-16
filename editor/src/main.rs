mod camera;
mod gui;

use engine::{
    app::App,
    plugin::{
        core::clock::ClockPlugin,
        graphics::{
            egui::{EguiPlugin, EguiSceneSelector},
            wgpu_renderer::WgpuRendererPlugin,
        },
        host::window::WinitWindowPlugin,
        scene::{
            primitives_plugin::PrimitivesPlugin, scene_plugin::ScenePlugin,
        },
    },
    scene::{camera::Camera, projection::Projection, scene::SceneDescriptor},
    types::Size,
};

//use camera::CameraPlugin;
use gui::GuiPlugin;

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
        //        .add_plugin(CameraPlugin)
        .add_plugin(GuiPlugin)
        .run();
}
