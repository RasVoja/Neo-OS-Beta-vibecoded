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

Taken from the specification; the rows from FilmCraft down are the November 2026 additions.

| Neo name | Does | Based on |
| --- | --- | --- |
| Cabinet | files | Haiku Tracker, libarchive, GVfs |
| Browser | web | WPE WebKit, Epiphany parts |
| Pigeon | mail | Geary core |
| Quill | writing | LibreOffice Writer |
| Abacus | spreadsheets | LibreOffice Calc |
| Canvas, Stage | drawing, slides | LibreOffice Draw, Impress |
| Reader | PDF, EPUB, MOBI, comics; PDF notes, merge and split; bookshelf | Poppler, Foliate, qpdf |
| Album | photo viewer and organiser | gThumb, with a Neo Kit interface |
| Gramophone | music: clean library with genre bins, visualisations and skins | GStreamer, HippoPlayer, projectM (see [MUSIC.md](MUSIC.md)) |
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
| DreamCatcher | mind map, document organiser, workflow companion | Ideas from Minder, VYM and Heimer, FreeMind format, Yjs or Automerge for live editing (see [DREAMCATCHER.md](DREAMCATCHER.md)) |
| Channel | YouTube and PeerTube without adverts or account | FreeTube, later yt-dlp and mpv (see [TOGETHER.md](TOGETHER.md)) |
| Meeting | video calls, classrooms | Jitsi Meet |
| Broadcast | screen recording, streaming | OBS Studio |
| Decks | DJ mixing | Mixxx |
| Quill Together | one document written by several people on the local network | Quill with a CRDT layer; Collabora Online for full office documents |
| Neo Plumber | sends a file, URL or address to the right program | Plan 9's plumber (see [PLAN9.md](PLAN9.md)) |
| School Bell | school adventure game | Original, a homage to *Skool Daze* (ZX Spectrum) |
| Spectrum box | ZX Spectrum emulator in Arcade | Fuse |
| SchoolDesk | class tasks, due dates, discussion | Neo's own, ideas from Kolibri and Moodle (see [EVERYDAY-APPS.md](EVERYDAY-APPS.md)) |
| Weather | forecast widget and program, radar | Open-Meteo, national weather services |
| Collection | catalogue of your music, films, books, games | SQLite, MusicBrainz; Discogs and TMDB optional |
| VideoCraft | video converter, batches and presets | HandBrake, FFmpeg |

The Serbian names of the system programs (Ormar, Pero, Abakus and so on) stay as in the specification.
