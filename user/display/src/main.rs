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
//! moving, wakes when it moves, and answers a click on her. After an
//! error (a message tagged `DIJANA_TIP`, from Neo DOS: an unknown command,
//! a missing file, a program that crashed or failed) she wakes and gives a
//! short tip. When the system shuts down (`QUIT` with `QUIT_GOOD_NIGHT`)
//! she says good night and curls up to sleep before the desk goes. On
//! `QUIT` the service ends with its clock redraws, plus the tips given
//! times 2^32.
//!
//! Windows: the amber dot at the left of a title bar is the close box. A
//! click on it asks the window's program to close (an `EVENT_QUIT`), or
//! hides the Welcome window, which the Neo menu's About brings back. The
//! Bar lists the windows, a button each in the order they opened; the
//! front window's button is lit and a click on a button brings its window
//! to the front.

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

/// The desk: a canvas over the back buffer, in ordinary memory. Everything
/// is drawn there first and only finished areas are copied to the screen.
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

/// The first piece of the compositor: the desk is drawn in a back buffer
/// in ordinary (cached) memory, and finished areas are copied to the
/// framebuffer with the pointer's arrow laid on top. The screen never
/// shows half-drawn layers (no flicker while a window is dragged), and
/// nothing is ever read back from video memory, which is slow on real
/// cards. The arrow lives only on the screen, so the desk under it never
/// needs to be saved and put back.
struct Compositor {
    front: *mut u32,
    back: *const u32,
    /// Where the arrow is on the screen.
    arrow: Option<(i32, i32)>,
    /// Pixels copied to the screen, for the log and tests.
    copied: u64,
}

impl Compositor {
    /// Copies `area` of the back buffer to the screen, with the arrow (when
    /// it is there) on top.
    fn present(&mut self, (x, y, w, h): Rect) {
        let (x0, y0, x1, y1) = (x.max(0), y.max(0), (x + w).min(W), (y + h).min(H));
        for yy in y0..y1 {
            for xx in x0..x1 {
                let i = (yy * W + xx) as usize;
                // Both buffers hold W x H pixels; the point is inside.
                let mut c = unsafe { self.back.add(i).read() };
                if let Some((ax, ay)) = self.arrow {
                    let (dx, dy) = (xx - ax, yy - ay);
                    if (0..ARROW_W).contains(&dx) && (0..ARROW_H).contains(&dy) {
                        match ARROW[dy as usize][dx as usize] {
                            b'#' => c = 0x000000,
                            b'o' => c = PAGE,
                            _ => {}
                        }
                    }
                }
                // One aligned 32-bit write per pixel: device memory (the
                // framebuffer on AArch64) faults on wider unaligned ones.
                unsafe { self.front.add(i).write_volatile(c) };
            }
        }
        self.copied += ((x1 - x0).max(0) * (y1 - y0).max(0)) as u64;
    }

    /// Moves the arrow: the old place gets the desk back, the new one the
    /// arrow.
    fn move_arrow(&mut self, to: Option<(i32, i32)>) {
        if to == self.arrow {
            return;
        }
        let old = self.arrow;
        self.arrow = to;
        for (x, y) in [old, to].into_iter().flatten() {
            self.present((x, y, ARROW_W, ARROW_H));
        }
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
    /// When it opened: the Bar lists windows in this order.
    seq: u32,
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
fn repaint(s: &Screen, area: Rect, windows: &[Option<Window>], now: (i32, i32, i32), dog: &Dijana, calc: &Calc) {
    s.set_clip(area);
    draw(s);
    clock(s, now);
    meter(s);
    calc.draw(s);
    for w in windows.iter().flatten() {
        w.draw(s);
    }
    bar(s, now, windows);
    dog.draw(s);
    menu(s);
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

    // The Meter tile's frame; the readings are drawn by `meter`.
    s.rect(MX + 5, MY + 6, MW, MH, mix(WALL_BOTTOM, 0, 1, 2));
    s.rect(MX, MY, MW, MH, TILE);
    s.frame(MX, MY, MW, MH, LINE);
    s.text(MX + 10, MY + 6, "Merač · Meter", 1, INK);
}

/// The Calculator tile (Kalkulator), bottom left: whole numbers, the four
/// operations, C to clear. Clicked like a pocket calculator; a division by
/// zero is answered by Dijana instead of a crash.
const CX: i32 = 8;
const CY: i32 = 382;
// Narrow enough that the Neo DOS window (from x = 150) leaves it whole.
const CW: i32 = 140;
const CH: i32 = 172;
pub const CALC: Rect = (CX, CY, CW, CH);
/// The keys, row after row.
const KEYS: [[u8; 4]; 4] = [*b"789/", *b"456*", *b"123-", *b"C0=+"];
const KEY_W: i32 = 28;
const KEY_H: i32 = 24;
const KEY_GAP: i32 = 4;
/// Where the keys start.
const KEYS_X: i32 = CX + 8;
const KEYS_Y: i32 = CY + 56;
/// The largest number shown (12 digits); beyond it the tile says so.
const CALC_MAX: i64 = 999_999_999_999;

/// What a key press did that the desk must know about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CalcSaid {
    Fine,
    DivideByZero,
    TooLarge,
}

/// The calculator's state: the number being typed (or the last result),
/// the number before the operator, and the operator.
struct Calc {
    shown: i64,
    before: i64,
    op: Option<u8>,
    /// True when the next digit starts a new number.
    fresh: bool,
    /// Shown instead of the number after an error.
    error: Option<&'static str>,
}

impl Calc {
    fn new() -> Calc {
        Calc { shown: 0, before: 0, op: None, fresh: true, error: None }
    }

    /// The key at a point on the screen, if any.
    fn key_at((px, py): (i32, i32)) -> Option<u8> {
        let (dx, dy) = (px - KEYS_X, py - KEYS_Y);
        if dx < 0 || dy < 0 {
            return None;
        }
        let (col, row) = (dx / (KEY_W + KEY_GAP), dy / (KEY_H + KEY_GAP));
        let inside = dx % (KEY_W + KEY_GAP) < KEY_W && dy % (KEY_H + KEY_GAP) < KEY_H;
        (inside && col < 4 && row < 4).then(|| KEYS[row as usize][col as usize])
    }

    /// Applies the pending operator to `before` and `shown`.
    fn apply(&mut self) -> CalcSaid {
        let Some(op) = self.op.take() else { return CalcSaid::Fine };
        let r = match op {
            b'+' => self.before.checked_add(self.shown),
            b'-' => self.before.checked_sub(self.shown),
            b'*' => self.before.checked_mul(self.shown),
            _ if self.shown == 0 => {
                self.error = Some("/ 0 ?");
                return CalcSaid::DivideByZero;
            }
            _ => self.before.checked_div(self.shown),
        };
        match r {
            Some(v) if v.abs() <= CALC_MAX => {
                self.shown = v;
                CalcSaid::Fine
            }
            _ => {
                self.error = Some("Too big·Veliko");
                CalcSaid::TooLarge
            }
        }
    }

    fn press(&mut self, key: u8) -> CalcSaid {
        if key == b'C' {
            *self = Calc::new();
            return CalcSaid::Fine;
        }
        if self.error.is_some() {
            // After an error only C helps; a digit starts over too.
            if !key.is_ascii_digit() {
                return CalcSaid::Fine;
            }
            *self = Calc::new();
        }
        match key {
            b'0'..=b'9' => {
                let d = (key - b'0') as i64;
                if self.fresh {
                    self.shown = d;
                    self.fresh = false;
                } else if self.shown.abs() * 10 + d <= CALC_MAX {
                    self.shown = self.shown * 10 + if self.shown < 0 { -d } else { d };
                }
                CalcSaid::Fine
            }
            b'=' => {
                let said = self.apply();
                self.fresh = true;
                said
            }
            op => {
                // A chain such as 2 + 3 * 4 works left to right.
                let said = if self.fresh { CalcSaid::Fine } else { self.apply() };
                self.before = self.shown;
                self.op = Some(op);
                self.fresh = true;
                said
            }
        }
    }

    fn draw(&self, s: &Screen) {
        s.rect(CX + 5, CY + 6, CW, CH, mix(WALL_BOTTOM, 0, 1, 2));
        s.rect(CX, CY, CW, CH, TILE);
        s.frame(CX, CY, CW, CH, LINE);
        s.text(CX + 10, CY + 6, "Kalkulator", 1, INK);
        // The display, numbers on the right.
        let (dx, dy, dw, dh) = (CX + 9, CY + 26, CW - 18, 22);
        s.rect(dx, dy, dw, dh, PAGE);
        s.frame(dx, dy, dw, dh, LINE);
        let mut buf = [0u8; 20];
        let text = match self.error {
            Some(e) => e,
            None if self.shown < 0 => {
                let n = number(self.shown.unsigned_abs(), &mut buf).len();
                let at = buf.len() - n - 1;
                buf[at] = b'-';
                core::str::from_utf8(&buf[at..]).unwrap_or("?")
            }
            None => number(self.shown as u64, &mut buf),
        };
        let width = 8 * text.chars().count() as i32;
        s.text(dx + dw - 6 - width, dy + 3, text, 1, if self.error.is_some() { AMBER } else { INK });
        for (row, keys) in KEYS.iter().enumerate() {
            for (col, &k) in keys.iter().enumerate() {
                let x = KEYS_X + col as i32 * (KEY_W + KEY_GAP);
                let y = KEYS_Y + row as i32 * (KEY_H + KEY_GAP);
                let operator = !k.is_ascii_digit();
                s.rect(
                    x,
                    y,
                    KEY_W,
                    KEY_H,
                    if k == b'=' {
                        ACCENT
                    } else if operator {
                        WINDOW
                    } else {
                        PAGE
                    },
                );
                s.frame(x, y, KEY_W, KEY_H, LINE);
                let label = [k];
                let c = if k == b'=' { PAGE } else { INK };
                s.text(x + 10, y + 4, core::str::from_utf8(&label).unwrap_or("?"), 1, c);
            }
        }
    }
}

/// The Meter tile (Merač), under the Clock: how busy the processor was in
/// the last second, how much memory is in use, how many threads run. Read
/// from the kernel once a second, with the clock.
const MX: i32 = TX;
const MY: i32 = TY + TH + 16;
const MW: i32 = TW;
const MH: i32 = 118;
const METER: Rect = (MX, MY, MW, MH);

/// The last readings: processor busy (percent), memory used and usable
/// (MiB), threads. Kept here so any repaint can draw them.
static CPU: AtomicU64 = AtomicU64::new(0);
static RAM_USED: AtomicU64 = AtomicU64::new(0);
static RAM_ALL: AtomicU64 = AtomicU64::new(0);
static THREADS: AtomicU64 = AtomicU64::new(0);

/// Reads the kernel's counters and works out the readings since `last`.
fn measure(last: &mut neo_rt::Stats) {
    let Ok(now) = neo_rt::stats() else { return };
    let ticks = now.ticks.saturating_sub(last.ticks);
    let idle = now.idle_ticks.saturating_sub(last.idle_ticks).min(ticks);
    if ticks > 0 {
        CPU.store((ticks - idle) * 100 / ticks, Ordering::Relaxed);
    }
    let mib = |frames: u64| frames * 4096 / (1024 * 1024);
    RAM_ALL.store(mib(now.ram_frames), Ordering::Relaxed);
    RAM_USED.store(mib(now.ram_frames.saturating_sub(now.free_frames)), Ordering::Relaxed);
    THREADS.store(now.threads, Ordering::Relaxed);
    *last = now;
}

/// A number into text without a heap.
fn number(n: u64, buf: &mut [u8; 20]) -> &str {
    let mut i = buf.len();
    let mut n = n;
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    core::str::from_utf8(&buf[i..]).unwrap_or("?")
}

/// A reading: label, value text, and a bar filled `part` of `whole`.
fn gauge(s: &Screen, y: i32, label: &str, value: &[&str], part: u64, whole: u64) {
    s.rect(MX + 1, y, MW - 2, 34, TILE);
    s.text(MX + 10, y, label, 1, MUTED);
    let mut x = MX + 54;
    for v in value {
        x = s.text(x, y, v, 1, INK);
    }
    let (bx, by, bw, bh) = (MX + 10, y + 19, MW - 20, 8);
    s.rect(bx, by, bw, bh, PAGE);
    s.frame(bx, by, bw, bh, LINE);
    let fill = if whole == 0 { 0 } else { ((bw - 2) as u64 * part.min(whole) / whole) as i32 };
    s.rect(bx + 1, by + 1, fill, bh - 2, if part * 10 > whole * 8 { AMBER } else { ACCENT });
}

fn meter(s: &Screen) {
    let (mut a, mut b, mut c) = ([0u8; 20], [0u8; 20], [0u8; 20]);
    let cpu = CPU.load(Ordering::Relaxed);
    gauge(s, MY + 28, "CPU", &[number(cpu, &mut a), " %"], cpu, 100);
    let (used, all) = (RAM_USED.load(Ordering::Relaxed), RAM_ALL.load(Ordering::Relaxed));
    gauge(s, MY + 62, "RAM", &[number(used, &mut b), " / ", number(all, &mut c), " MB"], used, all);
    let mut d = [0u8; 20];
    s.rect(MX + 1, MY + 96, MW - 2, 18, TILE);
    s.text(MX + 10, MY + 96, "Niti · Threads", 1, MUTED);
    let t = THREADS.load(Ordering::Relaxed);
    let n = number(t, &mut d);
    s.text(MX + MW - 10 - 8 * n.len() as i32, MY + 96, n, 1, INK);
}

/// The keyboard layout shown on the Bar (0 US, 1 SR), as the kernel last
/// reported it.
static LAYOUT: AtomicU64 = AtomicU64::new(0);
/// The Bar's layout switch, left of the time: a click switches the layout.
const LAYOUT_BOX: Rect = (W - 104, H - 32, 36, 24);

/// The Neo menu: opened by a click on Dijana's head and "Neo" at the left
/// end of the Bar; it opens upwards, over everything. A click on an item
/// does it and closes the menu; a click anywhere else only closes it.
const NEO_BOX: Rect = (0, H - 40, 70, 40);
const MENU_ITEMS: [(&str, &str); 3] = [("O sistemu", "About"), ("Pomoć", "Help"), ("Laku noć", "Shut down")];
const ITEM_H: i32 = 26;
const MENU: Rect = (4, H - 41 - 4 - (ITEM_H * MENU_ITEMS.len() as i32 + 8), 220, ITEM_H * MENU_ITEMS.len() as i32 + 8);
static MENU_OPEN: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// The menu item at a point, if the point is on one.
fn menu_item((px, py): (i32, i32)) -> Option<usize> {
    let (x, y, w, _) = MENU;
    let i = (py - y - 4).div_euclid(ITEM_H);
    ((x..x + w).contains(&px) && py >= y + 4 && (i as usize) < MENU_ITEMS.len()).then_some(i as usize)
}

fn menu(s: &Screen) {
    if !MENU_OPEN.load(Ordering::Relaxed) {
        return;
    }
    let (x, y, w, h) = MENU;
    s.rect(x + 4, y + 5, w, h, mix(WALL_BOTTOM, 0, 1, 2));
    s.rect(x, y, w, h, PAGE);
    s.frame(x, y, w, h, LINE);
    for (i, (sr, en)) in MENU_ITEMS.iter().enumerate() {
        let iy = y + 4 + i as i32 * ITEM_H;
        if i == MENU_ITEMS.len() - 1 {
            s.rect(x + 6, iy - 1, w - 12, 1, LINE);
        }
        let after = s.text(x + 14, iy + 5, sr, 1, INK);
        let after = s.text(after, iy + 5, " · ", 1, MUTED);
        s.text(after, iy + 5, en, 1, MUTED);
    }
}

/// The Bar's window list: a button per window, in the order they opened,
/// from just right of the Neo menu. The front window's button is lit; a
/// click on a button brings its window to the front.
const TASK_X: i32 = 80;
const TASK_W: i32 = 96;
const TASK_STEP: i32 = TASK_W + 8;
const TASK_Y: i32 = H - 32;
const TASK_H: i32 = 24;
/// The whole Bar, drawn again when windows open, close or change places.
const BAR_AREA: Rect = (0, H - 41, W, 41);

/// The windows' indices in the order they opened, and how many there are.
fn tasks(windows: &[Option<Window>]) -> ([usize; MAX_WINDOWS], usize) {
    let mut order = [0; MAX_WINDOWS];
    let mut n = 0;
    for (i, w) in windows.iter().enumerate() {
        if w.is_some() && n < MAX_WINDOWS {
            order[n] = i;
            n += 1;
        }
    }
    order[..n].sort_unstable_by_key(|&i| windows[i].map_or(0, |w| w.seq));
    (order, n)
}

/// The window whose Bar button is at a point, if any.
fn task_at(windows: &[Option<Window>], (px, py): (i32, i32)) -> Option<usize> {
    let (order, n) = tasks(windows);
    let k = (px - TASK_X).div_euclid(TASK_STEP);
    let inside = (TASK_Y..TASK_Y + TASK_H).contains(&py) && px >= TASK_X && (px - TASK_X) % TASK_STEP < TASK_W;
    (inside && (k as usize) < n).then(|| order[k as usize])
}

/// The name a window has on the Bar.
fn task_name(w: &Window) -> &str {
    match w.content {
        None => "Dobrodošli",
        Some(_) => core::str::from_utf8(&w.title).unwrap_or("?").trim_end_matches('\0'),
    }
}

/// The Bar, along the bottom, with the window list, the layout switch and
/// the time on it.
fn bar(s: &Screen, (h, m, _): (i32, i32, i32), windows: &[Option<Window>]) {
    s.rect(0, H - 40, W, 40, BAR);
    s.rect(0, H - 41, W, 1, LINE);
    // Dijana's head is the logo, then the name: together the Neo menu's
    // button, lit while the menu is open.
    if MENU_OPEN.load(Ordering::Relaxed) {
        s.rect(1, H - 38, 66, 36, WINDOW);
    }
    s.sprite(4, H - 31, &dijana::HEAD, dijana::HEAD_W, &dijana::PALETTE, 1);
    s.text(36, H - 28, "Neo", 1, INK);
    s.rect(70, H - 32, 1, 24, LINE);
    let (order, n) = tasks(windows);
    let front = (0..windows.len()).rev().find(|&i| windows[i].is_some());
    for (k, &i) in order[..n].iter().enumerate() {
        let Some(w) = windows[i] else { continue };
        let x = TASK_X + k as i32 * TASK_STEP;
        let lit = Some(i) == front;
        s.rect(x, TASK_Y, TASK_W, TASK_H, if lit { ACCENT } else { PAGE });
        s.frame(x, TASK_Y, TASK_W, TASK_H, LINE);
        s.text(x + 8, TASK_Y + 4, task_name(&w), 1, if lit { PAGE } else { INK });
    }
    let hm = [two(h), two(m)];
    let digits = [hm[0][0], hm[0][1], b':', hm[1][0], hm[1][1]];
    s.text(W - 56, H - 28, core::str::from_utf8(&digits).unwrap_or(""), 1, INK);
    let (x, y, w, bh) = LAYOUT_BOX;
    let serbian = LAYOUT.load(Ordering::Relaxed) == 1;
    s.rect(x, y, w, bh, if serbian { ACCENT } else { PAGE });
    s.frame(x, y, w, bh, LINE);
    s.text(x + 10, y + 4, if serbian { "SR" } else { "US" }, 1, if serbian { PAGE } else { INK });
}

/// The close box: the amber dot at the left of a title bar, and the square
/// around it that takes the click.
const CLOSE_DOT: (i32, i32) = (16, 14);

/// Whether a point is on the close box of a window at `at`.
fn on_close_box((x, y): (i32, i32), (px, py): (i32, i32)) -> bool {
    let (dx, dy) = (px - x - CLOSE_DOT.0, py - y - CLOSE_DOT.1);
    dx.abs() <= 7 && dy.abs() <= 7
}

/// A window's frame, shadow and title bar.
fn frame(s: &Screen, (x, y): (i32, i32), (w, wh): (i32, i32), title: &str) {
    s.rect(x + 6, y + 8, w, wh, mix(WALL_BOTTOM, 0, 1, 2)); // shadow
    s.rect(x, y, w, wh, WINDOW);
    s.frame(x, y, w, wh, LINE);
    s.rect(x + 1, y + TITLE_H, w - 2, 1, LINE);
    s.disc(x + CLOSE_DOT.0, y + CLOSE_DOT.1, 5, AMBER);
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
/// How long her good night stays on the screen before the desk goes.
const GOOD_NIGHT_MS: i32 = 1_500;

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

    /// The words for a tip after an error, if `kind` is one she knows.
    fn tip_words(kind: u64) -> Option<(&'static str, &'static str)> {
        Some(match kind {
            neo_rt::TIP_UNKNOWN_COMMAND => {
                ("Ne znam tu reč. HELP kaže sve reči.", "Not a command. HELP lists them all.")
            }
            neo_rt::TIP_NO_FILE => ("Nema tog fajla. DIR kaže šta ima.", "No such file. DIR shows the files."),
            neo_rt::TIP_PROGRAM_STOPPED => {
                ("Program je pao, ali sve ostalo radi.", "A program crashed; the rest is fine.")
            }
            neo_rt::TIP_PROGRAM_FAILED => ("Taj program je završio sa greškom.", "That program ended with an error."),
            _ => return None,
        })
    }

    /// After an error: she wakes up and gives a tip. Returns the area to
    /// draw again, or None for a kind she does not know.
    fn tip(&mut self, kind: u64) -> Option<Rect> {
        let (sr, en) = Self::tip_words(kind)?;
        self.still_ms = 0;
        if self.mood == Mood::Sleep {
            self.set(Mood::Idle);
        }
        self.say(sr, en);
        Some(union(DOG, BUBBLE))
    }

    /// The system is shutting down: she says good night and curls up.
    fn good_night(&mut self) -> Rect {
        self.set(Mood::Sleep);
        self.say("Laku noć! Vidimo se sutra.", "Good night! See you tomorrow.");
        union(DOG, BUBBLE)
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
    /// The next window's place in the Bar's list.
    seq: u32,
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
        if pressed & 1 != 0 {
            // A window's button on the Bar: that window comes to the front
            // and gets the keys.
            if let Some(i) = task_at(&self.windows, (px, py)) {
                let i = self.raise(i);
                let w = self.windows[i]?;
                self.set_focus(w.content.and_then(|c| c.events));
                return Some(union(w.area(), BAR_AREA));
            }
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
                let w = self.windows[i].unwrap_or(Window { at: (0, 0), title: [0; 8], seq: 0, content: None });
                let (x, y) = w.at;
                if pressed & 1 != 0 && on_close_box(w.at, (px, py)) {
                    match w.content {
                        // The Welcome window is ours: it just goes (the
                        // Neo menu's About brings it back).
                        None => return Some(union(self.remove(i), BAR_AREA)),
                        // A program is asked to close its own window; it
                        // answers with WINDOW_CLOSE. One without an event
                        // port cannot be asked, so its window stays.
                        Some(Content { events: Some(port), .. }) => {
                            let _ = neo_rt::try_send(port, neo_rt::EVENT_QUIT, 0, 0);
                        }
                        Some(_) => {}
                    }
                    return dirty.map(|d| union(d, BAR_AREA));
                }
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
        if let Some(d) = dirty {
            // The front window changed: so did the lit button.
            dirty = Some(union(d, BAR_AREA));
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
        Some(union(self.remove(i), BAR_AREA))
    }

    /// Takes window `i` off the desk; returns the area it leaves. The rest
    /// move down one; a program's memory stays mapped (the next window gets
    /// its own).
    fn remove(&mut self, i: usize) -> Rect {
        let area = self.windows[i].map_or((0, 0, 0, 0), |w| w.area());
        for j in i..MAX_WINDOWS - 1 {
            self.windows[j] = self.windows[j + 1];
        }
        self.windows[MAX_WINDOWS - 1] = None;
        self.grab = None;
        area
    }

    /// Brings the Welcome window back, or to the front if it is there.
    /// Returns the area to draw, or None if the desk is full.
    fn welcome(&mut self) -> Option<Rect> {
        let i = match self.windows.iter().position(|w| w.is_some_and(|w| w.content.is_none())) {
            Some(i) => self.raise(i),
            None => {
                let slot = self.windows.iter().position(|w| w.is_none())?;
                self.seq += 1;
                self.windows[slot] = Some(Window { at: (90, 100), title: [0; 8], seq: self.seq, content: None });
                slot
            }
        };
        self.set_focus(None);
        Some(union(self.windows[i]?.area(), BAR_AREA))
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
        self.seq += 1;
        let content = Some(Content { pixels: pixels as *const u32, w, h, owner, events });
        let win = Window { at, title, seq: self.seq, content };
        self.windows[slot] = Some(win);
        // A new window comes up in front and gets the keys.
        self.set_focus(events);
        Some(union(win.area(), BAR_AREA))
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
    // The back buffer: W x H pixels of ordinary memory.
    let pages = ((W * H * 4) as u64).div_ceil(4096);
    let Ok(back) = neo_rt::share(pages).and_then(neo_rt::map_device) else { return 3 };
    // The mode is W x H at 32 bits, the whole framebuffer is mapped, and
    // the back buffer is as large.
    let screen = unsafe { Screen::new(back as *mut u32, W, H) };
    let mut out = Compositor { front: fb as *mut u32, back: back as *const u32, arrow: None, copied: 0 };
    let welcome = Window { at: (90, 100), title: [0; 8], seq: 0, content: None };
    let mut desk = Desk {
        windows: [Some(welcome), None, None, None],
        grab: None,
        buttons: 0,
        title: None,
        events: None,
        focus: None,
        seq: 0,
    };
    let mut dog = Dijana::new();
    let mut calc = Calc::new();
    let mut readings = neo_rt::Stats::default();
    measure(&mut readings);
    let mut shown = time().unwrap_or((12, 0, 0));
    repaint(&screen, FULL, &desk.windows, shown, &dog, &calc);
    out.present(FULL);
    let mut redraws = 1; // the first drawing counts
    let mut pointer = None;
    let mut tips: i64 = 0;
    // Time left before the desk goes, once Dijana has said good night.
    let mut closing: Option<i32> = None;
    loop {
        // Orders, windows and pointer moves; what needs drawing is
        // gathered into one rectangle.
        let mut dirty: Option<Rect> = None;
        let mut add = |r: Rect| dirty = Some(dirty.map_or(r, |d| union(d, r)));
        while let Ok(Some(m)) = neo_rt::try_recv(CONTROL) {
            match m.tag {
                QUIT if m.a0 == neo_rt::QUIT_GOOD_NIGHT && closing.is_none() => {
                    // Shutting down: a moment for her good night first.
                    add(dog.good_night());
                    closing = Some(GOOD_NIGHT_MS);
                }
                QUIT => closing = Some(0),
                neo_rt::DIJANA_TIP => {
                    if let Some(r) = dog.tip(m.a0) {
                        tips += 1;
                        add(r);
                    }
                }
                neo_rt::POINTER_MOVED => {
                    let p = (m.a0 as i32, (m.a1 & 0xFFFF) as i32);
                    pointer = Some(p);
                    let buttons = m.a1 >> 16;
                    let clicked = buttons & !desk.buttons & 1 != 0;
                    // The Neo menu takes the click first.
                    let open = MENU_OPEN.load(Ordering::Relaxed);
                    let on_button = {
                        let (x, y, w, h) = NEO_BOX;
                        (x..x + w).contains(&p.0) && (y..y + h).contains(&p.1)
                    };
                    if clicked && (open || on_button) {
                        MENU_OPEN.store(!open && on_button, Ordering::Relaxed);
                        add(union(MENU, NEO_BOX));
                        if open {
                            match menu_item(p) {
                                Some(0) => {
                                    // The Welcome window comes back, in front.
                                    if let Some(r) = desk.welcome() {
                                        add(r);
                                    }
                                    dog.say("Neo Boing Platinum OS Lite", "Fortress · Neo DOS 0.3 · GPLv3");
                                    add(union(DOG, BUBBLE));
                                }
                                Some(1) => {
                                    dog.say("Pomoć: kucaj HELP u Neo DOS-u.", "Help: type HELP in Neo DOS.");
                                    add(union(DOG, BUBBLE));
                                }
                                Some(_) => {
                                    // Neo DOS ends; the kernel then stops us with a good night.
                                    let _ = neo_rt::shut_down(FOCUS);
                                }
                                None => {}
                            }
                        }
                        // The desk does not see this click, only that the button is down.
                        desk.buttons = buttons;
                        continue;
                    }
                    // A click on the Calculator, where no window covers it.
                    let key = Calc::key_at(p).filter(|_| clicked && desk.hit(p).is_none());
                    if let Some(r) = desk.pointer(p, buttons) {
                        add(r);
                    }
                    if let Some(k) = key {
                        add(CALC);
                        match calc.press(k) {
                            CalcSaid::Fine => {}
                            CalcSaid::DivideByZero => {
                                dog.say("Nulom se ne deli. Pritisni C.", "No dividing by zero. Press C.");
                                tips += 1;
                                add(union(DOG, BUBBLE));
                            }
                            CalcSaid::TooLarge => {}
                        }
                    }
                    if let Some(r) = dog.pointer(p, clicked).filter(|_| closing.is_none()) {
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
        if let Some(left) = closing {
            if left <= 0 {
                // Keys back to the console; tell every program with a
                // window that the desk is going.
                desk.set_focus(None);
                for w in desk.windows.iter().flatten() {
                    if let Some(Content { events: Some(port), .. }) = w.content {
                        let _ = neo_rt::try_send(port, neo_rt::EVENT_QUIT, 0, 0);
                    }
                }
                return redraws | tips << 32;
            }
            closing = Some(left - 20);
        }
        let now = time().unwrap_or((12, 0, 0));
        let clock_due = now != shown;
        // Draw in the back buffer, then copy what changed to the screen.
        if let Some(r) = dirty {
            repaint(&screen, r, &desk.windows, now, &dog, &calc);
            out.present(r);
        }
        if clock_due {
            // The tiles and the Bar's time, under windows that are over them.
            measure(&mut readings);
            for r in [(TX, TY, TW, TH), METER, (W - 60, H - 32, 48, 20)] {
                repaint(&screen, r, &desk.windows, now, &dog, &calc);
                out.present(r);
            }
            shown = now;
            redraws += 1;
        }
        out.move_arrow(pointer);
        neo_rt::sleep(2);
    }
}
