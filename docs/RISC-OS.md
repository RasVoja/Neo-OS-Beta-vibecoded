# RISC OS ideas on Neo

**English** · [Srpski](RISC-OS.sr.md) · Plan, 27 November 2026

RISC OS shaped the Bar and drag-to-save in Neo. This page lists what else is worth taking. Most of the RISC OS sources are open (Apache 2.0, to be confirmed per component), but the idea comes first: Neo borrows behaviour, not code. Items marked **specified** are in the specification; **new** are suggestions for the owner to accept or drop.

| # | Idea | State | What Neo gets |
| --- | --- | --- | --- |
| R1 | Icon bar and pinboard | specified | The Bar and the Shelves |
| R2 | Drag the icon from a save box to save | specified | No searching through a dialog |
| R3 | **Three-button mouse: Select, Menu, Adjust**. Menu opens the menu under the pointer anywhere; Adjust does the same action as Select but keeps the window or menu open | new | Fits the optional "right button for menus" already in the specification. Adjust is the new part, for example opening a folder in a new window without closing the old one |
| R4 | **Applications as folders** (`!App`): double-click runs it, Shift-double-click opens it to look inside | new | Neo's single-file programs keep their size; the view "open the program" shows what it contains and what it may do |
| R5 | Scripts in the boot sequence (Obey files) | covered | Lux and Neo DOS scripts |
| R6 | BBC BASIC as the first language | covered, with a question | Neo BASIC is the first language. Evaluate a BBC BASIC dialect (for example Brandy) for people with old listings |
| R7 | **Run RISC OS itself** in Arcade | new, to evaluate | RPCEmu or RISC OS on a Raspberry Pi board for old RISC OS programs, as a separate program |
| R8 | Small bundled apps (Edit, Draw, Paint) with a size budget | covered | Small programs with a memory budget, already a Neo design rule |

## Order

R3 and R4 are cheap and come with the desktop. R6 and R7 are evaluations, decided after M2.
