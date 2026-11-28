//! The terminal: Neo DOS in a window on the desk.
//!
//! Handle 0 is a port where the kernel copies what programs print on the
//! console (tag `TEXT + n`, n bytes packed in the two words); handle 1 lets
//! it type into the console, as a keyboard does. It opens an 80 x 25 window
//! through the display service, draws the text as it comes, and passes the
//! keys typed while its window has the focus on to the console, where
//! Neo DOS reads them.
//!
//! It ends when the display service stops, and returns how many bytes it
//! showed.

#![no_std]
#![no_main]

use neo_gfx::{Event, Window};
use neo_rt::{Error, Handle};

const TEXT_PORT: Handle = 0;
const TYPING: Handle = 1;
/// The kernel's tag for console text: `TEXT + n` carries n bytes.
const TEXT: u64 = 0x40;

const COLS: usize = 80;
const ROWS: usize = 25;
const CW: i32 = 8;
const CH: i32 = 16;
const PAGE: u32 = 0xFFFFFF;
const INK: u32 = 0x1B2125;
const CURSOR: u32 = 0xC86F00;

struct Term {
    win: Window,
    cells: [[char; COLS]; ROWS],
    /// A UTF-8 character still arriving: its bytes so far.
    pending: ([u8; 4], usize),
    col: usize,
    row: usize,
    shown: i64,
}

impl Term {
    fn draw_cell(&self, col: usize, row: usize) {
        let (x, y) = (col as i32 * CW, row as i32 * CH);
        let c = &self.win.canvas;
        c.rect(x, y, CW, CH, PAGE);
        let ch = self.cells[row][col];
        if ch != ' ' {
            c.text(x, y, ch.encode_utf8(&mut [0; 4]), 1, INK);
        }
    }

    fn draw_cursor(&self, on: bool) {
        if self.col < COLS {
            if on {
                let (x, y) = (self.col as i32 * CW, self.row as i32 * CH);
                self.win.canvas.rect(x, y + CH - 3, CW, 2, CURSOR);
            } else {
                self.draw_cell(self.col, self.row);
            }
        }
    }

    fn redraw(&self) {
        for row in 0..ROWS {
            for col in 0..COLS {
                self.draw_cell(col, row);
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row + 1 < ROWS {
            self.row += 1;
            return;
        }
        // Scroll up one line.
        self.cells.copy_within(1.., 0);
        self.cells[ROWS - 1] = [' '; COLS];
        self.redraw();
    }

    fn byte(&mut self, b: u8) {
        self.shown += 1;
        match b {
            b'\n' => self.newline(),
            b'\r' => self.col = 0,
            0x08 => self.col = self.col.saturating_sub(1),
            0x20..=0x7E => self.put(b as char),
            0x80..=0xF4 => {
                // UTF-8: collect the bytes until the character is whole.
                let (buf, n) = &mut self.pending;
                if b & 0xC0 != 0x80 {
                    *n = 0;
                }
                if *n < 4 {
                    buf[*n] = b;
                    *n += 1;
                }
                if let Some(c) = core::str::from_utf8(&buf[..*n]).ok().and_then(|s| s.chars().next()) {
                    *n = 0;
                    self.put(c);
                }
            }
            _ => {}
        }
    }

    fn put(&mut self, c: char) {
        if self.col >= COLS {
            self.newline();
        }
        self.cells[self.row][self.col] = c;
        self.draw_cell(self.col, self.row);
        self.col += 1;
    }
}

fn run() -> Result<i64, Error> {
    let win = Window::open("Neo DOS", COLS as i32 * CW, ROWS as i32 * CH)?;
    let mut t = Term { win, cells: [[' '; COLS]; ROWS], pending: ([0; 4], 0), col: 0, row: 0, shown: 0 };
    t.redraw();
    t.draw_cursor(true);
    t.win.drawn()?;
    loop {
        let mut changed = false;
        while let Some(m) = neo_rt::try_recv(TEXT_PORT)? {
            let n = m.tag.wrapping_sub(TEXT) as usize;
            if !(1..=16).contains(&n) {
                continue;
            }
            if !changed {
                t.draw_cursor(false);
                changed = true;
            }
            let mut bytes = [0u8; 16];
            bytes[..8].copy_from_slice(&m.a0.to_le_bytes());
            bytes[8..].copy_from_slice(&m.a1.to_le_bytes());
            for &b in &bytes[..n] {
                t.byte(b);
            }
        }
        while let Some(event) = t.win.poll()? {
            match event {
                Event::Quit => {
                    // The window goes with the program.
                    let _ = t.win.close();
                    return Ok(t.shown);
                }
                Event::Key(k) => {
                    let _ = neo_rt::type_to_console(TYPING, k);
                }
                Event::Click { .. } => {}
            }
        }
        if changed {
            t.draw_cursor(true);
            t.win.drawn()?;
        }
        neo_rt::sleep(2);
    }
}

#[no_mangle]
fn neo_main() -> i64 {
    run().unwrap_or(-1)
}
