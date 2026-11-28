# NBPOS feature list

**English** · [Srpski](FEATURES.sr.md) · Updated 27 November 2026

One page with everything NBPOS (Neo Boing Platinum OS) does or plans to do, tagged by scope. Details live in [STATUS.md](STATUS.md) (milestones), the specification (PDF) and the plan pages linked at the end.

## Scope rule

**Lite first.** Neo Boing Platinum OS Lite on x86_64 and ARM takes in everything from the earlier full and pro design that software can do on today's hardware. The goal is to prove the system on an existing platform first.

Everything that needs Neo hardware that does not exist (Neo 500, 2000, 4000, the Neo package, FPGA cores: concept hardware) stays **concept art**. It is kept in its own section here, and in documents, videos and examples it carries a visible "concept" label so it is never mistaken for what runs.

Tags: **[done]** works and is tested in CI · **[next]** being built · **[lite]** planned for Lite, can enter the plan now · **[concept]** needs hardware that does not exist (concept hardware).

Rule for the tags: anything that does not need Neo hardware is **[lite]** and can go into the development plan now.

## At a glance

| Area | [done] | [next] | [lite] | [concept] |
| --- | --- | --- | --- | --- |
| Kernel and programs | Fortress on x86_64 and AArch64, user programs, ports, capabilities | Genode on seL4 | Renewal, Chronos, Lock | |
| Boot and drivers | ISO for BIOS and UEFI, GOP desk, USB keyboard and mouse, PCI, ACPI | First boot on real PCs | Network, sound, storage, Raspberry Pi 4/5 | Neo boards, Satellite module |
| Desktop | First window, Clock and Meter tiles, Dijana, Neo DOS 0.3 | Compositor, Tiles, Shelf and Bar | Neo Kit, Neo Silver, First Desk, live language switch | Glass effects in hardware |
| Apps | | | Cabinet Pro, Album (gThumb), FilmCraft (Kdenlive), PhotoCraft (GIMP and Inkscape), office, mail, web | |
| Games | | | Neo's own games, Flatpak games, Arcade (MAME, DOSBox, ScummVM, RetroArch) | FPGA cores |
| Music | | | HippoPlayer, qBittorrent, Hydrogen, tracker, beatmaker | Sound chip |
| Help | Beta manual, HTML and PDF guide | | Help program, F1, pictures of the real screen | |

## 1. Works now [done]

- Fortress kernel boots on x86_64 and AArch64 (QEMU), with a serial console and a boot test in CI.
- CPU exceptions, own page tables, stack guard page, all physical memory managed, kernel heap, timer, preemptive scheduler.
- Programs in user mode, one address space each; a crash stops only that program.
- Named message ports and capabilities; shared-memory rings for large messages.
- Rust runtime (`neo-rt`), ELF loader, programs loaded from a disk image.
- User-mode services: console, keyboard (PS/2, virtio-input), framebuffer display, virtio block device with a read-only file system.
- Keyboard layouts US and Serbian Latin (č, ć, ž, š, đ), switched with Alt+Shift.
- Neo DOS 0.3 with BOOT: and DH0: volumes, line editing and history, BOOTLOG, USB and PCI.
- First window, dragged with the mouse, the Clock and Meter tiles, drawn through a back buffer (no flicker); Dijana on the Bar, with tips after errors and a good night at shutdown.
- Bootable hybrid ISO for BIOS and UEFI (GRUB, Multiboot2), tested in CI with SeaBIOS and OVMF.
- The desk on the screen the boot loader sets up (UEFI GOP or VBE).
- USB keyboard and mouse (xHCI, hubs, HID boot reports, hot-plug on ports and behind hubs); PCI buses and high BARs; ACPI tables (CPUs, interrupt routing, power off).
- Genode on seL4 trial: an unchanged Neo program runs as a Genode component; the `neo_ports` server.
- Contributor guide, twelve starter tasks, GPLv3-or-later.

## 2. Being built [next]

- First boot on real PCs (see [BUILD-ISO.md](BUILD-ISO.md)).
- Genode on seL4: wake-up by signal, shared memory, handles in messages, then Neo DOS.
- Desktop: the rest of the compositor, more tiles (Calculator), Shelf and Bar, drag and drop, Neo menu, command palette, then the Neo Silver look.

## 3. Planned for Lite [lite]

**System**
- Renewal (one signed A/B update with one-click return), Chronos (snapshots, "back to yesterday"), Lock (per-app privacy, outgoing traffic log), Keeper (backups), Find (one local index), Treasury (open-source app catalogue with reproducible builds).
- Linux box (applications domain) for Flatpak and POSIX programs; Bridge (FEX/Box64, Wine, Proton) on ARM; Android room (1.1).
- Native networking, audio and storage drivers for common hardware; Linux driver code only inside isolated components.
- Neo Revive: the system from a USB stick on old 64-bit UEFI PCs and Raspberry Pi 4/5 class boards, with a checked-computers list.
- Neighbourhood features: file sending to nearby computers, mail across the neighbourhood, borrowing power from bigger Neo machines (Co-op).
- RAD: in the base version, in levels ([RAD.md](RAD.md)).

**Desk and manner**
- Neo Kit (one toolkit, Easy/Pro switch), Lux (scripting port in every app), Datatypes (1 000+ formats, decoders in a sandbox), Undo everywhere.
- Retro style with the Calm level only, Neo Silver look, First Desk theme, Serbian Latin and Cyrillic and English (UK and US), voice and "Ask Neo" with local models.
- Tasks, Pads (templates), no Save button, Set aside, Jump key, universal Move/Copy/Properties/Again/Undo keys.
- **The language can be changed while the system runs**, without a restart and without losing open documents: from a menu-bar item and from Settings. Latin and Cyrillic script switch separately from the language; the keyboard layout is separate too. Programs show text only from language catalogs, never hard-coded strings. Programs from the Linux box follow where they can and may ask for a restart.
- Dijana as resident and guide that never interrupts ([MASCOT.md](MASCOT.md)); the Snake as a second resident for a test build ([RESIDENTS.md](RESIDENTS.md)).

**Apps (P0 first)**
- Cabinet (file manager, dual pane), Shell, Neo BASIC, Quill, Abacus, Pigeon (mail), Browser, Album, Gramophone, Cinema, Calendar and contacts, Keyring, Arcade (emulators and boxes instead of FPGA, with a Classic Mac emulator: [SYSTEM7-REFERENCE.md](SYSTEM7-REFERENCE.md)), Keeper, Readers.
- Added in November 2026: HippoPlayer engine, qBittorrent, Hydrogen and a beatmaker, tracker exchange ([MUSIC.md](MUSIC.md)); FilmCraft (video editor, based on Kdenlive), PhotoCraft (image editor, based on GIMP and Inkscape) and Album (image viewer, based on gThumb) ([CREATIVE-APPS.md](CREATIVE-APPS.md)).
- Ready-made free games through Flatpak in the Linux box: SuperTuxKart, 0 A.D., Luanti (Minetest), Frozen Bubble, Battle for Wesnoth; Arcade with MAME, DOSBox, ScummVM and RetroArch ([GAMES.md](GAMES.md)).
- Every program shows a simple name and says what it is based on, on its splash screen, in About, in Help and in the documentation ([APP-NAMES.md](APP-NAMES.md)).
- **Neo Boing Platinum OS Lite for children:** the same Lite with the Kids' room profile as default, a smaller program set and adult functions locked ([KIDS.md](KIDS.md)).

**Inspiration lists**
- AmigaOS, classic Mac, BeOS and Haiku, RISC OS, and modern macOS and Unix habits: see the plan pages below.

## 3b. Moved from Pro to Lite (27 November 2026)

Everything that does not wait for full hardware is now Lite. Where the Pro text named a Neo chip, Lite uses a software version.

| Pro idea | In Lite |
| --- | --- |
| All games ([GAMES.md](GAMES.md)) | Software versions; Warblade Neo waits for its licence |
| Arcade with FPGA cores | Arcade with emulators: MAME, DOSBox, ScummVM, RetroArch, FS-UAE, Classic Mac; plus open-source games through Flatpak |
| Workshop pack (video, photo, drawing) | FilmCraft (Kdenlive), PhotoCraft (GIMP and Inkscape), Album (gThumb) |
| Studio sound features | HippoPlayer, qBittorrent, Hydrogen, tracker, beatmaker ([MUSIC.md](MUSIC.md)) |
| Neo School, Neo Cards, Neo BASIC, languages course, news, phone link, Claude help | Same programs, on generic hardware (demos already in `docs/html/en/demo`) |
| Cabinet Pro and Magellan (rules and automation), Crunch (archives), Tiles and the Bar | Same; they are the first desktop work |
| Styles: Retro, Future Shock and the Crystal theme | Retro first; Future Shock and Crystal as the desktop allows. Crystal's host is the Snake ([RESIDENTS.md](RESIDENTS.md)) |
| Easy/Pro switch, Kids' room, large profile, accessibility | Lite, with the kids' version ([KIDS.md](KIDS.md)) |
| Sound themes (Neo basic, Crystal, Wood, Chip) | Lite, software mixer |
| Pro tools: Chronos, Renewal, Lock, Keeper, Find, Treasury | Lite (section 3 above) |
| VSA-6000 graphics, Neuron AI, Newton physics, Chameleon FPGA | Software rendering, local models on the CPU, software physics, emulators. The chips themselves: concept ([CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md)) |
| Help and user guide | In the system, HTML and PDF ([HELP.md](HELP.md)) |

## 4. Concept art only [concept]

- Neo 500 / 2000 / 4000 boards (concept hardware), the Neo 100 keyboard computer, the exchangeable Neo card and Autoconfig, the Neo package (VSA-6000, Zrak, Panorama, Newton, Kopernik, Neuron, Orfej, Courier, Chronos and Safe chips).
- FPGA cores (Chameleon service, Arcade cores): Lite uses emulators.
- Satellite x86 module, hardware privacy switches with LED, Eye cameras, cartridges and Pocket card, UHD Blu-ray and M-DISC bay, Sun station, Neo Automat, e-ink Dvolik.
- A RAD: that survives power loss (ultracapacitor or battery) and hardware-assisted glass and physics effects.

## Plan pages

[AMIGA.md](AMIGA.md) · [CLASSIC-MAC.md](CLASSIC-MAC.md) · [BEOS-HAIKU.md](BEOS-HAIKU.md) · [RISC-OS.md](RISC-OS.md) · [MUSIC.md](MUSIC.md) · [CREATIVE-APPS.md](CREATIVE-APPS.md) · [APP-NAMES.md](APP-NAMES.md) · [KIDS.md](KIDS.md) · [RAD.md](RAD.md) · [GAMES.md](GAMES.md) · [HELP.md](HELP.md) · [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md) · [RESIDENTS.md](RESIDENTS.md) · [SYSTEM7-REFERENCE.md](SYSTEM7-REFERENCE.md) · [BUILD-ISO.md](BUILD-ISO.md) · [HANDOFF.md](HANDOFF.md)
