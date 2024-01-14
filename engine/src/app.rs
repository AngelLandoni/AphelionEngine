use shipyard::World;

use crate::{
    plugin::Pluggable,
    schedule::{Scheduler, Schedule}, workload::run_update_workload,
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

impl<'app> App<'app> {
    /// Creates a new `App` instance. It utilizes a dummy run loop and requires
    /// configuration for actual rendering.
    pub fn new() -> Self {
        App {
            world: World::new(),
            run_loop: Box::new(dummy_run_loop),
            plugins: Vec::new(),
            scheduler: Scheduler::new()
        }
    }

    /// Runs the application, taking ownership of the `App`. After this point,
    /// the user is no longer in control of it.
    pub fn run(mut self) {
        // Extracts plugins to pass 'self' as a parameter. The plugins will be
        // discarded as they are no longer needed.
        let plugins = std::mem::take(&mut self.plugins);
        // Configure all pluggins.
        plugins
            .iter()
            .for_each(|p| p.configure(&mut self));

        // Take ownership of the run loop and execute it.
        let run_loop = std::mem::replace(
            &mut self.run_loop,
            Box::new(dummy_run_loop)
        );

        run_loop(&mut self);
    }

    /// A function that must be called each frame. This will be called from the 
    /// `run_loop` size as it is taking ownership of `App`. This allow us to use
    /// `winit` or any other window handler.
    /// 
    /// TODO(Angel): Check if update needs `mut`.
    pub fn update(&self) {
        run_update_workload(self);
    }
    
    /// Setups the main `RunLoop`.
    pub(crate) fn set_run_loop(&mut self,
                               run_loop: impl FnOnce(&mut App) + 'app) {
        self.run_loop = Box::new(run_loop);
    }
    
    /// Configures the system. This function must always be invoked from a 
    /// plugin.
    pub fn schedule(&mut self, 
                    schedule: Schedule,
                    configurator: impl Fn(&World) + 'app) {
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