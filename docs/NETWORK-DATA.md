# Built-in network services, drivers and data

**English** · [Srpski](NETWORK-DATA.sr.md) · Plan, 28 November 2026

NBPOS treats the network, file types and devices the way AmigaOS did: the system knows them, so every program gets them for free. This page lists the built-in network services, the way drivers are loaded, and what NBPOS offers for home networks and databases, with an honest note on how much each helps the user.

Everything here is **[lite]** (or **[done]** where marked) and runs on ordinary x86_64 and ARM computers. Nothing needs Neo hardware. The first versions of the network services run in the **Linux box** (the applications domain) and move to native Neo components later; licences are re-checked before any code is merged, and programs under GPL-2.0-only, AGPL or other licences ship as separate programs, never merged into NBPOS code (which is GPLv3-or-later).

Tags: **[done]** works and is tested in CI · **[next]** being built · **[lite]** planned for Lite.

## 1. The idea: datatypes, for the network too

On the Amiga (AmigaOS 3.0, 1992) a *datatype* taught the whole system a file format once: install the JPEG datatype and every program that shows pictures can open JPEG. NBPOS keeps that rule (Datatypes, 1 000+ formats, decoders in a sandbox) and stretches it to the network: a web page, a remote folder, a Flash animation or a Java program is one more type the system knows. A program does not carry its own browser engine, its own FTP code or its own translator; it asks the system.

**What the user gets:** one place to update and one set of privacy rules (Lock shows which service touches the network); smaller programs; the same right-click menu everywhere.

## 2. Built-in network services [lite]

| Service | What the user does | Based on (licence) | How much it helps |
| --- | --- | --- | --- |
| **Web view as a system service** | Help, mail, Readers, Cabinet previews and any program show web pages through one shared engine; the Browser uses the same one | WebKit (WPE WebKit; LGPL-2.1 and BSD) | High: one engine to keep secure, programs stay small |
| **Remote folders as desk icons** | Type `ftp://`, `sftp://`, `smb://` or `https://…/webdav` in Cabinet, or pick *Connect to server…*; the server appears as a volume icon on the desk, like an Amiga disk (`NAS:`, `WORK:`) and opens in Cabinet, Neo DOS and every program | Neo mount handlers; first in the Linux box with rclone (MIT) and Samba client libraries (GPLv3) | High: no extra FTP or NAS program to learn |
| **Local WebDAV server (opt-in)** | One switch in Settings shares a chosen folder with other computers and phones in the house; off by default, password on, only on the home network | a small WebDAV server (e.g. rclone serve webdav, MIT) | Medium: easy sharing without a cloud |
| **Flash content** | Old `.swf` games and animations open with a double-click, in a sandbox, with no network access unless allowed | Ruffle (MIT or Apache-2.0) | Medium for retro fans and old school material |
| **Java programs** | Double-click a `.jar` and it runs; the Java runtime is installed only the first time it is needed | OpenJDK (GPL-2.0 with the Classpath exception), separate program | Medium: old tools, games and school programs keep working |
| **Open Translation** | Right-click a document, page or selected text, *Translate to…* | LibreTranslate (AGPL-3.0) with Argos Translate models (MIT), local and offline; see [FEATURES.md](FEATURES.md) | High for a multilingual house; nothing goes to Google Translate or another cloud |

Every service is off the network until a program asks for it, and Lock lists each one with its outgoing traffic. A service that is not used is not started.

## 3. Only the drivers that are needed (mount and devs)

AmigaOS kept device drivers in `DEVS:` and started a device only when it was *mounted*, from its mount entry. Nothing was loaded for hardware that was not there. NBPOS follows the same rule:

- **[done] in part:** the Fortress kernel already starts a driver only when it finds its device: USB (xHCI) only if a USB controller is on the PCI bus, virtio input only for virtio input devices, the disk only when a disk answers. The boot log (`BOOTLOG` in Neo DOS) shows what was found and what was started.
- **[lite]:** drivers live as separate files in `DEVS:` (one small description per driver: PCI or USB class, vendor and device numbers). At start the system reads the PCI, USB and ACPI lists and starts only the matching drivers, each as its own isolated program. A device plugged in later (USB, Thunderbolt, a network share) is started at that moment and stopped when it is removed.
- **[lite]:** every volume is mounted on purpose and has a name: `DH0:`, `USB0:`, `NAS:`, `RAD:`. The user sees and can change the mount entry in plain text (Neo DOS `MOUNT`, `ASSIGN`, and the *Devices* page in Settings), as on the Amiga.

**What the user gets:** faster start (the yardstick is under 5 seconds to the desk on an SSD, see the list in [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md)), less memory used, a smaller attack surface, and a clear answer to "what is running on my computer?". A broken driver for one device cannot stop the rest.

## 4. Home networks and phones [lite]

| What | Use | Based on (licence) |
| --- | --- | --- |
| **SMB and NFS** | Shared folders on the home network and on a NAS appear as desk icons (section 2); printers and scanners shared from another computer | Samba client libraries (GPLv3), NFS client |
| **Syncthing** | Folders kept the same on two or more computers and phones, directly, without a cloud and without an account | Syncthing (MPL-2.0) |
| **Phone link** | Files, clipboard, notifications and the phone's camera on the computer; send a file to a phone nearby | KDE Connect (GPL-2.0 or later), LocalSend (Apache-2.0); see the [phone link demo](html/en/demo/neo-phone-link.html) |
| **Neighbourhood** | File sending and mail between NBPOS computers in the house, borrowing power (Co-op) | see [FEATURES.md](FEATURES.md) |

## 5. Databases [lite]

| What | Who it is for | Based on (licence) |
| --- | --- | --- |
| **SQLite, built in** | Every program: Cabinet Pro (Magellan rules, tags), Quill (recent files, templates), Calendar and contacts, Album, Find, Gramophone libraries. Programs share one tested library, the user never sees it | SQLite (public domain) |
| **Small database program** (later) | Home users: a collection, a recipe book, a club's members; tables and forms without writing SQL, saved as one SQLite file | own program on SQLite |
| **PostgreSQL and MariaDB, optional server pack** | Advanced users and developers: installed from Treasury only when asked, started only when used, visible in Lock | PostgreSQL (PostgreSQL licence), MariaDB (GPL-2.0, separate program) |

**How much it helps.** SQLite helps everybody, invisibly: programs remember things fast and safely, and a crash does not damage the data. The database program helps families, schools and clubs that keep lists. The server pack matters only to a small group (developers, small offices), so it is never installed by default.

## 6. Order of work

1. Native networking (Ethernet first, then Wi-Fi) in the Fortress, as an isolated driver; DHCP, DNS and TLS in one network service.
2. `DEVS:` descriptions and starting drivers by PCI and USB match (builds on what the kernel already does).
3. Mount handlers for SMB, SFTP, FTP and WebDAV through the Linux box; volume icons on the desk.
4. SQLite as a shared library for Neo programs.
5. Web view service (WPE WebKit) in the Linux box; then Ruffle, OpenJDK and LibreTranslate as installable services.
6. Syncthing, KDE Connect and LocalSend as Flatpak programs with Neo names; then the local WebDAV switch and the server pack.

Related pages: [FEATURES.md](FEATURES.md) · [UNIQUE-FEATURES.md](UNIQUE-FEATURES.md) · [AMIGA.md](AMIGA.md) · [BEOS-HAIKU.md](BEOS-HAIKU.md) · [RISC-OS.md](RISC-OS.md) · [CLASSIC-MAC.md](CLASSIC-MAC.md)
