# AmigaOS feature set on Neo

**English** · [Srpski](AMIGA.sr.md) · Plan, 18 November 2026

Neo takes ideas from AmigaOS: a fast shell with `LoadDesk`, a desktop of drawers and icons, the Directory Opus style of file management, and the ability to run classic Amiga software. This page lists the open projects we plan to use, what each one costs, and what its licence lets us do.

## The rule for licences

Neo is GPLv3-or-later. Projects under the AROS Public License (APL 1.1, based on the MPL 1.1) or under GPL-2.0-only cannot be merged into Neo's own code. They can still ship **as separate programs** on Neo, because a program running on an OS is not combined with it. So we keep each of them as its own program, in its own folder, under its own licence. GPL-2.0-or-later and BSD code can be merged directly.

## Items

| # | Item | Licence (checked 18 Nov 2026) | What Neo gets | Estimate (pm = person-months) |
| --- | --- | --- | --- | --- |
| A1 | **AmigaOS API layer** from AROS (exec, dos, intuition, graphics, icon, workbench libraries) hosted on Neo, the way AROS runs hosted on Linux | APL 1.1, separate program | Amiga programs rebuilt from source run on Neo natively. This is what A3 to A7 need, and it is the main cost | 60 pm |
| A2 | **AROS Shell and C: commands** behind Neo DOS | APL 1.1, separate programs | Real AmigaDOS commands (`List`, `Copy`, `Assign`, scripts) next to Neo DOS's own; Neo DOS 0.3 already has the line editor and history | 8 pm |
| A3 | **Zune** instead of MUI | APL 1.1 (part of AROS) | A MUI-compatible GUI toolkit, so MUI programs build on Neo. MUI itself is closed freeware, so we do not use it | 12 pm |
| A4 | **AROS Wanderer** as an optional "classic desk" mode | APL 1.1, separate program | Drawers, icons and a backdrop, the Workbench way, started with `LoadWB`; the Neo desk stays the default | 10 pm |
| A5 | **Directory Opus 5** ("dopus5allamigas", actively maintained in 2026) | APL 1.1, separate program | The original file manager for those who want it; Ormar Pro, Neo's own file manager, keeps borrowing its ideas | 10 pm |
| A6 | **Galileo** | GPL-2.0 fork of DOpus 5 (repository also carries GPLv3 text) | Evaluate only: it overlaps A5. We take fixes and ideas, not a second port | 2 pm |
| A7 | **Scalos** | BSD / GPL-2.0 / LGPL-2.0 (sources differ); inactive since 2023 | Ideas for the Neo desk (icon handling, plugins); an optional port only if a volunteer wants it | 8 pm (optional) |
| A8 | **FS-UAE** in Arkada | GPL-2.0-or-later, can be merged | Runs unmodified classic Amiga software and games (needs Kickstart ROMs from the user, or AROS ROM) | 6 pm |
| A9 | **Janus-UAE** coherent mode | GPL-2.0, separate program | Amiga windows appear as ordinary windows on the Neo desk instead of inside one emulator screen | 10 pm |

**Total:** about 126 pm, about 20 000 hours. About 20 pm of it overlaps with work the specification already counts (Neo DOS 8j, Ormar Pro 8k, Arkada), so the net addition is about **106 pm**, about 2% on top of the 5 350 pm estimate.

## Order

1. **With M2:** A8 FS-UAE in Arkada, and A2 shell commands. They are cheap and visible.
2. **After M3:** A1 the API layer, then A3 Zune, A4 Wanderer and A5 Directory Opus on top of it.
3. **Later or by volunteers:** A9 Janus-UAE coherent mode, A6 and A7.

The API layer waits until Neo runs on Genode and seL4 (decision 0001), so that it is written once, for the final kernel.

## Sources

- Directory Opus 5, APL 1.1: [THIRD_PARTY_NOTICES](https://raw.githubusercontent.com/BlitterStudio/dopus5/master/THIRD_PARTY_NOTICES.md), [Generation Amiga, April 2026](https://www.generationamiga.com/2026/04/22/the-return-of-directory-opus-amigas-legendary-file-manager-gets-new-life/)
- Galileo: [github.com/Hagbard-Celin/galileo](https://github.com/Hagbard-Celin/galileo)
- Scalos: [Wikipedia](https://en.wikipedia.org/wiki/Scalos), [SourceForge](https://sourceforge.net/projects/scalos/)
- Janus-UAE: [SourceForge](https://sourceforge.net/projects/janus-uae/)
- FS-UAE: [fs-uae.net](https://next.fs-uae.net/download/source)
