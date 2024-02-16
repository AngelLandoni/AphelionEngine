use shipyard::World;
use std::collections::HashMap;

/// Specifies the various steps comprising the application lifecycle.
#[derive(Eq, PartialEq, Hash)]
pub enum Schedule {
    PipelineConfiguration,
    SceneConfiguration,
    PipelineUniformsSetup,

    Start,
    InitFrame,

    Update,
    WindowEvent,
    BeforeRequestRedraw,
    RequestRedraw,
    AfterRequestRedraw,
    QueueSubmit,

    WindowResize,

    EndFrame,

    CursorDelta,
}

/// Holds a collection of `Workload`s to be executed alongside information about
/// when each should be triggered.
pub(crate) struct Scheduler<'a> {
    pub(crate) schedules: HashMap<Schedule, Vec<Box<dyn Fn(&World) + 'a>>>,
}

impl<'a> Scheduler<'a> {
    /// Creates a new instance of `Scheduler`.
    pub(crate) fn new() -> Self {
        Scheduler {
            schedules: HashMap::new(),
        }
    }

    /// Adds a new callback to be executed when it is needed.
    pub(crate) fn add_schedule(
        &mut self,
        schedule: Schedule,
        callback: impl Fn(&World) + 'a,
    ) {
        self.schedules
            .entry(schedule)
            .or_default()
            .push(Box::new(callback));
    }
}
