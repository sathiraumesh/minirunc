use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use nix::fcntl::{OFlag, open};
use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nix::sched::{CloneFlags, clone};
use nix::sys::signal::Signal;
use nix::sys::stat::Mode;
use nix::unistd::{Pid, chdir, execv, pivot_root, sethostname};

use crate::common;
use crate::container::{EXEC_FIFO, Spec};

/// Runs inside the new namespaces (started by `spawn`). Sets up the
/// container, waits for `start`, then becomes the user's process.
/// Only returns if something failed.
pub fn init(id: &str, bundle: &Path) -> Result<()> {
    let spec = Spec::load(bundle)?;
    let flags = namespace_flags(&spec)?;

    // Without these namespaces, the steps below would change the HOST.
    if !flags.contains(CloneFlags::CLONE_NEWNS) {
        bail!("config must include a \"mount\" namespace");
    }
    if !spec.hostname.is_empty() && !flags.contains(CloneFlags::CLONE_NEWUTS) {
        bail!("hostname is set but there is no \"uts\" namespace");
    }

    // Grab the fifo now: after the root switch, host paths are gone.
    let fifo = open_fifo(id)?;

    if !spec.hostname.is_empty() {
        sethostname(&spec.hostname).context("set hostname")?;
    }

    setup_rootfs(&bundle.join(&spec.root.path))?;
    wait_for_start(fifo)?;
    exec_process(&spec)
}

/// Opens exec.fifo as a handle only (O_PATH), which doesn't block.
fn open_fifo(id: &str) -> Result<OwnedFd> {
    let path = common::root_path(id).join(EXEC_FIFO);
    open(&path, OFlag::O_PATH | OFlag::O_CLOEXEC, Mode::empty())
        .with_context(|| format!("open {}", path.display()))
}

/// Makes `rootfs` the container's `/` and mounts a fresh /proc.
fn setup_rootfs(rootfs: &Path) -> Result<()> {
    // Keep our mount changes from leaking back to the host.
    mount(None::<&str>, "/", None::<&str>, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None::<&str>)
        .context("make / private")?;

    // pivot_root needs the new root to be a mount point, so bind it to itself.
    mount(Some(rootfs), rootfs, None::<&str>, MsFlags::MS_BIND | MsFlags::MS_REC, None::<&str>)
        .with_context(|| format!("bind mount {}", rootfs.display()))?;

    // Switch roots: new root on top, old root stacked under it at the same
    // place, then detach the old one.
    chdir(rootfs).context("chdir to rootfs")?;
    pivot_root(".", ".").context("pivot_root")?;
    umount2(".", MntFlags::MNT_DETACH).context("detach old root")?;
    chdir("/").context("chdir to /")?;

    // A /proc that only shows the container's processes.
    mount(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), None::<&str>)
        .context("mount /proc")?;

    Ok(())
}

/// Blocks until `minirunc start` opens the fifo for reading.
fn wait_for_start(fifo: OwnedFd) -> Result<()> {
    // Reopen our O_PATH handle for writing; this is the call that waits.
    let path = format!("/proc/self/fd/{}", fifo.as_raw_fd());
    let mut writer = OpenOptions::new()
        .write(true)
        .open(&path)
        .context("open exec.fifo for writing")?;
    writer.write_all(b"0").context("write exec.fifo")?;
    Ok(())
}

/// Replaces this process with `process.args` from the spec.
fn exec_process(spec: &Spec) -> Result<()> {
    let (program, args) = spec
        .process
        .args
        .split_first()
        .context("process.args is empty")?;

    let mut cmd = Command::new(program);
    cmd.args(args).env_clear();
    for var in &spec.process.env {
        if let Some((key, value)) = var.split_once('=') {
            cmd.env(key, value);
        }
    }
    if !spec.process.cwd.is_empty() {
        cmd.current_dir(&spec.process.cwd);
    }

    // exec only returns on failure.
    let err = cmd.exec();
    Err(err).with_context(|| format!("exec {program}"))
}

/// Stack size for the cloned child. It only needs enough room to call execv.
const CHILD_STACK_SIZE: usize = 1024 * 1024;

/// Starts `minirunc init <id> <bundle>` as a child process inside the new
/// namespaces from the spec. Returns the child's pid (as seen from the host).
pub fn spawn(id: &str, bundle: &Path, spec: &Spec) -> Result<Pid> {
    let flags = namespace_flags(spec)?;

    // Re-run our own binary. execv needs C strings, built here in the parent.
    let exe = CString::new("/proc/self/exe")?;
    let args = [
        exe.clone(),
        CString::new("init")?,
        CString::new(id)?,
        CString::new(bundle.as_os_str().as_bytes())?,
    ];

    // Runs in the child. execv only returns if it failed.
    let child = Box::new(|| {
        let Err(err) = execv(&exe, &args);
        eprintln!("minirunc: exec init: {err}");
        127
    });

    let mut stack = vec![0u8; CHILD_STACK_SIZE];

    // SAFETY: no CLONE_VM, so the child gets its own copy of memory (like fork),
    // and it only calls execv. SIGCHLD lets us wait for it like a normal child.
    let pid = unsafe { clone(child, &mut stack, flags, Some(Signal::SIGCHLD as i32)) }
        .context("clone container process")?;

    Ok(pid)
}

/// Turns the namespaces listed in the spec into clone(2) flags.
pub fn namespace_flags(spec: &Spec) -> Result<CloneFlags> {
    let mut flags = CloneFlags::empty();

    for ns in &spec.linux.namespaces {
        flags |= match ns.kind.as_str() {
            "pid" => CloneFlags::CLONE_NEWPID,
            "mount" => CloneFlags::CLONE_NEWNS,
            "uts" => CloneFlags::CLONE_NEWUTS,
            "network" => CloneFlags::CLONE_NEWNET,
            "ipc" => CloneFlags::CLONE_NEWIPC,
            other => bail!("unsupported namespace {other:?}"),
        };
    }

    Ok(flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_with(kinds: &[&str]) -> Spec {
        let namespaces = kinds
            .iter()
            .map(|k| serde_json::json!({ "type": k }))
            .collect();
        let json = serde_json::json!({ "linux": { "namespaces": namespaces } });
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn maps_known_namespaces() {
        let flags = namespace_flags(&spec_with(&["pid", "mount", "uts"])).unwrap();
        assert_eq!(
            flags,
            CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUTS
        );
    }

    #[test]
    fn rejects_unknown_namespace() {
        assert!(namespace_flags(&spec_with(&["user"])).is_err());
    }
}
