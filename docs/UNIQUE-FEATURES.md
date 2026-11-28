# What only NBPOS does

**English** · [Srpski](UNIQUE-FEATURES.sr.md) · Updated 28 November 2026

A separate list of the things Neo Boing Platinum OS (NBPOS) does differently from every current system we know of: Windows, macOS, the Linux desktops (GNOME, KDE), ChromeOS, Haiku and today's Amiga-like systems. Each item says honestly what comes closest, so the claim can be checked. "As far as we know" applies to every line: if you know a system that already does one of these, open an issue and we will correct the page.

Everything here is **[lite]** or **[done]** and runs on ordinary x86_64 and ARM computers; nothing needs Neo hardware.

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
| 12 | **Smart Drop** | While you drag, the system knows the type of the object (through Datatypes) and the target shows **before you let go** what it will do with it ("Album: add 3 photos", "Pigeon: attach", "DreamCatcher: link note"). It converts on the fly when the target needs another type (TIFF dropped on a web form becomes JPEG, a URL dropped on DreamCatcher becomes a link note). A **Shelf** on the screen edge parks files between programs. Modifiers are the same everywhere: none = move, Ctrl = copy, Alt = link (alias), Shift = ask. Program icons on the Bar are drop targets, as on the Amiga and the classic Mac | macOS drag with Option and Command, third-party shelves (Yoink, Dropover), the KDE "Move / Copy / Link" menu on drop; none shows the target's action in advance or converts on the way | [lite] |

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
| 11 | **Every program says what it is based on** | Simple names (Album, FilmCraft, PhotoCraft) and, on the splash screen, in About and in Help, the open project underneath (gThumb, Kdenlive, GIMP) ([APP-NAMES.md](APP-NAMES.md)) | Linux distributions show the original names only | [lite] |

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

A feature goes here only when we can name what comes closest and say plainly what is different. Ideas are welcome as issues; details of each feature stay on its own page ([FEATURES.md](FEATURES.md), [MULTI-MONITOR.md](MULTI-MONITOR.md), [AMIGA.md](AMIGA.md)).
