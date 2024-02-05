use shipyard::{UniqueView, UniqueViewMut};

use crate::{
    app::App,
    host::window::Window,
    plugin::Pluggable,
    scene::{
        asset_server::AssetServer,
        camera::Camera,
        keyboard::Keyboard,
        mouse::{Cursor, CursorDelta},
        perspective::Perspective,
    },
    schedule::Schedule,
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

        // Update aspect ratio when window is resized.
        app.schedule(Schedule::WindowResize, |world| {
            world.run(|w: UniqueView<Window>, mut p: UniqueViewMut<Perspective>| {
                p.update_aspect_ratio(w.size.width as f32 / w.size.height as f32);
            });
        });
    }
}
