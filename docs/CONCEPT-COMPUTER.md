# Two separate things: the system, and the ideal computer concept

**English** · [Srpski](CONCEPT-COMPUTER.sr.md) · Plan, 27 November 2026

## The decision

The owner decided to separate two things that have been mixed together in the documents, demos and videos:

1. **The operating system** (Neo Boing Platinum OS, with Neo Boing Platinum OS Lite as its first release). It runs on existing generic x86_64 and ARM hardware, and it is the **dominant** thing in the repository, the demos and the videos.
2. **The ideal computer concept** (Neo 500, 2000, 4000, the NeoPocket handheld, the Neo package, FPGA cores, Sun station and so on). A computer that would ship with the system, shown **fully apart**, labelled concept hardware. In it, **Neo Boing Platinum is only one extra feature**, and not the identity of the whole.

**The name.** The system is **Neo Boing Platinum OS** (decided 18 November 2026) and stands on its own on any PC or ARM board. On the concept computer (concept hardware) the preinstalled Neo Boing Platinum OS is one extra feature of the machine, next to its chips, cartridges and ROM; the machine is never a requirement of the system.

## The rule for every document, demo and video

| Question | If yes | If no |
| --- | --- | --- |
| Does it run today, or can it run on a generic PC or ARM board? | It belongs to **the system**. Main pages, main demos | Next question |
| Does it need hardware that does not exist (Neo boards, chips, FPGA)? | It belongs to **the concept**. Separate area, label "Concept" | Ask the owner |

Every concept page, demo and video shows a visible banner "Concept: needs hardware that does not exist". The system pages never mention concept hardware as a requirement. The system is shown running in QEMU or on a real PC; the concept is shown as drawings.

## Proposed layout of the repository

```
README.md             the system first; one short pointer to the concept
docs/                 the system: status, features, plans, help (this folder)
docs/html/…/demo/     system demos (Cabinet, Neo DOS, Tiles, Crunch, First Desk, languages …)
concept/              the ideal computer: its own README, overview, brochure, technical book
concept/demo/         hardware demos (board, package, Sun station, FPGA)
```

The six PDFs and the HTML pages now in `docs/pdf` and `docs/html` mix both. They are sorted in two steps:

1. **Cut** each into a system part and a concept part (the specification and the technical book have a clear software part and a hardware part).
2. **Move** the hardware parts and hardware demos to `concept/`.

**Done on 27 November 2026 (first step):** the README now puts the system first (user guide, system demos, system documents) and lists the overview, brochure, technical book, identity page and "Neo for everyone" demo in a separate section headed "concept hardware". Every mention of Neo 500/2000/4000 in the plan pages carries the label "(concept hardware)". The documents themselves are not yet cut or moved; that is the next step, done file by file so nothing is lost.

## What the system's main demos are

Already in the repository and about the system: Cabinet, Cabinet Magellan, Neo DOS, Tiles, Crunch, First Desk, languages, school, news, Claude help (distant future, not guaranteed), phone link, and the beta manual. These become the first row in the demo list.

## Videos

The repository has no video files, only documents, web demos and screenshots. The owner's videos are kept outside it and follow the same rule: system videos show the running system; the concept video is separate, with the banner.

## Concept videos (received 27 November 2026)

Six owner-made concept videos (about 7 minutes in total): Neo promo, Neo identity, Neo in everyday life, Neo ideas at work, Neo apps at work, and "Neo from click to chip". They show the concept computer (board with ARM, FPGA and VSA-6000, Studio in a box) together with system features (Gramophone, Vault, games, montage). They are **concept** material and are to be sorted file by file with the owner before they are linked from the README.

### What the videos mean for the system (from a first look at frames)

- **Belongs to the system (Lite, shown as demo material):** First Desk and the Neo Silver look, Gramophone, Vault (privacy), the games shelf, montage, three sound themes (Crystal, Wood, Chip), three fonts with Cyrillic and Latin, tiles with live data, the resident Dijana (moods: good morning, error, sleep).
- **Concept computer only (label as concept):** the board with ARM, FPGA and VSA-6000, ROM and NVMe cartridges, solar panel and battery, the Neo 500/2000/4000 models, "Studio in a box".
- Videos that mix both need a cut or an on-screen "concept" label before they are used as the system's main demo. Only frames of two of the six were looked at; the rest is unchecked.
