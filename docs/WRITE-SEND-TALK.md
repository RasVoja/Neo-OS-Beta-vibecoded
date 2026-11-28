# Write, send, talk: Quill, Letter and Talk

**English** · [Srpski](WRITE-SEND-TALK.sr.md) · Plan, 28 November 2026.

Three programs that work as one: **Quill** writes a document, **Letter** sends it by mail, **Talk** sends it to another Neo user on the network as a live file. Plus the classroom side of the same idea in **SchoolDesk**. Everything here is **[lite]**: software only, on ordinary x86_64 and ARM computers, no Neo hardware needed.

## Why HTML still matters

Hand-editing HTML sounds dated now that AI writes web sites and WordPress runs on templates. But HTML is still the best language for things that are not web sites: **documents, e-books (EPUB is HTML inside a ZIP), mail (every mail program shows HTML), help pages and printouts**. It is open, it is read by every device, and it keeps working in 30 years. So NBPOS does not build a web-site editor. It takes the good habits of the classic visual editors and puts them where they help everyone.

| Inspired by (platform, year) | What it did well | What NBPOS takes |
| --- | --- | --- |
| **Dreamweaver** (Macromedia, later Adobe; Windows and Mac, 1997) | Visual view and code view of the same page side by side | Quill's HTML mode can split into the visual page and its source, both editable |
| **RocketCake** (Ambiera; Windows and macOS, 2010s, free edition) | Responsive pages without writing code | Ready-made layouts (Pads) that look right on a phone and on a big screen |
| **MetalWeb** (Amiga, late 1990s) | A WYSIWYG HTML editor small enough for a classic Amiga | A small, fast editor; HTML mode starts in under a second |
| **YAM**, Yet Another Mailer (Amiga, 1995) | Filters, ARexx port, many identities, everything in plain files | Letter: rules, a Lux port, several identities, mail stored as ordinary files |
| **Iris** (MorphOS, 2020s) | A modern, fast mail client on a small system: IMAP, threads, HTML mail | Letter: threads, IMAP first, HTML shown and written in the same engine |
| **WookieChat**, IRC client (Amiga and MorphOS, 2000s) | Themes, a retro window, pictures shown in the chat | Talk: themes, a retro window, pictures inline |
| **BeOS Mail** (BeOS, 1998) | Mail folders as saved queries over message attributes | Letter: folders as saved searches |
| **Apple Bonjour / Rendezvous** (Mac OS X, 2002) | Computers find each other on the network with no setup | Talk: mDNS discovery, no account |
| **LocalSend** (Windows, macOS, Linux, Android, iOS, 2022) | Files between nearby devices with no cloud | Talk: the same protocol, so phones can join |
| **Veyon**, earlier iTALC (Linux and Windows, 2004 / 2017) | Open classroom management: screen view and broadcast | SchoolDesk: class view and broadcast |
| **Google Classroom** (web, 2014) and **Notion** (web, 2016) | Tasks with due dates; pages that mix text, lists and tasks | SchoolDesk tasks and DreamCatcher cards, on the local network, no account |
| **Markdown** (John Gruber, 2004), **Typora** (2015) | Plain text that reads like a document; marks hidden while you type | Quill's default Markdown WYSIWYG mode |

## Quill: three modes, one document [lite]

Quill (SR: Pero) gets three modes, switched by one button in the toolbar or Neo + 1/2/3:

1. **Plain text** for quick notes and for code: a monospaced font, nothing hidden.
2. **Markdown WYSIWYG** (the default): you see headings, bold text, lists and pictures, and the file is still ordinary Markdown underneath. Type `# ` and a heading appears; the marks show only on the line where the cursor is.
3. **HTML WYSIWYG**: full layout, tables, columns, styles, pictures that text flows around; a split view shows the source beside the page (the Dreamweaver habit).

Switching keeps the content. Going from HTML to Markdown warns before anything Markdown cannot hold (columns, colours) is simplified, and Undo brings it back.

**Export from the same document:** HTML (one file with pictures inside, or a folder), **PDF** (through the system web view's print engine), **EPUB** (for Reader and any e-book reader), plus ODT and DOCX through the LibreOffice core Quill already uses.

**Send as mail** (File › Send as mail, or drag the document onto Letter on the Bar): Quill hands the document to Letter as an HTML mail with the pictures inline (`multipart/related`, each picture by `cid:`), with a plain-text part added for old readers. Letter opens with the text in place; you only add the address. Smart Drop shows "Letter: send as mail" before you let go.

**Engine:** the Markdown modes use `pulldown-cmark` (MIT) and CommonMark with GitHub tables; the HTML mode uses the same WPE WebKit system web view as Browser, so what you write looks the same in Quill, Letter, Browser and the help pages.

## Letter: mail with a human face [lite]

**Letter** (SR: Pismo) is the system mail program. It replaces the earlier working name *Pigeon* (SR: Golub) from the specification.

- **WYSIWYG HTML writing** in the same WebKit engine as Quill's HTML mode; a plain-text-only setting for people and lists that want it.
- **Templates:** a letter, an invoice, a school notice, a newsletter, as Pads (stationery); your own template is any letter saved as a Pad.
- **Attachments you can see:** pictures, PDFs and documents show a preview inside the letter (through Datatypes) before you send and when you receive; no need to save a file to look at it.
- **Local drafts:** every letter is saved while you type, on your own disk, also with the network off; nothing goes to the server before you press Send. A 30-second send delay lets you take a letter back.
- **Rules and a Lux port** after YAM: sort, flag, reply and forward by rules; scripts from Neo DOS and other programs.
- **Folders as saved searches** (the BeOS Mail idea), threads and a unified inbox after Iris.
- **Send to a Neo user:** pick a person from Talk instead of an address. The document goes through Talk as a **live file**: the receiver opens it in Quill and edits it, and both see the changes (the same CRDT layer as Quill Together). A normal mail copy can go along for people who are not on NBPOS.
- **Protocols:** IMAP and SMTP first, JMAP later; OpenPGP and S/MIME signing and encryption; no account with us, no tracking pixels (remote pictures off until you allow them per sender).

**Based on:** the Geary core for IMAP and SMTP at first, `lettre` (MIT/Apache) for building mail with inline pictures, Sequoia-PGP (LGPL) for OpenPGP; the interface is Neo Kit.

## Talk: chat with no account [lite]

**Talk** (SR: Razgovor), with the look of **WookieChat**: themes (Platinum, Phosphor, First Desk, a chunky retro window), a classic channel and user list, and Dijana in the corner when someone writes to you.

- **Text with pictures:** GIF, JPG and PNG shown inline in the conversation, animated GIFs too (still when the system asks for reduced motion).
- **Files by drag and drop**, straight from one computer to the other (P2P), no server and no size limit beyond the disk; the **LocalSend** protocol (MIT), so phones with LocalSend can join.
- **Finds people by itself:** computers on the same network see each other through mDNS and DNS-SD (Avahi); **no account**, no phone number, just a name and a Dijana-style picture you pick.
- **Outside the home network (optional):** a bridge to **XMPP** or **Matrix** for friends elsewhere, turned on by the user, listed in Lock with its traffic.
- **Live files:** a Quill document or a DreamCatcher board sent in Talk opens as a shared document both people edit.
- **Safety:** messages on the local network are encrypted (Noise protocol), the other computer's key is shown as a short word list to compare once; in the Kids' room only contacts an adult approved.

## SchoolDesk on the school network [lite]

**SchoolDesk** (SR: Školska klupa) grows from tasks and due dates ([EVERYDAY-APPS.md](EVERYDAY-APPS.md)) into a whole classroom on the school's own network:

- **The teacher sees every computer in the class** as a tile: who is connected, which task is open; a click shows one pupil's screen, and the pupil sees a clear eye sign whenever the screen is watched.
- **Screen broadcast:** the teacher's screen (or one pupil's work, with permission) goes to every monitor in the room; Neo + B ends it.
- **Files stay in the school:** handing out and collecting files goes over the local network through Talk, not over the internet.
- **DreamCatcher tasks with a note on the desk:** a task from the teacher arrives as a DreamCatcher card and a quiet notification on the pupil's desk (Dijana holds up a small sign; she never interrupts).
- **Based on:** Veyon (GPL-2.0, open classroom management) for screen view and broadcast, Talk for files, DreamCatcher for tasks.

## Is plain text still worth it?

Yes, for **quick notes and code**: it opens anywhere, it is tiny, and nothing can break it. For almost everything else **Markdown is the better middle**: it is still plain text underneath, so it lasts and every program reads it, but it has headings, lists, links and pictures. AI models write Markdown naturally, and Git can compare two versions line by line, which a DOCX file cannot. So Quill opens in Markdown WYSIWYG by default, keeps plain text one click away, and uses HTML only when a document needs real layout or goes out as mail or an e-book.

## Demo and video

- **HTML demo** `docs/html/en/demo/neo-write-send-talk.html` (SR: `docs/html/sr/demo/neo-pisi-posalji-pricaj.html`): Quill's three modes on one document, Send as mail into Letter, Talk with an inline picture and a dragged file, and the SchoolDesk class view. It is a mock-up in the browser; nothing is sent.
- **Video** (about one minute, SR and EN) recorded from the demo: `docs/video/en/NBPOS-write-send-talk-en.mp4`, `docs/video/sr/NBPOS-pisi-posalji-pricaj-sr.mp4`. Both are labelled "plan, not yet coded".

## Coding steps and estimate

Coding steps 194 to 205 (steps 160 to 193 are already planned for Plan 9, the together apps and Smart Drop), after Neo Kit and the system web view:

| Step | Work |
| --- | --- |
| 194 | Quill plain-text mode and the mode switch |
| 195 | Quill Markdown WYSIWYG (pulldown-cmark, marks on the cursor line) |
| 196 | Quill HTML WYSIWYG in WPE WebKit, split source view |
| 197 | Export HTML, PDF and EPUB from one document |
| 198 | Letter: IMAP/SMTP, threads, local drafts, send delay |
| 199 | Letter: HTML writing, templates as Pads, attachment previews |
| 200 | Send as mail from Quill (inline pictures, plain-text part); rules and Lux port |
| 201 | Talk: mDNS discovery, encrypted local chat, inline pictures, themes |
| 202 | Talk: P2P file transfer (LocalSend), drag and drop |
| 203 | Live files between Neo users (Letter and Talk, CRDT layer) |
| 204 | Talk: optional XMPP and Matrix bridge |
| 205 | SchoolDesk: class view, screen broadcast, local files, DreamCatcher tasks |

About **24 person-months**: Quill modes and export 6, Letter 8, Talk 6, SchoolDesk classroom additions 4 (on top of SchoolDesk's 5 in [EVERYDAY-APPS.md](EVERYDAY-APPS.md)). Licences are checked again before code is merged; Veyon (GPL-2.0) and the Geary core (LGPL) run as separate programs.

See also [FEATURES.md](FEATURES.md), [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md) (items 17 and 18), [TOGETHER.md](TOGETHER.md), [APP-NAMES.md](APP-NAMES.md).
