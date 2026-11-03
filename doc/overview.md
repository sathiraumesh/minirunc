# Overview

## What is a container?

A container is a **normal Linux process** with two things added:

1. **Namespaces**: the process gets its own private view of parts of the
   system (process list, hostname, filesystem mounts, network, ...).
2. **A different root filesystem**: the process sees a folder (the bundle's
   `rootfs`) as `/`.

There is no "container" object in the kernel. A runtime like minirunc just
starts a process in the right way.

## The bundle

A bundle is a folder holding everything needed to run a container:

```
mybundle/
├── config.json   what to run and how (OCI format)
└── rootfs/       the container's filesystem (/bin, /etc, ...)
```

`config.json` is parsed into the `Spec` struct (`src/container/spec.rs`).

## The commands

minirunc follows runc's two-step model:

| Command | What it does |
|---|---|
| `create <id>` | Builds the container but does **not** run the program yet |
| `start <id>` | Tells the waiting container to run its program *(not written yet)* |
| `init` | Internal. Runs inside the container to set it up. Never typed by hand |

Why two steps? See [exec-fifo.md](exec-fifo.md).

## The full flow

```
HOST                                   INSIDE THE CONTAINER
────                                   ────────────────────
minirunc create abc
  ├─ check root, load config.json
  ├─ make /run/minirunc/abc/ + exec.fifo
  └─ spawn ──── clone(namespaces) ───► minirunc init abc /bundle
     (returns the PID, create exits)     ├─ set hostname
                                         ├─ switch / to rootfs, mount /proc
                                         └─ wait on exec.fifo   ⏸
minirunc start abc                             │
  └─ open exec.fifo for reading ──────────────►│ unblocked
                                         └─ exec /bin/sh  (the user's program)
```

## Where state is kept

Each container gets a folder under the root dir:

```
/run/minirunc/        root (override with MINIRUNC_ROOT)
└── abc/              one folder per container id
    └── exec.fifo
```

`MINIRUNC_ROOT` lets you use another folder, for example in tests.

## Current status

| Part | Status |
|---|---|
| `create`: config, state dir, fifo | done |
| `spawn` | written, **not yet called** by `create` |
| `init` | written, not yet run on Linux |
| `start` | not written |
| saving the PID | not written |
