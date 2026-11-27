# Classic Mac look and software on Neo

**English** · [Srpski](CLASSIC-MAC.sr.md) · Plan, 18 November 2026

Neo's look takes as many ideas as it can from the classic Mac OS (see the specification, chapter 8b, and the retro theme "First Desk"). This page plans two features around the open project [Kelsidavis/System7](https://github.com/Kelsidavis/System7), a reimplementation of Macintosh System 7 for x86 that boots through GRUB and Multiboot2, as Neo Lite now does.

## What the project is (checked 18 November 2026)

- **Licence:** MIT according to the project's listing. Before any of its code enters Neo, its LICENSE file and `docs/DERIVATION_APPENDIX.md` must be read.
- **How it was made:** by reverse-engineering the System 7 binaries with Ghidra and a language model. Whether parts of it derive from Apple's code is disputed in public comments, so its **provenance is not settled**.
- **State:** its author calls it "a booting skeleton". It runs in QEMU and is not yet a full system.

Two rules follow. We take **ideas and behaviour** from it freely. We take **code** only after the provenance check (item C3). Neo also never ships Apple's names, fonts, icons or artwork; the look is Neo's own drawing in the classic spirit.

## Items

| # | Item | What Neo gets | Estimate |
| --- | --- | --- | --- |
| C1 | **"First Desk" classic windows and controls** | Window frames, title bars with stripes, close and zoom boxes, scroll bars, buttons, check boxes and menus drawn in the classic style by neo-gfx, as Neo's own artwork. The same look as CSS for the HTML demos | 6 pm |
| C2 | **Classic Mac software in Arcade** | Arcade runs classic Mac programs and games through mature emulators: Basilisk II and SheepShaver (GPL-2.0) for 68k and PowerPC, and Mini vMac (GPL-2.0) for the early Macs. Each runs as a separate program and needs a Mac ROM the user owns | 6 pm |
| C3 | **System 7 reimplementation: evaluation** | Read its licence and derivation notes. If its provenance is clean, add it to Arcade as a way to start classic programs without an Apple ROM, and reuse its window manager behaviour as a reference for C1 | 1 pm |

**Total:** about 13 pm (person-months), about 2 100 hours. Part of C1 is already counted in the specification's theme work.

## Order

1. **With M3** (themes, Serbian and English complete): C1.
2. **After M3**, with the Amiga emulation in Arcade (see [AMIGA.md](AMIGA.md)): C2 and C3.

## Sources

- [Hackaday, 29 September 2025: Macintosh System 7 ported to x86 with LLM help](https://hackaday.com/2025/09/29/macintosh-system-7-ported-to-x86-with-llm-help)
- [OSS Insight: Kelsidavis/System7](https://ossinsight.io/analyze/Kelsidavis/System7)
