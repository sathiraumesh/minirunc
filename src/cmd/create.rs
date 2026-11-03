use std::path;

use anyhow::{Context, Result, bail};
use nix::unistd::geteuid;

use crate::container::{self, Spec};

pub fn run(id: &str, bundle: &str) -> Result<()> {
    if !geteuid().is_root() {
        bail!("create: must run as root");
    }

    let bundle = path::absolute(bundle)
        .with_context(|| format!("resolve bundle path {bundle}"))?;

    // Load the config first, so a bad bundle fails before anything is created.
    let spec = Spec::load(&bundle)?;
    let dir = container::create(id)?;

    println!("created {id} at {} (runs {:?})", dir.display(), spec.process.args);
    Ok(())
}
