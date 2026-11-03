use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Setup a continer from bundle without statrting it
    Create {
        /// ID of the container
        id: String,
        /// Directory holding the config.josn and the root file system
        #[arg(short, long, default_value = ".")]
        bundle: String,
    },
    /// Internal: runs inside the new namespaces and sets up the container.
    /// Started by `create`; not meant to be run by hand.
    #[command(hide = true)]
    Init {
        /// ID of the container
        id: String,
        /// Absolute path to the bundle
        bundle: String,
    },
}

impl Command {
    pub fn run(self) -> Result<()> {
        match self {
            Command::Create { id, bundle } => super::create::run(&id, &bundle),
            Command::Init { id, bundle } => super::init::run(&id, &bundle),
            _ => Ok(()),
        }
    }
}
