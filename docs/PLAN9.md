# Plan 9: a homage, and the ideas worth taking

**English** · [Srpski](PLAN9.sr.md) · Plan, 28 November 2026

**Plan 9 from Bell Labs** was the research system that the authors of Unix (Rob Pike, Ken Thompson, Dave Presotto, Phil Winterbottom and others) built at Bell Labs from the late 1980s. They named it after Ed Wood's film **Plan 9 from Outer Space** (1957, released 1959), often called the worst film ever made, and gave it a bunny mascot, Glenda. Few people ever used it, yet much of today's computing quietly comes from it: **UTF-8** (Pike and Thompson, 1992), the **9P** file protocol that Linux, QEMU and WSL use today, `/proc`, and the idea that network services can simply be files. Since 2021 the code is under the MIT licence (Plan 9 Foundation), and the 9front fork is still developed.

NBPOS takes the best of it in two ways: a homage to the system and to the film, and a short list of ideas that make Neo safer and simpler. All of it is **[lite]**: software only, no Neo hardware needed.

## The homage [lite]

**To the system**
- In the Fortress kernel and the Neo mount system the rule is Plan 9's: **everything is a file**, or more exactly, every resource a program uses has a name in its namespace, and the program reaches it only through a port it was given (section "Ideas" below).
- **`p9fs`**: Neo's internal network file transport speaks **9P2000** (with the `9P2000.L` dialect for the Linux box). Every page of these docs that mentions sharing folders between NBPOS computers, the Linux box or a virtual machine means `p9fs` underneath.
- About › Credits names Plan 9 from Bell Labs next to AmigaOS, classic Mac OS, BeOS and RISC OS, the systems Neo learns from.
- UTF-8 is the only text encoding inside Neo; the Serbian Latin and Cyrillic support in [FEATURES.md](FEATURES.md) rests on it, and the credit says where it came from.

**To the film** (small, never in the way, switched off in the Kids' room only if a grown-up wishes)
- **Plan 9 screen saver:** flying saucers on visible strings, swaying over a cardboard graveyard, with Criswell's line *"Future events such as these will affect you in the future"* in the corner. Drawn by Neo, no frames from the film.
- **Neo DOS:** `PLAN9` prints one line of thanks to Bell Labs and one quote from the film; `VERSION FULL` mentions both.
- **Cinema sample:** the film itself is widely listed as in the public domain in the US (its copyright was not renewed). If a licence check before release confirms it for the countries we ship to, Cinema can offer it as a free classic to download from the Treasury, never preinstalled.
- **Dijana** knows one joke about it, told at most once a year, on 22 July (the film's premiere in 1959, give or take the source).

## Ideas worth building into NBPOS [lite]

| Plan 9 idea | What it does there | Neo version | Where it helps |
| --- | --- | --- | --- |
| **Plumber** | A small service gets a piece of text, a file name or a URL and its context, and rules decide which program opens it and how (a file name with `:42` opens the editor at line 42) | **Neo Plumber** (SR: Razvodnik), a system service next to Datatypes. Datatypes knows *what* a file is; Plumber decides *who* opens it and *what to do*, also for URLs, mail addresses, phone numbers, map coordinates, `IMG_2041.jpg` in a chat line. Rules are a short text file and a page in Settings; every program can send to it (Neo Kit, one call) | Click any address anywhere and the right program opens; the Linux box programs get the same through `xdg-open` |
| **Union directories** | `bind -a` or `bind -b` puts several folders on one name, so the folders behave as one | **Union mounts** in the Neo mount system. Amiga users know half of it already: `ASSIGN LIBS: ADD` | App bundles see system files plus their own; a **theme** is one folder bound in front of the default one, so it replaces only what it has; user fonts and icons stack on the system ones |
| **Per-process namespace** | Every program has its own view of the file tree; what is not in it does not exist for it | Fortress already gives programs only the ports they hold; Neo shapes the file view the same way. A program sees `HOME:` only if Lock allows, a game sees its save folder, a downloaded program sees nothing else | The security base for Lock, the Kids' room and Flatpak programs: no hidden access, and Lock can show the whole view of a program on one page |
| **`/net` as files** | Network connections are files in a folder, so any program that can write a file can open a connection | Neo's network service shows its connections as a read-only tree, `NET:`, for Lock and Neo DOS (`DIR NET:` lists who talks to whom). Opening connections stays a port call | Lock's outgoing-traffic log and a plain answer to "what is this program talking to?" |
| **`cpu` command** | Your terminal, someone else's processor: the remote machine runs the program but sees your files and screen through your namespace | The **Co-op** feature (borrowing power from bigger NBPOS computers) works this way, over `p9fs` | A Raspberry Pi 5 sends a FilmCraft render to the family PC, files never copied by hand |
| **factotum** | One agent holds passwords and keys and does the logins for programs, which never see the secret | **Keyring** works as that agent for Neo programs | Programs never keep passwords of their own |
| **Venti and Fossil** | Every day's state of the disk is kept and can be opened as a folder | **Chronos** "back to yesterday" opens yesterday as a read-only folder, `CHRONOS:2026-11-27/` | Restoring a file is just copying it |
| **Acme and rio** | Text is the interface: any word can be executed or plumbed | Not the look (Neo stays Mac, Amiga and BeOS in manner), but one habit: in Quill, Neo DOS and Cabinet, middle-click on a word sends it to the Plumber | A power-user habit that costs nothing to learn for others |

What we do **not** take: the rio window look, the three-button-mouse-only habits, and the idea that users should live in a text editor. Neo is for children, adults and professionals; Plan 9's ideas go under the hood.

## Estimate

About 6 person-months on top of the planned work: Neo Plumber 2, union mounts and namespace view 2, `p9fs` (9P2000 and 9P2000.L) 1.5, homage items 0.5. `/net` view, factotum and Chronos folders are parts of Lock, Keyring and Chronos already in the estimate.

## Order

1. `p9fs` with the Linux box (it is the simplest shared-folder transport for it; QEMU and Linux already speak 9P).
2. Per-process namespace view in Lock, with union mounts for themes.
3. Neo Plumber with Neo Kit.
4. The homage items with the desktop (screen saver, `PLAN9`).

See also [FEATURES.md](FEATURES.md), [AMIGA.md](AMIGA.md), [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md).
