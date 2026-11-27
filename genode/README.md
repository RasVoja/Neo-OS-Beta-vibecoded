# Neo on Genode (trial)

Decision 0001 keeps Fortress as Neo Lite's kernel for now and starts a
trial of Neo programs on Genode and seL4 straight away. This folder is
that trial.

- `neo-hello/` is an ordinary neo-rt program, built as a static library
  with neo-rt's `genode` feature. It contains no Genode code.
- `repos/neo/` is a Genode repository. Its `neo_hello` component links the
  Rust library and implements neo-rt's system calls with Genode services:
  `exit` goes to the parent, `print` to the LOG session, and every other
  call answers "unknown call", as Fortress does for a call it lacks.
- `repos/neo/run/neo_hello.run` boots Genode with that one component and
  waits for it to exit with code 0.

## Running it

1. Get Genode 24.11 and its toolchain 23.05, then prepare the ports:
   `tool/ports/prepare_port sel4 grub2`.
2. `tool/create_builddir x86_64`, then in `build/x86_64/etc/build.conf`
   set `KERNEL ?= sel4`, `BOARD ?= pc`, comment out `-no-kvm` if your
   QEMU rejects it, and add
   `REPOSITORIES += /path/to/NeoOS/genode/repos/neo`.
3. `GENODE_DIR=/path/to/genode scripts/test-genode-x86_64.sh`

The expected output ends with:

```
[init -> neo_hello] Hello from Rust on Genode!
[init -> neo_hello] fib(50) = 12586269025
[init -> neo_hello] find_port: not provided (as expected)
[init -> neo_hello] neo_hello done
[init] child "neo_hello" exited with exit value 0
```

## Next

Ports, messages and memory sharing map to Genode RPC and dataspaces (see
the table in decision 0001); each call that gets mapped stops answering
"unknown call". The neodos shell is the next program to try.
