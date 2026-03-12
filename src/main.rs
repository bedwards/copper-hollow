#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;

mod bridge;
mod cli;
mod engine;
mod gui;
mod midi_export;
mod state;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = cli::Cli::parse();

    match &args.command {
        Some(cmd) => cli::commands::execute(cmd, &args),
        None => gui::run(),
    }
}
