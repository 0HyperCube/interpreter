#![feature(option_result_contains)]
#![allow(unused)]
#![feature(let_chains)]

#[macro_use]
extern crate log;

mod bytecode;
use std::path::Path;

pub use bytecode::*;

/// A simple CLI
fn main() {
	prelude::init_logger();

	let mut args = std::env::args();
	let mut path = args.next();

	// Cargo run feeds in the target folder which should be discarded
	if path.as_ref().filter(|path| path.starts_with("target")).is_some() {
		path = args.next();
	}

	if let Some(path) = path {
		// Error if the user has sent in too many arguments
		if args.next().is_some() {
			error!("Expected either path or nothing");
			std::process::exit(66);
		}
		info!("Running file {}", path);
		run_file(&path);
	} else {
		// Start REPL if no arguments
		info!("Welcome to the REPL");
		info!("Press enter to exit");
		repl();
	}
}
