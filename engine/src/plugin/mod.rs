pub mod core;
pub mod graphics;
pub mod host;
pub mod scene;

use crate::app::App;

pub trait Pluggable {
    fn configure(&self, app: &mut App);
}
