# System 7 references: window look, fonts, and a Classic Mac emulator in Arcade

**English** · [Srpski](SYSTEM7-REFERENCE.sr.md) · Plan, 27 November 2026

The owner asked to take three things from the Kelsidavis/System7 project. Here is what it turned out to be, and what Neo does with each point.

## What Kelsidavis/System7 is

An open-source reimplementation of Apple Macintosh System 7 for modern x86 hardware, written in C, booted through GRUB2 and Multiboot2, under the MIT licence (as listed by public repository statistics). It describes itself as an experimental proof of concept, built with the help of a language model. It is **not a CSS project**.

Two cautions: it shows the Chicago bitmap font and a rainbow Apple logo, which are Apple's. Neo copies neither. Neo takes the behaviour and the spirit, as the specification says, never Apple's graphics, fonts or trademarks.

## 1. CSS for classic windows (title bars, radio buttons, check boxes, scroll bars)

The CSS libraries for this look are separate projects (for example system7.css). Their licences and the licences of the fonts they ship were not confirmed, so **no code or font is copied**. Instead `docs/assets/first-desk.css` is written from scratch for the First Desk theme: striped title bar, close and zoom boxes, one-bit radio buttons and check boxes, rounded buttons with a double border for the default, and one-bit scroll bars. `first-desk-demo.html` shows them. It is for documents, demos and the web page; the real system draws its theme in Neo Kit.

*Not yet looked at in a browser; check it before use.*

## 2. Font and icon style for documentation and demos

Neo has its own bitmap font, "Neo System", planned in Latin and Cyrillic, plus its own pixel icons. Chicago and Geneva are Apple's; Neo does not use them or copies of them. Until Neo System exists, documents and demos use a monospace system font. The feel (heavy one-pixel strokes, tight spacing) is the guide for the new font, not the glyph shapes.

## 3. A Classic Mac emulator in Arcade

Arcade picks the way to run a program, and the user picks the program. Classic Mac OS becomes one more system there, in the way the Amiga is run today.

| Emulator | Licence | Runs |
| --- | --- | --- |
| Basilisk II | GPLv2 | 68k Macs, System 7 to 8.1 |
| SheepShaver | GPLv2 | PowerPC Mac OS 7.5.2 to 9 |
| Mini vMac (also as a libretro core) | GPLv2 | Compact Macs, System 3 to 7.5 |
| Snow | MIT | 68k Macs (128K to Mac II) |
| Advanced Mac Substitute | to check | old Mac programs without Apple ROM or system software |

**Not shipped:** Apple ROM images and Apple system software. Most of these emulators need both. Arcade asks the user for their own copies and shows a plain explanation, and then runs the program as with any other system. Emulators ship as separate programs, per the licence rule in [AMIGA.md](AMIGA.md).

## Order

The CSS and the demo are ready now. The emulators come with Arcade, after the Linux box; Advanced Mac Substitute is worth testing first because it needs no Apple files.
