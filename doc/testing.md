# Testing

## Unit tests (any OS)

```
cargo test
```

Linux-only code (`src/os/linux.rs`) is skipped on macOS. To at least
type-check it from a Mac:

```
rustup target add aarch64-unknown-linux-gnu    # once
cargo check --target aarch64-unknown-linux-gnu
```

## Running on Linux (Vagrant)

Namespaces and mounts only exist on Linux, so the real commands run in a VM.
The `Vagrantfile` in the project root sets up Ubuntu 24.04 with:

- Rust
- a test bundle at `/home/vagrant/bundle` (Alpine Linux rootfs + `config.json`)
- a `test-create` script

```
vagrant up                  # first time: create + set up the VM (several minutes)
vagrant ssh -c test-create  # build and run `create`
vagrant ssh                 # shell inside the VM
vagrant destroy -f          # delete the VM
```

The project folder is shared into the VM at `/vagrant`, so edits on the Mac
show up there immediately. The VM builds into `~/target` so it doesn't
clash with the Mac's `./target`.

### What `test-create` checks

1. `create test` succeeds and makes `/run/minirunc/test/` with `exec.fifo`
2. `create test` again fails: `container test already exists`
3. `create` without `sudo` fails: `must run as root`

Pass a different id with `vagrant ssh -c "test-create myid"`.
