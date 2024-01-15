use crate::{
    app::App,
    schedule::Schedule,
};

/// Coordinates all the update systems.
pub(crate) fn run_request_redraw_workload(app: &App) {
    // Update events.
    // Extract all the update callbacks from the user and execute them.
    if let Some(update_fns) = app.scheduler.schedules.get(&Schedule::RequestRedraw) {
        for func in update_fns {
            func(&app.world);
        }
    }
}

/// Coordinates all the update systems.
pub(crate) fn run_update_workload(app: &App) {
    // Update events.
    // Extract all the update callbacks from the user and execute them.
    if let Some(update_fns) = app.scheduler.schedules.get(&Schedule::Update) {
        for func in update_fns {
            func(&app.world);
        }
    }
}

pub(crate) fn run_submit_queue_workload(app: &App) {
    // Execute all the configs that should be triggered before the queue
    // gets submitted.
    if let Some(fsq_fns) = app.scheduler.schedules.get(&Schedule::BeforeSubmitQueue) {
        for func in fsq_fns {
            func(&app.world);
        }
    } 
}