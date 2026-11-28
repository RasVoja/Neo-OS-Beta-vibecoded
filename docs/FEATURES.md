# NBPOS feature list

**English** · [Srpski](FEATURES.sr.md) · Updated 28 November 2026

One page with everything NBPOS (Neo Boing Platinum OS) does or plans to do, tagged by scope. Details live in [STATUS.md](STATUS.md) (milestones), the specification (PDF) and the plan pages linked at the end.

## Scope rule

**Lite first.** Neo Boing Platinum OS Lite on x86_64 and ARM takes in everything from the earlier full and pro design that software can do on today's hardware. The goal is to prove the system on an existing platform first.

**Platforms: x86_64 and ARM only.** The yardsticks are a Raspberry Pi 5 (minimum), an AMD Zen 2 PC (reference) and an Intel Core 10th generation PC; what each one is expected to do is in section 4.

Dedicated Neo hardware that does not exist is a separate concept (concept hardware). It is kept only in the last section of this page, and in documents, videos and examples it carries a visible "concept" label so it is never mistaken for what runs.

Tags: **[done]** works and is tested in CI · **[next]** being built · **[lite]** planned for Lite, can enter the plan now · **[concept]** needs hardware that does not exist (concept hardware).

Rule for the tags: anything that does not need Neo hardware is **[lite]** and can go into the development plan now.

## At a glance

| Area | [done] | [next] | [lite] |
| --- | --- | --- | --- |
| Kernel and programs | Fortress on x86_64 and AArch64, user programs, ports, capabilities | Genode on seL4 | Renewal, Chronos, Lock |
| Boot and drivers | ISO for BIOS and UEFI, GOP desk, USB keyboard and mouse, PCI, ACPI | First boot on real PCs | Network, sound, storage, Raspberry Pi 5 (main ARM target) and Pi 4 |
| Desktop | First window, Clock, Meter and Calculator tiles, Dijana, Neo DOS 0.5 | Compositor, Tiles, Shelf and Bar | Neo Kit, Neo Silver, First Desk, live language switch, Open Translation |
| Apps | | | Cabinet Pro, Album (gThumb), FilmCraft (Kdenlive), PhotoCraft (GIMP and Inkscape), office, mail, web |
| Games | | | Neo's own games, Flatpak games, Arcade (MAME, DOSBox, ScummVM, RetroArch) |
| Music | | | HippoPlayer, qBittorrent, Hydrogen, tracker, beatmaker |
| Help | Beta manual, HTML and PDF guide | | Help program, F1, pictures of the real screen |

## 1. Works now [done]

- Fortress kernel boots on x86_64 and AArch64 (QEMU), with a serial console and a boot test in CI.
- CPU exceptions, own page tables, stack guard page, all physical memory managed, kernel heap, timer, preemptive scheduler.
- Programs in user mode, one address space each; a crash stops only that program.
- Named message ports and capabilities; shared-memory rings for large messages.
- Rust runtime (`neo-rt`), ELF loader, programs loaded from a disk image.
- User-mode services: console, keyboard (PS/2, virtio-input), framebuffer display, virtio block device with a file system that can be written.
- Keyboard layouts US and Serbian Latin (č, ć, ž, š, đ), switched with Alt+Shift.
- Neo DOS 0.5 with BOOT: and DH0: volumes, line editing and history, BOOTLOG, USB and PCI; WRITE, COPY, RENAME and DELETE on a writable DH0:, ECHO > and >>, scripts (EXECUTE and `s/startup-sequence`), DATE and TIME, and small tools for text files: SEARCH, FIND, HEAD, TAIL, WC, SORT, UNIQ, TAC, NUMBER, CMP, HEXDUMP, plus CALC, REPEAT, WAIT and ALIAS; wildcards in DIR, COPY and DELETE, TOUCH, REPLACE, variables (`SET`, `$name`), `IF`, `FOR`, and the line editor `EDIT`.
- First window, dragged with the mouse, the Clock, Meter and Calculator tiles, drawn through a back buffer (no flicker); Dijana on the Bar, with tips after errors and a good night at shutdown; the Neo menu (About, Help, Shut down); windows close with their close box, minimize to the Bar and are listed on it.
- Bootable hybrid ISO for BIOS and UEFI (GRUB, Multiboot2), tested in CI with SeaBIOS and OVMF.
- The desk on the screen the boot loader sets up (UEFI GOP or VBE).
- USB keyboard and mouse (xHCI, hubs, HID boot reports, hot-plug on ports and behind hubs); PCI buses and high BARs; ACPI tables (CPUs, interrupt routing, power off).
- Genode on seL4 trial: an unchanged Neo program runs as a Genode component; the `neo_ports` server.
- Contributor guide, twelve starter tasks, GPLv3-or-later.

## 2. Being built [next]

- First boot on real PCs (see [BUILD-ISO.md](BUILD-ISO.md)).
- Genode on seL4: wake-up by signal, shared memory, handles in messages, then Neo DOS.
- Desktop: the rest of the compositor, more tiles, resizing windows, Shelf and Bar, drag and drop, Neo menu, command palette, then the Neo Silver look.

## 3. Planned for Lite [lite]

**System**
- Renewal (one signed A/B update with one-click return), Chronos (snapshots, "back to yesterday"), Lock (per-app privacy, outgoing traffic log), Keeper (backups), Find (one local index), Treasury (open-source app catalogue with reproducible builds).
- Linux box (applications domain) for Flatpak and POSIX programs; Bridge (FEX/Box64, Wine, Proton) on ARM; Android room (1.1).
- Native networking, audio and storage drivers for common hardware; Linux driver code only inside isolated components.
- Neo Revive: the system from a USB stick on old 64-bit UEFI PCs and Raspberry Pi 4/5 class boards, with a checked-computers list.
- Neighbourhood features: file sending to nearby computers, mail across the neighbourhood, borrowing power from bigger computers running NBPOS (Co-op).
- RAD: in the base version, in levels ([RAD.md](RAD.md)).

**Desk and manner**
- Neo Kit (one toolkit, Easy/Pro switch), Lux (scripting port in every app), Datatypes (1 000+ formats, decoders in a sandbox), Undo everywhere.
- Retro style with the Calm level only, Neo Silver look, First Desk theme, Serbian Latin and Cyrillic and English (UK and US), voice and "Ask Neo" with local models.
- Tasks, Pads (templates), no Save button, Set aside, Jump key, universal Move/Copy/Properties/Again/Undo keys.
- **The language can be changed while the system runs**, without a restart and without losing open documents: from a menu-bar item and from Settings. Latin and Cyrillic script switch separately from the language; the keyboard layout is separate too. Programs show text only from language catalogs, never hard-coded strings. Programs from the Linux box follow where they can and may ask for a restart.
- **Open Translation [lite]:** right-click a document, a web page or selected text (in Cabinet, Quill, Browser, Readers and Pigeon) and choose *Translate to…*, then a major world language: English, Spanish, French, German, Portuguese, Italian, Russian, Chinese, Japanese, Korean, Arabic, Hindi, Turkish and more, Serbian where a model exists. The translation opens as a new document beside the original and keeps the layout where the format allows. The engine is **LibreTranslate** (AGPL-3.0, self-hosted) with **Argos Translate** models (MIT), as a local service, first in the Linux box. Language packs are downloaded once and then work offline. Nothing goes to Google Translate or any other cloud service; Lock shows that the service has no network access once the packs are in place. LibreTranslate ships as a separate program because of its licence; licences are re-checked before code is merged.
- Dijana as resident and guide that never interrupts ([MASCOT.md](MASCOT.md)); the Snake as a second resident for a test build ([RESIDENTS.md](RESIDENTS.md)).

**Apps (P0 first)**
- Cabinet (file manager, dual pane), Shell, Neo BASIC, Quill, Abacus, Pigeon (mail), Browser, Album, Gramophone, Cinema, Calendar and contacts, Keyring, Arcade (emulators and boxes, with a Classic Mac emulator: [SYSTEM7-REFERENCE.md](SYSTEM7-REFERENCE.md)), Keeper, Readers.
- Added in November 2026: HippoPlayer engine, qBittorrent, Hydrogen and a beatmaker, tracker exchange ([MUSIC.md](MUSIC.md)); FilmCraft (video editor, based on Kdenlive), PhotoCraft (image editor, based on GIMP and Inkscape) and Album (image viewer, based on gThumb) ([CREATIVE-APPS.md](CREATIVE-APPS.md)).
- Ready-made free games through Flatpak in the Linux box: SuperTuxKart, 0 A.D., Luanti (Minetest), Frozen Bubble, Battle for Wesnoth; Arcade with MAME, DOSBox, ScummVM and RetroArch ([GAMES.md](GAMES.md)).
- Every program shows a simple name and says what it is based on, on its splash screen, in About, in Help and in the documentation ([APP-NAMES.md](APP-NAMES.md)).
- **Neo Boing Platinum OS Lite for children:** the same Lite with the Kids' room profile as default, a smaller program set and adult functions locked ([KIDS.md](KIDS.md)).

**Inspiration lists**
- AmigaOS, classic Mac, BeOS and Haiku, RISC OS, and modern macOS and Unix habits: see the plan pages below.

## 3b. Moved from Pro to Lite (27 November 2026)

Everything that does not wait for full hardware is now Lite. Where the Pro text needed dedicated hardware, Lite uses a software version.

| Pro idea | In Lite |
| --- | --- |
| All games ([GAMES.md](GAMES.md)) | Software versions; Warblade Neo waits for its licence |
| Arcade | Emulators: MAME, DOSBox, ScummVM, RetroArch, FS-UAE, Classic Mac; plus open-source games through Flatpak |
| Workshop pack (video, photo, drawing) | FilmCraft (Kdenlive), PhotoCraft (GIMP and Inkscape), Album (gThumb) |
| Studio sound features | HippoPlayer, qBittorrent, Hydrogen, tracker, beatmaker ([MUSIC.md](MUSIC.md)) |
| Neo School, Neo Cards, Neo BASIC, languages course, news, phone link | Same programs, on generic hardware (demos already in `docs/html/en/demo`) |
| Claude help | **Distant future, not guaranteed.** Not planned for Lite; the demo only shows an idea |
| Cabinet Pro and Magellan (rules and automation), Crunch (archives), Tiles and the Bar | Same; they are the first desktop work |
| Styles: Retro, Future Shock and the Crystal theme | Retro first; Future Shock and Crystal as the desktop allows. Crystal's host is the Snake ([RESIDENTS.md](RESIDENTS.md)) |
| Easy/Pro switch, Kids' room, large profile, accessibility | Lite, with the kids' version ([KIDS.md](KIDS.md)) |
| Sound themes (Neo basic, Crystal, Wood, Chip) | Lite, software mixer |
| Pro tools: Chronos, Renewal, Lock, Keeper, Find, Treasury | Lite (section 3 above) |
| Graphics, AI and physics that the Pro text gave to dedicated chips | The computer's own graphics card (Mesa, Vulkan) or software rendering, local models on the processor, software physics, emulators |
| Help and user guide | In the system, HTML and PDF ([HELP.md](HELP.md)) |

## 4. Performance on today's hardware (estimate)

NBPOS Lite is made for x86_64 and ARM (AArch64) computers that exist today. Three machines are the yardstick: a **Raspberry Pi 5** as the minimum, an **AMD Zen 2** PC as the reference, and an **Intel Core 10th generation** PC as its Intel equivalent.

These are engineering estimates for the planned Lite 1.0 desktop, worked out from what comparable systems (Haiku, light Linux desktops, Raspberry Pi OS) do on the same hardware. They are targets, not measurements: today's code shows the first desk only, and the Raspberry Pi 5 port has not started yet (ARM runs in QEMU `virt`). Reports from real machines ([REAL-PC-TEST.md](REAL-PC-TEST.md)) will replace them with measured numbers.

| | **Raspberry Pi 5**, 8 GB (minimum) | **Intel Core 10th gen** (Comet Lake, e.g. Core i5-10400 with UHD 630 graphics), 16 GB | **AMD Zen 2** (Ryzen 3000, e.g. Ryzen 5 3600 with a mid-range graphics card), 16 GB (reference) |
| --- | --- | --- | --- |
| Processor | 4 × Cortex-A76, 2.4 GHz | 6 cores, 12 threads, up to 4.3 GHz | 6 cores, 12 threads, up to 4.2 GHz |
| Start to desk | about 10–15 s from an NVMe or USB SSD, 20–30 s from microSD | about 5–8 s from an SSD | about 5–8 s from an SSD |
| Smooth desk at 60 Hz | 1080p with all effects; 1440p and 4K with glass and shadows turned down | 1440p with all effects; 4K with fewer effects | 4K with all effects; 1440p at 120–144 Hz |
| Pointer and window drag | 60 frames a second, pointer within one frame | the same | the same, up to 144 Hz on fast monitors |
| Windows open before anything slows | about 15–20 | about 40 | 50 or more |
| Native Neo program starts (Cabinet, Quill, Abacus) | under 1 s | under 0.3 s | under 0.3 s |
| Memory for the desk and a few programs | about 400–600 MB | about 400–600 MB | about 400–600 MB |
| Linux box programs (Flatpak) | Office, browser with about 10 tabs, Album (gThumb), PhotoCraft (GIMP) fine; FilmCraft (Kdenlive) 1080p with proxy files; 3–4 heavy programs at once | all of them; FilmCraft 4K with proxy files | all of them; FilmCraft 4K, rendering on the graphics card |
| Games | Frozen Bubble, Wesnoth and Neo's 2D games at full speed; SuperTuxKart and Luanti 720p–1080p on low, 30–60 fps; 0 A.D. small maps | SuperTuxKart and Luanti 1080p medium; 0 A.D. medium maps | 1080p–1440p on high |
| Arcade (emulators) | DOSBox, ScummVM, FS-UAE (Amiga 500 and 1200) at full speed; MAME for most 1980s and early 1990s games; RetroArch up to PlayStation 1 | MAME for nearly all 2D games; RetroArch up to Nintendo 64 and Dreamcast | up to PlayStation 2 and GameCube in most games |
| Local models ("Ask Neo", speech) | speech with small models; a 1–3 B language model at a few tokens a second | a 3–4 B model at about 8–12 tokens a second | a 7–8 B model at about 6–10 tokens a second on the processor, faster on the graphics card |
| Open Translation, one page (~500 words) | about 5–15 s | about 1–3 s | about 1–3 s |
| Power at the desk | about 4–8 W | about 25–40 W (whole PC) | about 40–60 W with a graphics card |

Notes:
- A **Raspberry Pi 5 with 4 GB** runs the same desk, with fewer Linux box programs open at once. A Raspberry Pi 4 and old PCs (Neo Revive, about 2012 on) run Lite with the effects off.
- **Zen 2 with built-in graphics** (Ryzen 4000 and 5000 APUs such as the Ryzen 5 4600G, and laptops with the Ryzen 5 4500U) sit between the Intel and the Ryzen columns: the processor of the reference, graphics close to Intel UHD 630 or a little better.
- The Intel column is the equivalent of the AMD reference for the processor; the difference between the two columns is mostly the graphics card.

## Plan pages

[AMIGA.md](AMIGA.md) · [CLASSIC-MAC.md](CLASSIC-MAC.md) · [BEOS-HAIKU.md](BEOS-HAIKU.md) · [RISC-OS.md](RISC-OS.md) · [MUSIC.md](MUSIC.md) · [CREATIVE-APPS.md](CREATIVE-APPS.md) · [APP-NAMES.md](APP-NAMES.md) · [KIDS.md](KIDS.md) · [RAD.md](RAD.md) · [GAMES.md](GAMES.md) · [HELP.md](HELP.md) · [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md) · [RESIDENTS.md](RESIDENTS.md) · [SYSTEM7-REFERENCE.md](SYSTEM7-REFERENCE.md) · [BUILD-ISO.md](BUILD-ISO.md) · [HANDOFF.md](HANDOFF.md)

## Concept hardware (separate, not needed)

> **Concept hardware.** A vision of a future dedicated computer. None of it exists, nothing above depends on it, and every feature above runs on ordinary x86_64 and ARM computers. Claude integration is not part of it either: it is a distant-future idea, not guaranteed.

- **Neo 500, 2000 and 4000** boards, the **NeoPocket** handheld and the Neo 100 keyboard computer, with the exchangeable Neo card and Autoconfig.
- The Neo package of chips (VSA-6000 graphics, Zrak, Panorama, Newton physics, Kopernik, Neuron AI, Orfej sound, Courier, Chronos and Safe), FPGA cores (Chameleon service, Arcade cores) and the Satellite x86 module.
- Hardware privacy switches with LED, Eye cameras, cartridges and the Pocket card, UHD Blu-ray and M-DISC bay, Sun station, Neo Automat, e-ink Dvolik.
- Hardware extras for system features: a RAD: that survives power loss, glass and physics effects in hardware. On ordinary computers the same features run in software (section 3b).

Details: [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md).
