# Together: video, calls, streaming and making things with other people

**English** · [Srpski](TOGETHER.sr.md) · Plan, 28 November 2026 · **[lite]**

The owner asked what NBPOS offers today's children and adults for video, streaming and web meetings, and what does the most for productivity, good fun and a healthy bond between people through a computer, while raising its educational and creative side. This page is the answer and the plan.

## What helps most (in this order)

1. **Making something together** beats watching something alone. A shared mind map for a school project, a story written by two people at once, a LAN game in one room: the computer is the table people sit around, not the wall between them.
2. **Talking without a middleman.** Calls with no account, no adverts and no recording by a company; a link is enough.
3. **Creating rather than consuming.** Recording, editing, mixing and streaming your own work teaches more than any feed.
4. **Feeds you choose.** Subscriptions you picked, no endless recommendations, no autoplay into the next video (the rule already in the news feed: no push, no propaganda).
5. **Rest.** Dijana's 20-20-20 break and "Good night" time apply here too ([KIDS.md](KIDS.md)).

## New programs [lite]

All of them are free software that runs on x86_64 and ARM today, first as Flatpak programs in the Linux box, each with a simple name and the "based on" line ([APP-NAMES.md](APP-NAMES.md)).

| Neo name | Does | Based on | Licence (to re-check) |
| --- | --- | --- | --- |
| **Channel** | YouTube and PeerTube videos without adverts and without an account: local subscriptions, playlists, no tracking, SponsorBlock, download for later | **FreeTube** (desktop counterpart of NewPipe) now; later a Neo Kit interface over **yt-dlp** and **mpv**, like **Pipeline** does on GNOME | FreeTube AGPL-3.0, yt-dlp Unlicense, mpv GPL-2.0+/LGPL, Pipeline GPL-3.0 |
| **Meeting** | Video calls and classrooms without accounts; a link or a room name is enough; can run on a home or school NBPOS computer for the local network | **Jitsi Meet** | Apache-2.0 |
| **Broadcast** | Recording the screen and streaming (lessons, games, music), scenes and overlays | **OBS Studio** | GPL-2.0+ |
| **Decks** | DJ mixing with two or four decks, controllers, recording a mix | **Mixxx** | GPL-2.0+ |
| **Quill Together** | Writing one document at the same time on the local network, "Google Docs without Google" | Quill plus a CRDT layer (Yjs or Automerge, MIT); for full office documents, Collabora Online run on one NBPOS computer in the house | MIT; Collabora Online MPL-2.0 |
| **DreamCatcher Together** | One mind map edited by several people on the local network | [DREAMCATCHER.md](DREAMCATCHER.md) | MIT layer |

### Is there a NewPipe for the desktop?

Yes. **NewPipe** itself is an Android app (GPL-3.0); it will run in the Android room (version 1.1). On the desktop the closest equivalent is **FreeTube**: no adverts, no Google account, subscriptions kept on your computer, optional routing through Invidious. **Pipeline** (GTK) does the same for YouTube and PeerTube together. Channel starts as FreeTube and becomes a Neo Kit program on yt-dlp and mpv, so YouTube and **PeerTube** (decentralised video, AGPL-3.0) look the same in one list.

**Be plain about it.** YouTube's terms do not allow third-party players that skip adverts, and YouTube changes its site often, so such players break now and then until they are fixed. Channel is therefore a **separate program** that the user installs from the Treasury, not part of the base system, and Neo makes no promise that it will always work. Channel's About page also suggests supporting the people you watch directly (their own pages, Patreon, PeerTube donations).

## For children (Kids' room)

- **Channel:** only channels a grown-up added, no search, no comments, no recommendations; PeerTube instances made for schools can be added.
- **Meeting:** only rooms and contacts a grown-up approved; the camera and microphone light and the Lock log show when they are on.
- **Broadcast and Decks:** recording is on, live streaming to the internet is off unless a grown-up switches it on.
- **Together programs** work on the local network by default; nothing leaves the house or the classroom.

## Network and privacy

Together programs find each other on the local network (mDNS), share files over `p9fs` ([PLAN9.md](PLAN9.md)) and need no server on the internet. Lock shows which program talks to which computer.

## Estimate and order

1. Flatpak versions of Channel (FreeTube), Meeting, Broadcast and Decks, as soon as the Linux box, sound and the camera work: about 2 person-months for packaging, Lock rules and Kids' room limits.
2. Quill Together and DreamCatcher Together after Quill and DreamCatcher: about 4 person-months for the shared CRDT layer.
3. Channel as a Neo Kit program on yt-dlp and mpv: about 2 person-months.
