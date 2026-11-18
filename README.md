# Neo Boing Platinum OS

**English** · [Srpski](README.sr.md)

Neo Boing Platinum OS is the operating system of the Neo computer family. It aims to be one system with a human face, the same for a child, an adult and a professional, and the same from a handheld to a workstation. It is meant to be the only computer in the house, covering mail, web, office, photos, music, learning, games and retro, with no account and no telemetry.

> **Status: the code is here and open for contributions.** The *Fortress* kernel boots on x86_64 and AArch64 under QEMU. It runs Rust programs in user mode with capabilities and message ports, and has a keyboard (US and Serbian Latin), a mouse, a screen with windows, a disk, a terminal and the Neo DOS command line. The first Neo program also runs on Genode and seL4 (see [genode/](genode/README.md)). [docs/STATUS.md](docs/STATUS.md) tracks what is done and planned in each area; [docs/PROGRESS.md](docs/PROGRESS.md) is the step-by-step development log.

> **About the name.** Until 18 November 2026 the system was called *NeoOS* (or *Neo OS*). Other, unrelated projects use that name, so it is now **Neo Boing Platinum OS**. The Neo computers (Neo 500, 2000, 4000), Neo Lite, Neo DOS and the other Neo names stay as they are. *Boing* here is a nod to the Amiga spirit; the project does not use the Amiga Boing Ball logo, which is a trademark of its owner.

## Goal documents

| Document | English | Srpski |
| --- | --- | --- |
| Neo Boing Platinum OS 1.0: specification and development plan | [PDF](docs/pdf/en/Neo-Boing-Platinum-OS-1.0-specification.pdf) | [PDF](docs/pdf/sr/Neo-Boing-Platinum-OS-1.0-specifikacija.pdf) |
| The Neo family of computers (overview) | [PDF](docs/pdf/en/Neo-overview.pdf) | [PDF](docs/pdf/sr/Neo-pregled.pdf) |
| Promo brochure | [PDF](docs/pdf/en/Neo-promo-brochure.pdf) | [PDF](docs/pdf/sr/Neo-promo-brosura.pdf) |
| What else Neo can do | [PDF](docs/pdf/en/Neo-what-else-it-can-do.pdf) | [PDF](docs/pdf/sr/Neo-sta-jos-ume.pdf) |
| Fed up? (what Neo removes from everyday computing) | [PDF](docs/pdf/en/Neo-fed-up.pdf) | [PDF](docs/pdf/sr/Neo-dosadilo-vam-je.pdf) |
| Neo inside (technical book; the Crystal theme with DJ Nick is a concept, a possibility, not an agreement) | [PDF](docs/pdf/en/Neo-technical-book.pdf) | [PDF](docs/pdf/sr/Neo-tehnicka-knjiga.pdf) |

The specification is a working draft. Every figure in it is an engineering estimate, not a measurement.

## Concept pages and demos

The same documents as web pages, and demos that run in a browser. GitHub shows HTML as source, so download the folder (or the repository as a ZIP) and open the file in a browser. Everything is a concept and a simulation, not the real system.

| | English | Srpski |
| --- | --- | --- |
| All demos (start here) | [neo-demos.html](docs/html/en/demo/neo-demos.html) | [neo-demoi.html](docs/html/sr/demo/neo-demoi.html) |
| Cabinet Pro and Duo (file manager) | [neo-cabinet.html](docs/html/en/demo/neo-cabinet.html) | [neo-ormar.html](docs/html/sr/demo/neo-ormar.html) |
| Neo DOS (command line) | [neo-dos.html](docs/html/en/demo/neo-dos.html) | [neo-dos.html](docs/html/sr/demo/neo-dos.html) |
| Tiles and the Bar | [neo-tiles.html](docs/html/en/demo/neo-tiles.html) | [neo-ploce.html](docs/html/sr/demo/neo-ploce.html) |
| Cabinet Pro: Magellan (rules and automation) | [neo-cabinet-magellan.html](docs/html/en/demo/neo-cabinet-magellan.html) | [neo-ormar-magellan.html](docs/html/sr/demo/neo-ormar-magellan.html) |
| Crunch (compression and archives) | [neo-crunch.html](docs/html/en/demo/neo-crunch.html) | [neo-zgusni.html](docs/html/sr/demo/neo-zgusni.html) |
| First Desk (the retro theme) | [neo-first-desk.html](docs/html/en/demo/neo-first-desk.html) | [neo-prvi-sto.html](docs/html/sr/demo/neo-prvi-sto.html) |
| Neo Boing Platinum OS beta manual (work in progress) | [neo-manual-beta.html](docs/html/en/neo-manual-beta.html) | [neo-prirucnik-beta.html](docs/html/sr/neo-prirucnik-beta.html) |
| Neo desk, school, languages, news, Claude help, phone link | [docs/html/en/demo](docs/html/en/demo) | [docs/html/sr/demo](docs/html/sr/demo) |
| Overview, brochure, technical book, specification as web pages | [docs/html/en](docs/html/en) | [docs/html/sr](docs/html/sr) |

## Where the work starts: Neo Boing Platinum OS Lite

The full Neo Boing Platinum OS targets Neo hardware that does not exist yet (Neo 500, 2000 and 4000). **Neo Boing Platinum OS Lite** is the profile for small and existing machines, so it is the part that can be built and tested now:

- **Neo 100**: a small ARM computer inside a keyboard.
- **Neo Revive**: the system run from a USB stick on old 64-bit PCs (UEFI, roughly 2012 on, 2 GB RAM or more) and on Raspberry Pi 4/5 class boards.
- **Phones**: experimental, only where open drivers exist.

Lite ships with the Neo desktop. The first development target is generic **x86_64 under QEMU**, with generic **AArch64** (QEMU `virt`) second.

## Architecture in one table

| Layer | What it is |
| --- | --- |
| Apps and games | File manager, browser, office, Arcade, programs from the Treasury catalogue |
| Neo Kit and Lux | One windowing toolkit, themes, Datatypes, a scripting port in every app |
| System services | Exec-N (message ports), Dispatcher, Chronos (snapshots), Renewal (A/B updates), Lock (privacy), Keeper (backups), Find |
| Drivers | Native drivers in Rust; Linux driver code only inside isolated components |
| Fortress | seL4-class microkernel that hands out memory, CPU time and capabilities |
| Neo ROM and Safe | Boot loader, signed A/B system image, keys |

Design rules: responsiveness before throughput; one system, one API, one look; every app scriptable; small and understandable, with a size and memory budget for every system app. The system is written in Rust.

The desktop look, *Neo Silver*, follows classic Mac OS with the best of BeOS, Haiku, MorphOS and the Amiga. Every name, icon and drawing is Neo Boing Platinum OS's own. Serbian (Latin and Cyrillic) and English (UK and US) are the first languages, and a localisation kit makes adding any other language easy.

## Built on open source

Anything that already exists as a mature open project is used as the foundation; only what is new is written from scratch, and changes go back upstream. Candidates include Genode on seL4, Linux driver code in isolated components, Btrfs and EROFS, Mesa, Smithay, the Haiku Interface Kit, PipeWire, QEMU, LLVM and Rust.

## How to help

Start with [CONTRIBUTING.md](CONTRIBUTING.md): what you need, how to build and run the boot tests, and the rules a change must keep. [docs/STARTER-TASKS.md](docs/STARTER-TASKS.md) lists twelve starter tasks, each with its test, and each is also an issue. Questions, ideas and comments on the specification are welcome as issues too.

## Licence

Neo Boing Platinum OS is free software under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE).
