# What only NBPOS does

**English** · [Srpski](UNIQUE-FEATURES.sr.md) · Updated 28 November 2026

A separate list of the things Neo Boing Platinum OS (NBPOS) does differently from every current system we know of: Windows, macOS, the Linux desktops (GNOME, KDE), ChromeOS, Haiku and today's Amiga-like systems. Each item says honestly what comes closest, so the claim can be checked. "As far as we know" applies to every line: if you know a system that already does one of these, open an issue and we will correct the page.

Everything here is **[lite]** or **[done]** and runs on ordinary x86_64 and ARM computers; nothing needs Neo hardware. The one exception in the list of proven ideas is handwriting (Newton), kept there only as a distant-future idea.

Tags: **[done]** works and is tested in CI · **[lite]** planned for Lite.

## Desk and screens

| # | Feature | What is different | Closest existing thing | Status |
| --- | --- | --- | --- | --- |
| 1 | **Widgets as a screen behind** (Pro mode) | Widgets live on a layer of their own between the desk and the programs. Pull a program down partly and the widgets are there; the layer stays where you drop it | macOS Dashboard (a separate Space, removed in 2019), the Windows widget board that slides in from the side | [lite] |
| 2 | **Amiga screen drag on today's PCs** | Pull a full-screen program down by its title bar and see what is behind it; it stays where you leave it ([AMIGA.md](AMIGA.md), A10) | AmigaOS and MorphOS on their own hardware; no mainstream PC system | [lite] |
| 3 | **Windows remember their monitor by its identity** | Each window knows the monitor by maker, model and serial number, and goes back to it when the monitor is plugged in again ([MULTI-MONITOR.md](MULTI-MONITOR.md)) | Windows 11 and macOS sometimes on reconnect, not per program | [lite] |
| 4 | **Monitor profiles in one click** | "Work", "Cinema", "Presenting": monitors, resolutions, the Bar and which program goes on which monitor, all in one click | DisplayFusion and similar paid third-party tools; nothing built in | [lite] |
| 5 | **Video wall without special software** | Many monitors or projectors as one picture, with bezel correction, laid out by dragging; later across several NBPOS computers | NVIDIA Mosaic and AMD Eyefinity (some cards only), dedicated video-wall programs | [lite] |
| 6 | **Show one window on another monitor** | A live copy of only one window goes to the projector or TV; the rest of your desk stays private | presenter view inside single programs, window sharing in call programs | [lite] |
| 15 | **Smart Drop** | While you drag, the system knows the type of the object (through Datatypes) and the target shows **before you let go** what it will do with it ("Album: add 3 photos", "Pigeon: attach", "DreamCatcher: link note"). It converts on the fly when the target needs another type (TIFF dropped on a web form becomes JPEG, a URL dropped on DreamCatcher becomes a link note). A **Shelf** on the screen edge parks files between programs. Modifiers are the same everywhere: none = move, Ctrl = copy, Alt = link (alias), Shift = ask. Program icons on the Bar are drop targets, as on the Amiga and the classic Mac | macOS drag with Option and Command, third-party shelves (Yoink, Dropover), the KDE "Move / Copy / Link" menu on drop; none shows the target's action in advance or converts on the way | [lite] |

## Language and documents

| # | Feature | What is different | Closest existing thing | Status |
| --- | --- | --- | --- | --- |
| 7 | **Open Translation by right-click, offline** | Any document, page or selection to a major world language, by a local open engine (LibreTranslate with Argos models); nothing goes to a cloud service | Firefox local translation (web pages only), macOS Translate (selections, Apple's models) | [lite] |
| 8 | **Language changes while you work** | The whole system switches language without a restart and without closing documents; Latin and Cyrillic switch apart from the language | GNOME, KDE and Windows ask you to sign out or restart | [lite] |

## Character and manner

| # | Feature | What is different | Closest existing thing | Status |
| --- | --- | --- | --- | --- |
| 9 | **Dijana, a resident that never interrupts** | The logo is a character who lives on the Bar, greets, sleeps, gives a tip after an error and says good night, but never pops up in your way ([MASCOT.md](MASCOT.md)) | Clippy (it interrupted), desktop pets (not part of the system) | [done] first part |
| 10 | **Three reset habits at once** | Ctrl+Alt+Del (DOS and Windows), Ctrl+Neo+Esc (Amiga) and both Shift keys + Esc for a hard reset (RISC OS) all work | each system has its own one | [done] |
| 15 | **A sense of humour you can switch off** | The platinum ball, Plan 9 and Hitchhiker winks, lines for the day from science fiction and fantasy, haiku from the system, a name for your computer, your own motto and a Silly mode; one switch turns them all off and the real message always comes first ([EASTER-EGGS.md](EASTER-EGGS.md)). Inspired by **Plan 9 from Bell Labs** (and the film *Plan 9 from Outer Space*), with intent | `fortune` on Unix, hidden games in old systems | [done] first part (`PLAN9`, `ULLA`, `BOING`, `KNIGHT`, `KOAN`, `HAIKU` in Neo DOS) |
| 11 | **Every program says what it is based on** | Simple names (Album, FilmCraft, PhotoCraft) and, on the splash screen, in About and in Help, the open project underneath (gThumb, Kdenlive, GIMP) ([APP-NAMES.md](APP-NAMES.md)) | Linux distributions show the original names only | [lite] |

## Built-in services

| # | Feature | What is different | Closest existing thing | Status |
| --- | --- | --- | --- | --- |
| 12 | **The network as a datatype** | Web view, remote folders (FTP, SFTP, SMB, WebDAV), Flash (Ruffle), Java (`.jar`) and translation are system services every program uses, the way Amiga datatypes made a file format known to every program ([NETWORK-DATA.md](NETWORK-DATA.md)) | macOS WebKit and GVfs/KIO on Linux each cover a part; none treats them as one family with one privacy list | [lite] |
| 13 | **A remote server is a disk icon with a name** | `NAS:` or `WORK:` on the desk, the same in Cabinet, Neo DOS and every program, mounted on purpose and listed in plain text | AmigaOS mount list; Linux and macOS mount network shares but hide them in a sidebar | [lite] |
| 14 | **Only the drivers your hardware needs** | A driver starts only when its device is found, and a device plugged in later starts its driver then; the list of what runs is visible ([NETWORK-DATA.md](NETWORK-DATA.md)) | AmigaOS `DEVS:` and mount entries; Linux loads modules on demand but the default image carries thousands | [done] in part |

## Proven ideas, 1982 to today

Not new, and that is the point: each of these made life easier on a system people loved, and most of them were lost when those systems faded. NBPOS brings them back on ordinary computers. Years are the first public release that had the idea.

| Year | Idea and where it comes from | What it gives the user | In NBPOS | Status |
| --- | --- | --- | --- | --- |
| 1982 | **Instant start**, ZX Spectrum and Commodore 64 (system in ROM): ready to type in about a second | No waiting; the computer feels like an appliance | Goal: Lite to the desk in under 5 s from an SSD on the reference PC; only the needed drivers start | [lite] |
| 1983 | **Undo and document-centric work**, Apple Lisa | Mistakes cost nothing; you open documents, not programs | Undo everywhere, Pads (stationery), no Save button ([CLASSIC-MAC.md](CLASSIC-MAC.md)) | [lite] |
| 1985 | **RAM disk that survives a warm restart**, Amiga `RAM:` and later `RAD:` | Scratch work kept across a restart | `RAD:` ([RAD.md](RAD.md)) | [lite] |
| 1985 | **Mount entries and `DEVS:`**, AmigaOS | Each device is named, started only when used, and easy to see | `DEVS:`, `MOUNT`, `ASSIGN`, driver per device ([NETWORK-DATA.md](NETWORK-DATA.md)) | [done] in part |
| 1987 | **A scripting port in every program**, ARexx (bundled with AmigaOS 2.0 in 1990) | Programs work together: one script tells Quill, Cabinet and the mail program what to do | Lux port in every program, with ARexx-style commands, from Neo DOS and from Quill | [lite] |
| 1988 | **The program is a folder that knows how to start**, RISC OS `!Run` and `!Boot` | Install by copying one icon, remove by deleting it; no installer, no leftovers | Neo app bundle: one folder with the program, its icon, file types and start script | [lite] |
| 1989 | **Drag to save**, RISC OS | Save by dragging the file icon from the program straight into any folder | Cabinet and every Neo Kit program ([RISC-OS.md](RISC-OS.md)) | [lite] |
| 1989 | **Services menu**, NeXTSTEP | Any program offers its actions (translate, send to phone, make a PDF) to every other program | Right-click menu fed by system services, Open Translation first | [lite] |
| 1991 | **Alias that follows the original**, classic Mac System 7 | A link keeps working after the file is moved or renamed | Neo link, tracked by file identity, not by path | [lite] |
| 1992 | **Datatypes**, AmigaOS 3.0 | The system knows the file type by its content, not by its extension; every program opens every known format | Datatypes, 1 000+ formats, decoders in a sandbox; the network too ([NETWORK-DATA.md](NETWORK-DATA.md)) | [lite] |
| 1992 | **Everything is a file, network devices too**, Plan 9 from Bell Labs | One way to reach a disk, a server or a device | The philosophical base of the Neo mount system: volumes, servers and devices all open through the same names | [lite] |
| 1993 | **Handwriting recognition**, Apple Newton | Write on the screen and it becomes text | Distant future, only mentioned here; not planned for Lite | idea |
| 1997 | **Spring-loaded folders**, Mac OS 8 | Drag a file onto a folder, wait, and it opens so you can go deeper | Cabinet and the desk | [lite] |
| 1997 | **Extended attributes**, BeOS file system (BFS), today in Haiku | Rating, tags, date taken and author live on the file itself and are searchable, with no separate database | Attributes on NBPOS volumes, used by Album, Gramophone, Find and Cabinet ([BEOS-HAIKU.md](BEOS-HAIKU.md)) | [lite] |
| 2007 | **Quick look with the space bar**, Mac OS X 10.5 | See any file without opening a program | In Cabinet, through datatypes | [lite] |

## Accessibility (a promise, not a unique claim)

Other systems have most of these too; they are here because the owner asked for them as a firm part of the plan, not as an extra. Coding steps 150 to 159 in [PROGRESS.md](PROGRESS.md). All **[lite]**.

| Feature | What NBPOS does | Status |
| --- | --- | --- |
| **High-contrast theme, built in** | A high-contrast light and dark theme in the theme system from the first desktop release, also for Neo DOS and the boot screens; colour-blind-safe palettes for the Bar, tiles and charts | [lite], steps 150–151 |
| **Keyboard-only use** | Every window, menu, the Bar, the Neo menu and every Neo Kit control can be reached and used without a mouse; a visible focus ring everywhere; sticky keys, slow keys and mouse keys | [lite], steps 152–153 |
| **Screen reader (AT-SPI2)** | Neo Kit gives every control a name and role over **AT-SPI2**, the same accessibility bus the Linux desktops use, so **Orca** (GPL-2.0+) reads Neo programs and the Linux box programs alike; Neo DOS output can be spoken too | [lite], steps 154–156 |
| **Scalable fonts and interface** | Nothing is fixed to pixels: the interface scales from 100 to 300 percent, fonts separately from it; vector icons; the large profile from the specification uses it | [lite], steps 157–158 |
| **Calm and readable** | Reduced motion respected everywhere (already in the screen-drag plan), sound alerts also shown on screen, captions for Dijana and for videos in the help, an optional dyslexia-friendly font (OpenDyslexic, OFL) | [lite], step 159 |

## For Gen Z and the generations after

What young people expect from a computer, done without accounts, adverts or a cloud they do not own. Most of it exists elsewhere one piece at a time; together, local-first and with one simple face, it is the NBPOS difference. All **[lite]**.

| They want | NBPOS gives | Page |
| --- | --- | --- |
| Video calls with friends and class | **Meeting** (Jitsi Meet), a link is enough, no account | [TOGETHER.md](TOGETHER.md) |
| YouTube without adverts and tracking | **Channel** (FreeTube), with PeerTube in the same list | [TOGETHER.md](TOGETHER.md) |
| Streaming their games, music, lessons | **Broadcast** (OBS Studio), **Decks** (Mixxx) | [TOGETHER.md](TOGETHER.md) |
| Notion and Obsidian style notes and planning | **DreamCatcher**: mind map, linked notes, quick notes, light task board | [DREAMCATCHER.md](DREAMCATCHER.md), [EVERYDAY-APPS.md](EVERYDAY-APPS.md) |
| Google Classroom style school work | **SchoolDesk**, on the school's own network | [EVERYDAY-APPS.md](EVERYDAY-APPS.md) |
| Sharing with the phone and friends without a cloud | **LocalSend** and **Syncthing** (already in the specification's phone link), KDE Connect | [FEATURES.md](FEATURES.md) |
| Music that looks good | **Gramophone** with visualisations and skins | [EVERYDAY-APPS.md](EVERYDAY-APPS.md) |
| No waiting | Target: **under 5 seconds** from the boot loader to the desk on an NVMe SSD (today's estimate for the reference PC is 5–8 s, [FEATURES.md](FEATURES.md) section 4; to be measured on real PCs) | [REAL-PC-TEST.md](REAL-PC-TEST.md) |

## How to add to this list

A feature goes here only when we can name what comes closest and say plainly what is different. Ideas are welcome as issues; details of each feature stay on its own page ([FEATURES.md](FEATURES.md), [MULTI-MONITOR.md](MULTI-MONITOR.md), [NETWORK-DATA.md](NETWORK-DATA.md), [AMIGA.md](AMIGA.md)).
