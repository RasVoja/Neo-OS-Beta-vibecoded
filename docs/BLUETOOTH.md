# Bluetooth in NBPOS Lite

**English** · [Srpski](BLUETOOTH.sr.md) · Updated 28 November 2026

## Where it stands (step 120, the start)

- USB Bluetooth adapters (class e0.01.01, such as the Intel 8087:0032 on many PCs) are **found and named** in `USB` and in the boot log: "Bluetooth adapter (not driven yet)".
- Nothing talks to them yet. Wireless mice and keyboards in Bluetooth mode do not work yet; in 2.4 GHz dongle mode (Razer, Logitech) they are plain USB devices and work as such.

## Plan

1. HCI over USB (commands on endpoint 0, events and ACL data on interrupt and bulk endpoints): reset, read the address, scan.
2. HID over GATT and classic HID: pair a keyboard and a mouse (the first goal: the user's Razer devices in Bluetooth mode).
3. A Bluetooth panel in the desk (list, pair, forget), with Dijana's help.
4. Later: audio (A2DP) and file transfer.

Tag: **[next]** for step 1, **[lite]** for the rest.
