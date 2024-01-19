pub mod renderer;
pub mod window;
pub mod iced;
pub mod egui;

use crate::app::App;

pub trait Pluggable {
    fn configure(&self, app: &mut App);
}