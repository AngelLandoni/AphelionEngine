use crate::{
    plugin::Pluggable,
    scene::{
        asset_server::AssetServer,
        camera::Camera,
        keyboard::Keyboard,
        mouse::{Cursor, CursorDelta},
        perspective::Perspective
    },
    app::App, 
};

pub struct ScenePlugin;

impl Pluggable for ScenePlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_unique(Camera::default());
        app.world.add_unique(Perspective::default());
        app.world.add_unique(Keyboard::default());
        app.world.add_unique(Cursor::default());
        app.world.add_unique(CursorDelta::default());
        app.world.add_unique(AssetServer::default());
    }
}