# NeoOS: status and targets

**English** · [Srpski](STATUS.sr.md) · Updated 14 November 2026

This page tracks what is done and what is planned, area by area. It also sets the **contribution threshold**: the point at which the code moves from the private working repository into this one, so that others can build it, run it and add to it.

## Milestones

| Milestone | Meaning | State |
| --- | --- | --- |
| **M0: kernel seed** | A small kernel that boots, runs isolated Rust programs and passes an automatic boot test | Done |
| **M1: open for contributions** | The code moves here. Two architectures, programs loaded from disk, keyboard, screen, a minimal shell and a first window, with a contributor guide and starter tasks | 21 of 22 targets (95%) |
| **M2: Neo OS Lite alpha** | Boots from USB on a real PC (Neo Revive), network, Neo Kit windows, file manager, first everyday apps | Planned |
| **M3: Neo OS Lite beta** | Updates (Renewal), privacy (Lock), backups (Keeper), Treasury catalogue, Serbian and English complete | Planned |

## M1 targets by area

### Kernel (Fortress) · 5 of 6

- [x] Boots on x86_64 under QEMU, with a serial console and a boot test in CI
- [x] CPU exceptions, the kernel's own page tables, a stack guard page
- [x] All physical memory managed, a kernel heap
- [x] Timer, preemptive scheduler, sleeping and parking threads
- [x] Boots on AArch64 (QEMU `virt`) with the same boot test: page tables, timer, scheduler and message ports run there too
- [x] Decision recorded: when the bootstrap kernel gives way to Genode on seL4: accepted 2026-11-15, with the Genode trial started at once. The first Neo program already runs unchanged as a Genode component on seL4 under QEMU (2026-11-18)

### Programs and messages (Exec-N seed) · 5 of 5

- [x] Programs in user mode, each in its own address space, with system calls; a crash stops only that program
- [x] Named message ports and capabilities (send and receive rights, lookup permission)
- [x] Rust runtime for programs, ELF loader
- [x] Programs loaded from a disk image instead of being built into the kernel (boot image and virtio disk)
- [x] Shared-memory rings for large messages: a lock-free ring in a shared block with a port as the doorbell; a self-test sends a megabyte through a 64 KiB ring on both architectures

### Drivers and services · 4 of 4

- [x] Console service in user mode: the only program that touches the serial port
- [x] Keyboard input: PS/2 driver in user mode on x86_64, virtio-input on AArch64; US and Serbian Latin layouts (Alt+Shift or a click on the Bar switches; the Bar shows US or SR), with č, ć, ž, š, đ typed into Neo DOS and its window
- [x] Simple framebuffer service (pixels on screen): display service in user mode on x86_64 (VGA) and AArch64 (ramfb), the same program on both
- [x] Block device (virtio) and a read-only file system

### Shell and desktop · 2 of 2

- [x] Neo DOS, minimal: list, show and run files (Neo DOS 0.2, with the BOOT: and DH0: volumes, on both architectures; the command line can be edited with the arrows, Home, End and Delete, and Up and Down bring back earlier commands)
- [x] First window on screen and the Clock tile (desktop roadmap D0 to D3): the window is drawn and can be dragged with the mouse, and the Clock tile ticks, on both architectures

### Ready for contributors · 2 of 5

- [x] One-command boot test, run by CI on every change
- [x] Step-by-step progress log
- [x] Contributor guide: building, code style, how to add a program or a service (in English and Serbian; it moves here with the code)
- [x] Licence chosen for the code: GNU GPL version 3 or later, for this repository and the development one
- [ ] At least ten starter tasks as issues, each with a clear test: twelve are written, each with its test, and are filed as issues the day the code moves here

## After M1

- **Hardware path:** UEFI boot on real PCs, then Raspberry Pi 4/5, then the Neo 100.
- **Desktop:** compositor, tiles (Clock, Calculator, Meter), Shelf and Bar, drag and drop, Neo menu and command palette, then the Neo Silver look.
- **Files:** the dual-pane file manager, compression formats, Btrfs and EROFS.
- **Everyday apps:** browser, office, mail, photos and music through mature open projects.

## Overall progress against the specification

The specification estimates NeoOS 1.0 at 5 350 person-months. The coding done so far is about 2 412 engineering hours, roughly 15.1 person-months or 0.28% of that estimate. This counts only the kernel seed. Part of it may be replaced when the system moves to Genode on seL4, so it is an upper bound.
