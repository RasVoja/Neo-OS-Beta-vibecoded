//! Drawing for Neo programs: a `Canvas` over 32-bit pixels (0xRRGGBB) with
//! rectangles, discs, rings, lines, text in the Neo 8x16 font and
//! palette sprites (Dijana's frames, in `dijana`), all kept inside a clip
//! rectangle; and `Window`, which opens a window through the display
//! service and hands its events to the program.
//!
//! The display service draws the desk with the same canvas, over the
//! framebuffer; a program draws over the memory it shares with the service.

#![no_std]

pub mod dijana;
pub mod font;

use core::cell::Cell;
use neo_rt::{Error, Handle};

/// x, y, width, height.
pub type Rect = (i32, i32, i32, i32);

/// Pixels to draw on: `w` x `h`, row after row.
pub struct Canvas {
    px: *mut u32,
    pub w: i32,
    pub h: i32,
    /// Drawing only touches this rectangle: x0, y0, x1, y1 (exclusive).
    clip: Cell<(i32, i32, i32, i32)>,
}

impl Canvas {
    /// A canvas over `w` x `h` pixels at `px`.
    ///
    /// # Safety
    /// `px` must point to at least `w * h` writable pixels that stay valid
    /// as long as the canvas.
    pub unsafe fn new(px: *mut u32, w: i32, h: i32) -> Canvas {
        Canvas { px, w, h, clip: Cell::new((0, 0, w, h)) }
    }

    /// The clip rectangle as x0, y0, x1, y1.
    pub fn clip(&self) -> (i32, i32, i32, i32) {
        self.clip.get()
    }

    /// Limits drawing to a rectangle (inside the canvas).
    pub fn set_clip(&self, (x, y, w, h): Rect) {
        self.clip.set((x.max(0), y.max(0), (x + w).min(self.w), (y + h).min(self.h)));
    }

    /// Lets drawing reach the whole canvas again.
    pub fn clear_clip(&self) {
        self.clip.set((0, 0, self.w, self.h));
    }

    pub fn get(&self, x: i32, y: i32) -> u32 {
        if (0..self.w).contains(&x) && (0..self.h).contains(&y) {
            // Inside the canvas, checked above.
            unsafe { self.px.add((y * self.w + x) as usize).read_volatile() }
        } else {
            0
        }
    }

    pub fn put(&self, x: i32, y: i32, c: u32) {
        let (x0, y0, x1, y1) = self.clip.get();
        if (x0..x1).contains(&x) && (y0..y1).contains(&y) {
            // Inside the clip, which is inside the canvas.
            unsafe { self.px.add((y * self.w + x) as usize).write_volatile(c) };
        }
    }

    pub fn rect(&self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        let (x0, y0, x1, y1) = self.clip.get();
        let (xa, xb) = (x.max(x0), (x + w).min(x1));
        for yy in y.max(y0)..(y + h).min(y1) {
            for xx in xa..xb {
                // Inside the clip, which is inside the canvas.
                unsafe { self.px.add((yy * self.w + xx) as usize).write_volatile(c) };
            }
        }
    }

    /// A one-pixel outline.
    pub fn frame(&self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        self.rect(x, y, w, 1, c);
        self.rect(x, y + h - 1, w, 1, c);
        self.rect(x, y, 1, h, c);
        self.rect(x + w - 1, y, 1, h, c);
    }

    pub fn disc(&self, cx: i32, cy: i32, r: i32, c: u32) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    self.put(cx + x, cy + y, c);
                }
            }
        }
    }

    /// A circle two pixels wide.
    pub fn ring(&self, cx: i32, cy: i32, r: i32, c: u32) {
        for y in -r..=r {
            for x in -r..=r {
                let d = x * x + y * y;
                if d <= r * r && d > (r - 2) * (r - 2) {
                    self.put(cx + x, cy + y, c);
                }
            }
        }
    }

    /// A line `thick` pixels wide (Bresenham, stamped with small squares).
    pub fn line(&self, x0: i32, y0: i32, x1: i32, y1: i32, thick: i32, c: u32) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            self.rect(x - thick / 2, y - thick / 2, thick, thick, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Indexed pixels (`w` wide, row after row; index 0 is transparent)
    /// through `palette`, each one drawn as a `scale` x `scale` square.
    /// Dijana's frames are drawn this way.
    pub fn sprite(&self, x: i32, y: i32, pixels: &[u8], w: i32, palette: &[u32], scale: i32) {
        for (i, &p) in pixels.iter().enumerate() {
            if p != 0 {
                let (col, row) = (i as i32 % w, i as i32 / w);
                let c = palette.get(p as usize).copied().unwrap_or(0);
                self.rect(x + col * scale, y + row * scale, scale, scale, c);
            }
        }
    }

    /// Text in the 8x16 font, `scale` times as large; returns where the
    /// next character would go. Characters the font lacks leave a gap.
    pub fn text(&self, mut x: i32, y: i32, s: &str, scale: i32, c: u32) -> i32 {
        for ch in s.chars() {
            if let Some(i) = font::CHARS.chars().position(|f| f == ch) {
                for (row, bits) in font::GLYPHS[i].iter().enumerate() {
                    for col in 0..8 {
                        if bits >> (7 - col) & 1 != 0 {
                            self.rect(x + col * scale, y + row as i32 * scale, scale, scale, c);
                        }
                    }
                }
            }
            x += 8 * scale;
        }
        x
    }
}

/// What a window's program hears about.
pub enum Event {
    /// A click at x, y inside the window, with the buttons just pressed.
    Click { x: i32, y: i32, buttons: u64 },
    /// A key typed while the window has the focus.
    Key(u8),
    /// The display service is stopping.
    Quit,
}

/// A window of this program's own, opened through the display service.
pub struct Window {
    pub canvas: Canvas,
    display: Handle,
    events: Handle,
}

impl Window {
    /// Opens a `w` x `h` window titled `title` (up to 8 bytes are shown).
    /// The program must be allowed to look ports up.
    pub fn open(title: &str, w: i32, h: i32) -> Result<Window, Error> {
        let display = neo_rt::find_port("display")?;
        let pages = ((w * h * 4) as u64).div_ceil(4096);
        let shared = neo_rt::share(pages)?;
        let px = neo_rt::map_device(shared)? as *mut u32;
        // The block holds w x h pixels and stays mapped for the program's life.
        let canvas = unsafe { Canvas::new(px, w, h) };
        let mut t = [0u8; 8];
        let n = title.len().min(8);
        t[..n].copy_from_slice(&title.as_bytes()[..n]);
        let events = neo_rt::new_port(16)?;
        neo_rt::send(display, neo_rt::WINDOW_TITLE, u64::from_le_bytes(t), 0)?;
        neo_rt::send_handle(display, neo_rt::WINDOW_EVENTS, 0, events)?;
        neo_rt::send_handle(display, neo_rt::WINDOW_OPEN, (w as u64) << 16 | h as u64, shared)?;
        Ok(Window { canvas, display, events })
    }

    /// Asks the display service to show what was drawn.
    pub fn drawn(&self) -> Result<(), Error> {
        neo_rt::send(self.display, neo_rt::WINDOW_DRAWN, 0, 0)
    }

    /// The next event, if one is waiting.
    pub fn poll(&self) -> Result<Option<Event>, Error> {
        loop {
            let Some(m) = neo_rt::try_recv(self.events)? else { return Ok(None) };
            return Ok(Some(match m.tag {
                neo_rt::EVENT_CLICK => {
                    Event::Click { x: (m.a0 >> 16) as i32, y: (m.a0 & 0xFFFF) as i32, buttons: m.a1 }
                }
                neo_rt::EVENT_KEY => Event::Key(m.a0 as u8),
                neo_rt::EVENT_QUIT => Event::Quit,
                _ => continue,
            }));
        }
    }

    /// Takes the window off the screen.
    pub fn close(self) -> Result<(), Error> {
        neo_rt::send(self.display, neo_rt::WINDOW_CLOSE, 0, 0)
    }
}
