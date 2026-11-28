# Games in Lite

**English** · [Srpski](GAMES.sr.md) · Plan, 27 November 2026

The owner asked to move the games, and as many earlier Pro features as possible, into the **Lite** version. Rule: a game goes into Lite if it runs in software on generic x86_64 or ARM hardware. Every game below runs in software on such hardware; how well each class of game runs on a Raspberry Pi 5, an Intel Core 10th generation and an AMD Zen 2 PC is estimated in [FEATURES.md, section 4](FEATURES.md#4-performance-on-todays-hardware-estimate).

Taken from the Neo games page of the overview. "Original" means an original game of the genre; licensed games need the rights holder's written licence before they ship.

## Neo's own games [lite]

| Game | Genre | Source | Lite plan |
| --- | --- | --- | --- |
| Grandmaster | Chess | Original, Stockfish engine (GPL) | **Lite.** Engine plus Neo Kit board; 3D board only if the desktop has 3D, otherwise a flat board |
| Solitaire and Minesweeper | Classic pack (Solitaire, FreeCell, Hearts, Minesweeper) | Original, genre of the Windows classics | **Lite, from first start.** Smallest and cheapest; first game to ship |
| Crystal | Falling blocks, hexagonal | Original rules (Tetris is licensed) | **Lite.** Sound and particles tied to music, from Gramophone |
| Cosmos Pinball | Pinball | Original | **Lite.** Software physics, several tables |
| Forest Bane | 2D platformer | Original | **Lite.** Colour 2D graphics |
| Block 70 | First-person shooter | Original, Quake engine (GPL) | **Lite, later.** Needs 3D in the Linux box or a native renderer |
| Uskoks | Naval adventure, Adriatic, 16th century | Original | **Lite, later.** Largest game; start with trade and sailing |
| Midnight Poker | Cards, 18+ | Original | **Lite, adult profile only.** Hidden from children's profiles ([KIDS.md](KIDS.md)); needs an age-checked profile |
| School Bell | School adventure, side view, ZX Spectrum style | **Original**, a homage to *Skool Daze* (Microsphere, 1984, ZX Spectrum) | **Lite.** Your own pupil in a school full of teachers and classmates who follow a timetable; instead of a catapult, you win by answering questions from Neo School (maths, languages, history) and by helping classmates. Spectrum look (8 colours, attribute clash as an optional filter), Spectrum-style beeper sound. Educational on purpose: the questions come from Neo School packs, teachers can add their own |
| Warblade Neo | Shoot 'em up | **Licensed**, rights holder named in the spec | **Not until the licence exists.** Original ran in a 68030 core; a software remake is possible once licensed |

## What the showcase meant, and what changes

In the specification each game also "showed a part of the hardware". On Lite every game runs in software or on the computer's own graphics card, so the line "Shows: …" is dropped from Lite documents (see the note at the end).

## Ready-made games for Lite [lite]

Added 27 November 2026. These games already exist as free software for x86_64 and ARM Linux, so they do not wait for Neo games or Neo hardware. They run in the **Linux box** through **Flatpak** (Flathub), appear on the games shelf under a simple name with the "based on" line ([APP-NAMES.md](APP-NAMES.md)), and get only what Lock allows: their own save folder, sound, and network only when the game needs it. 3D games use Mesa (on old PCs its software renderer, slower but working).

| Shown as | Based on | Genre | Licence (to re-check) | Notes |
| --- | --- | --- | --- | --- |
| Kart | SuperTuxKart | Kart racing | GPL-3.0 | 3D; local split screen and network play |
| Empires | 0 A.D. | Historical real-time strategy | GPL-2.0+ code, CC BY-SA art | 3D; needs more memory, adult and teen profiles |
| Blocks | Luanti (formerly Minetest) | Block world building | LGPL-2.1+ code, CC BY-SA media | Kids' room version with a locked server list ([KIDS.md](KIDS.md)) |
| Bubbles | Frozen Bubble | Puzzle | GPL-2.0 | 2D, small, good for first start |
| Wesnoth | Battle for Wesnoth | Turn-based fantasy strategy | GPL-2.0+ | 2D; campaigns teach slowly, suits the Calm level |

## Arcade and emulators [lite]

Arcade (emulators and boxes) runs these through Flatpak first and natively later. The user brings their own ROMs, disks and game files; Neo ships none.

| In Arcade | Based on | Licence (to re-check) | For |
| --- | --- | --- | --- |
| DOS box | DOSBox (DOSBox Staging as the candidate) | GPL-2.0+ | Old PC games and programs from DOS |
| Adventure box | ScummVM | GPL-3.0+ | Point-and-click adventures; many free games (Beneath a Steel Sky, Flight of the Amazon Queen) can be offered from the Treasury |
| Arcade machines | MAME (Multiple Arcade Machine Emulator), the standalone program first, the libretro core later | GPL-2.0+ as a whole; many source files also BSD-3-Clause | Coin-op arcade machines and many old computers and consoles, on x86_64 and ARM (Flathub has it). The user brings the ROMs; only ROMs whose owners allow free use are offered (the list on mamedev.org) |
| Retro box | RetroArch (libretro cores) | GPL-3.0; each core has its own licence | Consoles and home computers; only cores with a free licence are shipped |
| Classic Mac | from [SYSTEM7-REFERENCE.md](SYSTEM7-REFERENCE.md) | see there | The classic Mac emulators |
| Amiga | FS-UAE, from [AMIGA.md](AMIGA.md) | GPL-2.0+ | The Amiga emulator |
| ZX Spectrum | Fuse (the Free Unix Spectrum Emulator), standalone and as a libretro core | GPL-2.0+ | The Spectrum 48K, 128K and +3. Amstrad allows the Spectrum ROMs to be shipped with emulators, so this box works from the first start; many authors have released their Spectrum games for free distribution |

### The ZX Spectrum heritage

Arcade (MAME and the retro boxes) is partly a homage to the home computers of the 1980s, and the **ZX Spectrum** most of all: in Yugoslavia and across Europe it was the first computer of a whole generation, often a home-made clone, with games loaded from cassette. *Skool Daze* (1984) and its sequel *Back to Skool* (1985) by David Reidy and Keith Warrington are the classic example: a school that lives by its timetable, with teachers who have names and habits.

- **School Bell** above is Neo's own original game in that genre, not a copy: its own school, its own characters, its own rules.
- **Pyskool** by Richard Dymond (GPL-3.0) is an open-source remake of *Skool Daze* and *Back to Skool* in Python. As far as we know it builds its graphics and sounds from the original game files, which are not free, so Neo can offer Pyskool in the Treasury but the user brings the game files, as with any ROM. His **SkoolKit** (GPL-3.0), a toolkit for taking Spectrum games apart, is a good tool for the Neo School course on how old games work.
- The Spectrum box (Fuse) can run the original *Skool Daze* when the user has a copy.

Programs under GPL-2.0-only or with other licences ship as separate programs, never merged into the system (the rule in [AMIGA.md](AMIGA.md)).

Downloading game and music files through **qBittorrent** is also a Lite app; see [MUSIC.md](MUSIC.md) (M2).

## Order

1. Ready-made games and Arcade through Flatpak, as soon as the Linux box runs (needs the Linux box, sound and the games shelf).
2. Solitaire and Minesweeper (needs Neo Kit and the desktop).
3. Grandmaster, Crystal, Cosmos Pinball, Forest Bane.
4. Block 70, Uskoks, Midnight Poker.
5. Warblade Neo only with a licence.

Games use the same help rules as other programs ([HELP.md](HELP.md)): the simple name, the "based on" line, and a first page of "How to play".

## Concept hardware (separate)

> On the concept computers (Neo 500, 2000, 4000 and the NeoPocket handheld; concept hardware, not needed) each game would also show off a Neo chip or an FPGA core, and Arcade would get FPGA cores. None of this is needed for Lite ([CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md)).
