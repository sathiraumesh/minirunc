use std::fs::{self, DirBuilder};
use std::io::ErrorKind;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;

use crate::common;

pub const EXEC_FIFO: &str = "exec.fifo";

/// Creates the state directory for container `id`, with its exec FIFO inside.
/// Returns the path of the state directory.
pub fn create(id: &str) -> Result<PathBuf> {
    validate_id(id)?;

    let dir = common::root_path(id);
    make_state_dir(&dir, id)?;

    // Don't leave a half-made container behind if the FIFO fails.
    if let Err(err) = make_exec_fifo(&dir) {
        let _ = fs::remove_dir_all(&dir);
        return Err(err);
    }

    Ok(dir)
}

/// Rejects ids that are empty or could escape the root dir (like "../etc").
fn validate_id(id: &str) -> Result<()> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');

    if id.is_empty() || id == "." || id == ".." || !id.chars().all(allowed) {
        bail!("invalid container id {id:?}: use letters, digits, '-', '_' or '.'");
    }
    Ok(())
}

/// Creates the root dir if needed, then the container's own dir.
/// Creating the container dir fails if it already exists, so two
/// `create` calls with the same id can't both succeed.
fn make_state_dir(dir: &Path, id: &str) -> Result<()> {
    let root = dir.parent().context("state dir has no parent")?;
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(root)
        .with_context(|| format!("create root dir {}", root.display()))?;

    match DirBuilder::new().mode(0o700).create(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            bail!("container {id} already exists")
        }
        Err(e) => Err(e).with_context(|| format!("create state dir {}", dir.display())),
    }
}

fn make_exec_fifo(dir: &Path) -> Result<()> {
    let fifo = dir.join(EXEC_FIFO);
    mkfifo(&fifo, Mode::from_bits_truncate(0o600))
        .with_context(|| format!("create {}", fifo.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_ids() {
        for id in ["abc", "my-container", "web_1", "v1.2"] {
            assert!(validate_id(id).is_ok(), "{id} should be valid");
        }
    }

    #[test]
    fn rejects_bad_ids() {
        for id in ["", ".", "..", "../etc", "a/b", "has space"] {
            assert!(validate_id(id).is_err(), "{id:?} should be invalid");
        }
    }
}
