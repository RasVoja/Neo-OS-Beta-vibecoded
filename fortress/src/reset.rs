//! Reset keys, on a USB keyboard (all of them are kept, for every habit):
//!
//! - Ctrl+Alt+Del: soft reset (DOS and Windows),
//! - Ctrl + Neo + Esc (the key with the Neo/Windows logo, either side): soft
//!   reset (the Amiga's Ctrl+Amiga+Amiga; many keyboards have one logo key),
//! - left Shift + right Shift + Esc: hard reset (RISC OS style): the chipset
//!   restarts the whole machine, devices included.

use core::sync::atomic::{AtomicU32, Ordering};

const L_CTRL: u16 = 29;
const R_CTRL: u16 = 97;
const L_ALT: u16 = 56;
const R_ALT: u16 = 100;
const L_SHIFT: u16 = 42;
const R_SHIFT: u16 = 54;
const L_NEO: u16 = 125;
const R_NEO: u16 = 126;
const DELETE: u16 = 111;
const ESC: u16 = 1;

static HELD: AtomicU32 = AtomicU32::new(0);

fn bit(code: u16) -> u32 {
    match code {
        L_CTRL => 1,
        R_CTRL => 2,
        L_ALT => 4,
        R_ALT => 8,
        L_SHIFT => 16,
        R_SHIFT => 32,
        L_NEO => 64,
        R_NEO => 128,
        _ => 0,
    }
}

/// What the keys held and the one just pressed ask for: `Some(hard)`.
pub fn wanted(held: u32, code: u16) -> Option<bool> {
    let ctrl = held & 3 != 0;
    let alt = held & 12 != 0;
    let neo = held & 192 != 0;
    let both_shift = held & 16 != 0 && held & 32 != 0;
    if ctrl && alt && code == DELETE {
        Some(false)
    } else if ctrl && neo && code == ESC {
        Some(false)
    } else if both_shift && code == ESC {
        Some(true)
    } else {
        None
    }
}

/// A key went down or up (key codes as in neo-keys). Restarts the PC when
/// a reset combination is complete.
pub fn key(code: u16, down: bool) {
    let b = bit(code);
    if b != 0 {
        if down {
            HELD.fetch_or(b, Ordering::Relaxed);
        } else {
            HELD.fetch_and(!b, Ordering::Relaxed);
        }
    }
    if !down {
        return;
    }
    // The modifier just pressed counts as held already.
    if let Some(hard) = wanted(HELD.load(Ordering::Relaxed), code) {
        crate::kprintln!("{} reset (keyboard).", if hard { "Hard" } else { "Soft" });
        crate::arch::reset(hard);
    }
}
