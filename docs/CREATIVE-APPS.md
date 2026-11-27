# Creative apps: FilmCraft, PhotoCraft and Album

**English** · [Srpski](CREATIVE-APPS.sr.md) · Plan, updated 27 November 2026

All three are **[lite]**: they run on generic x86_64 and ARM hardware and do not wait for Neo hardware. Each one first runs as a mature free program in the **Linux box** (through Flatpak), under its simple Neo name, with the "based on" line on its splash screen, in About and in Help ([APP-NAMES.md](APP-NAMES.md)). A Neo Kit interface comes after the desktop exists. Licences must be re-checked in each project's own repository before any code is merged.

| Shown as | Based on | Licence (to re-check) | What it does |
| --- | --- | --- | --- |
| **FilmCraft** | Kdenlive | GPL-3.0+ (some parts GPL-2.0+) | Video editor with a timeline |
| **PhotoCraft** | GIMP (raster) and Inkscape (vector) | GPL-3.0+ / GPL-2.0+ | Image editor, one entry for photos and drawings |
| **Album** | gThumb | GPL-2.0+ | Image viewer and organiser |

## FilmCraft: video editor (Kdenlive)

- **Plan.** FilmCraft is Kdenlive under the Neo name. It is mature, runs on x86_64 and ARM, and is on Flathub, so it can ship with the first Lite that has the Linux box. This replaces the earlier "Montage (Kdenlive)" slot in the specification; the program is the same, the name shown is FilmCraft.
- **Neo touches.** Opens files from Cabinet and Album, saves renders to the user's Films folder, uses Datatypes for formats when they exist, and shows Neo help pages.
- **Later.** A Neo Kit interface ("NeoGUI") over the same engine (MLT). The Rust video editor of the ArtCraft project, also called FilmCraft, is watched as a possible later engine; it is still early (versions 0.2 to 0.5 in reports).
- **Needs first.** Linux box, audio server, GPU or software rendering through Mesa.

## PhotoCraft: image editor (GIMP and Inkscape)

- **Plan.** PhotoCraft is a thin Neo wrapper with one entry on the Shelf and one first screen: "Edit a photo" opens GIMP, "Draw or design" opens Inkscape. The user sees one program, PhotoCraft; About and Help name both programs it is based on.
- **Neo touches.** A shared recent-files list, the same Open and Save places, Neo help pages with pictures, and an Easy profile that hides the expert menus where the programs allow it.
- **Later.** A Neo Kit interface; the "Brush" slot in the Workshop pack (Krita) stays as an option for drawing on a tablet.
- **Needs first.** Linux box; both programs are on Flathub.

## Album: image viewer (gThumb)

**Decided 27 November 2026: the image viewer is gThumb, shown as Album.**

- gThumb is free software (GPL-2.0+), so it can ship inside Neo and be changed.
- It gets the library, albums, folders and quick-fix feel in the Neo style: a timeline of photos, albums as Pads, one-click rotate, crop and light fixes. Bigger edits open in PhotoCraft.
- Album is also the default viewer for pictures opened from Cabinet, mail and the web.
- Needs first: the Linux box (or a native GTK port later), and Datatypes for formats.

## Order

1. Linux box with Flatpak.
2. Album (gThumb), the smallest and most used.
3. FilmCraft (Kdenlive) and PhotoCraft (GIMP and Inkscape).
4. Neo Kit interfaces for all three, after the desktop.
