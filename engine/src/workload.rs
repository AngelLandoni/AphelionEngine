use crate::{
    app::App,
    schedule::Schedule, graphics::passes::debug_gui_pass::debug_gui_pass_system,
};

/// Coordinates all the update systems.
pub(crate) fn run_update_workload(app: &App) {
    // Update events.
    // Extract all the update callbacks from the user and execute them.
    if let Some(update_fns) = app.scheduler.schedules.get(&Schedule::Update) {
        for func in update_fns {
            func(&app.world);
        }
    }
    
    // Run passes

    // Debug gui
    app.world.run(debug_gui_pass_system);
    // Run custom passes?
    // End frame.
}