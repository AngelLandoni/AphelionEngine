pub mod core;
pub mod host;
pub mod graphics;
pub mod scene;

use crate::app::App;

pub trait Pluggable {
    fn configure(&self, app: &mut App);
}