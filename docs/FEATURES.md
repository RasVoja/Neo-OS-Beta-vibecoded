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
| Boot and drivers | ISO for BIOS and UEFI, GOP desk, USB keyboard and mouse, PCI, ACPI | First boot on real PCs | Network services (web view, remote folders, SMB, WebDAV), drivers only for found devices, network, sound, storage, Raspberry Pi 5 (main ARM target) and Pi 4 |
| Desktop | First window, Clock, Meter and Calculator tiles, Dijana, Neo DOS 0.6 | Compositor, Tiles, Shelf and Bar | Neo Kit, Neo Silver, First Desk, live language switch, Open Translation |
| Apps | | | Cabinet Pro, Album (gThumb), FilmCraft (Kdenlive), PhotoCraft (GIMP and Inkscape), DreamCatcher, Channel (FreeTube), Meeting (Jitsi), Broadcast (OBS), Decks (Mixxx), office, mail, web |
| Games | | | Neo's own games (with School Bell), Flatpak games, Arcade (MAME, DOSBox, ScummVM, RetroArch, ZX Spectrum) |
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
- Neo DOS 0.6 with BOOT: and DH0: volumes, line editing and history, BOOTLOG, USB and PCI; WRITE, COPY, RENAME and DELETE on a writable DH0:, ECHO > and >>, scripts (EXECUTE and `s/startup-sequence`), DATE and TIME, and small tools for text files: SEARCH, FIND, HEAD, TAIL, WC, SORT, UNIQ, TAC, NUMBER, CMP, HEXDUMP, plus CALC, REPEAT, WAIT, ALIAS, DU, NEWEST and CRC; wildcards in DIR, COPY and DELETE, TOUCH, REPLACE, variables (`SET`, `$name`), `IF`, `FOR`, and the line editor `EDIT`.
- First window, dragged with the mouse, the Clock, Meter and Calculator tiles, drawn through a back buffer (no flicker); Dijana on the Bar, with tips after errors and a good night at shutdown; the Neo menu (About, Help, Shut down); windows close with their close box, minimize to the Bar and are listed on it.
- Bootable hybrid ISO for BIOS and UEFI (GRUB, Multiboot2), tested in CI with SeaBIOS and OVMF.
- The desk on the screen the boot loader sets up (UEFI GOP or VBE).
- USB keyboard and mouse (xHCI, hubs, HID boot reports, hot-plug on ports and behind hubs); PCI buses and high BARs; ACPI tables (CPUs, interrupt routing, power off).
- Genode on seL4 trial: an unchanged Neo program runs as a Genode component; the `neo_ports` server.
- Contributor guide, twelve starter tasks, GPLv3-or-later.
- Dijana's Bin: DELETE keeps a copy in the Bin (BIN, RESTORE, EMPTYBIN); NVMe disks: a Neo disk or a blank disk is written as DH0:, other disks are never written (memory disk instead); USB 3 hubs.

## 1b. Keyboard shortcuts and reset [done]

USB keyboards only for now (the PS/2 driver is a user program and does not see these keys yet). All three reset combinations work, so every habit does:

| Keys | What happens | Origin |
| --- | --- | --- |
| Ctrl + Alt + Del | Soft reset (primary) | DOS, Windows |
| Ctrl + Neo + Esc (Neo = the key with the logo, Win on a PC) | Soft reset | Amiga's Ctrl+Amiga+Amiga (one logo key is enough) |
| Left Shift + Right Shift + Esc | Hard reset: the chipset restarts the whole machine, devices included | RISC OS style |

Other shortcuts: Alt+Shift switches the keyboard layout (US, Serbian Latin).

## 2. Being built [next]

- First boot on real PCs (see [BUILD-ISO.md](BUILD-ISO.md)).
- Genode on seL4: wake-up by signal, shared memory, handles in messages, then Neo DOS.
- Desktop: the rest of the compositor (then screen drag and window slide), more tiles, resizing windows, Shelf and Bar, drag and drop, Neo menu, command palette, then the Neo Silver look.

## 3. Planned for Lite [lite]

**System**
- Renewal (one signed A/B update with one-click return), Chronos (snapshots, "back to yesterday"), Lock (per-app privacy, outgoing traffic log), Keeper (backups), Find (one local index), Treasury (open-source app catalogue with reproducible builds).
- Linux box (applications domain) for Flatpak and POSIX programs; Bridge (FEX/Box64, Wine, Proton) on ARM; Android room (1.1).
- Native networking, audio and storage drivers for common hardware; Linux driver code only inside isolated components.
- Neo Revive: the system from a USB stick on old 64-bit UEFI PCs and Raspberry Pi 4/5 class boards, with a checked-computers list.
- Neighbourhood features: file sending to nearby computers, mail across the neighbourhood, borrowing power from bigger computers running NBPOS (Co-op).
- RAD: in the base version, in levels ([RAD.md](RAD.md)).

**Network, drivers and data** (added 28 November 2026; details in [NETWORK-DATA.md](NETWORK-DATA.md))
- **Built-in network services, in the datatypes spirit:** web view as a system service (WPE WebKit), remote folders over FTP, SFTP, SMB and WebDAV mounted as named disk icons on the desk (Amiga style: `NAS:`, `WORK:`), an opt-in local WebDAV server, Flash content through Ruffle, Java programs (`.jar` by double-click) through OpenJDK, and Open Translation by right-click. Every program uses them; Lock lists each one with its traffic.
- **Only the drivers that are needed (mount and devs):** a driver starts only when its device is found (already true in the Fortress for USB, virtio input and the disk); drivers described in `DEVS:`, volumes mounted on purpose with `MOUNT` and `ASSIGN`, devices plugged in later started at that moment.
- **Home networks and phones:** SMB and NFS for home shares and NAS, Syncthing for sync without a cloud, KDE Connect and LocalSend for the phone.
- **Databases:** SQLite built in for Cabinet Pro, Quill, Calendar, Album and Find; later a small database program for home lists; PostgreSQL and MariaDB as an optional server pack for advanced users.
- **Proven ideas from 1982 to today** (instant start, ARexx-style ports, RISC OS app folders, Mac aliases, BeOS attributes, Plan 9 mounts and more): the list is in [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md#proven-ideas-1982-to-today).

**Desk and manner**
- Neo Kit (one toolkit, Easy/Pro switch), Lux (scripting port in every app), Datatypes (1 000+ formats, decoders in a sandbox), Undo everywhere.
- Retro style with the Calm level only, Neo Silver look, First Desk theme, Serbian Latin and Cyrillic and English (UK and US), voice and "Ask Neo" with local models.
- **Smart Drop [lite]** (unique feature, added 28 November 2026): drag and drop that knows the type of what you drag (Datatypes) and shows on the target, before you let go, what will happen; converts on the fly (TIFF to JPEG for a web form, a URL to a link note in DreamCatcher); a **Shelf** on the screen edge as a parking place between programs; none = move, Ctrl = copy, Alt = link, Shift = ask; program icons on the Bar accept drops. Neo Kit gives it to every program; Linux box programs get the basics through the usual drag-and-drop protocols. About 4 person-months, coding steps 190 to 193 ([UNIQUE-FEATURES.md](UNIQUE-FEATURES.md)).
- Tasks, Pads (templates), no Save button, Set aside, Jump key, universal Move/Copy/Properties/Again/Undo keys.
- **The language can be changed while the system runs**, without a restart and without losing open documents: from a menu-bar item and from Settings. Latin and Cyrillic script switch separately from the language; the keyboard layout is separate too. Programs show text only from language catalogs, never hard-coded strings. Programs from the Linux box follow where they can and may ask for a restart.
- **Open Translation [lite]:** right-click a document, a web page or selected text (in Cabinet, Quill, Browser, Readers and Pigeon) and choose *Translate to…*, then a major world language: English, Spanish, French, German, Portuguese, Italian, Russian, Chinese, Japanese, Korean, Arabic, Hindi, Turkish and more, Serbian where a model exists. The translation opens as a new document beside the original and keeps the layout where the format allows. The engine is **LibreTranslate** (AGPL-3.0, self-hosted) with **Argos Translate** models (MIT), as a local service, first in the Linux box. Language packs are downloaded once and then work offline. Nothing goes to Google Translate or any other cloud service; Lock shows that the service has no network access once the packs are in place. LibreTranslate ships as a separate program because of its licence; licences are re-checked before code is merged.
- **Screen drag [lite]** (added 28 November 2026), after the Amiga: pull the title bar of a full-screen program down and the desk, or the program behind it, shows underneath; it stays where you drop it. A depth button, Super+M for the next screen, three fingers down on touch. Optional Windows-style slide: windows slide into the Bar when they minimise and back out when they open, with No motion / Calm / Brisk, and no motion when the system asks for reduced motion. Done in the compositor (a full-screen program is one surface that it moves), after "a buffer per window". See [AMIGA.md](AMIGA.md) (A10) and the [demo](html/en/demo/neo-screen-drag.html).
- **Widgets as a screen behind [lite, Pro mode] (unique feature)** (added 28 November 2026): in the Pro mode, under advanced settings, the widgets (Clock, Meter, Calendar, notes, weather) can be pinned to a screen of their own, a separate compositor surface between the desk and the programs. Pull a program down with the screen-drag gesture and the widgets are there; pull the widget screen down too and the desk shows. The user picks which widgets go on which screen. As far as we know no current system does this: macOS Dashboard (until 2019) had widgets in a separate Space and Windows has a widget board that slides in from the side, but neither has a layer you pull down partly and leave where you dropped it. Same compositor work as screen drag, plus a settings page. Shown in the [demo](html/en/demo/neo-screen-drag.html) ("Widgets as a screen behind").
- Dijana as resident and guide that never interrupts ([MASCOT.md](MASCOT.md)); the Snake as a second resident for a test build ([RESIDENTS.md](RESIDENTS.md)).

**Apps (P0 first)**
- Cabinet (file manager, dual pane), Shell, Neo BASIC, Quill, Abacus, Pigeon (mail), Browser, Album, Gramophone, Cinema, Calendar and contacts, Keyring, Arcade (emulators and boxes, with a Classic Mac emulator: [SYSTEM7-REFERENCE.md](SYSTEM7-REFERENCE.md)), Keeper, Readers.
- Added in November 2026: HippoPlayer engine, qBittorrent, Hydrogen and a beatmaker, tracker exchange ([MUSIC.md](MUSIC.md)); FilmCraft (video editor, based on Kdenlive), PhotoCraft (image editor, based on GIMP and Inkscape) and Album (image viewer, based on gThumb) ([CREATIVE-APPS.md](CREATIVE-APPS.md)).
- Ready-made free games through Flatpak in the Linux box: SuperTuxKart, 0 A.D., Luanti (Minetest), Frozen Bubble, Battle for Wesnoth; Arcade with MAME, DOSBox, ScummVM, RetroArch and a ZX Spectrum box (Fuse); **School Bell**, Neo's own school adventure and a homage to the Spectrum's *Skool Daze*, with questions from Neo School ([GAMES.md](GAMES.md)).
- Every program shows a simple name and says what it is based on, on its splash screen, in About, in Help and in the documentation ([APP-NAMES.md](APP-NAMES.md)).
- **Neo Boing Platinum OS Lite for children:** the same Lite with the Kids' room profile as default, a smaller program set and adult functions locked ([KIDS.md](KIDS.md)).

**Ideas, video and people together** (added 28 November 2026)
- **DreamCatcher [lite]**, "Every idea has its place": a mind map, document organiser and workflow companion in one. Ideas, files from Cabinet Pro, Quill documents, links and tasks are nodes on a board; the same board shows as a map, an outline (opens in Quill) or a task list; Neo + D catches an idea from anywhere; several people can edit one board on the local network. About 8 person-months ([DREAMCATCHER.md](DREAMCATCHER.md)).
- **Channel [lite]:** YouTube and PeerTube without adverts or an account, the desktop counterpart of NewPipe, based on **FreeTube** first and later on yt-dlp and mpv. A separate program from the Treasury, because YouTube's terms do not allow such players and it can break.
- **Meeting** (Jitsi Meet, video calls without accounts), **Broadcast** (OBS Studio, recording and streaming), **Decks** (Mixxx, DJ mixing), **PeerTube** inside Channel.
- **Quill Together and DreamCatcher Together:** one document or one mind map edited by several people on the local network at once, "Google Docs without Google", with no server on the internet.
- **Everyday apps after programs people love** (Keep, Obsidian, Shortcut, Classroom, Dub, Sonique, AccuWeather, Discogs and IMDb, PDFGear, Icecream Ebook Reader, HandBrake): DreamCatcher notes, graph mode and tasks, **SchoolDesk**, **Weather**, **Collection**, Reader with PDF tools and a bookshelf, **VideoCraft**, Gramophone with genre bins and visualisations; offline-first and without accounts, about 20 person-months ([EVERYDAY-APPS.md](EVERYDAY-APPS.md)).
- **Accessibility** (high contrast, keyboard-only use, screen reader over AT-SPI2, scalable interface) and **what Gen Z gets**: [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md); coding steps 150 to 159.
- What does the most for healthy fun and learning with other people, and the Kids' room limits for all of these: [TOGETHER.md](TOGETHER.md).

**Plan 9 ideas** (added 28 November 2026, homage to Plan 9 from Bell Labs and to Ed Wood's film *Plan 9 from Outer Space*)
- **"Everything is a file"** is the base of the Neo mount system: every resource a program uses has a name in its own namespace.
- **`p9fs`:** the internal network file transport speaks 9P2000 (9P2000.L for the Linux box): shared folders between NBPOS computers, the Linux box and virtual machines, and the Co-op work like Plan 9's `cpu` command.
- **Neo Plumber:** a service that gets a file name, URL, address or phone number and its context and decides which program opens it and how; Datatypes says what a file is, the Plumber says who opens it.
- **Union mounts** (several folders seen as one, for app bundles and themes) and a **per-process namespace** (every program sees only what it needs), the security base for Lock and the Kids' room.
- A small homage: a Plan 9 screen saver with flying saucers on strings, `PLAN9` in Neo DOS, credits in About. About 6 person-months ([PLAN9.md](PLAN9.md)).

**Monitors and video walls** (added 28 November 2026)
- **Several monitors [lite]:** one continuous desk across all monitors (windows carry on across the edge at the right size, even with mixed resolutions); every window remembers its monitor by the monitor's identity and goes back to it when the monitor is plugged in again; right-click the title bar for **Move to › Monitor 2**, or Neo + Shift + arrow; "always open here" rules per program; one Bar or a Bar on every monitor.
- **Monitor profiles [lite]:** "Work", "Cinema", "Presenting", "Gaming" save the monitors, resolutions, the Bar and which program goes on which monitor; one click to switch, offered (not forced) when the monitors are plugged in.
- **Video wall [lite]:** many monitors or projectors as one picture, drawn by the compositor as one large frame split across the screens, with bezel correction; later across several NBPOS computers (Co-op). **Show on › Projector** sends a live copy of one window only. Programs can ask for a monitor through Neo Kit, the user's rules win. Details and estimates (about 18 person-months on top of the graphics drivers): [MULTI-MONITOR.md](MULTI-MONITOR.md).

**Inspiration lists**
- AmigaOS, classic Mac, BeOS and Haiku, RISC OS, Plan 9 from Bell Labs, the ZX Spectrum, and modern macOS and Unix habits: see the plan pages below.

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

## 5. What only NBPOS does (unique features)

Kept on a page of its own, [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md), with what comes closest in other systems for every item. In short:

1. Widgets as a screen behind the programs (Pro mode)
2. Amiga screen drag on today's PCs
3. Windows remember their monitor by its identity
4. Monitor profiles in one click, with which program goes where
5. Video wall without special software
6. One window shown on another monitor, the rest of the desk private
7. Open Translation by right-click, offline
8. The language changes while you work
9. Dijana, a resident that never interrupts [done, first part]
10. Three reset habits at once (DOS, Amiga, RISC OS) [done]
11. Every program says what it is based on
12. The network as a datatype
13. A remote server is a disk icon with a name
14. Only the drivers your hardware needs
15. Smart Drop: the target shows what it will do before you let go, converts on the fly, a Shelf for parking, the same modifier keys everywhere

## Plan pages

[NETWORK-DATA.md](NETWORK-DATA.md) · [AMIGA.md](AMIGA.md) · [CLASSIC-MAC.md](CLASSIC-MAC.md) · [BEOS-HAIKU.md](BEOS-HAIKU.md) · [RISC-OS.md](RISC-OS.md) · [MUSIC.md](MUSIC.md) · [CREATIVE-APPS.md](CREATIVE-APPS.md) · [APP-NAMES.md](APP-NAMES.md) · [KIDS.md](KIDS.md) · [RAD.md](RAD.md) · [GAMES.md](GAMES.md) · [HELP.md](HELP.md) · [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md) · [RESIDENTS.md](RESIDENTS.md) · [SYSTEM7-REFERENCE.md](SYSTEM7-REFERENCE.md) · [BUILD-ISO.md](BUILD-ISO.md) · [HANDOFF.md](HANDOFF.md) · [MULTI-MONITOR.md](MULTI-MONITOR.md) · [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md) · [PLAN9.md](PLAN9.md) · [DREAMCATCHER.md](DREAMCATCHER.md) · [TOGETHER.md](TOGETHER.md) · [EVERYDAY-APPS.md](EVERYDAY-APPS.md)

## Concept hardware (separate, not needed)

> **Concept hardware.** A vision of a future dedicated computer. None of it exists, nothing above depends on it, and every feature above runs on ordinary x86_64 and ARM computers. Claude integration is not part of it either: it is a distant-future idea, not guaranteed.

- **Neo 500, 2000 and 4000** boards, the **NeoPocket** handheld and the Neo 100 keyboard computer, with the exchangeable Neo card and Autoconfig.
- The Neo package of chips (VSA-6000 graphics, Zrak, Panorama, Newton physics, Kopernik, Neuron AI, Orfej sound, Courier, Chronos and Safe), FPGA cores (Chameleon service, Arcade cores) and the Satellite x86 module.
- Hardware privacy switches with LED, Eye cameras, cartridges and the Pocket card, UHD Blu-ray and M-DISC bay, Sun station, Neo Automat, e-ink Dvolik.
- Hardware extras for system features: a RAD: that survives power loss, glass and physics effects in hardware. On ordinary computers the same features run in software (section 3b).

Details: [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md).
