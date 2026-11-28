# Several monitors and video walls

**English** · [Srpski](MULTI-MONITOR.sr.md) · Plan, 28 November 2026

How Neo Boing Platinum OS (NBPOS) Lite plans to work with two or more monitors, and with a video wall made of many screens. Everything here is **[lite]**: it runs on ordinary x86_64 and ARM computers and needs no Neo hardware. Nothing on this page works yet; it is the plan the compositor follows.

## The problem today

Most systems give you two choices: **clone** (every monitor shows the same thing) or **extend** (one wide desk). After that you are on your own:

- Windows land on whatever monitor the system picks, and after you unplug a monitor and plug it back in they often stay piled on the first one.
- There is no simple way to say "this program always opens on the right monitor" or "this window full screen on the TV".
- A different arrangement for different moments (work, film, presenting) means opening display settings and moving everything by hand each time.
- A video wall (many screens acting as one picture) needs a special graphics card mode (NVIDIA Mosaic, AMD Eyefinity) or paid software.

## The NBPOS way

### 1. One continuous desk

All monitors make one desk. You drag a window from one monitor to the next, and it carries on naturally across the edge, at the right size even when the monitors have different resolutions or scaling (a 4K monitor next to a 1080p one: the window keeps its size in centimetres, not in pixels). The pointer does not get stuck on corners where the screens do not line up; it slides along the edge.

### 2. Every window remembers its monitor

Each window remembers the monitor it was on, by that monitor's identity (maker, model and serial number from the monitor's own data, EDID), not by "monitor 1" or "monitor 2". Unplug the laptop from the desk monitor and the windows move to the laptop screen; plug it back in and they go back where they were. Close a program and open it again, and it opens on the same monitor, in the same place.

### 3. Move to a monitor in one click

- Right-click the title bar: **Move to › Monitor 2 (Dell, right)**, with the monitor's name and a small picture of the layout.
- Keyboard: **Neo + Shift + arrow** moves the window to the monitor in that direction; **Neo + Shift + number** to monitor 1, 2, 3.
- In the same menu: **Always open here** makes it a rule for that program (the rule is shown and removed in Settings › Monitors).

### 4. The Bar: on every monitor or only on one

The user chooses:

- **One Bar** on the main monitor (classic, calm).
- **A Bar on every monitor**, each listing the windows that are on that monitor (Windows 11 style), or all windows (MorphOS and KDE style).
- The Neo menu and the clock can stay on the main monitor only, so the other monitors stay clean for work or film.

### 5. Monitor profiles

A profile saves the whole arrangement: which monitors are on, their resolution, refresh rate and position, which one is main, where the Bar is, and **which programs go on which monitor**. Switching takes one click on the Bar or one key.

| Profile (examples) | What it does |
| --- | --- |
| **Work** | All monitors on; mail and calendar on the left, the editor in the middle, Neo DOS on the right |
| **Cinema** | Only the TV on, at its best refresh rate; Cinema full screen; notifications quiet |
| **Presenting** | Laptop screen shows notes, the projector shows only the slides window (see 7) |
| **Gaming** | One monitor at its highest refresh rate, the others dimmed or off |

A profile can switch itself on when its monitors are connected (plug in the TV and **Cinema** is offered, not forced). Profiles live in the user's settings, so Chronos snapshots and Keeper backups keep them.

### 6. Video wall without special software

Several monitors, or projectors, act as **one big picture**. The compositor draws one large frame and sends each screen its own piece of it.

- Bezel correction: the gap of the monitor frames is counted in, so a line drawn across the wall stays straight.
- Mixed screens are allowed; the wall is laid out in Settings › Monitors by dragging screens on a grid, as they hang on the wall.
- Any program can use the wall: a picture, a film, a map, a slide show, a game, or the desk itself.
- Larger walls (more outputs than one graphics card has) through several NBPOS computers in the same neighbourhood (Co-op, see [FEATURES.md](FEATURES.md)), synchronised over the local network. This comes after the single-computer wall.

### 7. Show one window on another monitor

**Show on › Projector** puts a live copy of one window on another monitor (or the wall) full screen, while you keep working on your own screen. Useful for teaching, presenting and showing a film at a party. Nothing else from your desk is ever shown there, so notifications and other windows stay private.

### 8. A hint for programs (API)

Programs written with Neo Kit can ask, they cannot force:

- "open me on the monitor with the pointer", "on the main monitor", or "on the monitor where my other window is";
- "show me full screen on monitor N" (Cinema, Arcade, slide shows);
- "I am a presenter program: notes here, picture on the other monitor";
- "tell me the size and position of the video wall".

The system and the user's rules always win, so a program cannot take over a screen the user did not give it. Programs from the Linux box get the same behaviour through the usual Wayland and X11 ways where they support them.

### 9. Screens per monitor (Amiga idea)

With screen drag ([AMIGA.md](AMIGA.md), A10) each monitor has its own stack of screens: pull a full-screen program down on one monitor and the others stay as they are.

## What has to be built

| Piece | Depends on | Estimate |
| --- | --- | --- |
| Finding monitors, reading EDID, modes and hot-plug | native display drivers or the Linux driver domain (DRM/KMS) | part of the graphics driver work |
| One desk across monitors, mixed resolution and scaling, pointer across edges | the compositor (a buffer per window) | about 6 person-months |
| Windows remember their monitor, "Move to", keys, rules | the above | about 1.5 person-months |
| The Bar on one or every monitor | the Bar | about 1 person-month |
| Monitor profiles | the above | about 1 person-month |
| Show one window on another monitor | the compositor | about 1 person-month |
| Video wall on one computer (bezel correction, layout page) | the above | about 3 person-months |
| Video wall across several computers | Co-op, network | about 4 person-months |
| Neo Kit API hints | Neo Kit | about 0.5 person-month |

About **18 person-months** in all, on top of the graphics drivers. These are estimates, not measured work.

## Where things stand today

Fortress uses only the one screen the boot loader sets up (UEFI GOP or VBE); there is no native graphics driver yet, so only one monitor is used even when several are connected. Multi-monitor work starts when the compositor has a buffer per window and the first native or Linux-domain display driver can list the monitors.

Yardsticks: a Raspberry Pi 5 has two micro HDMI outputs (two 4K monitors at 60 Hz). An Intel Arc B580 card, as on the reference test machine ([TEST-MACHINES.md](TEST-MACHINES.md)), has four outputs, enough for a small video wall from one computer.

## Honest comparison

| Feature | Closest existing thing |
| --- | --- |
| Windows remember their monitor | Windows 11 and macOS remember on reconnect in some cases, not by monitor identity and not per program |
| "Always open here" rules | KDE window rules (hidden, technical), DisplayFusion (paid, Windows only) |
| Monitor profiles with windows | DisplayFusion and similar third-party tools; no system does it built in |
| Video wall | NVIDIA Mosaic and AMD Eyefinity (only some cards), dedicated video-wall software |
| Show one window elsewhere | presenter view inside PowerPoint or Impress, window sharing in call programs |

See also [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md).
