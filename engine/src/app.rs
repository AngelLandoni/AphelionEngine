use shipyard::World;

use crate::{
    host::events::{Event, WindowEvent},
    plugin::Pluggable,
    schedule::{Schedule, Scheduler},
    workload::{
        finish_frame_workload, init_frame_workload,
        run_after_request_redraw_workload, run_before_request_redraw_workload,
        run_before_start, run_generic_event_workload,
        run_pipeline_configuration, run_pipeline_uniform_configuration,
        run_request_redraw_workload, run_scene_configuration,
        run_submit_queue_workload, run_update_workload,
        run_window_event_workload, start_frame_workload, update_cursor_delta,
        update_cursor_position, update_keyboard_events, update_mouse_events,
        update_mouse_wheel_delta, update_mouse_wheel_step_delta,
        update_window_size,
    },
};

/// This class represents the application, serving as the container for global
/// configuration settings. Additionally, it plays a crucial role in supplying
/// custom configurations to the engine.
pub struct App<'app> {
    /// The main `ECS`.
    pub world: World,
    /// This is the main run loop responsible for keeping the application alive
    /// and dispatching events.
    run_loop: Box<dyn FnOnce(&mut App) + 'app>,
    /// Conatins all the plugins to be configured.
    plugins: Vec<Box<dyn Pluggable + 'app>>,
    /// Contains all the workloads to be performed.
    pub(crate) scheduler: Scheduler<'app>,
}

impl<'app> Default for App<'app> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'app> App<'app> {
    /// Creates a new `App` instance. It utilizes a dummy run loop and requires
    /// configuration for actual rendering.
    pub fn new() -> Self {
        // TODO(Angel): Find a better place for this.

        let world = World::new();

        App {
            world,
            run_loop: Box::new(dummy_run_loop),
            plugins: Vec::new(),
            scheduler: Scheduler::new(),
        }
    }

    /// Runs the application, taking ownership of the `App`. After this point,
    /// the user is no longer in control of it.
    pub fn run(mut self) {
        // Extracts plugins to pass 'self' as a parameter. The plugins will be
        // discarded as they are no longer needed.
        let plugins = std::mem::take(&mut self.plugins);
        // Configure all pluggins.
        plugins.iter().for_each(|p| p.configure(&mut self));

        // Configure pipelines.
        run_pipeline_configuration(&self);
        // Configure scenes.
        run_scene_configuration(&self);
        // Configure pipelines uniforms.
        run_pipeline_uniform_configuration(&self);
        // Configure all the rest of stuff which depends of the pipelines.
        run_before_start(&self);

        // Take ownership of the run loop and execute it.
        let run_loop =
            std::mem::replace(&mut self.run_loop, Box::new(dummy_run_loop));

        run_loop(&mut self);
    }

    /// A function what must be called everytime there is an event. In case
    /// of inmediate mode it must be called once per frame.
    pub fn tick(&mut self, event: &Event) {
        run_generic_event_workload(self);

        match event {
            Event::Window(w_event) => {
                match w_event {
                    WindowEvent::CloseRequested => {
                        println!("Close window");
                    }

                    WindowEvent::RequestRedraw => {
                        start_frame_workload(self);
                        init_frame_workload(self);

                        run_update_workload(self);

                        run_before_request_redraw_workload(self);
                        run_request_redraw_workload(self);
                        run_after_request_redraw_workload(self);

                        run_submit_queue_workload(self);

                        finish_frame_workload(self);
                    }

                    WindowEvent::CursorMoved(x, y) => {
                        update_cursor_position(self, x, y);
                    }

                    WindowEvent::Resized(width, height) => {
                        update_window_size(self, width, height);
                    }

                    WindowEvent::UnknownOrNotImplemented => {}
                }

                run_window_event_workload(self);
            }

            Event::Keyboard(event) => {
                update_keyboard_events(self, event);
            }

            Event::Mouse(event) => {
                update_mouse_events(self, event);
            }

            Event::CursorMotion(x, y) => {
                update_cursor_delta(self, x, y);
            }

            Event::MouseWheelMotion(x, y) => {
                update_mouse_wheel_delta(self, x, y);
            }

            Event::MouseWheelStepMotion(x, y) => {
                update_mouse_wheel_step_delta(self, x, y);
            }

            Event::UnknownOrNotImplemented => {}
        }
    }

    /// Setups the main `RunLoop`.
    pub(crate) fn set_run_loop(
        &mut self,
        run_loop: impl FnOnce(&mut App) + 'app,
    ) {
        self.run_loop = Box::new(run_loop);
    }

    /// Configures the system. This function must always be invoked from a
    /// plugin.
    pub fn schedule(
        &mut self,
        schedule: Schedule,
        configurator: impl Fn(&World) + 'app,
    ) {
        self.scheduler.add_schedule(schedule, configurator);
    }

    /// Inserts a new `Plugin` into the application.
    pub fn add_plugin(mut self, plugin: impl Pluggable + 'app) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }
}

/// Dummy free function, serves as a replacement to remove the actual run loop.
fn dummy_run_loop(_app: &mut App) {}
