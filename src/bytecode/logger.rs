use std::sync::Once;

use log::{LevelFilter, Metadata, Record};

static LOGGER: SimpleLogger = SimpleLogger;
static LOGGER_INIT: Once = Once::new();

/// Initalise a simple costom logging implementation
pub fn init_logger() {
	LOGGER_INIT.call_once(|| {
		let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace));
	});
}

/// A simple logger that just prints to stdout using some colours.
struct SimpleLogger;

impl log::Log for SimpleLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.target() != "rustyline"
	}

	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			let col = match record.level() {
				log::Level::Error => 91,
				log::Level::Warn => 93,
				log::Level::Info => 94,
				log::Level::Debug => 92,
				log::Level::Trace => 32,
			};
			let level = format!("\x1b[{col}m[{}]", record.level());

			if matches!(record.target(), "Stack" | "Disassembly" | "Source Error") {
				print!("{:<12}\x1b[90m [{}]\x1b[39m: {}", level, record.target(), record.args());
			} else {
				let file = record.file().unwrap_or_default();
				let line = record.line().unwrap_or_default();
				print!("{:<12}\x1b[90m {}:{}\x1b[39m: {}", level, file, line, record.args());
				if !matches!(record.target(), "nonew") {
					println!();
				}
			}
		}
	}

	fn flush(&self) {}
}
