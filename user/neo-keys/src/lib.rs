//! Key numbers to characters, for every Neo keyboard driver: the PS/2
//! driver in user mode and the virtio keyboard in the kernel. Scan code
//! set 1 (PS/2 with translation) and Linux key codes (virtio-input) agree
//! for these keys, so one table serves both.
//!
//! Two layouts: US English and Serbian Latin (QWERTZ, with č, ć, ž, š and
//! đ on the keys right of the letters). Left Alt + Shift switches between
//! them, as on other systems. In Serbian Latin only the letters move; the
//! digits and symbols stay where the US layout has them for now.

#![no_std]

/// Keys 0-57 without and with Shift, US layout.
const PLAIN: &[u8; 58] = b"\0\x1b1234567890-=\x08\tqwertyuiop[]\n\0asdfghjkl;'`\0\\zxcvbnm,./\0*\0 ";
const SHIFTED: &[u8; 58] = b"\0\x1b!@#$%^&*()_+\x08\tQWERTYUIOP{}\n\0ASDFGHJKL:\"~\0|ZXCVBNM<>?\0*\0 ";
const LSHIFT: u16 = 42;
const RSHIFT: u16 = 54;
const LALT: u16 = 56;
const CAPS: u16 = 58;
// Keys outside the typing block (Linux key codes).
const HOME: u16 = 102;
const UP: u16 = 103;
const LEFT: u16 = 105;
const RIGHT: u16 = 106;
const END: u16 = 107;
const DOWN: u16 = 108;
const DELETE: u16 = 111;

/// PS/2 sends these keys as 0xE0 followed by a scan code that, alone,
/// would be a key on the number pad. Turns that second code (without the
/// release bit) into the key code `Keys` understands.
pub fn ps2_extended(code: u8) -> Option<u16> {
    Some(match code {
        0x47 => HOME,
        0x48 => UP,
        0x4B => LEFT,
        0x4D => RIGHT,
        0x4F => END,
        0x50 => DOWN,
        0x53 => DELETE,
        _ => return None,
    })
}

/// Serbian Latin: the keys that differ from US, as (key, plain, shifted).
const SERBIAN: [(u16, char, char); 7] = [
    (21, 'z', 'Z'), // where US has Y
    (44, 'y', 'Y'), // where US has Z
    (26, 'š', 'Š'), // [
    (27, 'đ', 'Đ'), // ]
    (39, 'č', 'Č'), // ;
    (40, 'ć', 'Ć'), // '
    (43, 'ž', 'Ž'), // \
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Layout {
    #[default]
    Us,
    SerbianLatin,
}

impl Layout {
    /// The number the kernel keeps for it: 0 US, 1 Serbian Latin.
    pub fn number(self) -> u64 {
        self as u64
    }

    pub fn from_number(n: u64) -> Layout {
        if n & 1 == 1 {
            Layout::SerbianLatin
        } else {
            Layout::Us
        }
    }
}

/// Shift, Alt and Caps Lock state, the layout, and the translation itself.
#[derive(Default)]
pub struct Keys {
    shift: bool,
    alt: bool,
    caps: bool,
    pub layout: Layout,
}

impl Keys {
    /// Feeds one key event; returns the character it types, if any.
    pub fn key(&mut self, code: u16, down: bool) -> Option<char> {
        match code {
            LSHIFT | RSHIFT => {
                if down && !self.shift && self.alt {
                    self.switch();
                }
                self.shift = down;
            }
            LALT => {
                if down && !self.alt && self.shift {
                    self.switch();
                }
                self.alt = down;
            }
            CAPS if down => self.caps = !self.caps,
            _ if !down || self.alt => {}
            k if (k as usize) < PLAIN.len() => {
                if self.layout == Layout::SerbianLatin {
                    if let Some(&(_, plain, shifted)) = SERBIAN.iter().find(|&&(key, _, _)| key == k) {
                        let upper = self.shift ^ self.caps;
                        return Some(if upper { shifted } else { plain });
                    }
                }
                let plain = PLAIN[k as usize];
                let upper = self.shift ^ (self.caps && plain.is_ascii_lowercase());
                let c = if upper { SHIFTED[k as usize] } else { plain };
                return (c != 0).then_some(c as char);
            }
            _ => {}
        }
        None
    }

    /// Feeds one key event and writes the bytes it types into `out`: a
    /// letter as UTF-8, and the arrows, Home, End and Delete as the escape
    /// sequences a VT100 terminal sends (`ESC [ A` for up, `ESC [ 3 ~` for
    /// Delete), so programs read them the same way from the serial line.
    /// Returns how many bytes it wrote.
    pub fn key_bytes(&mut self, code: u16, down: bool, out: &mut [u8; 4]) -> usize {
        let seq: &[u8] = match code {
            _ if !down => b"",
            UP => b"\x1b[A",
            DOWN => b"\x1b[B",
            RIGHT => b"\x1b[C",
            LEFT => b"\x1b[D",
            HOME => b"\x1b[H",
            END => b"\x1b[F",
            DELETE => b"\x1b[3~",
            _ => b"",
        };
        if !seq.is_empty() {
            out[..seq.len()].copy_from_slice(seq);
            return seq.len();
        }
        match self.key(code, down) {
            Some(c) => c.encode_utf8(out).len(),
            None => 0,
        }
    }

    fn switch(&mut self) {
        self.layout = match self.layout {
            Layout::Us => Layout::SerbianLatin,
            Layout::SerbianLatin => Layout::Us,
        };
    }
}
