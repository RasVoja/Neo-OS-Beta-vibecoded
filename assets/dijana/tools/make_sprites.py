from PIL import Image, ImageDraw
import json

W, H = 64, 48
P = dict(
    o=(43,26,18,255),      # outline
    t=(201,128,60,255),    # tan
    s=(160,96,42,255),     # tan shade
    w=(244,239,230,255),   # white
    g=(214,205,188,255),   # white shade
    k=(34,30,36,255),      # black saddle
    n=(30,30,38,255),      # nose
    e=(58,34,22,255),      # eye
    p=(224,138,138,255),   # pink lid
    r=(205,50,40,255),     # red toy
    z=(120,170,230,255),   # Z
    y=(244,200,60,255),    # star on the scarf
)

def new():
    im = Image.new("RGBA", (W, H), (0,0,0,0))
    return im, ImageDraw.Draw(im)

def px(d, x, y, c):
    d.point((x, y), fill=P[c])

def rect(d, x0, y0, x1, y1, c):
    d.rectangle((x0, y0, x1, y1), fill=P[c])

def ell(d, x0, y0, x1, y1, c):
    d.ellipse((x0, y0, x1, y1), fill=P[c])

def stand(tail=0, blink=False, head_dy=0, nose_twitch=0, breathe=0, ear_sway=0):
    im, d = new()
    gy = 43  # ground
    # shadow
    rect(d, 14, 44, 52, 45, 'g')
    # tail (left, up)
    tail_pts = {0: [(10,22),(9,20),(8,18),(8,16)],
                1: [(10,22),(8,21),(7,19),(5,18)],
                2: [(10,22),(9,19),(10,17),(11,15)]}[tail]
    for x,y in tail_pts:
        rect(d, x-2, y-2, x+2, y+2, 'o')
    for x,y in tail_pts:
        rect(d, x-1, y-1, x+1, y+1, 't')
    # legs (behind then front)
    for lx in (16, 22):                     # rear legs
        rect(d, lx-1, 32, lx+5, gy, 'o'); rect(d, lx, 33, lx+4, gy-1, 'w')
    for lx in (40, 46):                     # front legs
        rect(d, lx-1, 32, lx+5, gy, 'o'); rect(d, lx, 33, lx+4, gy-1, 'w')
    # body
    b = breathe
    ell(d, 10, 18-b, 52, 36+b, 'o')
    ell(d, 11, 19-b, 51, 35+b, 'w')
    # saddle (black) and tan patch
    ell(d, 13, 19-b, 42, 30, 'k')
    ell(d, 33, 20-b, 50, 31, 't')
    rect(d, 12, 31, 50, 34+b, 'w')
    # chest
    ell(d, 40, 26, 53, 36+b, 'w')
    # head
    hy = head_dy
    ell(d, 40, 10+hy, 58, 29+hy, 'o')
    ell(d, 41, 11+hy, 57, 28+hy, 't')
    # snout
    ell(d, 50, 17+hy, 63, 29+hy, 'o')
    ell(d, 51, 18+hy, 62, 28+hy, 't')
    rect(d, 51, 24+hy, 60, 28+hy, 'w')       # white muzzle
    # blaze
    rect(d, 47, 11+hy, 49, 18+hy, 'w')
    rect(d, 50, 18+hy, 52, 24+hy, 'w')
    rect(d, 47, 12+hy, 48, 13+hy, 'r'); px(d, 48, 13+hy, 'o')   # ladybug
    # nose
    nx = 60 + nose_twitch
    rect(d, nx-2, 18+hy, nx+1, 22+hy, 'n')
    px(d, nx-2, 18+hy, 'w')
    # eye
    if blink:
        rect(d, 50, 16+hy, 53, 16+hy, 'o')
    else:
        rect(d, 49, 15+hy, 53, 18+hy, 'o')
        rect(d, 50, 15+hy, 52, 17+hy, 'e')
        px(d, 51, 15+hy, 'w')
        rect(d, 49, 18+hy, 53, 19+hy, 'p')   # droopy pink lower lid
    # ear (long, dangling)
    sw = ear_sway
    ell(d, 38+sw, 12+hy, 48+sw, 40+hy, 'o')
    ell(d, 39+sw, 13+hy, 47+sw, 39+hy, 's')
    rect(d, 41+sw, 16+hy, 44+sw, 34+hy, 't')
    # scarf with a star, over the ear and the chest
    rect(d, 36, 26, 51, 29, 'o'); rect(d, 37, 27, 50, 28, 'r')
    rect(d, 46, 29, 51, 30, 'o'); rect(d, 47, 29, 50, 31, 'r')
    rect(d, 47, 32, 50, 33, 'o'); rect(d, 48, 32, 49, 33, 'r'); px(d, 48, 30, 'y')
    return im

def sleep(frame):
    im, d = new()
    b = 1 if frame in (1,2) else 0
    rect(d, 8, 44, 58, 45, 'g')
    # body lying
    ell(d, 6, 26-b, 46, 44, 'o')
    ell(d, 7, 27-b, 45, 43, 'w')
    ell(d, 9, 27-b, 36, 38, 'k')
    ell(d, 28, 28-b, 44, 40, 't')
    # tail resting
    rect(d, 2, 36, 8, 40, 'o'); rect(d, 3, 37, 7, 39, 't')
    # rear leg
    ell(d, 14, 36, 30, 44, 'o'); ell(d, 15, 37, 29, 43, 'w')
    # front paws forward
    rect(d, 40, 38, 60, 44, 'o'); rect(d, 41, 39, 59, 43, 'w')
    # head resting on paws
    ell(d, 38, 24, 58, 42, 'o'); ell(d, 39, 25, 57, 41, 't')
    ell(d, 50, 32, 63, 43, 'o'); ell(d, 51, 33, 62, 42, 't')
    rect(d, 52, 38, 61, 42, 'w')
    rect(d, 47, 25, 49, 31, 'w'); rect(d, 50, 31, 52, 36, 'w')
    rect(d, 58, 33, 62, 36, 'n')
    rect(d, 47, 26, 48, 27, 'r'); px(d, 48, 27, 'o')   # ladybug
    # closed eye
    rect(d, 48, 31, 52, 31, 'o'); px(d, 48, 32, 'o'); px(d, 52, 32, 'o')
    rect(d, 33, 33, 37, 41, 'o'); rect(d, 34, 34, 36, 40, 'r'); px(d, 35, 36, 'y')   # scarf
    # ear flopped down
    ell(d, 36, 26, 46, 46, 'o'); ell(d, 37, 27, 45, 45, 's'); rect(d, 39, 30, 42, 42, 't')
    def Z(x, y):
        for i in range(4): px(d, x+i, y, 'z'); px(d, x+i, y+3, 'z')
        px(d, x+2, y+1, 'z'); px(d, x+1, y+2, 'z')
    # Zs
    if frame == 1: Z(55, 20)
    if frame == 2: Z(55, 20); Z(58, 12)
    if frame == 3: Z(58, 12)
    return im

anims = {
  'idle':  [dict(tail=0), dict(tail=1), dict(tail=2, blink=True), dict(tail=1)],
  'sniff': [dict(head_dy=6, nose_twitch=0, ear_sway=1), dict(head_dy=8, nose_twitch=1, ear_sway=2),
            dict(head_dy=8, nose_twitch=0, ear_sway=2), dict(head_dy=6, nose_twitch=1, ear_sway=1)],
  'sleep': [0,1,2,3],
}
durations = {'idle':[300,250,150,250], 'sniff':[200,200,200,200], 'sleep':[500,500,500,500]}

frames, meta, row = [], {}, 0
for name, fs in anims.items():
    for i, f in enumerate(fs):
        im = sleep(f) if name == 'sleep' else stand(**f)
        frames.append((name, i, im))

sheet = Image.new("RGBA", (W*4, H*3), (0,0,0,0))
atlas_frames = {}
for idx, (name, i, im) in enumerate(frames):
    r, c = idx // 4, idx % 4
    sheet.paste(im, (c*W, r*H))
    atlas_frames[f"dijana_{name}_{i}"] = {
        "frame": {"x": c*W, "y": r*H, "w": W, "h": H}, "rotated": False, "trimmed": False,
        "spriteSourceSize": {"x":0,"y":0,"w":W,"h":H}, "sourceSize": {"w":W,"h":H},
        "duration": durations[name][i]}
tags = [{"name": n, "from": k*4, "to": k*4+3, "direction": "pingpong" if n=='sniff' else "forward"}
        for k, n in enumerate(anims)]
atlas = {"frames": atlas_frames,
         "meta": {"app": "NBPOS Dijana generator", "version": "1.0", "image": "dijana-sheet.png",
                  "format": "RGBA8888", "size": {"w": W*4, "h": H*3}, "scale": "1",
                  "frameTags": tags}}
import os
out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "pixel") + os.sep
sheet.save(out + "dijana-sheet.png")
sheet.resize((W*4*4, H*3*4), Image.NEAREST).save(out + "dijana-sheet-x4.png")
json.dump(atlas, open(out + "dijana-sheet.json", "w"), indent=2)

# GIF previews at x6 on a platinum grey background
def gif(name):
    fr = []
    for n, i, im in frames:
        if n != name: continue
        bg = Image.new("RGBA", (W, H), (221,221,221,255)); bg.alpha_composite(im)
        fr.append(bg.resize((W*6, H*6), Image.NEAREST).convert("P"))
    fr[0].save(out + f"dijana-{name}.gif", save_all=True, append_images=fr[1:],
               duration=durations[name], loop=0, disposal=2)
for n in anims: gif(n)
# single static icon
frames[0][2].save(out + "dijana-idle.png")
frames[0][2].resize((W*8, H*8), Image.NEAREST).save(out + "dijana-idle-x8.png")
print("ok")
