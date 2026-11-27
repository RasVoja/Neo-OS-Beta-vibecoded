# Residents, and the Snake (Zmija) as the second one

**English** · [Srpski](RESIDENTS.sr.md) · Plan, 27 November 2026

Dijana is the first resident. The specification (chapter 8c) already names the first candidate for another one: the snake from the song "Ide zmija" by DJ Nick DeeTronic, as a dancing resident beside Gramophone. The owner has talked with DJ Nick and he agrees to a **test build**.

The specification also lists the Crystal theme (by DJ Nick, Nikola Tomić, Deetronic Design Studio, for AmigaOS 4) as a third global style, whose host is the Snake. The written licence for the Snake should say whether it also covers the Crystal theme, its mouse, icons, backgrounds and boot sound.

## What a resident is

A package of animations, sounds and rules made with a tool from the Neo SDK. It only receives events from the system; it sees no files and no network, and shows no adverts. Residents are bought once and without subscription; community residents are free under a free licence; well-known characters need an official licence.

## What is needed for the Snake

| # | Need | Notes |
| --- | --- | --- |
| 1 | **A written licence** | Draft in the private folder (`nbpos-private/ZMIJA-LICENCA-NACRT.sr.md`): covers the character only, only for this OS, a test phase first. Needs a lawyer for intellectual property in Serbia before signing |
| 2 | **Who holds the rights** | The character, the song (composition, lyrics) and the recording may belong to different people: co-authors, a publisher, a label. Get a short written confirmation of who owns what |
| 3 | **The artwork** | Who draws the Snake, and a written transfer or licence from that artist. Pixel, vector and animation versions, like Dijana's |
| 4 | **The song** | Safest: do not ship it. The Snake dances to whatever plays in Gramophone (beat detection). If a short excerpt is wanted, composition and recording each need a licence |
| 5 | **The resident framework** | Event feed, package format (sprite sheet, atlas, rules file), Neo SDK tool, a picker in Settings, a store entry. Not built yet; comes with the desktop |
| 6 | **Test build rules** | Closed testing with named testers, marked TEST, with an end date. The Snake's files live in a **separate package with its own licence file** and are **not** committed to the public GPL repository |
| 7 | **Keep licences apart** | The GPL covers Neo's code. It gives no right to the Snake, and the Snake's licence gives no right over the code |

## Order

Resident framework with Dijana first; then the Snake as the second package, in a private test build; the commercial terms are decided only after the test.
