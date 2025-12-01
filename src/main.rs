use clap::Parser;

mod cli;
mod compressors;
mod core;
mod crypto;
mod operations;
mod utils;

use cli::{execute, CliArgs};
use utils::init_logger;

fn main() {
    // Initialize logging
    init_logger();

    // Parse command-line arguments
    let args = CliArgs::parse();

    // Execute command
    match execute(args) {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
