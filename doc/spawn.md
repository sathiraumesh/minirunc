# `spawn`: starting the container process

Code: `src/os/linux.rs`, `spawn` and `namespace_flags`.

```rust
pub fn spawn(id: &str, bundle: &Path, spec: &Spec) -> Result<Pid>
```

Starts `minirunc init <id> <bundle>` as a child process **already inside**
new namespaces, and returns its PID.

## Namespaces

A namespace gives a process its own private copy of one part of the system.

| config.json | Flag | The container gets its own... |
|---|---|---|
| `pid` | `CLONE_NEWPID` | process list (it is PID 1, can't see host processes) |
| `mount` | `CLONE_NEWNS` | filesystem mounts |
| `uts` | `CLONE_NEWUTS` | hostname |
| `network` | `CLONE_NEWNET` | network interfaces, IPs, ports |
| `ipc` | `CLONE_NEWIPC` | shared memory, message queues |

## Step 1: names → flags

`namespace_flags` turns the list in `config.json` into one `CloneFlags` value.
Each flag is one bit; `|=` switches that bit on:

```
"pid"   → flags = 0b001
"uts"   → flags = 0b011
"mount" → flags = 0b111
```

An unknown name (e.g. `"user"`) is an error rather than silently ignored.

## Step 2: `clone`

`clone` is a Linux system call that creates a new process, like `fork`,
but with flags choosing what the child gets fresh copies of.

```rust
unsafe { clone(child, &mut stack, flags, Some(SIGCHLD)) }
```

| Argument | Meaning |
|---|---|
| `child` | function the child runs (it calls `execv`, see below) |
| `stack` | memory for the child's stack (1 MB) |
| `flags` | which namespaces to create |
| `SIGCHLD` | parent is notified when the child exits, like a normal child |

- In the **parent**, `clone` returns the child's PID.
- The **child** runs the `child` function instead.

**Why `unsafe`:** Rust can't check `clone` is used correctly. It is safe
here because the child gets its own copy of memory (no `CLONE_VM`) and only
calls `execv`.

## Step 3: the child re-runs minirunc

The child immediately does:

```
execv /proc/self/exe  init  <id>  <bundle>
```

`/proc/self/exe` always points to the running program, so the child becomes
a fresh copy of minirunc, running the hidden `init` command.

### Why run itself again?

Some setup must happen **inside** the new namespaces before the user's
program runs (hostname, rootfs, `/proc`, waiting for `start`):

- The **parent** can't do it: it's outside, so it would change the host.
- The **user's program** (`/bin/sh`) can't do it: it doesn't know how.
- So the child becomes **minirunc again**, in `init` mode, to do the setup.

Starting a fresh program (instead of running setup code straight after
`clone`) also means the child starts with clean memory and none of the
parent's state. runc works the same way.

See [init.md](init.md) for what `init` does.
