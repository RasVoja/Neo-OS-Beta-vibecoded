# Trying Neo Boing Platinum OS Lite on a real PC

**English** · [Srpski](REAL-PC-TEST.sr.md) · 27 November 2026

The ISO runs in QEMU on every change. What QEMU cannot show is how it behaves on real machines: other USB controllers, other firmware, other ACPI tables. This page is for anyone who wants to try it and tell us what happened. It takes about ten minutes and writes nothing to your disks.

## What you need

- A PC with an x86_64 processor (Intel or AMD), BIOS or UEFI.
- A USB stick of 64 MB or more; everything on it is erased.
- A USB keyboard and mouse (PS/2 works too).
- A phone, to photograph the screen.

## 1. Write the image

Download `neo-boing-platinum-os-lite-x86_64.iso` (or build it: `./scripts/make-usb-image.sh`) and write it to the stick with balenaEtcher, Raspberry Pi Imager, Rufus in **DD mode**, or `dd if=… of=/dev/sdX bs=4M` on Linux.

## 2. Boot it

1. **Windows with BitLocker:** if you change Secure Boot, Windows may later ask for its recovery key. Find the key first (your Microsoft account, "BitLocker recovery key").
2. The image is not signed: switch **Secure Boot off** for the test, and back on afterwards.
3. Pick the stick in the one-time boot menu (often F8, F11, F12 or Esc while the PC starts).
4. GRUB shows two entries. The first, **Neo Boing Platinum OS Lite**, starts the desk; the second runs the boot self-test and is useful when the first shows nothing.

## 3. What you should see

- The desk: wallpaper, the Welcome window, the Clock tile, the Neo DOS window and the Bar at the bottom.
- **Dijana** on the Bar, right of the middle, saying hello. Move the mouse near her: she sniffs. Leave the mouse alone for half a minute: she falls asleep.
- Typing goes into Neo DOS. Try `HELP`, `INFO`, `DIR`.
- Unplug the USB keyboard and plug it in again (or into another port): it should keep working. Ports of a USB hub (front panels, monitors, keyboards with ports) count too.

## 4. Read what the kernel found

A real PC has no serial line to read boot messages from, so Neo DOS shows them on the screen:

| Command | Shows |
| --- | --- |
| `BOOTLOG` | the first 20 lines; `BOOTLOG 2`, `BOOTLOG 3` … the next pages |
| `BOOTLOG USB` | only the lines about USB: the controller, its interrupts (MSI, MSI-X or polling), each device |
| `BOOTLOG ACPI` | processors, interrupt routing, whether power off works |
| `BOOTLOG PCI` | how many devices and buses were found |
| `BOOTLOG Screen` | which framebuffer the desk uses |
| `USB` | the USB devices set up right now, one a line (port, ids, speed, what it is) |
| `PCI` | every PCI device, one a line (place, vendor:device, class and its name) |

Photograph every page of `BOOTLOG` and the output of `USB` and `PCI`, then type `ENDCLI`: on a PC with working ACPI the machine switches off by itself.

## 5. Tell us

Open an issue with the **Real PC report** template (or write in Serbian, that is fine). Add:

- the PC or motherboard model, processor, BIOS or UEFI;
- how far it got: GRUB menu, black screen, desk, typing;
- the photos of `BOOTLOG`, `USB` and `PCI`;
- what did not work: the keyboard, the mouse, a port, power off.

Every report, also "black screen after GRUB", helps: it says where to look.

## Known limits

- No network, sound, disk or Wi-Fi drivers yet; nothing is saved.
- USB 3 hubs are listed, not walked; their keyboards still work through the hub's USB 2 side.
- The screen must be 800 x 600 at 32 bits; GRUB asks the firmware for it. If the firmware cannot, the desk does not appear (the self-test entry says so).
