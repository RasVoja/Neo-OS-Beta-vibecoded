# Everyday apps: the good ideas of programs people love

**English** · [Srpski](EVERYDAY-APPS.sr.md) · Plan, 28 November 2026 · **[lite]**

The owner listed programs people like and asked for their best ideas in NBPOS. Rule: we take the **idea**, not the look or the name, build it with Neo Kit on free software, and keep it **offline-first, no account, no cloud required**. Every program keeps the simple name and the "based on" line ([APP-NAMES.md](APP-NAMES.md)).

| Inspiration | Idea we take | Goes into NBPOS as | Based on (licence to re-check) |
| --- | --- | --- | --- |
| **Google Keep** | Catch a note in two seconds, colours, checklists, pin a note to the desk | **DreamCatcher Notes**: Neo + D catches a note; colour, checklist, reminder; a pinned note shows as a widget on the desk (or on the widget screen behind, Pro mode) | DreamCatcher ([DREAMCATCHER.md](DREAMCATCHER.md)) |
| **Obsidian** | Notes as plain files, `[[links]]` between them, a graph of the links | **DreamCatcher graph mode** plus Quill: a folder of Markdown notes, `[[links]]` in Quill, and DreamCatcher draws the map of links; nothing locked in a database | Plain Markdown; graph view in Neo Kit |
| **Shortcut** (project tracker) | Stories, a board, small iterations, without the weight of Jira | **DreamCatcher tasks**: a light tracker inside a board: To do / Doing / Done, labels, who does it, a due date; for a family, a class or a small team, not a company | DreamCatcher |
| **Google Classroom** | A task with a due date, a discussion, files sorted by type | **SchoolDesk** (SR: Školska klupa): the teacher's NBPOS computer hosts the class on the local network; tasks, due dates, a discussion per task, files sorted by type (documents, pictures, DreamCatcher boards); no Google account, nothing leaves the school unless the school wants it | Neo's own, with ideas and exchange formats from **Kolibri** (MIT, offline-first learning) and **Moodle** (GPL-3.0, import and export of courses) |
| **Dub Music Player** | A clean library without clutter, genre "bins", instant search | **Gramophone**, library view: one tidy list, genres as bins you drop albums into, search as you type, no store and no streaming upsell | Gramophone ([MUSIC.md](MUSIC.md)), metadata from **MusicBrainz** (CC0 data) |
| **Sonique 2** | A player with visualisations and skins | **Gramophone**, player view: full-screen visualisations (projectM, LGPL-2.1, compatible with MilkDrop presets) and skins through the Neo theme system; one small "Sonique-like" round skin as a homage | Gramophone, projectM |
| **AccuWeather** | A clear forecast, radar animation | **Weather** (SR: Vreme): a desk widget and a program of its own; hourly and 14-day forecast, radar animation, the last forecast kept for offline; no location leaves the computer except the town you typed | **Open-Meteo** (data CC BY 4.0, server AGPL-3.0, can be self-hosted), radar from national weather services where their terms allow (for Serbia RHMZ), **RainViewer** as an option |
| **Discogs and IMDb** | A catalogue of your own records and films, credits, release details | **Collection** (SR: Kolekcija): a local SQLite database of your music, films, books and games, scanned from your folders or typed in; offline-first; filling in details from MusicBrainz (CC0) and, if the user turns them on, the Discogs API and TMDB | SQLite (public domain), MusicBrainz; Discogs and TMDB only as optional online sources, under their terms |
| **PDFGear** | A free PDF tool: read, annotate, merge, split, no cloud | **Reader** (SR: Listaj), PDF tools: highlight, notes, drawing, forms, signatures; merge, split, reorder, compress; all on the computer | Poppler (GPL-2.0+), **qpdf** (Apache-2.0), ideas from **Xournal++** (GPL-2.0+) |
| **Icecream Ebook Reader** | A bookshelf, bookmarks, reading progress | **Reader**, books: a shelf for EPUB, MOBI, FB2, CBZ and PDF, bookmarks and notes, reading progress, an optional daily reading goal (Dijana mentions it at most once a day, only if asked) | **Foliate** (GPL-3.0) |
| **HandBrake** | Convert video in batches with ready presets, clean interface | **VideoCraft** (SR: VideoCraft): a converter next to FilmCraft: drop files, pick a preset ("for phone", "for TV", "small for mail", "archive"), run a batch; uses the graphics card when it can | **HandBrake** (GPL-2.0), FFmpeg |

## Estimate and order

About 20 person-months in all: DreamCatcher Notes, graph mode and tasks 4 (on top of DreamCatcher's 8), SchoolDesk 5, Gramophone library and visualisations 2, Weather 1.5, Collection 2.5, Reader PDF tools and shelf 3, VideoCraft 2 (first as HandBrake in Flatpak, about 0.2).

1. Flatpak first, as soon as the Linux box runs: HandBrake (VideoCraft), Foliate (Reader books), projectM.
2. With the desktop and Neo Kit: Weather widget, DreamCatcher Notes, Reader PDF tools.
3. After DreamCatcher and Quill: graph mode and tasks; then SchoolDesk (it needs the "together" layer from [TOGETHER.md](TOGETHER.md)); Collection.
