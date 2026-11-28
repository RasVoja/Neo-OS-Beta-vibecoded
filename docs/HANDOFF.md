# Handoff: how to continue coding NBPOS

**English** · [Srpski](HANDOFF.sr.md) · 27 November 2026

For a developer, or an AI assistant with a small context window (for example a free account), who continues without the earlier chats.

## Read in this order

1. [README.md](../README.md): what NBPOS is and what Lite means.
2. [STATUS.md](STATUS.md): milestones and what is done.
3. [CONTRIBUTING.md](../CONTRIBUTING.md): build, test, code rules, how to add a program or a service.
4. [decisions/0001-fortress-and-genode.md](decisions/0001-fortress-and-genode.md): why the kernel will give way to Genode on seL4.
5. [FEATURES.md](FEATURES.md): scope, **Lite first**, concept art kept apart.
6. This page. [PROGRESS.md](PROGRESS.md) is about 50 KB: read only its last section.

## Rules for every step

- One step, one branch, named `claude/neo-lite-stepNN-short-name`, one pull request.
- Keep the boot tests green: `./scripts/test-boot-x86_64.sh`, `./scripts/test-boot-aarch64.sh`, `./scripts/test-usb-x86_64.sh`. Update the file counts in the scripts when you add a program.
- Format with `cargo fmt` in the root and in `user/`.
- At the end of a step, add an entry to PROGRESS.md and tick the line in STATUS.md.
- Documents come in English and Serbian (`X.md` and `X.sr.md`), and the Serbian one is Latin script.
- The code is GPLv3-or-later. Anything under another licence ships as a separate program (rule in [AMIGA.md](AMIGA.md)).
- Own names, icons and drawings only: no third-party or licensed names in the system.
- Label concept art as concept everywhere (docs, videos, examples).
- System pages speak only of x86_64 and ARM (reference machines: Raspberry Pi 5, AMD Zen 2, Intel Core 10th gen). Neo hardware (Neo 500, 2000, 4000, NeoPocket, chips, FPGA) goes only in a short "Concept hardware" section at the end of a page.
- Claude integration is always marked "(distant future, not guaranteed)"; never promise it.
- Do not claim a feature works until a test shows it.
- Every text the user sees comes from a language catalog, never a hard-coded string, so the language can be switched while the system runs ([FEATURES.md](FEATURES.md)).

## Queue (in this order)

1. **First boot on real PCs**, starting with the reference machine: [BUILD-ISO.md](BUILD-ISO.md), sections 4 and 5. Needs the owner's hands; the assistant fixes what the photos and logs show.
2. **Genode on seL4:** wake-up by signal, shared memory, handles in messages; then Neo DOS as a Genode component.
3. **Desktop:** compositor, tiles (Clock, Calculator, Meter), Shelf and Bar, drag and drop, Neo menu, command palette, then Neo Silver.
4. **Files:** dual-pane Cabinet, archives, Btrfs and EROFS; `RAD:` level 1 ([RAD.md](RAD.md)).
5. **Drivers for everyday use:** storage (NVMe, AHCI), network, audio. Everything below needs them.
6. **Linux box** (applications domain). Needed by the photo viewer, HippoPlayer, qBittorrent and Hydrogen.
7. **Image viewer, FilmCraft and PhotoCraft** ([CREATIVE-APPS.md](CREATIVE-APPS.md)). Album is gThumb, FilmCraft is Kdenlive, PhotoCraft is GIMP and Inkscape.
8. **Games** ([GAMES.md](GAMES.md)), starting with Solitaire and Minesweeper. **Help** ([HELP.md](HELP.md)): the one-source HTML/PDF build exists; screenshots test and the in-system Help program follow the desktop.
9. **Music and apps** in the order of [MUSIC.md](MUSIC.md).
10. **Kids' version** ([KIDS.md](KIDS.md)) once the desktop, Easy layout and Lock Keys exist, and the **app name and credits rule** ([APP-NAMES.md](APP-NAMES.md)) from the first program on.
11. **Installer** beside Windows 11 ([BUILD-ISO.md](BUILD-ISO.md), section 6).
12. Plan pages in the order each states: [AMIGA.md](AMIGA.md), [CLASSIC-MAC.md](CLASSIC-MAC.md), [BEOS-HAIKU.md](BEOS-HAIKU.md), [RISC-OS.md](RISC-OS.md).

## Open choices that are the owner's

The `RISC-OS.md` items marked **new** (BeOS and Haiku items H9 to H11 are accepted); sorting the documents into the system and the concept ([CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md)), the beatmaker's model, the canonical look of Dijana ([MASCOT.md](MASCOT.md)), and the speed build profile.

## Paste this at the start of a session

> You are continuing NBPOS (Neo Boing Platinum OS), Lite first. Read README.md, docs/STATUS.md, CONTRIBUTING.md and docs/HANDOFF.md. Work on the first unchecked item of the queue in HANDOFF.md, one branch and one step. Keep the boot tests green, update PROGRESS.md and STATUS.md, write documents in English and Serbian, and say plainly what you did not test.
