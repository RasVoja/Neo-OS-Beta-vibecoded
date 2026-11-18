# Decision 0001: when Fortress gives way to Genode on seL4

*Srpski: [0001-fortress-and-genode.sr.md](0001-fortress-and-genode.sr.md)*

**Status:** accepted (2026-11-15), with one change by the project owner: the Genode trial (the third trigger below) starts now, before M1 closes, instead of waiting for the other two triggers.

**Trial:** the first neo-rt program runs unchanged as a Genode component on seL4 under QEMU (`genode/`, 2026-11-18).

## Context

The README plans Neo Boing Platinum OS on the Genode OS framework over the seL4
microkernel. Fortress, the kernel in this repository, began as a bootstrap:
a small kernel to learn the hardware path and to have something testable
from the first day. Forty steps in, it has two architectures, user-mode
drivers, capabilities, message ports, shared memory and rings, a desk with
windows, and Neo DOS, all covered by boot tests in CI.

Staying on Fortress for too long would mean writing, by ourselves, what
seL4 already proves (isolation, a verified kernel) and what Genode already
has (hundreds of drivers, a GUI stack, ports of Mesa and Chromium-class
software). Switching too early would stop the visible progress that brings
contributors, while Genode's own learning curve is steep.

## Options

1. **Switch now.** Stop Fortress work and start over on Genode/seL4.
2. **Never switch.** Grow Fortress into the full kernel.
3. **Switch at a clear point, keeping the user side.** Build the user side
   (programs, `neo-rt`, `neo-gfx`, `neo-keys`, `neo-ring`, the display
   service, Neo DOS) so that it does not depend on Fortress's internals,
   then move it onto Genode when the triggers below are met. Fortress
   stays as the teaching kernel and the "Revive" fallback.

## Decision

Option 3. The user side keeps talking to the kernel only through `neo-rt`,
so `neo-rt` is the one place that changes when the kernel does. Its calls
map onto Genode sessions as follows:

| `neo-rt` today | On Genode |
| --- | --- |
| ports, `send`, `recv`, `try_send` | RPC and signals between components |
| `share`, `send_handle`, `map_device` | dataspaces and capability passing |
| `neo-ring` over a shared block | a dataspace plus a signal (the Genode packet stream is the same idea) |
| files, volumes | the VFS and file system sessions |
| display protocol, windows | a GUI session (nitpicker or its successor) |
| keyboard, pointer, layout | input sessions; `neo-keys` keeps the layouts |
| `spawn`, capabilities given at start | the init component's routing |

**Triggers for the switch.** Start the move when *all three* hold:

- M1 is closed: the code is public and contributors have starter tasks.
- A first real hardware target is chosen (a PC with UEFI or a Raspberry
  Pi), because that is where Fortress would need drivers Genode already
  has.
- One Neo program (the display service or Neo DOS) has been built against
  Genode as a trial, through a `neo-rt` back end, and its boot test passes
  under QEMU.

Until then, new Fortress kernel work is limited to what the user side
needs and to keeping the tests green. No new kernel drivers are written
in Fortress that Genode already has.

## Consequences

- Kernel-side hours on Fortress count toward the specification's Fortress
  and Exec-N items only in part, as the hours report already says.
- `neo-rt` must not leak Fortress details (handle numbers, call numbers)
  into programs beyond what is documented.
- The boot tests stay; under Genode they become run scripts that check the
  same lines.
