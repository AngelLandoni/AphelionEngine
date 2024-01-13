pub mod renderer;
pub mod window;
pub mod iced;

use crate::app::App;

pub trait Pluggable {
    fn configure(&self, app: &mut App);
}