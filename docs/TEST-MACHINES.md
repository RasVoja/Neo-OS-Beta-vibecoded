# Test machines

Machines the project knows from their own reports. What is named here is
what Fortress and Neo DOS can say about them today; none of it is a promise
of a driver. (This page is about ordinary PCs. Neo hardware is concept
only, see [CONCEPT-COMPUTER.md](CONCEPT-COMPUTER.md).)

## Threadripper PRO workstation with an Intel Arc B580

From a system report sent by the tester.

| Part | What it is |
|---|---|
| CPU | AMD Ryzen Threadripper PRO 3945WX (Castle Peak, Zen 2, 12 cores, 24 threads, socket SP3) |
| Board | AMD WRX80 platform (PCI-Express 4.0) |
| Memory | 64 GB DDR4 |
| Graphics | Intel Arc B580 Graphics (Battlemage family, BMG-G21, ASRock board, 12 GB GDDR6, 192-bit, PCI 8086:E20B on bus 3, x8 link) |
| PCI ids seen | AMD 1022:1480 root complex, 1481 IOMMU, 1482 host bridge, 1483 and 1484 bridges; Intel 8086:E2F0 and E2FF (the card's PCIe bridges) |

What Neo Boing Platinum OS Lite does with it today:

- **Screen:** the ISO asks the boot loader for 1920 x 1080 at 32 bits
  (UEFI GOP, which Arc cards provide). Whatever mode the loader gives is
  used, and the desk fills it (up to 2560 x 1440; a larger screen gets the
  desk in the middle and the rest black).
- **Graphics driver:** none yet. Arc cards have no native driver in
  Fortress; the screen is the boot loader's framebuffer, without
  acceleration. A native Arc (Xe2) driver is a very large piece of work and
  is not planned in the near steps.
- **Recognition:** the boot log and `PCI` in Neo DOS name the AMD Zen 2
  platform and Arc B580 and B570 cards, so reports show them plainly
  (`Platform: AMD Starship/Matisse root complex ...`, `Graphics: Intel Arc
  B580 Graphics ... at 03:00.0`).
- **Disks:** an NVMe driver exists since step 72 but has only been tried in
  QEMU. It reads the machine's NVMe disk and never writes to it unless the
  disk is a Neo disk, so the owner's own system is safe. Nothing can be
  installed on it yet.
- **Not there yet on this machine:** more than one CPU core, and sound.

How to send a report: [REAL-PC-TEST.md](REAL-PC-TEST.md).
