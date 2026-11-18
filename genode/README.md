# Neo on Genode (trial)

Decision 0001 keeps Fortress as Neo Lite's kernel for now and starts a
trial of Neo programs on Genode and seL4 straight away. This folder is
that trial.

- `neo-hello/`, `neo-echo-server/`, `neo-echo-client/` are ordinary neo-rt
  programs, built as static libraries with neo-rt's `genode` feature (one
  Cargo workspace, `Cargo.toml` here). They contain no Genode code.
- `repos/neo/` is a Genode repository:
  - `lib/neo_genode` is the runtime every Neo component links. It
    implements neo-rt's system calls with Genode services: `exit` goes to
    the parent, `print` to the LOG session, `sleep` to the Timer, and the
    port calls (`find_port`, `new_port`, `send`, `try_send`, `recv`,
    `close`) to the `neo_ports` server. Every other call answers "unknown
    call", as Fortress does for a call it lacks. A component's starting
    handles come from its config (`<port handle="0" name="echo"
    create="yes"/>`), the way Fortress hands a program its handles.
  - `server/neo_ports` holds the ports of all Neo components, by name.
  - `app/neo_hello`, `app/neo_echo_server`, `app/neo_echo_client` are the
    programs' components.
  - `run/neo_hello.run` boots one program; `run/neo_echo.run` boots the
    echo pair, which exchange 100 messages through two ports.

## Running it

1. Get Genode 24.11 and its toolchain 23.05, then prepare the ports:
   `tool/ports/prepare_port sel4 grub2`.
2. `tool/create_builddir x86_64`, then in `build/x86_64/etc/build.conf`
   set `KERNEL ?= sel4`, `BOARD ?= pc`, comment out `-no-kvm` if your
   QEMU rejects it, and add
   `REPOSITORIES += /path/to/NeoOS/genode/repos/neo`.
3. `GENODE_DIR=/path/to/genode scripts/test-genode-x86_64.sh`

The expected output includes:

```
[init -> neo_hello] Hello from Rust on Genode!
[init -> neo_hello] fib(50) = 12586269025
[init -> neo_hello] find_port: not provided (as expected)
[init] child "neo_hello" exited with exit value 0
[init -> neo_echo_client] echo client: 100 of 100 answers right
[init -> neo_echo_server] echo server: answered 100 messages
```

`neo_hello` has no route to `neo_ports`, so Genode also logs that the
session could not be made; that is the point of its `find_port` check.

## Next

Waiting is polled once a millisecond for now; a signal from `neo_ports`
will replace it. Then shared memory (Genode dataspaces) and passing
handles in messages, and after that the neodos shell.
