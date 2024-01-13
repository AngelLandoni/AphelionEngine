use shipyard::UniqueView;

use crate::graphics::components::UniqueRenderer;

/// A system used to execute the gui pass.
pub(crate) fn debug_gui_pass_system(gpu: UniqueView<UniqueRenderer>) {
    println!("Running internal pass");
}