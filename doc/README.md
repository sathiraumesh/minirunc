# minirunc docs

How minirunc works, one topic per file.

| File | Topic |
|---|---|
| [overview.md](overview.md) | The big picture: what a container is and how the commands fit together |
| [create.md](create.md) | `minirunc create`: what it does, step by step |
| [spawn.md](spawn.md) | `spawn` and `clone`: starting a process inside new namespaces |
| [init.md](init.md) | `minirunc init`: the setup that runs inside the container |
| [exec-fifo.md](exec-fifo.md) | `exec.fifo`: why the container waits for `start` |
| [testing.md](testing.md) | Running minirunc in the Vagrant Linux VM |

## Where the code lives

```
src/
├── main.rs            entry point; sets MINIRUNC_ROOT
├── cmd/               one file per CLI command
│   ├── root.rs        the list of subcommands (clap)
│   ├── create.rs      `create`
│   └── init.rs        `init` (hidden)
├── container/         container state on disk
│   ├── spec.rs        config.json → Spec
│   └── state.rs       state dir + exec.fifo
├── common/utils.rs    root dir helpers
└── os/linux.rs        Linux-only: namespaces, spawn, init
```
