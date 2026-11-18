#!/usr/bin/env python3
"""Turns the AArch64 kernel ELF file into a raw "Image" that loaders using
the arm64 Linux boot protocol accept (QEMU passes the device tree and an
initrd only to such images). Copies each loadable segment to its offset
from the lowest physical address; .bss is not stored, the header's
image_size covers it.

usage: elf2image.py kernel.elf kernel.img
"""
import struct
import sys

src, dst = sys.argv[1], sys.argv[2]
elf = open(src, "rb").read()
if elf[:4] != b"\x7fELF" or elf[4] != 2 or elf[5] != 1:
    sys.exit("not a 64-bit little-endian ELF file")
phoff, = struct.unpack_from("<Q", elf, 32)
phentsize, phnum = struct.unpack_from("<HH", elf, 54)
segments = []
for i in range(phnum):
    p_type, _flags, offset, _vaddr, paddr, filesz, _memsz, _align = struct.unpack_from(
        "<IIQQQQQQ", elf, phoff + i * phentsize)
    if p_type == 1 and filesz > 0:  # PT_LOAD
        segments.append((paddr, elf[offset:offset + filesz]))
base = min(p for p, _ in segments)
end = max(p + len(d) for p, d in segments)
image = bytearray(end - base)
for paddr, data in segments:
    image[paddr - base:paddr - base + len(data)] = data
if image[56:60] != b"ARM\x64":
    sys.exit("the kernel has no arm64 Image header")
open(dst, "wb").write(image)
