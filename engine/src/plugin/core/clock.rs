use std::time::Instant;

use shipyard::{Unique, UniqueViewMut};

use crate::{
    plugin::Pluggable,
    schedule::Schedule
};

#[derive(Unique)]
pub struct Clock {
    /// Contains the start time of the past frame.
    last_frame_time: Instant,
    /// Contains the delta time calculated based on the current time.
    delta: f64,
    /// Determines if the engine should be limited or not.
    limit_frame_rate: bool,
    /// Defines the execution limit per seconds.
    target_frame_rate: f64,
}

impl Clock {
    fn new(limit_frame_rate: bool, target_frame_rate: u64) -> Self {
        Clock { 
            last_frame_time: Instant::now(),
            delta: 0.0,
            limit_frame_rate,
            target_frame_rate: 1.0 / target_frame_rate as f64,
        }
    }
}

impl Clock {
    fn update(&mut self) {
        self.delta = Instant::now()
            .duration_since(self.last_frame_time)
            .as_secs_f64();

        self.last_frame_time = Instant::now();
    }

    pub fn delta_seconds(&self) -> f64 {
        self.delta
    }

    pub fn delta_milliseconds(&self) -> f64 {
        self.delta * 1000.0
    }
}

/// Traks the elapsed time between frames. This can be used to know the delta
/// time.
pub struct ClockPlugin {
    limit_frame_rate: bool,
    target_frame_rate: u64,
}

impl Default for ClockPlugin {
    fn default() -> Self {
        Self {
            limit_frame_rate: true,
            target_frame_rate: 60,
        }
    }
}

impl ClockPlugin {    
    pub fn new(limit_frame_rate: bool, target_frame_rate: u64) -> Self {
        Self {
            limit_frame_rate,
            target_frame_rate,
        }
    }
}

impl Pluggable for ClockPlugin {
    fn configure(&self, app: &mut crate::app::App) {
        app.world.add_unique(Clock::new(
            self.limit_frame_rate,
            self.target_frame_rate,
        ));

        app.schedule(Schedule::BeforeRequestRedraw, |world| {
            world.run(calculate_clock_step_system);
        });
    }
}

fn calculate_clock_step_system(mut clock: UniqueViewMut<Clock>) {
    clock.update();
}