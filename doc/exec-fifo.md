# `exec.fifo`: why the container waits

File: `/run/minirunc/<id>/exec.fifo`, made by `create`.

## What a FIFO is

A FIFO (named pipe) looks like a file but is a **meeting point between two
processes**. Nothing is stored on disk.

The key rule: **opening it for writing blocks until someone opens it for
reading** (and the other way round).

## How minirunc uses it

```
create                     init (in container)              start
──────                     ───────────────────              ─────
make exec.fifo
spawn init ─────────────►  setup...
exit                       open fifo for writing  ⏸ blocks
                                   │
                                   │◄────────────────────── open fifo for reading
                           unblocked, write "0"
                           exec /bin/sh
```

The blocked process uses **no CPU**. The kernel puts it to sleep.

## Why wait at all?

It follows runc and the OCI standard, which split starting a container into
**two steps**:

| Step | State afterwards |
|---|---|
| `create` | container fully set up, program **not running yet** |
| `start` | program running |

The gap between them lets other tools prepare the container **before any
user code runs**, using its PID:

- set up networking (create an interface, move it into the container)
- apply cgroup limits (memory, CPU, number of processes)
- run hooks

If the program started straight away, it would briefly run without its
network or limits.

**Race analogy:** `create` = runners at the starting line; the wait = holding
still while officials check everything; `start` = the starting gun.

## Do you need it?

Not strictly. A simpler runtime can have a single `run` command that sets up
and starts the program at once, with no fifo and no `start`. minirunc keeps
the two-step model because it follows runc.

## Testing without `start`

Until `start` exists, unblock a created container by hand:

```
sudo cat /run/minirunc/<id>/exec.fifo
```

`cat` opens the fifo for reading, which is exactly what `start` will do.
