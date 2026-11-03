# `minirunc create`

```
sudo minirunc create <id> --bundle <path>
```

Builds a container from a bundle **without starting its program**.

Code: `src/cmd/create.rs` → `src/container/state.rs`.

## Steps

### 1. Check for root

```rust
if !geteuid().is_root() { bail!("create: must run as root"); }
```

**Why:** making namespaces, mounting filesystems and writing to `/run`
all need root. Failing early gives a clear message instead of a confusing
"permission denied" halfway through.

### 2. Make the bundle path absolute

```rust
let bundle = path::absolute(bundle)?;   // "." → "/home/me/mybundle"
```

**Why:** the bundle path is passed to the container's `init` process later.
That process may run from a different directory, where `.` would mean
something else. An absolute path means the same thing everywhere.

### 3. Load `config.json`

```rust
let spec = Spec::load(&bundle)?;
```

Reads `<bundle>/config.json`, parses it, and checks that `process.args`
and `root.path` are not empty.

**Why first:** if the config is broken, we fail **before** creating
anything on disk, so nothing needs cleaning up.

### 4. Create the state directory

```rust
let dir = container::create(id)?;   // → /run/minirunc/<id>
```

Inside `container::create`:

1. **Validate the id**: only letters, digits, `-`, `_`, `.`; not `.` or
   `..`.
   **Why:** the id becomes part of a path. An id like `../../etc` would
   otherwise escape the root dir.
2. **Make the root dir** (`/run/minirunc`) if missing.
3. **Make the container dir** (`/run/minirunc/<id>`), permissions `0700`.
   This step fails if the folder already exists, which becomes
   `container <id> already exists`.
   **Why not check `exists()` first:** two `create` calls at the same moment
   could both pass the check. Letting the OS refuse is race-free.
4. **Make `exec.fifo`** inside it, permissions `0600`.
   See [exec-fifo.md](exec-fifo.md) for what it is for.
   If this fails, the container dir is deleted, so a failed `create` leaves
   nothing behind.

### 5. (Next) Spawn the container process

Not wired in yet. `create` will call `os::spawn`, which starts
`minirunc init` inside new namespaces, then save the returned PID.
See [spawn.md](spawn.md).

## Errors you may see

| Message | Cause |
|---|---|
| `create: must run as root` | run with `sudo` |
| `read <bundle>/config.json` | bundle path wrong or no config.json |
| `invalid <bundle>/config.json` | `process.args` or `root.path` empty |
| `invalid container id "..."` | id has `/`, spaces, etc. |
| `container <id> already exists` | id already used; pick another or delete `/run/minirunc/<id>` |
