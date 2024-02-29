use std::sync::Mutex;

use engine::plugin::Pluggable;

pub struct GuiLogger;

impl engine::log::Log for GuiLogger {
    fn enabled(&self, metadata: &engine::log::Metadata) -> bool {
        metadata.level() <= engine::log::STATIC_MAX_LEVEL
    }

    fn log(&self, record: &engine::log::Record) {
        if self.enabled(record.metadata()) {
            if let Ok(mut lock) = LOG.lock() {
                lock.push((record.level(), record.args().to_string()))
            }
        }
    }

    fn flush(&self) {
        // Not needed as it is used by a inmediate mode GUI.
    }
}

/// Contains a global pool which contains all the logs.
static LOG: Mutex<Vec<(engine::log::Level, String)>> = Mutex::new(Vec::new());

/// Reads the global logs. Read triggers a copy of the log.
pub fn read_logs() -> Vec<(engine::log::Level, String)> {
    if let Ok(lock) = LOG.lock() {
        lock.clone()
    } else {
        Vec::new()
    }
}

/// Removes all the logs from the pool.
pub fn clean_logs() {
    if let Ok(mut lock) = LOG.lock() {
        lock.clear()
    }
}

pub struct GuiLoggerPlugin;

impl Pluggable for GuiLoggerPlugin {
    fn configure(&self, _: &mut engine::app::App) {
        engine::log::set_logger(&GuiLogger)
            .map(|_| engine::log::set_max_level(engine::log::LevelFilter::Info))
            .unwrap();
    }
}
