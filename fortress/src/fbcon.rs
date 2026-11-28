//! The kernel's text on the screen.
//!
//! On a real PC there is no serial line to read, so when the machine ends
//! in a black screen nobody can tell where the boot stopped. This draws the
//! kernel's log (everything `kprintln!` prints) on the boot loader's
//! framebuffer, from the moment the page tables are up until the display
//! service takes the screen over; a panic or a CPU exception brings it back,
//! so the last words stay readable. White 8 x 16 letters on black, and when
//! the screen is full it is cleared and filled again from the top, so the
//! newest lines are always the ones to read.
//!
//! The framebuffer may sit high in the address space (a 64-bit BAR of a
//! graphics card), so it is mapped with `map_device`.

#[cfg(target_arch = "x86_64")]
#[path = "../../user/neo-gfx/src/font.rs"]
mod font;

#[cfg(target_arch = "x86_64")]
mod real {
    use core::sync::atomic::{AtomicBool, Ordering};

    use super::font;

    const CELL_W: u64 = 8;
    const CELL_H: u64 = 16;
    const WHITE: u32 = 0x00FF_FFFF;
    const BLACK: u32 = 0;

    struct Screen {
        base: u64,
        width: u64,
        height: u64,
        pitch: u64,
        col: u64,
        row: u64,
    }

    static mut SCREEN: Option<Screen> = None;
    static ON: AtomicBool = AtomicBool::new(false);

    /// The kernel half of the work: map the framebuffer, clear it, show the
    /// log so far and keep showing new lines.
    pub fn init() {
        let Some(fb) = crate::pvh::boot_framebuffer() else { return };
        if fb.kind != 1 || fb.bpp != 32 || fb.width < 320 || fb.height < 200 || fb.pitch < fb.width * 4 {
            return;
        }
        let size = fb.pitch as u64 * fb.height as u64;
        let Ok(base) = crate::arch::paging::map_device(fb.addr, size) else {
            crate::kprintln!("Screen text: the framebuffer at {:#x} cannot be mapped.", fb.addr);
            return;
        };
        // Safe: one core, before the display service and any other thread.
        unsafe {
            *core::ptr::addr_of_mut!(SCREEN) = Some(Screen {
                base,
                width: fb.width as u64,
                height: fb.height as u64,
                pitch: fb.pitch as u64,
                col: 0,
                row: 0,
            })
        };
        clear();
        replay(0);
        ON.store(true, Ordering::Relaxed);
        crate::kprintln!(
            "Screen text: the log is drawn on the boot screen {}x{} (framebuffer at {:#x}).",
            fb.width,
            fb.height,
            fb.addr
        );
    }

    fn screen() -> Option<&'static mut Screen> {
        // Safe: only the one core writes the log, and only when it has the console.
        unsafe { (*core::ptr::addr_of_mut!(SCREEN)).as_mut() }
    }

    fn clear() {
        let Some(s) = screen() else { return };
        for y in 0..s.height {
            for x in 0..s.width {
                put_pixel(s, x, y, BLACK);
            }
        }
        s.col = 0;
        s.row = 0;
    }

    fn put_pixel(s: &Screen, x: u64, y: u64, color: u32) {
        let p = (s.base + y * s.pitch + x * 4) as *mut u32;
        unsafe { p.write_volatile(color) };
    }

    /// The last `bytes` of the log (all of it for 0), drawn.
    fn replay(bytes: usize) {
        let mut chunk = [0u8; 256];
        let mut offset = 0;
        // Find where the log ends, then start `bytes` before that.
        if bytes > 0 {
            let mut total = 0;
            loop {
                let n = crate::console::read_log(total, &mut chunk);
                if n == 0 {
                    break;
                }
                total += n;
            }
            offset = total.saturating_sub(bytes);
        }
        loop {
            let n = crate::console::read_log(offset, &mut chunk);
            if n == 0 {
                break;
            }
            for &b in &chunk[..n] {
                draw(b);
            }
            offset += n;
        }
    }

    fn draw(b: u8) {
        let Some(s) = screen() else { return };
        match b {
            b'\r' => {}
            b'\n' => newline(s),
            0x20..=0x7E => {
                if (s.col + 1) * CELL_W > s.width {
                    newline(s);
                }
                glyph(s, b as char);
                s.col += 1;
            }
            _ => {}
        }
    }

    fn newline(s: &mut Screen) {
        s.col = 0;
        s.row += 1;
        if (s.row + 1) * CELL_H > s.height {
            clear_in(s);
        }
    }

    fn clear_in(s: &mut Screen) {
        for y in 0..s.height {
            for x in 0..s.width {
                put_pixel(s, x, y, BLACK);
            }
        }
        s.col = 0;
        s.row = 0;
    }

    fn glyph(s: &Screen, ch: char) {
        let Some(i) = font::CHARS.chars().position(|c| c == ch) else { return };
        let (x0, y0) = (s.col * CELL_W, s.row * CELL_H);
        for (dy, bits) in font::GLYPHS[i].iter().enumerate() {
            for dx in 0..8 {
                let on = bits & (0x80 >> dx) != 0;
                put_pixel(s, x0 + dx as u64, y0 + dy as u64, if on { WHITE } else { BLACK });
            }
        }
    }

    /// One byte of kernel text.
    pub fn put(b: u8) {
        if ON.load(Ordering::Relaxed) {
            draw(b);
        }
    }

    /// The display service takes the screen: stop drawing text.
    pub fn stop() {
        ON.store(false, Ordering::Relaxed);
    }

    /// A panic or exception: take the screen back, clear it and show the end of the log.
    pub fn resume() {
        if screen().is_none() {
            return;
        }
        ON.store(false, Ordering::Relaxed);
        clear();
        replay(3000);
        ON.store(true, Ordering::Relaxed);
    }
}

#[cfg(target_arch = "x86_64")]
pub use real::{init, put, resume, stop};

#[cfg(not(target_arch = "x86_64"))]
pub fn init() {}
#[cfg(not(target_arch = "x86_64"))]
pub fn put(_b: u8) {}
#[cfg(not(target_arch = "x86_64"))]
pub fn resume() {}
#[cfg(not(target_arch = "x86_64"))]
pub fn stop() {}
