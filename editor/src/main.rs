mod camera;
mod gui;
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

//use camera::CameraPlugin;
use gui::GuiPlugin;
use workbench_scene::WorkbenchScenePlugin;

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
                should_render_grid: true,
            },
            sub_scenes: vec![
                SceneDescriptor {
                    label: "Workbench Scene".to_owned(),
                    id: "WorkbenchScene".to_owned(),
                    camera: Camera::default(),
                    projection: Projection::default(),
                    resolution: Some(Size::new(2048, 1600)),
                    should_render_grid: true,
                },
                SceneDescriptor {
                    label: "Landscape Scene".to_owned(),
                    id: "LandscapeScene".to_owned(),
                    camera: Camera::default(),
                    projection: Projection::default(),
                    resolution: Some(Size::new(1024, 768)),
                    should_render_grid: true,
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
        .run();
}
