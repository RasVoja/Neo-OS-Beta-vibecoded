//! The boot file system: a read-only image that the boot loader puts in
//! memory next to the kernel (the "initrd"), so that programs no longer
//! have to be built into the kernel.
//!
//! The format is a plain POSIX tar archive (ustar), which every system can
//! create. Each entry is a 512-byte header followed by the file's bytes,
//! rounded up to 512; two empty headers end the archive. Only regular files
//! are listed; directories and anything else are skipped. Every header's
//! checksum is verified and every size is checked against the image, so a
//! damaged image is refused instead of being read past its end.

use crate::sync::SpinLock;

const BLOCK: usize = 512;

static IMAGE: SpinLock<Option<&'static [u8]>> = SpinLock::new(None);

/// Takes the boot image the loader handed over; returns it, or why it was
/// refused. The memory must stay reserved for as long as the kernel runs.
///
/// # Safety
/// `[start, end)` must be readable memory that nothing else will change.
pub unsafe fn init(start: u64, end: u64) -> Result<BootFs<'static>, &'static str> {
    let fs = BootFs::from_range(start, end)?;
    *IMAGE.lock() = Some(fs.data);
    Ok(fs)
}

/// The boot image, if one was given.
pub fn image() -> Option<BootFs<'static>> {
    IMAGE.lock().map(|data| BootFs { data })
}

pub struct BootFs<'a> {
    data: &'a [u8],
}

pub struct File<'a> {
    /// Full path inside the image, without a leading "./".
    name: [u8; 256],
    name_len: usize,
    /// The last part of the path, as stored in the image.
    base: &'a [u8],
    pub data: &'a [u8],
}

impl<'a> File<'a> {
    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("?")
    }

    /// The file's own name without its directory ("hello-rust").
    pub fn base_name(&self) -> &'a str {
        let start = self.base.iter().rposition(|&c| c == b'/').map_or(0, |i| i + 1);
        core::str::from_utf8(&self.base[start..]).unwrap_or("?")
    }
}

/// Reads an octal number field, as tar stores sizes.
pub(crate) fn octal(field: &[u8]) -> Option<u64> {
    let mut v: u64 = 0;
    let mut seen = false;
    for &c in field {
        match c {
            b'0'..=b'7' => {
                v = v.checked_mul(8)?.checked_add((c - b'0') as u64)?;
                seen = true;
            }
            b' ' | 0 if seen => break,
            b' ' | 0 => {}
            _ => return None,
        }
    }
    seen.then_some(v)
}

pub(crate) fn c_field(field: &[u8]) -> &[u8] {
    let len = field.iter().position(|&c| c == 0).unwrap_or(field.len());
    &field[..len]
}

/// The header checksum: the sum of all bytes with the checksum field
/// itself counted as spaces.
pub(crate) fn checksum_ok(h: &[u8]) -> bool {
    let Some(want) = octal(&h[148..156]) else { return false };
    let sum: u64 = h.iter().enumerate().map(|(i, &b)| if (148..156).contains(&i) { 32 } else { b as u64 }).sum();
    sum == want
}

impl<'a> BootFs<'a> {
    /// Checks that `data` is a well-formed archive.
    pub fn new(data: &'a [u8]) -> Result<BootFs<'a>, &'static str> {
        let fs = BootFs { data };
        let mut it = fs.entries();
        for e in &mut it {
            e?;
        }
        Ok(fs)
    }

    /// # Safety
    /// `[start, end)` must be readable memory that stays untouched.
    unsafe fn from_range(start: u64, end: u64) -> Result<BootFs<'static>, &'static str> {
        if start == 0 || end <= start {
            return Err("empty");
        }
        BootFs::new(core::slice::from_raw_parts(start as *const u8, (end - start) as usize))
    }

    fn entries(&self) -> Entries<'a> {
        Entries { data: self.data, at: 0 }
    }

    /// The regular files in the image.
    pub fn files(&self) -> impl Iterator<Item = File<'a>> + '_ {
        self.entries().filter_map(|e| e.ok())
    }

    pub fn find(&self, name: &str) -> Option<&'a [u8]> {
        self.files().find(|f| f.name() == name).map(|f| f.data)
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

struct Entries<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Iterator for Entries<'a> {
    type Item = Result<File<'a>, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let h = self.data.get(self.at..self.at + BLOCK)?;
            if h.iter().all(|&b| b == 0) {
                return None; // end of archive
            }
            if !checksum_ok(h) {
                self.at = self.data.len();
                return Some(Err("bad header checksum"));
            }
            let Some(size) = octal(&h[124..136]) else {
                self.at = self.data.len();
                return Some(Err("bad size field"));
            };
            let body = self.at + BLOCK;
            let Some(end) = body.checked_add(size as usize).filter(|&e| e <= self.data.len()) else {
                self.at = self.data.len();
                return Some(Err("file runs past the end of the image"));
            };
            self.at = body + (size as usize).next_multiple_of(BLOCK);
            let kind = h[156];
            if kind != b'0' && kind != 0 {
                continue; // directories, links and the like
            }

            let mut name = [0u8; 256];
            let mut n = 0;
            let prefix = c_field(&h[345..500]);
            let base = c_field(&h[0..100]);
            for part in [prefix, if prefix.is_empty() { &[][..] } else { b"/" }, base] {
                name[n..n + part.len()].copy_from_slice(part);
                n += part.len();
            }
            let (skip, name_len) = if name.starts_with(b"./") { (2, n - 2) } else { (0, n) };
            name.copy_within(skip..n, 0);
            let base = c_field(&h[0..100]);
            return Some(Ok(File { name, name_len, base, data: &self.data[body..end] }));
        }
    }
}
