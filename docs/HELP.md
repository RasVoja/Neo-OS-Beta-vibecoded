# Help and user guide: inside the system, as HTML and as PDF

**English** · [Srpski](HELP.sr.md) · Plan, 27 November 2026

The owner asked that the user guide and help be as available as possible: **inside the system** as graphic messages and pictures of the interface, and **outside it** as HTML and PDF.

## Read the guide now

| | English | Srpski |
| --- | --- | --- |
| **User guide, start page** | [guide/html-en/index.html](../guide/html-en/index.html) | [guide/html-sr/index.html](../guide/html-sr/index.html) |
| The whole guide as one PDF | [NBPOS-guide-en.pdf](../guide/NBPOS-guide-en.pdf) | [NBPOS-vodic-sr.pdf](../guide/NBPOS-vodic-sr.pdf) |
| Beta manual of the desktop | [neo-manual-beta.html](html/en/neo-manual-beta.html) | [neo-prirucnik-beta.html](html/sr/neo-prirucnik-beta.html) |
| Feature list | [FEATURES.html](../guide/html-en/FEATURES.html) | [FEATURES.sr.html](../guide/html-sr/FEATURES.sr.html) |
| Games | [GAMES.html](../guide/html-en/GAMES.html) | [GAMES.sr.html](../guide/html-sr/GAMES.sr.html) |
| Creative apps (FilmCraft, PhotoCraft, Album) | [CREATIVE-APPS.html](../guide/html-en/CREATIVE-APPS.html) | [CREATIVE-APPS.sr.html](../guide/html-sr/CREATIVE-APPS.sr.html) |
| Music | [MUSIC.html](../guide/html-en/MUSIC.html) | [MUSIC.sr.html](../guide/html-sr/MUSIC.sr.html) |
| For children | [KIDS.html](../guide/html-en/KIDS.html) | [KIDS.sr.html](../guide/html-sr/KIDS.sr.html) |
| Bootable image and first boot | [BUILD-ISO.html](../guide/html-en/BUILD-ISO.html) | [BUILD-ISO.sr.html](../guide/html-sr/BUILD-ISO.sr.html) |
| System demos in the browser | [neo-demos.html](html/en/demo/neo-demos.html) | [neo-demoi.html](html/sr/demo/neo-demoi.html) |

The HTML pages open from a downloaded copy of the repository, a folder or a USB stick, with no server and no network. On GitHub, download the repository as a ZIP first, because GitHub shows HTML as source.

## One source, three forms

Every help page is written once, in Markdown, in English and in Serbian (`help/en/…`, `help/sr/…`), with pictures next to it. A build step makes the three forms from that single source, so they cannot disagree.

| Form | For | How |
| --- | --- | --- |
| **In the system** | Everyone, offline, at the moment of need | The Help program and context help (below) |
| **HTML** | Reading in any browser, on the web page, on another computer | `scripts/build-docs.sh html`: one page per topic, an index and a search box, no server needed, opens from a folder or a USB stick |
| **PDF** | Printing, sending, reading on paper | `scripts/build-docs.sh pdf`: the whole guide as one book, with contents and the pictures |

## Inside the system

1. **Help key.** One key (F1 and a button in the menu bar) opens help for what is on screen: the window, the dialog, the error.
2. **Graphic messages.** Instead of a wall of text, help shows a **picture of the real interface** with a numbered arrow or frame on the part being explained, and two or three short lines. The pictures are captured from the real system by a test (below), so they do not go out of date.
3. **"Show me".** Where a task is a few steps, the Help program can point at the real control (a frame pulses around it), one step at a time. It never clicks for the user.
4. **Dijana** reads the "Hint" level (as in [KIDS.md](KIDS.md)) and points to the right help page; she does not repeat it.
5. **Error messages** carry the NEO code and a link to the help page of that code. In Easy mode the code is behind "grown-up details".
6. **First start tour:** five short screens with pictures, skippable, available again from Help.
7. **Language.** Help follows the system language and changes with it while the system runs ([FEATURES.md](FEATURES.md)). If a page has no translation, it shows English and says so.
8. **Offline always.** Help is part of the image. No account, no network.
9. **Accessibility.** Every picture has a text description; pages work with the screen reader and the large profile; nothing important is only in a picture.

## Three layers of depth

- **Quick start** (one screen per program): what it is, the three things you will do first.
- **User guide** (a few pages per program): the tasks, with pictures.
- **Reference** (technical): shortcuts, formats, Neo DOS commands. This is the part for experts.

## Every program has the same first help page

"What this program is based on", with the same facts as About ([APP-NAMES.md](APP-NAMES.md)), then "How to start". Games begin with "How to play" ([GAMES.md](GAMES.md)).

## Pictures from the real system

A boot test runs the system in QEMU, opens each program in its main states and saves a screenshot with a name from a list (`help/shots.list`). The Help program shows these files, and the HTML and PDF builds include the same files. When the interface changes, the pictures change with the next build. Today the repository already has screenshots of this kind in `docs/img/`.

An arrow, frame or number is added on top by a small script from a text file (`help/annotations/…`) that holds the coordinates, so the original screenshot is never edited by hand.

## Beginning today

The repository already has a beta manual (HTML) and demos in `docs/html/`, and six PDF documents. The first step is to move these into the one-source build, then add the in-system Help program once the desktop exists.

## Build tool

`scripts/build-docs.sh` uses `pandoc` for HTML and `wkhtmltopdf` for PDF, both free software. It is a first version and has been tested only on the documents in this folder. `scripts/build-guide.py` builds the published guide in `guide/` (HTML for both languages with an index, and one PDF per language); it needs only pandoc and, for the PDF, wkhtmltopdf or a headless Chrome. Run it after changing any page in `docs/`.

## Order

1. One-source build for HTML and PDF (done as a first version).
2. Screenshot test and annotation script.
3. Help program, F1, error codes with links (after the desktop and Neo Kit).
4. "Show me" and the first-start tour.
