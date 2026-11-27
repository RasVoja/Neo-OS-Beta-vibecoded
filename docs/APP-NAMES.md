# App names and credits: what each program is based on

**English** · [Srpski](APP-NAMES.sr.md) · Rule, 27 November 2026

## The rule

1. **Simple names.** A program carries a short everyday name (Quill, Cabinet, Album), not the upstream name.
2. **Say what it is based on.** Wherever the program introduces itself, it also names its base:
   - **Splash screen:** the Neo name and one small line, for example "based on LibreOffice Writer".
   - **About:** the base program and its version, its licence, a link to the upstream source, and what Neo changed.
   - **Help:** the first page is "What this program is based on", with the same facts and a thanks to the authors.
   - **Documentation and the Treasury entry:** the same line, in the same words.
3. **Third-party programs that are not rebuilt keep their own name** (for example Wesnoth, Hydrogen). They still get the "based on" line, written as "made by".
4. Credits are kept in one file per program (`CREDITS`), and all four places read it, so they cannot disagree.
5. A program under a different licence is a separate program (see [AMIGA.md](AMIGA.md)); its licence text is shown in About.

## Names

Taken from the specification; the last four are the November 2026 additions.

| Neo name | Does | Based on |
| --- | --- | --- |
| Cabinet | files | Haiku Tracker, libarchive, GVfs |
| Browser | web | WPE WebKit, Epiphany parts |
| Pigeon | mail | Geary core |
| Quill | writing | LibreOffice Writer |
| Abacus | spreadsheets | LibreOffice Calc |
| Canvas, Stage | drawing, slides | LibreOffice Draw, Impress |
| Reader | PDF, EPUB, comics | Poppler, Foliate |
| Album | photo viewer and organiser | gThumb, with a Neo Kit interface |
| Gramophone | music | GStreamer, HippoPlayer (see [MUSIC.md](MUSIC.md)) |
| Cinema | video | mpv |
| Wave | audio editor | Audacity engine |
| Calendar | notes, calendar | Evolution Data Server, CalDAV |
| Keyring | passwords | KeePassXC format |
| Arcade | games, retro | RetroArch, libretro, MAME, DOSBox, ScummVM, FS-UAE |
| Neo BASIC | first programs | SmallBASIC core |
| Keeper | backups | Btrfs snapshots, restic |
| FilmCraft | video editing | Kdenlive (later a Neo Kit interface) |
| PhotoCraft | image editing and drawing | GIMP and Inkscape (later a Neo Kit interface) |
| Tracker | module music | libopenmpt, with ideas from Schism Tracker and MilkyTracker |
| Drums | drum machine | Hydrogen |
| Torrent | downloads | qBittorrent |
| Kart, Empires, Blocks, Bubbles, Wesnoth | games | SuperTuxKart, 0 A.D., Luanti, Frozen Bubble, Battle for Wesnoth (see [GAMES.md](GAMES.md)) |

The Serbian names of the system programs (Ormar, Pero, Abakus and so on) stay as in the specification.
