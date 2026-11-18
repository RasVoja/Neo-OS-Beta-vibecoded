//! hello-window: the first program with a window of its own.
//!
//! It opens a window through the display service (neo-gfx `Window`): a
//! teal page with a greeting, a white disc and a progress bar that fills in
//! ten steps. Then it answers events:
//!
//! - a left click turns the disc amber, or white again;
//! - typed keys appear on a line under the disc (Backspace removes one);
//! - Esc or a right click closes the window.
//!
//! It also ends when the display service stops, or after 30 seconds with
//! nothing happening. It returns ten times the colour changes plus the
//! length of the typed line.

#![no_std]
#![no_main]

use neo_gfx::{Event, Window};
use neo_rt::Error;

const W: i32 = 200;
const H: i32 = 120;
const TEAL: u32 = 0x0D6F78;
const WHITE: u32 = 0xFFFFFF;
const AMBER: u32 = 0xC86F00;
const STEPS: i32 = 10;
/// Ticks with nothing happening before it ends by itself (30 s).
const IDLE: u64 = 3000;
/// The typed line: where it is and how many characters fit.
const LINE_Y: i32 = 86;
const LINE_MAX: usize = 23;

fn run() -> Result<i64, Error> {
    let win = Window::open("Zdravo!", W, H)?;
    let c = &win.canvas;
    c.rect(0, 0, W, H, TEAL);
    c.text(8, 4, "Zdravo, Neo!", 1, WHITE);
    let disc = |color: u32| c.disc(W / 2, H / 2, 24, color);
    disc(WHITE);
    win.drawn()?;
    // The progress bar along the bottom, in ten steps.
    for step in 1..=STEPS {
        c.rect(8, H - 12, (W - 16) * step / STEPS, 8, AMBER);
        win.drawn()?;
        neo_rt::sleep(5);
    }
    let (mut changes, mut idle, mut amber) = (0i64, 0, false);
    let mut line = [0u8; LINE_MAX];
    let mut len = 0;
    let show = |line: &[u8]| {
        c.rect(0, LINE_Y, W, 16, TEAL);
        c.text(8, LINE_Y, core::str::from_utf8(line).unwrap_or(""), 1, WHITE);
    };
    while idle < IDLE {
        let Some(event) = win.poll()? else {
            neo_rt::sleep(2);
            idle += 2;
            continue;
        };
        idle = 0;
        match event {
            Event::Quit => return Ok(changes * 10 + len as i64),
            Event::Click { buttons, .. } if buttons & 2 != 0 => break,
            Event::Key(0x1B) => break,
            Event::Click { buttons, .. } if buttons & 1 != 0 => {
                amber = !amber;
                disc(if amber { AMBER } else { WHITE });
                changes += 1;
            }
            Event::Key(0x08) if len > 0 => {
                len -= 1;
                show(&line[..len]);
            }
            Event::Key(k @ 0x20..=0x7E) if len < LINE_MAX => {
                line[len] = k;
                len += 1;
                show(&line[..len]);
            }
            _ => continue,
        }
        win.drawn()?;
    }
    win.close()?;
    Ok(changes * 10 + len as i64)
}

#[no_mangle]
fn neo_main() -> i64 {
    run().unwrap_or(-1)
}
