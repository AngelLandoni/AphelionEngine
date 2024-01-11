pub mod renderer;
pub mod window;

use crate::app::App;

pub trait Pluggable {
    fn configure(&self, app: &mut App);
}