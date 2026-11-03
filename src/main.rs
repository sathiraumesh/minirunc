use std::env;

use anyhow::Result;
use clap::Parser;

mod cmd;
mod common;
mod container;
mod os;

/// A small OCI-style container runtime, modelled on runc.
#[derive(Parser)]
#[command(name = "minic", version, about)]
struct Cli {
    #[command(subcommand)]
    command: cmd::Command,
}

fn main() -> Result<()> {
    let root = common::root_dir(env::var(common::ROOT_VAR).ok());
    // safe here: no other threads yet
    unsafe {
        env::set_var(common::ROOT_VAR, &root);
    }

    Cli::parse().command.run()
}
