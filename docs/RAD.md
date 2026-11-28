# RAD: in the base version

**English** · [Srpski](RAD.sr.md) · Plan, 27 November 2026

`RAD:` is the fast disk in memory that keeps its contents across a restart (the "Worktable" in Easy mode). The owner decided it belongs in the **base Lite version**, not in concept art. This page splits it into levels that software can deliver on an ordinary x86_64 PC or ARM board.

| Level | What | Needs | State |
| --- | --- | --- | --- |
| 1 | `RAM:` and `RAD:` as memory-backed volumes in Neo DOS and Cabinet; every block has a checksum, a damaged volume is not mounted and a message explains why | the memory-backed block service, the file system service | to do |
| 2 | "Save RAD: at shutdown": the contents are written to disk (compressed) and come back at next start | a writable storage driver (NVMe or AHCI), which does not exist yet | blocked |
| 3 | **Survives a restart that Neo starts itself** (restart after an update, "Restart" in the menu): Fortress keeps a reserved region, jumps back to its entry point without the firmware resetting memory, finds the region by a header and checks it | a reserved region from the memory map, a restart path that re-initialises devices (xHCI and others), a test in QEMU | to do |

**What a plain PC cannot promise:** a hardware reset or a power cycle may clear memory, or may not; the firmware decides. So level 3 is promised only for restarts that Neo itself performs. Everything else falls back to level 2, or to an honest message.

## Settings

- Size: a share of memory, adjustable. A suggestion is one eighth of RAM by default (4 GB on a 32 GB PC, 1 GB on an 8 GB Raspberry Pi 5); only the files it holds use memory.
- Lock labels apply. Without a hardware key store, files labelled Private are not kept in `RAD:` unless it is encrypted with a passphrase.
- Boot from it, as on the Amiga, for quick tests of images.

## Test

A CI boot test that writes to `RAD:`, restarts through Fortress, and reads the content back, in QEMU with BIOS and with UEFI.

## Concept hardware (separate)

> A `RAD:` that survives power loss (a fourth level, with an ultracapacitor or battery and a boot ROM) belongs to the concept computers only (concept hardware, not needed; [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md)).
