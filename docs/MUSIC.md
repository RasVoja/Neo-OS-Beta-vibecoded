# Music and everyday apps: HippoPlayer, qBittorrent, Hydrogen, trackers

**English** · [Srpski](MUSIC.sr.md) · Plan, 27 November 2026

Decisions of the project owner, with what each one needs first. Licences are as found on 27 November 2026 and must be read again in the project's own files before any code is merged. NBPOS is GPLv3-or-later; a program under another licence ships as a **separate program** (see the rule in [AMIGA.md](AMIGA.md)).

**Scope: all six items are [lite].** None of them needs Neo hardware; they run on generic x86_64 and ARM PCs, first through Flatpak in the Linux box, later natively where it pays off. qBittorrent is also how Gramophone and the tracker fetch large free music collections, such as Creative Commons albums and module archives, with Lock limiting it to the download folder.

| # | Item | Licence (to re-check) | Plan | Needs first |
| --- | --- | --- | --- | --- |
| M1 | **HippoPlayer** as the audio engine of Gramophone | MIT and Apache per the Arch package; plugins such as UADE carry their own licences | Use the player core and its replay plugins for Amiga, C64 and chip formats. Neo's own front end (Gramophone) replaces its Qt window. Fits the "Paula mode" for MOD music in the specification | Audio driver and a software mixer; Linux box until the core is built natively |
| M2 | **qBittorrent** as the torrent app | GPL, version to be checked | Qt program, so it runs in the Linux box. Lock gives it network access and one download folder (the Letterbox), nothing else | Network stack, Linux box, Lock |
| M3 | **Hydrogen** drum machine | GPL-2.0-or-later | Separate program in the Linux box, audio through Neo's audio server | Audio server, Linux box |
| M4 | **Beatmaker with a language model** | to be chosen | A local model (no cloud) writes drum patterns from a text prompt and hands them to Hydrogen as a song file or MIDI. Always a preview first, always undoable. On Lite it runs on the CPU with a small model | M3, local-model runtime (also needed by Ask Neo) |
| M5 | **Tracker** (OpenMPT with features of Schism Tracker and MilkyTracker) | OpenMPT BSD-3-Clause; Schism and MilkyTracker GPL (to check) | **Decided 27 November 2026: a Neo tracker**, built on libopenmpt, that adds Schism's and MilkyTracker's keyboard workflow and features. OpenMPT itself is Windows-only; running it in the Windows box stays an optional fallback | Audio server, MIDI ports |
| M6 | **Mutual exchange** between tracker, Hydrogen and the beatmaker | n/a | Live MIDI between programs through named Exec-N ports (`midi.*`, plus USB MIDI devices). Stems as WAV files: each program exports to one shared Stems folder, others see new files at once (a Cabinet trigger) and can import them | M3 and M5, message ports (done) |

## Order

1. Audio driver and audio server (everything below needs sound).
2. M1 HippoPlayer in Gramophone, the first audible result.
3. Network, Linux box, then M2 qBittorrent.
4. M3 Hydrogen, M5 tracker, M6 exchange, then M4 the beatmaker.

## Open questions

- Which Neo audio API do the Linux box programs see (PipeWire-compatible, as the specification says)?
- Build HippoPlayer's core natively on Neo, or keep it in the Linux box for good?
- Which model, and how large, for M4 on an ordinary processor (from a Raspberry Pi 5 to a Ryzen 5 3600)?

## Concept hardware (separate)

> A sound chip on the concept computers (Neo 500, 2000, 4000, NeoPocket; concept hardware, not needed) would only make these programs faster ([CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md)).
