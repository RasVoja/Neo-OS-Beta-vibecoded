#!/usr/bin/env python3
"""Makes user/neo-gfx/src/dijana.rs: Dijana's pixel frames for the desk,
from the sprite sheet in assets/dijana/pixel/ (12 frames of 64x48, idle,
sniff and sleep, with their durations from the JSON atlas), plus a small
head for the Bar, cut from the first idle frame.

    scripts/make-dijana.py > user/neo-gfx/src/dijana.rs

Pixels are stored as palette indices, 0 for transparent, so the frames
take 3 KiB each.
"""
import json
import os
from PIL import Image

here = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "dijana", "pixel")
sheet = Image.open(os.path.join(here, "dijana-sheet.png")).convert("RGBA")
atlas = json.load(open(os.path.join(here, "dijana-sheet.json")))
frames = list(atlas["frames"].items())
W, H = 64, 48
# The head, for the Bar: a box around it in the first idle frame.
HEAD = (36, 6, 28, 21)

palette = [None]
def index(px):
    r, g, b, a = px
    if a < 128:
        return 0
    c = (r << 16) | (g << 8) | b
    if c not in palette:
        palette.append(c)
    return palette.index(c)

out = []
for name, f in frames:
    fr = f["frame"]
    assert (fr["w"], fr["h"]) == (W, H), name
    out.append([index(sheet.getpixel((fr["x"] + x, fr["y"] + y))) for y in range(H) for x in range(W)])
f0 = frames[0][1]["frame"]
hx, hy, hw, hh = HEAD
head = [index(sheet.getpixel((f0["x"] + hx + x, f0["y"] + hy + y))) for y in range(hh) for x in range(hw)]

def rows(data, width):
    for i in range(0, len(data), width):
        yield "        " + ", ".join(str(v) for v in data[i:i + width]) + ","

print("//! Dijana, the logo and first resident of Neo Boing Platinum OS, in")
print("//! pixels: made by scripts/make-dijana.py from")
print("//! assets/dijana/pixel/dijana-sheet.png. Do not edit by hand.")
print()
print("/// Frame size in pixels.")
print("pub const W: i32 = %d;" % W)
print("pub const H: i32 = %d;" % H)
print()
print("/// Colours (0xRRGGBB); index 0 is transparent.")
print("#[rustfmt::skip]")
print("pub const PALETTE: [u32; %d] = [0, %s];" % (len(palette), ", ".join("0x%06X" % c for c in palette[1:])))
print()
for tag in atlas["meta"]["frameTags"]:
    print("/// The %s animation: frames %d to %d (%s)." % (tag["name"], tag["from"], tag["to"], tag["direction"]))
    print("pub const %s: (usize, usize) = (%d, %d);" % (tag["name"].upper(), tag["from"], tag["to"]))
print()
print("/// How long each frame stays, in milliseconds.")
print("pub const DURATION_MS: [u32; %d] = [%s];" % (len(frames), ", ".join(str(f["duration"]) for _, f in frames)))
print()
print("/// The frames, row after row, as palette indices.")
print("#[rustfmt::skip]")
print("pub static FRAMES: [[u8; %d]; %d] = [" % (W * H, len(out)))
for (name, _), data in zip(frames, out):
    print("    // %s" % name)
    print("    [")
    for r in rows(data, W):
        print(r)
    print("    ],")
print("];")
print()
print("/// Her head, for the Bar: %d x %d pixels." % (hw, hh))
print("pub const HEAD_W: i32 = %d;" % hw)
print("pub const HEAD_H: i32 = %d;" % hh)
print("#[rustfmt::skip]")
print("pub static HEAD: [u8; %d] = [" % (hw * hh))
for r in rows(head, hw):
    print(r)
print("];")
