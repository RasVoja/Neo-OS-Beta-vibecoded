# BeOS and Haiku ideas on Neo

**English** · [Srpski](BEOS-HAIKU.sr.md) · Plan, 27 November 2026

What to take from BeOS and Haiku. Haiku's code is MIT-licensed (to be confirmed per component), so it can be merged into Neo, which is GPLv3-or-later. Items marked **specified** are already in the specification (chapter 8); H9 to H11 were suggestions from this page and were **accepted by the owner on 27 November 2026**.

| # | Idea | Source | State | What Neo gets |
| --- | --- | --- | --- | --- |
| H1 | Tracker as the file manager base | Haiku Tracker | specified | Cabinet grows from it, with the spatial windows |
| H2 | Interface Kit as the toolkit base | Haiku | specified | Neo Kit's starting point |
| H3 | Window tabs, Stack and Tile | Haiku | specified | Drag a tab onto another window to make one; windows glued by an edge move together |
| H4 | Replicants | BeOS | specified | A part of a program (clock, meter) dragged onto the desk lives there |
| H5 | Attributes and live queries | BFS, Tracker | specified | A saved search is a folder that refreshes itself; attribute columns in Cabinet |
| H6 | Workspaces, up to nine | BeOS | specified | Each with its own resolution and background |
| H7 | One thread per window | BeAPI | specified | A frozen window never blocks the others or the menu bar |
| H8 | Vector icons (HVIF) | Haiku | specified | Small, sharp at any size |
| H9 | **Media Kit ideas**: a graph of media nodes with low latency | BeOS Media Kit | accepted | Model for Neo's audio server, which [MUSIC.md](MUSIC.md) needs: nodes for players, effects and MIDI, sample-accurate timing |
| H10 | **MIME types that carry attributes and preferred apps** | BeOS | accepted | Feeds Datatypes: each type names its default viewer, icon and extra columns |
| H11 | **Mail as queries** | BeOS Mail | accepted | Pigeon shows folders as saved queries over message attributes, so nothing is copied |
| H12 | Package files mounted as layers | Haiku packages | partly specified | Neo's unpacked single-file programs already do this; keep the "see what the program installs" view |
| H13 | Terminal and Process Controller | Haiku | specified in spirit | Shell and Caretaker |

## Do not take

- BFS itself (Neo uses Btrfs with extended attributes) and Haiku's app_server (Neo's compositor is its own, on Wayland).

## Order

H1 to H3 and H5 arrive with the desktop and Cabinet work. H9 is decided while the audio server is designed, before [MUSIC.md](MUSIC.md) item M1.
