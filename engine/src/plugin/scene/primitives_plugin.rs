use shipyard::UniqueView;

use crate::{
    plugin::Pluggable,
    scene::asset_server::AssetServer,
    app::App, 
};

/// Allocates and setups all the default primitives (Triangle, Quad, Cube, Cone,
/// Donut, etc) to be used in the rendering step.
/// 
/// This plugin requires the `AssetServer` therfore the `ScenePlugin` must be
/// configured first.
pub struct PrimitivesPlugin;

impl Pluggable for PrimitivesPlugin {
    fn configure(&self, app: &mut App) {
        let a_server = match app.world.borrow::<UniqueView<AssetServer>>() {
            Ok(s) => s,
            Err(_) => {
                println!("Primitives are not configured, AssetServer not configured");
                return;
            }
        };
    }
}