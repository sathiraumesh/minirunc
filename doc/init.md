# `minirunc init`: setup inside the container

Code: `src/cmd/init.rs` → `src/os/linux.rs` (`init` and helpers).

```
minirunc init <id> <bundle>     (hidden; started only by spawn)
```

`init` is the first process inside the new namespaces. It sets the container
up, waits for `start`, then **replaces itself** with the user's program.

It's hidden from `--help` because nobody should run it by hand. On non-Linux
systems it just returns an error.

## Steps

### 1. Load the config and check namespaces

```rust
if !flags.contains(CLONE_NEWNS)  { bail!(...) }   // need "mount"
if hostname set && !CLONE_NEWUTS { bail!(...) }   // need "uts"
```

**Why:** without a mount namespace, the rootfs switch below would change
the **host's** mounts. Without a uts namespace, setting the hostname would
rename the **host**. These checks stop that.

### 2. Grab a handle to `exec.fifo`

```rust
open(path, O_PATH | O_CLOEXEC, ...)
```

- `O_PATH`: get a reference to the file without really opening it,
  so this **doesn't block**.
- `O_CLOEXEC`: closed automatically when the user's program starts.

**Why now:** after step 4 the host's folders are gone from view, so
`/run/minirunc/<id>/exec.fifo` can't be reached by path any more. A handle
opened before still works.

### 3. Set the hostname

```rust
sethostname(&spec.hostname)
```

Only changes the container's hostname, because it's in its own uts namespace.

### 4. Switch the root filesystem

`setup_rootfs` makes the bundle's `rootfs` folder the container's `/`:

| Step | Why |
|---|---|
| mount `/` as `MS_PRIVATE` (recursive) | so mount changes don't leak back to the host |
| bind-mount `rootfs` onto itself | `pivot_root` requires the new root to be a mount point |
| `chdir(rootfs)` + `pivot_root(".", ".")` | swap roots: rootfs becomes `/`, old root is stacked underneath |
| `umount2(".", MNT_DETACH)` | detach the old root so the host filesystem is unreachable |
| `chdir("/")` | move into the new root |
| mount `proc` on `/proc` | a `/proc` showing only the container's processes |

**`pivot_root` vs `chroot`:** `chroot` only changes how paths are looked
up and is easy to escape. `pivot_root` really swaps the root mount, and
detaching the old root removes the host filesystem completely.

### 5. Wait for `start`

```rust
wait_for_start(fifo)?;
```

Re-opens the fifo for writing through `/proc/self/fd/<n>`. That open
**blocks** until `minirunc start` opens the other end.
See [exec-fifo.md](exec-fifo.md).

### 6. Run the user's program

`exec_process` builds a `Command` from `config.json`:

| config.json | Rust |
|---|---|
| `process.args` | `Command::new(args[0]).args(rest)` |
| `process.env` | `env_clear()` then `env(key, value)` for each `KEY=VALUE` |
| `process.cwd` | `current_dir(cwd)` |

then calls `exec()`, which **replaces** the init process with the program.
The PID stays the same, so the program becomes PID 1 in the container.

`exec()` only returns if it failed (e.g. program not found).

**Why `env_clear`:** the program should see only the variables from the
config, not the host's.
