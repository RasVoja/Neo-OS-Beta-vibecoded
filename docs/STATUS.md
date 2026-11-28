# Neo Boing Platinum OS: status and targets

**English** · [Srpski](STATUS.sr.md) · Updated 27 November 2026

This page tracks what is done and what is planned, area by area. It also sets the **contribution threshold**: the point at which the code moves from the private working repository into this one, so that others can build it, run it and add to it. The threshold was reached on 18 November 2026, and development continues here.

## Tags

Every item below and in [FEATURES.md](FEATURES.md) carries one tag: **[done]** works and is tested in CI · **[next]** being built now · **[lite]** planned for Neo Boing Platinum OS Lite on generic x86_64 and ARM, can enter the plan now · **[concept]** needs Neo hardware that does not exist (concept hardware). Anything that does not need Neo hardware is [lite].

## Milestones

| Milestone | Meaning | State |
| --- | --- | --- |
| **M0: kernel seed** | A small kernel that boots, runs isolated Rust programs and passes an automatic boot test | [done] |
| **M1: open for contributions** | The code moves here. Two architectures, programs loaded from disk, keyboard, screen, a minimal shell and a first window, with a contributor guide and starter tasks | [done], 22 of 22 targets (18 November 2026) |
| **M2: Neo Boing Platinum OS Lite alpha** | Boots from USB on a real PC (Neo Revive), network, Neo Kit windows, file manager, first everyday apps, the Linux box with Flatpak (first games, Album, FilmCraft, PhotoCraft, qBittorrent) | [next]: real-PC boot work has started |
| **M3: Neo Boing Platinum OS Lite beta** | Updates (Renewal), privacy (Lock), backups (Keeper), Treasury catalogue, Help program with F1, Serbian and English complete | [lite] |
| **AmigaOS feature set** | AROS API layer, shell and C: commands, Zune, Wanderer, Directory Opus 5, FS-UAE and Janus-UAE, about 106 person-months net; see [AMIGA.md](AMIGA.md) | [lite]: FS-UAE and shell commands with M2, the rest after M3 |
| **Classic Mac look and software** | Classic windows and controls for the "First Desk" theme, classic Mac emulators in Arcade, an evaluation of the System 7 reimplementation (Kelsidavis/System7, MIT); about 13 person-months; see [CLASSIC-MAC.md](CLASSIC-MAC.md) | [lite]: the theme with M3, the rest after M3 |
| **Games, creative and music apps in Lite** | Ready-made games through Flatpak, Arcade (MAME, DOSBox, ScummVM, RetroArch), Neo's own games; FilmCraft (Kdenlive), PhotoCraft (GIMP and Inkscape), Album (gThumb); HippoPlayer, qBittorrent, Hydrogen, tracker; see [GAMES.md](GAMES.md), [CREATIVE-APPS.md](CREATIVE-APPS.md), [MUSIC.md](MUSIC.md) | [lite]: Flatpak programs with M2, Neo Kit versions after the desktop |
| **Concept computer** | Neo 500, 2000, 4000, Neo 100, the Neo package of chips, FPGA cores; see [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md) | [concept]: not a blocker for the system |

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
- [x] At least ten starter tasks as issues, each with a clear test: twelve, issues #3 to #14

## After M1

- **Hardware path [next]:** UEFI boot on real PCs, then Raspberry Pi 4/5 [lite], then the Neo 100 [concept].
  - [x] A bootable PC image (ISO for CD or USB stick): GRUB loads Fortress with Multiboot2, on BIOS and UEFI; tested in CI with SeaBIOS and OVMF
  - [x] The desk on the screen the boot loader sets up (UEFI GOP or VBE), tested with no VGA card at all
  - [x] USB keyboard and mouse (xHCI controller, device setup, HID boot reports, interrupts by MSI or MSI-X)
  - [x] Real-PC groundwork: every PCI bus and high BARs, USB hubs, ACPI tables (CPUs, interrupt routing, power off)
  - [x] Ready for testers: USB devices plugged in and pulled out while running (on the PC's ports and behind hubs), the boot messages and the USB and PCI devices on screen (`BOOTLOG`, `USB`, `PCI` in Neo DOS), a test guide and a report template ([REAL-PC-TEST.md](REAL-PC-TEST.md))
  - [ ] First boot on real PCs
- **Genode on seL4 (decision 0001) [next]:**
  - [x] An unchanged Neo program runs as a Genode component
  - [x] Ports and messages between Neo programs (the neo_ports server)
  - [ ] Wake-up by signal, shared memory, handles in messages; then Neo DOS
- **Desktop [next]:** [done] Dijana, the logo and first resident, lives on the Bar (greets, sniffs, sleeps, answers a click, gives a tip after an error, says good night at shutdown); [done] the compositor's first piece (the desk is drawn in memory and copied to the screen, no flicker); [done] the Meter tile (processor, memory, threads) under the Clock and the Calculator tile; next the rest of the compositor, Shelf and Bar, drag and drop, Neo menu and command palette, then the Neo Silver look.
- **Files [lite]:** Cabinet Pro (dual pane, Magellan rules), Crunch (compression formats), Btrfs and EROFS.
- **Everyday apps [lite]:** browser, office, mail, Album (gThumb), Gramophone with HippoPlayer, qBittorrent, through mature open projects in the Linux box first.
- **Games [lite]:** SuperTuxKart, 0 A.D., Luanti, Frozen Bubble, Battle for Wesnoth through Flatpak; Arcade with MAME, DOSBox, ScummVM, RetroArch; then Neo's own games ([GAMES.md](GAMES.md)).
- **Help [lite]:** the user guide as HTML and PDF already exists in [guide/](../guide/html-en/index.html) [done]; the Help program, F1 and pictures of the real screen come with the desktop ([HELP.md](HELP.md)).
- **Concept computer [concept]:** Neo boards and chips stay concept hardware; Lite uses software versions instead ([CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md)).

## Overall progress against the specification

The specification estimates Neo Boing Platinum OS 1.0 at 5 350 person-months. The coding done so far is about 3 100 engineering hours, roughly 19.4 person-months or 0.36% of that estimate. This counts the kernel seed and the first desk (with Dijana). Part of it may be replaced when the system moves to Genode on seL4, so it is an upper bound.
