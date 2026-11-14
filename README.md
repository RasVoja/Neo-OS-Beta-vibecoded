# NeoOS

**English** · [Srpski](README.sr.md)

NeoOS is the operating system of the Neo computer family. It aims to be one system with a human face, the same for a child, an adult and a professional, and the same from a handheld to a workstation. It is meant to be the only computer in the house, covering mail, web, office, photos, music, learning, games and retro, with no account and no telemetry.

> **Status: early, being coded in the open soon.** The first kernel, *Fortress*, boots on x86_64 under QEMU. It runs programs written in Rust in user mode, with capabilities and message ports, and a console service. Code is developed in a private repository for now and moves here once it reaches the contribution threshold described in [docs/STATUS.md](docs/STATUS.md). That page also tracks what is done and what is planned in each area.

## Goal documents

| Document | English | Srpski |
| --- | --- | --- |
| NeoOS 1.0: specification and development plan | [PDF](docs/pdf/en/NeoOS-1.0-specification.pdf) | [PDF](docs/pdf/sr/NeoOS-1.0-specifikacija.pdf) |
| The Neo family of computers (overview) | [PDF](docs/pdf/en/Neo-overview.pdf) | [PDF](docs/pdf/sr/Neo-pregled.pdf) |
| Promo brochure | [PDF](docs/pdf/en/Neo-promo-brochure.pdf) | [PDF](docs/pdf/sr/Neo-promo-brosura.pdf) |
| What else Neo can do | [PDF](docs/pdf/en/Neo-what-else-it-can-do.pdf) | [PDF](docs/pdf/sr/Neo-sta-jos-ume.pdf) |
| Fed up? (what Neo removes from everyday computing) | [PDF](docs/pdf/en/Neo-fed-up.pdf) | [PDF](docs/pdf/sr/Neo-dosadilo-vam-je.pdf) |

The specification is a working draft. Every figure in it is an engineering estimate, not a measurement.

## Where the work starts: Neo OS Lite

The full NeoOS targets Neo hardware that does not exist yet (Neo 500, 2000 and 4000). **Neo OS Lite** is the profile for small and existing machines, so it is the part that can be built and tested now:

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

The desktop look, *Neo Silver*, follows classic Mac OS with the best of BeOS, Haiku, MorphOS and the Amiga. Every name, icon and drawing is NeoOS's own. Serbian (Latin and Cyrillic) and English (UK and US) are the first languages, and a localisation kit makes adding any other language easy.

## Built on open source

Anything that already exists as a mature open project is used as the foundation; only what is new is written from scratch, and changes go back upstream. Candidates include Genode on seL4, Linux driver code in isolated components, Btrfs and EROFS, Mesa, Smithay, the Haiku Interface Kit, PipeWire, QEMU, LLVM and Rust.

## How to help

Code is not here yet; see [docs/STATUS.md](docs/STATUS.md) for when it arrives and which areas will be open first. Until then, issues with questions, ideas and comments on the specification are welcome.

## Licence

The licence of this repository is not final yet. The [LICENSE](LICENSE) file currently holds the GNU GPL version 2.
