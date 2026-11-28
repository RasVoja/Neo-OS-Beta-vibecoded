# DreamCatcher: every idea has its place

**English** · [Srpski](DREAMCATCHER.sr.md) · Plan, 28 November 2026 · **[lite]**

DreamCatcher is a new NBPOS program: a **mind map**, a **document organiser** and a **workflow companion** in one window. It catches ideas before they fly away, shows how documents, people and tasks are tied together, and walks with you from the first sketch to the finished work. Tagline: **"Every idea has its place."**

The name is a working name; a trademark check comes before release (several products already use "Dreamcatcher").

## What it does

- **Nodes on a board.** Every idea is a node: a word, a sentence, a picture, a voice note, a file from Cabinet, a document from Quill, a link, a task. Nodes are joined by lines that say how they are related ("needs", "part of", "contradicts", "next").
- **Documents are real nodes.** Drag a file from Cabinet onto the board and it is linked, not copied; rename or move the file and the board still finds it (through Find, the local index). Double-click opens it in its own program, through the Neo Plumber ([PLAN9.md](PLAN9.md)).
- **From map to outline to tasks.** The same board shows as a mind map, as an outline (which Quill opens as the skeleton of a document), or as a task list with three columns: *To do*, *Doing*, *Done*. A node becomes a task with one key.
- **Workflow companion.** A board can have steps ("Research › Draft › Review › Send"). DreamCatcher shows which step you are on, what is still open, and offers the Pad (template) that fits the next step. No deadlines nagging: Dijana mentions an open step at most once a day, and only if you asked her to.
- **Catch quickly.** A key combination from anywhere (Neo + D) opens a small catch window: type or speak an idea, it lands on the "Inbox" board and waits to be sorted.
- **Together on the local network.** Two or more NBPOS computers in the same home or classroom can open one board and edit it at the same time, each with their own colour pointer, without an account and without a server on the internet (see [TOGETHER.md](TOGETHER.md)).
- **Plain file format.** A board is a folder with one text file (nodes and links) and its pictures, so Keeper, Chronos and `p9fs` handle it like any other folder; export to OPML, Markdown, FreeMind `.mm`, PNG, SVG and PDF.

## Works with

| Program | How |
| --- | --- |
| Quill (Pero) | An outline opens as a new document; headings in a document can be sent back as nodes |
| Cabinet Pro (Ormar Pro) | Drag files in; a board can be a "smart folder" in Cabinet; Magellan rules can put new files on a board |
| Calendar (Rokovnik) | A task with a date shows in the calendar |
| Find (Traži) | Searches boards, nodes and the documents they point to |
| Neo School | Ready boards for school projects and for learning a language |
| Kids' room | A simple mode with big nodes, pictures and stickers, no task columns ([KIDS.md](KIDS.md)) |

## Based on

The mind-map engine follows open-source programs: **Minder** (GPL-3.0) and **VYM** (GPL-2.0) for the map, **Heimer** (GPL-3.0) for its simple style, and the FreeMind format for exchange. Live editing on the network uses a CRDT library (**Yjs** or **Automerge**, both MIT). The Neo interface is new, written with Neo Kit. Licences are checked again before code is taken ([APP-NAMES.md](APP-NAMES.md)).

## Estimate and order

About 8 person-months: board and nodes 3, Cabinet, Quill and Find links 2, outline and task views 1.5, live editing on the local network 1.5. It needs Neo Kit and the desktop first, so it comes after Cabinet and Quill. A first version can start earlier as a Flatpak program in the Linux box (Minder with a Neo theme) so the idea can be tried.
