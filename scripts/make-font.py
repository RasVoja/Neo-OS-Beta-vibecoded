#!/usr/bin/env python3
"""Makes user/neo-gfx/src/font.rs: an 8x16 bitmap font for the display
service, drawn from DejaVu Sans Mono (Bitstream Vera licence, free to use
and redistribute). Covers ASCII 32-126 and the Serbian Latin letters.

    scripts/make-font.py /path/to/DejaVuSansMono.ttf > user/neo-gfx/src/font.rs
"""
import sys
from PIL import Image, ImageDraw, ImageFont

CHARS = "".join(chr(c) for c in range(32, 127)) + "čćžšđČĆŽŠĐ·"
font = ImageFont.truetype(sys.argv[1], 13)
rows = []
for ch in CHARS:
    img = Image.new("L", (8, 16), 0)
    d = ImageDraw.Draw(img)
    l, t, r, b = d.textbbox((0, 0), ch, font=font)
    x = (8 - (r - l)) // 2 - l
    d.text((x, 0), ch, font=font, fill=255)
    rows.append([sum(1 << (7 - c) for c in range(8) if img.getpixel((c, y)) > 110) for y in range(16)])

print("//! An 8x16 bitmap font, made by scripts/make-font.py from DejaVu Sans")
print("//! Mono (Bitstream Vera licence). Do not edit by hand.")
print()
print("/// The characters, in the same order as `GLYPHS`.")
print("pub const CHARS: &str = %s;" % ('"' + CHARS.replace("\\", "\\\\").replace('"', '\\"') + '"'))
print()
print("/// One glyph per character: 16 rows, bit 7 is the leftmost pixel.")
print("pub const GLYPHS: [[u8; 16]; %d] = [" % len(rows))
for r in rows:
    print("    [" + ", ".join("0x%02X" % v for v in r) + "],")
print("];")
