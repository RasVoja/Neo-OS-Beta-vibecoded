# Neo Boing Platinum OS Lite for children

**English** · [Srpski](KIDS.sr.md) · Plan, 27 November 2026

A version of Lite with a simplified interface and fewer functions, for children. It is **not a separate system**: it is the same Lite with the "Kids' room" profile as the default, a smaller set of programs and the adult features locked. The specification already has the pieces (Easy mode, Easy layout, Kids' room, large profile, "Delete without fear", Dijana's seven tasks); this page puts them together. The name is open ("Neo Kids" is only a working name).

## How it looks

- Starts in the **Easy layout**: a large Shelf with six things and the clock, big buttons, words next to every icon.
- One main action per screen, as the specification requires; a menu bar with few items.
- Dijana is more present, and help is set to "Hint".
- Delete without questions, with "Undo" strip and the Bin (already specified).
- No Pro names, no technical error codes on screen: the NEO code stays behind a "grown-up details" button.

## Two age bands (suggestion, for the owner to accept or drop)

| Band | Roughly | Desk |
| --- | --- | --- |
| Little | 4 to 7 | Six tiles: draw, music, stories, games, photos, Dijana. No files view, no typing needed to start anything |
| Big | 8 to 12 | Adds writing, Neo BASIC and Neo Cards, files in a simple view, the typing game, and a safe browser if an adult switches it on |

## What is switched off or locked

Behind a grown-up key (the Lock "Keys"):

- The Pro switch, Neo DOS and the Shell (the course "The command line for children" in Neo School still works inside the course).
- Installing programs, the Linux box, the Windows and Android rooms, torrents.
- The browser, mail and chat, unless an adult enables them; the browser can be limited to a list of sites the adult chooses.
- The poker game (18+ profile only).
- Changing the profile or system settings.

Lock already shows what leaves the machine, so a grown-up can see it. No accounts, no adverts, no telemetry, as for every Neo.

## Programs in the set

Draw and PhotoCraft in a simple mode, Quill (large, few tools), Neo BASIC, Neo Cards, Crystal, Solitaire and Minesweeper, Grandmaster chess, the Forest platformer, Gramophone, Album and the viewer, Reader, the typing game, Dijana's Day screen saver, **School Bell** (Neo's own school adventure, a homage to the ZX Spectrum's *Skool Daze*, with questions from Neo School: [GAMES.md](GAMES.md)), **DreamCatcher** in its simple mode with big nodes and stickers ([DREAMCATCHER.md](DREAMCATCHER.md)), and, when a grown-up switches them on, **Channel** with approved channels only and **Meeting** with approved contacts only ([TOGETHER.md](TOGETHER.md)). In school use, Classroom mode (version 1.1) applies.

## Time and rest

Dijana's 20-20-20 break and a "Good night" time set by an adult. These are reminders and lock-out choices for a grown-up to set, not a promise of control.

## Be plain about limits

No filter is complete, and an adult nearby is still needed. Before release, check the rules for children's data that apply in the countries where it ships. Local-only data and no accounts help, but that is a legal question for the pre-release review the specification already plans.

## How it is built

- The same ISO with a preset: the first-start choice "Who uses the computer: Kids' room" is stored as the default, and the program set comes from a list. In [BUILD-ISO.md](BUILD-ISO.md) terms it is a profile, for example `scripts/profiles/kids.env`.
- It needs the desktop, the Easy layout and Lock's Keys first, so it comes after the desktop work in the [HANDOFF.md](HANDOFF.md) queue.
