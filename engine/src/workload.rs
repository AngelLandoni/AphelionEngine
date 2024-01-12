use crate::{
    app::App,
    schedule::Schedule,
};

/// Coordinates all the update systems.
pub(crate) fn run_update_workload(app: &mut App) {
    // Update events.
    // Extract all the update callbacks from the user and execute them.
    if let Some(update_fns) = app.scheduler.schedules.get(&Schedule::Update) {
        for func in update_fns {
            func(&app.world);
        }
    }
    // Run passes
    // Run custom passes?
    // End frame.
}