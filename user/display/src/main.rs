//! The display service: the first pixels of Neo OS Lite.
//!
//! Handle 0 is the framebuffer (memory the service maps into its own
//! address space), handle 1 is the service's own port (a message with tag
//! `QUIT` ends it) and handle 2 is the clock chip, for the time on the
//! Clock tile: the CMOS ports 0x70-0x71 on a PC, the PL031's registers on
//! AArch64. On a PC under QEMU, handle 4 is the card's mode registers (the
//! Bochs display's index and data ports, 0x1CE-0x1D0). When the boot loader
//! has already set the mode (UEFI GOP or VBE, as on a real PC) there is no
//! handle 4 and the service keeps that mode; on AArch64 the kernel has
//! already set the mode (ramfb).
//!
//! It sets 800 x 600 at 32 bits per pixel and draws the desk: wallpaper,
//! the Bar (with Dijana's head as the logo), a first window, the Clock
//! tile (Sat) and Dijana herself, the first resident, who lives on the
//! Bar. Then it keeps the
//! clock going: it reads the clock chip a few times a second and redraws
//! the tile and the Bar's time when the second changes. It draws the
//! pointer's arrow where the kernel says the pointer is (messages tagged
//! `POINTER_MOVED`), keeping what the arrow covers. On `QUIT` it ends with
//! the number of clock redraws.
//!
//! Dijana greets once at start-up (a speech bubble, as the specification
//! says: she speaks up by herself only at first start, after an error and
//! when called). She breathes and wags (idle), sniffs when the pointer
//! comes close, falls asleep after half a minute without the pointer
//! moving, wakes when it moves, and answers a click on her.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU64, Ordering};
use neo_gfx::dijana;
use neo_rt::{Error, Handle};

const FRAMEBUFFER: Handle = 0;
const CONTROL: Handle = 1;
const CLOCK: Handle = 2;
const FOCUS: Handle = 3;
#[cfg(target_arch = "x86_64")]
const DISPI: Handle = 4;
/// The message that ends the service.
pub const QUIT: u64 = 2;

const W: i32 = 800;
const H: i32 = 600;

// The Neo palette (0xRRGGBB).
const WALL_TOP: u32 = 0x5A8086;
const WALL_BOTTOM: u32 = 0x1C3236;
const WINDOW: u32 = 0xECECEA;
const PAGE: u32 = 0xFFFFFF;
const INK: u32 = 0x1B2125;
const MUTED: u32 = 0x56616A;
const LINE: u32 = 0x9AA19C;
const ACCENT: u32 = 0x0D6F78;
const AMBER: u32 = 0xC86F00;
const TILE: u32 = 0xF4F6F3;
const BAR: u32 = 0xE4E8E5;

/// The screen: a canvas over the framebuffer.
type Screen = neo_gfx::Canvas;

/// The arrow: `#` ink, `o` white, blank shows what is under it.
const ARROW: [&[u8; 12]; 19] = [
    b"#           ",
    b"##          ",
    b"#o#         ",
    b"#oo#        ",
    b"#ooo#       ",
    b"#oooo#      ",
    b"#ooooo#     ",
    b"#oooooo#    ",
    b"#ooooooo#   ",
    b"#oooooooo#  ",
    b"#ooooooooo# ",
    b"#oooooo#####",
    b"#ooo#oo#    ",
    b"#oo# #oo#   ",
    b"#o#  #oo#   ",
    b"##    #oo#  ",
    b"#     #oo#  ",
    b"       #o#  ",
    b"       ##   ",
];
const ARROW_W: i32 = 12;
const ARROW_H: i32 = 19;

/// The pointer's arrow, with what it covers kept so it can be put back.
struct Cursor {
    at: Option<(i32, i32)>,
    under: [u32; (ARROW_W * ARROW_H) as usize],
}

impl Cursor {
    fn hide(&mut self, s: &Screen) {
        if let Some((x, y)) = self.at.take() {
            for (i, &c) in self.under.iter().enumerate() {
                let (dx, dy) = (i as i32 % ARROW_W, i as i32 / ARROW_W);
                s.put(x + dx, y + dy, c);
            }
        }
    }

    fn show(&mut self, s: &Screen, (x, y): (i32, i32)) {
        for dy in 0..ARROW_H {
            for dx in 0..ARROW_W {
                self.under[(dy * ARROW_W + dx) as usize] = s.get(x + dx, y + dy);
                match ARROW[dy as usize][dx as usize] {
                    b'#' => s.put(x + dx, y + dy, 0x000000),
                    b'o' => s.put(x + dx, y + dy, PAGE),
                    _ => {}
                }
            }
        }
        self.at = Some((x, y));
    }
}

fn mix(a: u32, b: u32, t: i32, n: i32) -> u32 {
    let ch = |s: u32| {
        let (ca, cb) = ((a >> s) & 0xFF, (b >> s) & 0xFF);
        ((ca as i32 + (cb as i32 - ca as i32) * t / n) as u32) << s
    };
    ch(16) | ch(8) | ch(0)
}

/// The PC: the Bochs display's mode registers and the CMOS clock.
#[cfg(target_arch = "x86_64")]
mod pc {
    use super::{Error, CLOCK, DISPI, H, W};

    // Bochs display ("DISPI") registers.
    const INDEX: u16 = 0;
    const DATA: u16 = 1;
    const REG_ID: u16 = 0;
    const REG_XRES: u16 = 1;
    const REG_YRES: u16 = 2;
    const REG_BPP: u16 = 3;
    const REG_ENABLE: u16 = 4;
    const ENABLED: u16 = 0x01;
    const LINEAR: u16 = 0x40;

    fn dispi(reg: u16, val: u16) -> Result<(), Error> {
        neo_rt::io_out16(DISPI, INDEX, reg)?;
        neo_rt::io_out16(DISPI, DATA, val)
    }

    pub fn set_mode() -> Result<(), Error> {
        // No handle for the registers: the boot loader has set the mode.
        match neo_rt::io_out16(DISPI, INDEX, REG_ID) {
            Err(Error::BadHandle) => return Ok(()),
            r => r?,
        }
        let id = neo_rt::io_in16(DISPI, DATA)?;
        if !(0xB0C0..=0xB0CF).contains(&id) {
            return Err(Error::NotFound);
        }
        dispi(REG_ENABLE, 0)?;
        dispi(REG_XRES, W as u16)?;
        dispi(REG_YRES, H as u16)?;
        dispi(REG_BPP, 32)?;
        dispi(REG_ENABLE, ENABLED | LINEAR)
    }

    /// Hours, minutes and seconds from the clock chip (it may count in BCD).
    pub fn time() -> Result<(i32, i32, i32), Error> {
        let read = |reg: u8| -> Result<u8, Error> {
            neo_rt::io_out(CLOCK, 0, reg)?;
            neo_rt::io_in(CLOCK, 1)
        };
        // Wait while the chip is updating, so the three values belong together.
        while read(0x0A)? & 0x80 != 0 {}
        let bcd = read(0x0B)? & 0x04 == 0;
        let conv = |v: u8| if bcd { (v >> 4) * 10 + (v & 0x0F) } else { v } as i32;
        Ok((conv(read(0x04)? & 0x7F), conv(read(0x02)?), conv(read(0x00)?)))
    }
}

/// AArch64 (QEMU `virt`): the kernel has set the mode (ramfb); the clock is
/// the PL031, whose first register counts seconds (UTC).
#[cfg(target_arch = "aarch64")]
mod arm {
    use super::{Error, CLOCK};

    pub fn set_mode() -> Result<(), Error> {
        Ok(())
    }

    pub fn time() -> Result<(i32, i32, i32), Error> {
        static mut RTC: *const u32 = core::ptr::null();
        // Only this one thread uses it: map the chip on first use.
        let rtc = unsafe {
            if RTC.is_null() {
                RTC = neo_rt::map_device(CLOCK)? as *const u32;
            }
            RTC
        };
        let secs = unsafe { rtc.read_volatile() } % 86_400;
        Ok(((secs / 3600) as i32, (secs / 60 % 60) as i32, (secs % 60) as i32))
    }
}

#[cfg(target_arch = "aarch64")]
use arm::{set_mode, time};
#[cfg(target_arch = "x86_64")]
use pc::{set_mode, time};

/// Sine of `deg` degrees, times 1000 (a table every 6 degrees is enough
/// for clock hands).
fn sin1000(deg: i32) -> i32 {
    const T: [i32; 16] = [0, 105, 208, 309, 407, 500, 588, 669, 743, 809, 866, 914, 951, 978, 995, 1000];
    let d = deg.rem_euclid(360);
    let q = |a: i32| T[(a / 6) as usize];
    match d {
        0..=90 => q(d),
        91..=180 => q(180 - d),
        181..=270 => -q(d - 180),
        _ => -q(360 - d),
    }
}

fn hand(s: &Screen, cx: i32, cy: i32, deg: i32, len: i32, thick: i32, c: u32) {
    let x = cx + len * sin1000(deg) / 1000;
    let y = cy - len * sin1000(deg + 90) / 1000;
    s.line(cx, cy, x, y, thick, c);
}

fn two(n: i32) -> [u8; 2] {
    [b'0' + (n / 10) as u8, b'0' + (n % 10) as u8]
}

/// x, y, width, height.
type Rect = (i32, i32, i32, i32);

const FULL: Rect = (0, 0, W, H);
/// The first window's size; its place changes when it is dragged.
const WIN_W: i32 = 460;
const WIN_H: i32 = 270;
/// The title bar's height: grab a window there to drag it.
const TITLE_H: i32 = 28;
/// At most this many windows: the first one and three from programs.
const MAX_WINDOWS: usize = 4;
/// The largest window a program may open.
const MAX_CONTENT: (i32, i32) = (640, 400);

/// A program's window: its pixels live in memory it shares with us.
#[derive(Clone, Copy)]
struct Content {
    pixels: *const u32,
    w: i32,
    h: i32,
    /// The program that owns it.
    owner: u64,
    /// Where its events go, if it asked for them.
    events: Option<neo_rt::Handle>,
}

#[derive(Clone, Copy)]
struct Window {
    at: (i32, i32),
    title: [u8; 8],
    /// None for the first window, which this service draws itself.
    content: Option<Content>,
}

impl Window {
    fn size(&self) -> (i32, i32) {
        match self.content {
            None => (WIN_W, WIN_H),
            Some(c) => (c.w + 2, c.h + TITLE_H + 2),
        }
    }

    /// The window with its shadow.
    fn area(&self) -> Rect {
        let (w, h) = self.size();
        (self.at.0, self.at.1, w + 6, h + 8)
    }

    fn draw(&self, s: &Screen) {
        let (x, y) = self.at;
        let Some(c) = self.content else {
            return welcome(s, self.at);
        };
        let (w, wh) = self.size();
        frame(s, (x, y), (w, wh), core::str::from_utf8(&self.title).unwrap_or("?").trim_end_matches('\0'));
        // Only rows and columns inside the clip are copied.
        let (cx0, cy0, cx1, cy1) = s.clip();
        let (ox, oy) = (x + 1, y + TITLE_H + 1);
        for row in (cy0 - oy).max(0)..(cy1 - oy).min(c.h) {
            for col in (cx0 - ox).max(0)..(cx1 - ox).min(c.w) {
                // Inside the shared block, whose size was checked.
                let px = unsafe { c.pixels.add((row * c.w + col) as usize).read_volatile() };
                s.put(ox + col, oy + row, px & 0xFF_FFFF);
            }
        }
    }
}

/// The smallest rectangle around both.
fn union(a: Rect, b: Rect) -> Rect {
    let (x0, y0) = (a.0.min(b.0), a.1.min(b.1));
    let (x1, y1) = ((a.0 + a.2).max(b.0 + b.2), (a.1 + a.3).max(b.1 + b.3));
    (x0, y0, x1 - x0, y1 - y0)
}

/// Draws everything inside `area`, back to front: wallpaper, the Clock
/// tile, the windows (first to last), then the Bar and Dijana, which stay
/// on top.
fn repaint(s: &Screen, area: Rect, windows: &[Option<Window>], now: (i32, i32, i32), dog: &Dijana) {
    s.set_clip(area);
    draw(s);
    clock(s, now);
    for w in windows.iter().flatten() {
        w.draw(s);
    }
    bar(s, now);
    dog.draw(s);
    s.set_clip(FULL);
}

fn draw(s: &Screen) {
    // Wallpaper: a vertical gradient.
    for y in 0..H {
        s.rect(0, y, W, 1, mix(WALL_TOP, WALL_BOTTOM, y, H));
    }

    // The Clock tile's frame; the face is drawn by `clock`.
    s.rect(TX + 5, TY + 6, TW, TH, mix(WALL_BOTTOM, 0, 1, 2));
    s.rect(TX, TY, TW, TH, TILE);
    s.frame(TX, TY, TW, TH, LINE);
    s.text(TX + 10, TY + 6, "Sat · Clock", 1, INK);
}

/// The keyboard layout shown on the Bar (0 US, 1 SR), as the kernel last
/// reported it.
static LAYOUT: AtomicU64 = AtomicU64::new(0);
/// The Bar's layout switch, left of the time: a click switches the layout.
const LAYOUT_BOX: Rect = (W - 104, H - 32, 36, 24);

/// The Bar, along the bottom, with the layout switch and the time on it.
fn bar(s: &Screen, (h, m, _): (i32, i32, i32)) {
    s.rect(0, H - 40, W, 40, BAR);
    s.rect(0, H - 41, W, 1, LINE);
    // Dijana's head is the logo, then the name.
    s.sprite(4, H - 31, &dijana::HEAD, dijana::HEAD_W, &dijana::PALETTE, 1);
    s.text(36, H - 28, "Neo", 1, INK);
    s.rect(70, H - 32, 1, 24, LINE);
    s.rect(80, H - 32, 96, 24, PAGE);
    s.frame(80, H - 32, 96, 24, LINE);
    s.text(88, H - 28, "Dobrodošli", 1, INK);
    let hm = [two(h), two(m)];
    let digits = [hm[0][0], hm[0][1], b':', hm[1][0], hm[1][1]];
    s.text(W - 56, H - 28, core::str::from_utf8(&digits).unwrap_or(""), 1, INK);
    let (x, y, w, bh) = LAYOUT_BOX;
    let serbian = LAYOUT.load(Ordering::Relaxed) == 1;
    s.rect(x, y, w, bh, if serbian { ACCENT } else { PAGE });
    s.frame(x, y, w, bh, LINE);
    s.text(x + 10, y + 4, if serbian { "SR" } else { "US" }, 1, if serbian { PAGE } else { INK });
}

/// A window's frame, shadow and title bar.
fn frame(s: &Screen, (x, y): (i32, i32), (w, wh): (i32, i32), title: &str) {
    s.rect(x + 6, y + 8, w, wh, mix(WALL_BOTTOM, 0, 1, 2)); // shadow
    s.rect(x, y, w, wh, WINDOW);
    s.frame(x, y, w, wh, LINE);
    s.rect(x + 1, y + TITLE_H, w - 2, 1, LINE);
    s.disc(x + 16, y + 14, 5, AMBER);
    s.disc(x + 32, y + 14, 5, LINE);
    s.text(x + 48, y + 6, title, 1, INK);
}

/// The first window, its top left corner at `(x, y)`.
fn welcome(s: &Screen, (x, y): (i32, i32)) {
    let (w, wh) = (WIN_W, WIN_H);
    frame(s, (x, y), (w, wh), "Dobrodošli · Welcome");
    s.rect(x + 8, y + 36, w - 16, wh - 44, PAGE);
    // The logo: Dijana, then the name.
    s.sprite(x + 20, y + 52, &dijana::FRAMES[0], dijana::W, &dijana::PALETTE, 1);
    s.text(x + 96, y + 50, "Neo Boing", 2, ACCENT);
    s.text(x + 96, y + 78, "Platinum OS Lite", 2, ACCENT);
    s.text(x + 24, y + 124, "Zdravo! Ovo je prvi prozor Neo-a.", 1, INK);
    s.text(x + 24, y + 144, "Nacrtao ga je program u korisničkom", 1, INK);
    s.text(x + 24, y + 164, "režimu, a ne jezgro.", 1, INK);
    s.text(x + 24, y + 194, "Hello! This is Neo's first window, drawn", 1, MUTED);
    s.text(x + 24, y + 214, "by a program in user mode, not the kernel.", 1, MUTED);
    s.rect(x + 24, y + 244, 120, 2, AMBER);
}

const TX: i32 = 590;
const TY: i32 = 100;
const TW: i32 = 170;
const TH: i32 = 214;

/// The clock face and the digits under it.
fn clock(s: &Screen, (h, m, sec): (i32, i32, i32)) {
    let (cx, cy, r) = (TX + TW / 2, TY + 98, 62);
    s.disc(cx, cy, r, PAGE);
    s.ring(cx, cy, r, INK);
    for i in 0..12 {
        let len = if i % 3 == 0 { 10 } else { 5 };
        let deg = i * 30;
        let (ox, oy) = (sin1000(deg), sin1000(deg + 90));
        s.line(
            cx + (r - 3) * ox / 1000,
            cy - (r - 3) * oy / 1000,
            cx + (r - 3 - len) * ox / 1000,
            cy - (r - 3 - len) * oy / 1000,
            2,
            INK,
        );
    }
    hand(s, cx, cy, (h % 12) * 30 + m / 2, 34, 4, INK);
    hand(s, cx, cy, m * 6, 50, 3, INK);
    hand(s, cx, cy, sec * 6, 54, 1, AMBER);
    s.disc(cx, cy, 3, AMBER);
    let hms = [two(h), two(m), two(sec)];
    let digits = [hms[0][0], hms[0][1], b':', hms[1][0], hms[1][1], b':', hms[2][0], hms[2][1]];
    s.rect(cx - 34, TY + TH - 26, 68, 18, TILE);
    s.text(cx - 32, TY + TH - 24, core::str::from_utf8(&digits).unwrap_or(""), 1, INK);
}

/// Dijana lives on the Bar, left of the layout switch, with her head above
/// its edge: like the Bar she stays in front of the windows, so they never
/// hide her.
const DOG_SCALE: i32 = 1;
const DOG_X: i32 = W - 200;
const DOG_Y: i32 = H - 50;
const DOG: Rect = (DOG_X, DOG_Y, dijana::W * DOG_SCALE, dijana::H * DOG_SCALE);
/// Her speech bubble, above and left of her, with its tail; also in front.
const BUBBLE: Rect = (DOG_X - 200, DOG_Y - 62, 316, 62);
/// How close the pointer comes before she sniffs at it.
const SNIFF_REACH: i32 = 48;
/// She falls asleep after this long without the pointer moving.
const SLEEP_AFTER_MS: i32 = 30_000;
/// How long a speech bubble stays.
const SAY_MS: i32 = 7_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mood {
    Idle,
    Sniff,
    Sleep,
}

/// The first resident: which animation, which frame, and what she says.
struct Dijana {
    mood: Mood,
    frame: usize,
    /// Pingpong direction for the sniff (+1 or -1).
    step: isize,
    /// Time left on this frame.
    left_ms: i32,
    /// Time since the pointer last moved.
    still_ms: i32,
    /// Two lines (Serbian, English) and how long they stay.
    says: Option<(&'static str, &'static str)>,
    say_ms: i32,
}

impl Dijana {
    fn new() -> Dijana {
        let mut d = Dijana { mood: Mood::Idle, frame: 0, step: 1, left_ms: 0, still_ms: 0, says: None, say_ms: 0 };
        d.set(Mood::Idle);
        // She speaks up by herself at start-up.
        d.say("Zdravo! Ja sam Dijana.", "Hello! I'm Dijana. Welcome.");
        d
    }

    fn range(mood: Mood) -> (usize, usize) {
        match mood {
            Mood::Idle => dijana::IDLE,
            Mood::Sniff => dijana::SNIFF,
            Mood::Sleep => dijana::SLEEP,
        }
    }

    fn set(&mut self, mood: Mood) {
        self.mood = mood;
        self.frame = Self::range(mood).0;
        self.step = 1;
        self.left_ms = dijana::DURATION_MS[self.frame] as i32;
    }

    fn say(&mut self, sr: &'static str, en: &'static str) {
        self.says = Some((sr, en));
        self.say_ms = SAY_MS;
    }

    /// Time passes; returns the area to draw again, if anything changed.
    fn tick(&mut self, ms: i32) -> Option<Rect> {
        let mut dirty = None;
        self.still_ms = self.still_ms.saturating_add(ms);
        if self.mood != Mood::Sleep && self.still_ms >= SLEEP_AFTER_MS {
            self.set(Mood::Sleep);
            dirty = Some(DOG);
        }
        if self.says.is_some() {
            self.say_ms -= ms;
            if self.say_ms <= 0 {
                self.says = None;
                dirty = Some(union(dirty.unwrap_or(BUBBLE), BUBBLE));
            }
        }
        self.left_ms -= ms;
        if self.left_ms <= 0 {
            let (first, last) = Self::range(self.mood);
            self.frame = if self.mood == Mood::Sniff {
                // There and back.
                if (self.frame == last && self.step > 0) || (self.frame == first && self.step < 0) {
                    self.step = -self.step;
                }
                (self.frame as isize + self.step) as usize
            } else if self.frame >= last {
                first
            } else {
                self.frame + 1
            };
            self.left_ms = dijana::DURATION_MS[self.frame] as i32;
            dirty = Some(union(dirty.unwrap_or(DOG), DOG));
        }
        dirty
    }

    /// The pointer moved (and maybe clicked on her): she wakes, sniffs when
    /// it is close, answers a click. Returns the area to draw again.
    fn pointer(&mut self, (px, py): (i32, i32), clicked: bool) -> Option<Rect> {
        self.still_ms = 0;
        let (x, y, w, h) = DOG;
        let r = SNIFF_REACH;
        let near = (x - r..x + w + r).contains(&px) && (y - r..y + h + r).contains(&py);
        let on = (x..x + w).contains(&px) && (y..y + h).contains(&py);
        let mut dirty = None;
        let mood = match (self.mood, near) {
            (_, true) => Mood::Sniff,
            (Mood::Sniff | Mood::Sleep, false) => Mood::Idle,
            (m, false) => m,
        };
        if mood != self.mood {
            self.set(mood);
            dirty = Some(DOG);
        }
        if clicked && on {
            self.say("Vau! Pomoć: kucaj HELP u Neo DOS-u.", "Woof! For help, type HELP in Neo DOS.");
            dirty = Some(union(dirty.unwrap_or(BUBBLE), BUBBLE));
        }
        dirty
    }

    fn draw(&self, s: &Screen) {
        let f = &dijana::FRAMES[self.frame];
        s.sprite(DOG_X, DOG_Y, f, dijana::W, &dijana::PALETTE, DOG_SCALE);
        if let Some((sr, en)) = self.says {
            let (bx, by, bw, bh) = BUBBLE;
            let body = bh - 12;
            s.rect(bx, by, bw, body, PAGE);
            s.frame(bx, by, bw, body, INK);
            // The tail points down to her head.
            for d in 0..10 {
                s.rect(bx + bw - 60 + d, by + body - 1 + d, 12 - d, 1, PAGE);
                s.put(bx + bw - 60 + d, by + body - 1 + d, INK);
                s.put(bx + bw - 48, by + body - 1 + d, INK);
            }
            s.text(bx + 10, by + 6, sr, 1, INK);
            s.text(bx + 10, by + 26, en, 1, MUTED);
        }
    }
}

/// The windows, and dragging them: which one the pointer holds and where.
struct Desk {
    windows: [Option<Window>; MAX_WINDOWS],
    /// The window being dragged, where it is going, and where the pointer
    /// grabbed it.
    grab: Option<(usize, (i32, i32), (i32, i32))>,
    buttons: u64,
    /// A title and an event port waiting for their window, and who sent
    /// them.
    title: Option<(u64, [u8; 8])>,
    events: Option<(u64, neo_rt::Handle)>,
    /// The event port typed keys go to, if a program's window has the
    /// focus; otherwise they go to the console (Neo DOS).
    focus: Option<neo_rt::Handle>,
}

impl Desk {
    /// The topmost window under the point, if any.
    fn hit(&self, (px, py): (i32, i32)) -> Option<usize> {
        (0..MAX_WINDOWS).rev().find(|&i| {
            self.windows[i].is_some_and(|w| {
                let ((x, y), (ww, wh)) = (w.at, w.size());
                (x..x + ww).contains(&px) && (y..y + wh).contains(&py)
            })
        })
    }

    /// Puts window `i` in front of the others; returns its new index.
    fn raise(&mut self, i: usize) -> usize {
        let last = (0..MAX_WINDOWS).rev().find(|&j| self.windows[j].is_some()).unwrap_or(i);
        let w = self.windows[i];
        for j in i..last {
            self.windows[j] = self.windows[j + 1];
        }
        self.windows[last] = w;
        last
    }

    /// Takes one pointer report. A press raises the window under the
    /// pointer (returns its area to draw again), grabs it by its title bar,
    /// or tells its program about a click inside.
    fn pointer(&mut self, (px, py): (i32, i32), buttons: u64) -> Option<Rect> {
        let pressed = buttons & !self.buttons & 3;
        self.buttons = buttons;
        let (lx, ly, lw, lh) = LAYOUT_BOX;
        if pressed & 1 != 0 && (lx..lx + lw).contains(&px) && (ly..ly + lh).contains(&py) {
            // The kernel switches it and tells us back (LAYOUT_CHANGED);
            // the focus stays where it was.
            let _ = neo_rt::keyboard_layout(FOCUS, neo_rt::LAYOUT_SWITCH);
            return None;
        }
        let mut dirty = None;
        if pressed != 0 {
            // The window clicked gets the keys; the desk gives them back
            // to the console.
            let target =
                self.hit((px, py)).and_then(|i| self.windows[i]).and_then(|w| w.content).and_then(|c| c.events);
            self.set_focus(target);
            if let Some(mut i) = self.hit((px, py)) {
                if self.windows.get(i + 1).is_some_and(|w| w.is_some()) {
                    i = self.raise(i);
                    dirty = self.windows[i].map(|w| w.area());
                }
                let w = self.windows[i].unwrap_or(Window { at: (0, 0), title: [0; 8], content: None });
                let (x, y) = w.at;
                if py < y + TITLE_H {
                    if pressed & 1 != 0 {
                        self.grab = Some((i, w.at, (px - x, py - y)));
                    }
                } else if let Some(Content { events: Some(port), w: cw, h: ch, .. }) = w.content {
                    let (cx, cy) = (px - x - 1, py - y - TITLE_H - 1);
                    if (0..cw).contains(&cx) && (0..ch).contains(&cy) {
                        // Never wait for a slow program.
                        let _ = neo_rt::try_send(port, neo_rt::EVENT_CLICK, (cx as u64) << 16 | cy as u64, pressed);
                    }
                }
            }
        }
        if buttons & 1 == 0 {
            self.grab = None;
        }
        if let Some((i, _, (gx, gy))) = self.grab {
            let (ww, _) = self.windows[i].map(|w| w.size()).unwrap_or((0, 0));
            // Keep the title bar on the desk, above the Bar.
            let to = ((px - gx).clamp(40 - ww, W - 40), (py - gy).clamp(0, H - 41 - TITLE_H));
            self.grab = Some((i, to, (gx, gy)));
        }
        dirty
    }

    /// Sends typed keys to `port`, or to the console with `None`.
    fn set_focus(&mut self, port: Option<neo_rt::Handle>) {
        if self.focus != port {
            self.focus = port;
            let _ = neo_rt::focus_keys(FOCUS, port);
        }
    }

    /// A program closes its window; returns the area it leaves.
    fn close(&mut self, owner: u64) -> Option<Rect> {
        let i = self.windows.iter().position(|w| w.is_some_and(|w| w.content.is_some_and(|c| c.owner == owner)))?;
        let w = self.windows[i]?;
        if let Some(Content { events: Some(port), .. }) = w.content {
            if self.focus == Some(port) {
                self.set_focus(None);
            }
            let _ = neo_rt::close(port);
        }
        // The rest move down one; the memory stays mapped (the next window
        // gets its own).
        for j in i..MAX_WINDOWS - 1 {
            self.windows[j] = self.windows[j + 1];
        }
        self.windows[MAX_WINDOWS - 1] = None;
        self.grab = None;
        Some(w.area())
    }

    /// A program opens a window with its pixels in shared memory `handle`.
    /// Returns the area to draw, or None if it was refused.
    fn open(&mut self, owner: u64, size: u64, handle: Option<neo_rt::Handle>) -> Option<Rect> {
        let handle = handle?;
        let (w, h) = ((size >> 16) as i32 & 0xFFFF, size as i32 & 0xFFFF);
        let slot = self.windows.iter().position(|w| w.is_none());
        let bytes = neo_rt::file_size(handle).ok();
        let fits = bytes.is_some_and(|b| b >= (w * h * 4) as u64);
        let ok = slot.is_some()
            && fits
            && (1..=MAX_CONTENT.0).contains(&w)
            && (1..=MAX_CONTENT.1).contains(&h)
            && self.windows.iter().flatten().all(|win| win.content.is_none_or(|c| c.owner != owner));
        let pixels = if ok { neo_rt::map_device(handle).ok() } else { None };
        // The mapping keeps the memory; the handle is not needed any more.
        let _ = neo_rt::close(handle);
        let (slot, pixels) = (slot?, pixels?);
        let title = match self.title.take() {
            Some((from, t)) if from == owner => t,
            _ => *b"Program\0",
        };
        // New windows cascade from the lower right of the desk; a large one
        // moves left and up until it fits above the Bar.
        let (ow, oh) = (w + 2, h + TITLE_H + 2);
        let at = (
            (570 - 24 * (slot as i32 - 1)).min(W - ow - 8).max(8),
            (340 - 24 * (slot as i32 - 1)).min(H - 41 - oh - 8).max(8),
        );
        let events = match self.events.take() {
            Some((from, port)) if from == owner => Some(port),
            Some((_, port)) => {
                let _ = neo_rt::close(port);
                None
            }
            None => None,
        };
        let win = Window { at, title, content: Some(Content { pixels: pixels as *const u32, w, h, owner, events }) };
        self.windows[slot] = Some(win);
        // A new window comes up in front and gets the keys.
        self.set_focus(events);
        Some(win.area())
    }

    /// The area of `owner`'s window, to show what it drew.
    fn drawn(&self, owner: u64) -> Option<Rect> {
        self.windows.iter().flatten().find(|w| w.content.is_some_and(|c| c.owner == owner)).map(|w| w.area())
    }
}

#[no_mangle]
fn neo_main() -> i64 {
    if set_mode().is_err() {
        return 1;
    }
    let Ok(fb) = neo_rt::map_device(FRAMEBUFFER) else { return 2 };
    // The mode is W x H at 32 bits, and the whole framebuffer is mapped.
    let screen = unsafe { Screen::new(fb as *mut u32, W, H) };
    let welcome = Window { at: (90, 100), title: [0; 8], content: None };
    let mut desk = Desk {
        windows: [Some(welcome), None, None, None],
        grab: None,
        buttons: 0,
        title: None,
        events: None,
        focus: None,
    };
    let mut dog = Dijana::new();
    let mut shown = time().unwrap_or((12, 0, 0));
    repaint(&screen, FULL, &desk.windows, shown, &dog);
    let mut redraws = 1; // the first drawing counts
    let mut cursor = Cursor { at: None, under: [0; (ARROW_W * ARROW_H) as usize] };
    let mut pointer = None;
    loop {
        // Orders, windows and pointer moves; what needs drawing is
        // gathered into one rectangle.
        let mut dirty: Option<Rect> = None;
        let mut add = |r: Rect| dirty = Some(dirty.map_or(r, |d| union(d, r)));
        while let Ok(Some(m)) = neo_rt::try_recv(CONTROL) {
            match m.tag {
                QUIT => {
                    // Keys back to the console; tell every program with a
                    // window that the desk is going.
                    desk.set_focus(None);
                    for w in desk.windows.iter().flatten() {
                        if let Some(Content { events: Some(port), .. }) = w.content {
                            let _ = neo_rt::try_send(port, neo_rt::EVENT_QUIT, 0, 0);
                        }
                    }
                    return redraws;
                }
                neo_rt::POINTER_MOVED => {
                    let p = (m.a0 as i32, (m.a1 & 0xFFFF) as i32);
                    pointer = Some(p);
                    let buttons = m.a1 >> 16;
                    let clicked = buttons & !desk.buttons & 1 != 0;
                    if let Some(r) = desk.pointer(p, buttons) {
                        add(r);
                    }
                    if let Some(r) = dog.pointer(p, clicked) {
                        add(r);
                    }
                }
                neo_rt::LAYOUT_CHANGED => {
                    LAYOUT.store(m.a0 & 1, Ordering::Relaxed);
                    add(LAYOUT_BOX);
                }
                neo_rt::WINDOW_EVENTS => {
                    if let Some(h) = m.handle {
                        if let Some((_, old)) = desk.events.replace((m.sender, h)) {
                            let _ = neo_rt::close(old);
                        }
                    }
                }
                neo_rt::WINDOW_CLOSE => {
                    if let Some(r) = desk.close(m.sender) {
                        add(r);
                    }
                }
                neo_rt::WINDOW_TITLE => desk.title = Some((m.sender, m.a0.to_le_bytes())),
                neo_rt::WINDOW_OPEN => {
                    if let Some(r) = desk.open(m.sender, m.a0, m.handle) {
                        add(r);
                    }
                }
                neo_rt::WINDOW_DRAWN => {
                    if let Some(r) = desk.drawn(m.sender) {
                        add(r);
                    }
                }
                _ => {
                    // Not for us: a handle that came with it is closed.
                    if let Some(h) = m.handle {
                        let _ = neo_rt::close(h);
                    }
                }
            }
        }
        if let Some((i, to, _)) = desk.grab {
            if let Some(w) = desk.windows[i].as_mut() {
                if w.at != to {
                    let old = w.area();
                    w.at = to;
                    add(union(old, w.area()));
                }
            }
        }
        // Each turn of the loop is about 20 ms (it sleeps two ticks).
        if let Some(r) = dog.tick(20) {
            add(r);
        }
        let now = time().unwrap_or((12, 0, 0));
        let clock_due = now != shown;
        if clock_due || pointer != cursor.at || dirty.is_some() {
            // Take the arrow off, draw, put it back on top.
            cursor.hide(&screen);
            if let Some(r) = dirty {
                repaint(&screen, r, &desk.windows, now, &dog);
            }
            if clock_due {
                // The tile and the Bar's time, under windows that are over
                // them.
                repaint(&screen, (TX, TY, TW, TH), &desk.windows, now, &dog);
                repaint(&screen, (W - 60, H - 32, 48, 20), &desk.windows, now, &dog);
                shown = now;
                redraws += 1;
            }
            if let Some(p) = pointer {
                cursor.show(&screen, p);
            }
        }
        neo_rt::sleep(2);
    }
}
