use std::borrow::BorrowMut;

use shipyard::UniqueViewMut;

use crate::{
    app::App, host::{
        components::UniqueWindow,
        events::KeyboardEvent
    },
    scene::mouse::Cursor,
    schedule::Schedule
};

/// Coordinates all the update systems.
pub(crate) fn run_before_request_redraw_workload(app: &App) {
    // Update events.
    // Extract all the update callbacks from the user and execute them.
    if let Some(update_fns) = app.scheduler.schedules.get(&Schedule::BeforeRequestRedraw) {
        for func in update_fns {
            func(&app.world);
        }
    }
}

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
pub(crate) fn run_after_request_redraw_workload(app: &App) {
    // Update events.
    // Extract all the update callbacks from the user and execute them.
    if let Some(update_fns) = app.scheduler.schedules.get(&Schedule::AfterRequestRedraw) {
        for func in update_fns {
            func(&app.world);
        }
    }
}

pub(crate) fn run_window_event_workload(app: &App) {
    // Update events.
    // Extract all the update callbacks from the user and execute them.
    if let Some(update_fns) = app.scheduler.schedules.get(&Schedule::WindowEvent) {
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
    if let Some(fsq_fns) = app.scheduler.schedules.get(&Schedule::QueueSubmit) {
        for func in fsq_fns {
            func(&app.world);
        }
    } 
}

pub(crate) fn start_frame_workload(app: &App) {
    if let Some(s_fns) = app.scheduler.schedules.get(&Schedule::Start) {
        for func in s_fns {
            func(&app.world);
        }
    } 
}

pub(crate) fn init_frame_workload(app: &App) {
    if let Some(if_fns) = app.scheduler.schedules.get(&Schedule::InitFrame) {
        for func in if_fns {
            func(&app.world);
        }
    } 
}

pub(crate) fn finish_frame_workload(app: &App) {
    if let Some(ef_fns) = app.scheduler.schedules.get(&Schedule::EndFrame) {
        for func in ef_fns {
            func(&app.world);
        }
    } 
}


pub(crate) fn update_cursor_position(app: &mut App, x: &f64, y: &f64) {
    let storage = app.world.borrow_mut();

    let mut cursor: UniqueViewMut<Cursor> = storage
        .borrow::<UniqueViewMut<Cursor>>()
        .expect("Unable to adquire cursor");

    cursor.x = *x;
    cursor.y = *y;
}

pub(crate) fn update_window_size(app: &mut App, width: &u32, height: &u32) {
    {
        let storage = app.world.borrow_mut();

        let mut size = storage
            .borrow::<UniqueViewMut<UniqueWindow>>()
            .expect("Unable to adquire cursor");
        
        size.host_window.size.width = *width;
        size.host_window.size.height = *height;
    }
    
    if let Some(w_u_fns) = app.scheduler.schedules.get(&Schedule::WindowResize) {
        for func in w_u_fns {
            func(&app.world);
        }
    }
}

/// Updates the state of the keys in the globat keyboard state.
pub(crate) fn update_keyboard_events(app: &mut App, keyboard: &KeyboardEvent) {
    let mut k = app
        .world
        .borrow::<UniqueViewMut<crate::scene::keyboard::Keyboard>>()
        .expect("Unable to acquire Keyboard resource");

    match keyboard {
        KeyboardEvent::Pressed(key) => k.register_key(key.clone()),
        KeyboardEvent::Released(key) => k.remove_key(key),
    }
}