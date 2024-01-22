pub mod core;
pub mod host;
pub mod graphics;

use crate::app::App;

pub trait Pluggable {
    fn configure(&self, app: &mut App);
}