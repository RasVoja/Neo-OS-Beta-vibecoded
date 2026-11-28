# Bootable image, optimized build and installing beside Windows 11

**English** · [Srpski](BUILD-ISO.sr.md) · Plan, 27 November 2026

## 1. What exists

`scripts/make-usb-image.sh` builds `target/neo-lite-x86_64.iso`: a hybrid ISO that boots from a CD or from a USB stick on PCs with BIOS or UEFI. GRUB loads Fortress with Multiboot2 and the boot image as a module. CI boots it with SeaBIOS and with OVMF (`scripts/test-usb-x86_64.sh`). It needs `grub-pc-bin`, `grub-efi-amd64-bin`, `xorriso` and `mtools`.

The image is a live system: nothing is written to any disk by itself. There is no installer yet. Neo DOS can write files to a disk that the machine offers as writable (a virtio disk without the read-only flag); the boot image is always read-only.

## 2. Build and test

```sh
./scripts/test-boot-x86_64.sh     # builds everything, boots 11 times
./scripts/make-usb-image.sh       # the ISO
./scripts/test-usb-x86_64.sh      # boots the ISO with BIOS and with UEFI
```

Write it to a USB stick with `dd`, balenaEtcher or Raspberry Pi Imager.

## 3. Optimized build

Cargo.toml builds release with `opt-level = "s"` (small). A speed profile is in `scripts/profiles/speed.env`: opt-level 3, fat LTO, one codegen unit. It is **untested**; source it, build, and run the boot tests before trusting it.

**CPU-specific flags are not safe yet.** The `x86_64-unknown-none` target is built without SSE and AVX. Flags such as `-C target-cpu=znver2` can turn them on, and the kernel does not yet save and restore that state. Revisit this when the kernel handles FPU/SSE state, and then only for user programs first. The same applies to the Genode side later.

## 4. The reference machine

`scripts/profiles/ras-vojas.env` records the owner's computer: Threadripper PRO 3945WX (24 threads), WRX80 Creator R2.0 board, 64 GB, Intel Arc B580, NVMe and a 16 TB disk, Windows 11, UEFI.

What to expect on it today:

| Part | Expectation |
| --- | --- |
| Screen | Through the UEFI GOP framebuffer, with no acceleration. The Arc B580 needs no driver for that |
| USB keyboard and mouse | Depends on which xHCI controller the ports hang off. The board has several; try more than one port |
| PCI and ACPI | Many PCI buses; the code reads all of them. Unknown how it behaves on this board, hence the test |
| Network, audio, disks | No drivers yet. The 10G cards, Wi-Fi 6E and USB audio come later (M2 and after) |

## 5. First boot on the reference machine

The step-by-step guide for any PC, with what to look at and how to report it, is [REAL-PC-TEST.md](REAL-PC-TEST.md). Neo DOS shows the kernel's boot messages with `BOOTLOG`.

The step "First boot on real PCs" in [STATUS.md](STATUS.md) is still open, and this machine is the first candidate.

1. Back up what matters. Nothing here writes to disks, but a reboot is a reboot.
2. **BitLocker first.** Windows 11 Home may have device encryption on. Changing the Secure Boot setting can make it ask for the recovery key. Find the key (your Microsoft account, "BitLocker recovery key") or suspend BitLocker before you touch the firmware settings.
3. The image is not signed for Secure Boot. Turn Secure Boot off for the test, boot the USB stick from the one-time boot menu, and turn it on again afterwards.
4. Write down or photograph what the screen shows, and where it stops. Add the photo and the answers to the questions in section 4 to `docs/PROGRESS.md`.
5. If the screen is black, try the second menu entry (the boot self-test), another USB port, and a plain USB stick written with `dd`.

## 6. Installing beside Windows 11

**Rules for every stage:** never write to the Windows partitions or the Windows EFI System Partition; use only free space or a partition made for Neo; show the plan and ask before writing; the undo removes only what the installer added.

| Stage | What | State |
| --- | --- | --- |
| 0 | Live USB or CD, writes nothing | works |
| 1 | **Manual dual boot, no new code.** In Windows shrink a data volume (the owner has a large second volume on the NVMe), make a small FAT32 partition for Neo, copy the `boot/` tree of the ISO there with GRUB, add a firmware boot entry. How to add the entry depends on the firmware; to be tested on the reference machine and written down here | to do |
| 2 | **Installer program** in Neo: lists disks, offers only free space, asks to type the disk name to confirm, writes the A/B image and GRUB to its own partition, adds the boot entry, offers an undo | blocked: needs NVMe and AHCI drivers and a writable file system |
| 3 | Updates through Renewal onto the second half of the A/B image | after the installer |

The Raspberry Pi 5 (and the Pi 4) comes after the PC path.
