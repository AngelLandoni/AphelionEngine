mod camera;
mod gui;
mod utils;
mod workbench_scene;

use camera::CameraPlugin;
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

use gui::GuiPlugin;
use utils::log::GuiLoggerPlugin;
use workbench_scene::WorkbenchScenePlugin;

pub fn main() {
    App::new()
        .add_plugin(WinitWindowPlugin::new("My game", 2048, 1200))
        .add_plugin(WgpuRendererPlugin)
        .add_plugin(ScenePlugin {
            main: SceneDescriptor {
                label: "Main Scene".to_owned(),
                id: "MainScene".to_owned(),
                camera: Camera::default(),
                projection: Projection::default(),
                resolution: None,
                should_render_grid: true,
                should_render_sky: false,
            },
            sub_scenes: vec![
                SceneDescriptor {
                    label: "Workbench Scene".to_owned(),
                    id: "WorkbenchScene".to_owned(),
                    camera: Camera::default(),
                    projection: Projection::default(),
                    resolution: Some(Size::new(2048, 1200)),
                    should_render_grid: true,
                    should_render_sky: true,
                },
                SceneDescriptor {
                    label: "Landscape Scene".to_owned(),
                    id: "LandscapeScene".to_owned(),
                    camera: Camera::default(),
                    projection: Projection::default(),
                    resolution: Some(Size::new(30, 30)),
                    should_render_grid: true,
                    should_render_sky: false,
                },
            ],
        })
        .add_plugin(ClockPlugin)
        .add_plugin(PrimitivesPlugin)
        .add_plugin(EguiPlugin {
            scene: EguiSceneSelector::Main,
        })
        .add_plugin(CameraPlugin)
        .add_plugin(WorkbenchScenePlugin)
        .add_plugin(GuiPlugin)
        .add_plugin(GuiLoggerPlugin)
        .run();
}
