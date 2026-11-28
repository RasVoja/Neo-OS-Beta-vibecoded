//! The disk's file system: a ustar archive, the same format as the boot
//! image, read from the disk sector by sector instead of from memory, and
//! now also written.
//!
//! New files are appended before the end of the archive; a file written
//! again is appended again and the latest copy wins; a deleted file is
//! an empty entry with mode 0, which hides the earlier copies. Nothing is
//! ever rewritten in place, so the archive stays one that `tar` reads (it
//! lists every copy, but extracts them in order, so the last one wins
//! there too), and a crash can lose at most the file being written: its
//! data goes first, the end marker next, and the header that makes it
//! visible last.
//!
//! Only the headers are read to list the disk; a file's bytes are read
//! when it is opened, so the disk can be much larger than memory.

use crate::bootfs::{c_field, checksum_ok, octal};
use crate::sync::SpinLock;
use crate::virtio_blk::{self, SECTOR};
use alloc::string::String;
use alloc::vec::Vec;

pub struct Entry {
    pub name: String,
    /// First sector of the file's bytes.
    pub sector: u64,
    pub size: u64,
}

/// Most bytes in one file written to the disk.
pub const MAX_FILE: usize = 4 * 1024 * 1024;
/// Longest file name (the ustar name field).
pub const MAX_NAME: usize = 100;

/// One header found on the disk.
struct Raw {
    name: String,
    sector: u64,
    size: u64,
    /// Mode 0 and no bytes: the file of this name was deleted.
    deleted: bool,
}

/// Reads all headers; also returns the sector where the next one goes.
fn scan() -> Result<(Vec<Raw>, u64), &'static str> {
    let total = virtio_blk::sectors().ok_or("no disk")?;
    let mut out = Vec::new();
    let mut h = [0u8; SECTOR];
    let mut at = 0u64;
    while at < total {
        virtio_blk::read(at, &mut h)?;
        if h.iter().all(|&b| b == 0) {
            break; // end of archive
        }
        if !checksum_ok(&h) {
            return Err("bad header checksum");
        }
        let size = octal(&h[124..136]).ok_or("bad size field")?;
        let body = at + 1;
        let body_sectors = size.div_ceil(SECTOR as u64);
        if body + body_sectors > total {
            return Err("file runs past the end of the disk");
        }
        if h[156] == b'0' || h[156] == 0 {
            let prefix = c_field(&h[345..500]);
            let base = c_field(&h[0..100]);
            let mut name = String::new();
            for part in [prefix, if prefix.is_empty() { &[][..] } else { b"/" }, base] {
                name.push_str(core::str::from_utf8(part).map_err(|_| "file name is not UTF-8")?);
            }
            let name = name.strip_prefix("./").map(String::from).unwrap_or(name);
            let deleted = size == 0 && octal(&h[100..108]) == Some(0);
            out.push(Raw { name, sector: body, size, deleted });
        }
        at = body + body_sectors;
    }
    Ok((out, at))
}

/// The files on the disk: the latest copy of each, without deleted ones,
/// in the order their latest copies were written.
pub fn list() -> Result<Vec<Entry>, &'static str> {
    let (raw, _) = scan()?;
    let mut out: Vec<Entry> = Vec::new();
    for (i, r) in raw.iter().enumerate() {
        // Only the last header of a name counts.
        if raw[i + 1..].iter().any(|later| later.name == r.name) || r.deleted {
            continue;
        }
        out.push(Entry { name: r.name.clone(), sector: r.sector, size: r.size });
    }
    Ok(out)
}

/// Reads a whole file from the disk.
pub fn read_file(name: &str) -> Result<Vec<u8>, &'static str> {
    let entry = list()?.into_iter().find(|e| e.name == name).ok_or("no such file")?;
    let mut data = alloc::vec![0u8; (entry.size as usize).next_multiple_of(SECTOR)];
    for (i, chunk) in data.chunks_mut(8 * SECTOR).enumerate() {
        virtio_blk::read(entry.sector + 8 * i as u64, chunk)?;
    }
    data.truncate(entry.size as usize);
    Ok(data)
}

/// Bytes of the disk that are still free for new files.
pub fn free_bytes() -> Result<u64, &'static str> {
    let total = virtio_blk::sectors().ok_or("no disk")?;
    let (_, end) = scan()?;
    // A header and the end marker are needed besides the data.
    Ok(total.saturating_sub(end + 3) * SECTOR as u64)
}

/// A ustar header for a regular file.
fn header(name: &str, size: u64, mode: u64) -> [u8; SECTOR] {
    let mut h = [0u8; SECTOR];
    h[..name.len()].copy_from_slice(name.as_bytes());
    let put = |h: &mut [u8; SECTOR], at: usize, len: usize, v: u64| {
        // Octal digits, zero padded, ending in a NUL.
        let mut v = v;
        for i in (0..len - 1).rev() {
            h[at + i] = b'0' + (v & 7) as u8;
            v >>= 3;
        }
    };
    put(&mut h, 100, 8, mode);
    put(&mut h, 108, 8, 0); // uid
    put(&mut h, 116, 8, 0); // gid
    put(&mut h, 124, 12, size);
    put(&mut h, 136, 12, crate::clock::unix_seconds()); // modification time, from the machine's clock
    h[156] = b'0';
    h[257..263].copy_from_slice(b"ustar\0");
    h[263..265].copy_from_slice(b"00");
    // The checksum counts its own field as spaces; it is written as six
    // octal digits, a NUL and a space.
    let sum: u64 = h.iter().enumerate().map(|(i, &b)| if (148..156).contains(&i) { 32 } else { b as u64 }).sum();
    put(&mut h, 148, 7, sum);
    h[155] = b' ';
    h
}

/// Only one writer at a time (reading the headers and appending must not
/// interleave).
static WRITER: SpinLock<()> = SpinLock::new(());

fn check_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() || name.len() > MAX_NAME {
        return Err("bad file name length");
    }
    if name.starts_with('/') || name.ends_with('/') || name.contains("//") || name.contains('\0') {
        return Err("bad file name");
    }
    if name.split('/').any(|p| p == "." || p == "..") {
        return Err("bad file name");
    }
    Ok(())
}

/// Appends `data` to the disk as `name`; an earlier file of that name is
/// hidden by it. The programs in `bin/` are kept: they cannot be replaced.
pub fn write_file(name: &str, data: &[u8]) -> Result<(), &'static str> {
    check_name(name)?;
    if name.starts_with("bin/") {
        return Err("bin/ is kept as it is");
    }
    if data.len() > MAX_FILE {
        return Err("file too large");
    }
    append(name, data, 0o644)
}

/// Deletes a file: appends the mark that hides it.
pub fn delete_file(name: &str) -> Result<(), &'static str> {
    check_name(name)?;
    if name.starts_with("bin/") {
        return Err("bin/ is kept as it is");
    }
    if !list()?.iter().any(|e| e.name == name) {
        return Err("no such file");
    }
    append(name, &[], 0)
}

fn append(name: &str, data: &[u8], mode: u64) -> Result<(), &'static str> {
    if !virtio_blk::writable() {
        return Err("the disk is read-only");
    }
    let _only_one = WRITER.lock();
    let total = virtio_blk::sectors().ok_or("no disk")?;
    let (_, end) = scan()?;
    let body = data.len().div_ceil(SECTOR) as u64;
    // The header, the data, and a sector of zeros to end the archive.
    if end + 1 + body + 1 > total {
        return Err("the disk is full");
    }
    // 1. The data. 2. The new end of the archive (a zero sector, and a
    // second one when there is room, as tar likes). 3. The header, which
    // makes the file exist.
    let mut at = end + 1;
    for chunk in data.chunks(8 * SECTOR) {
        let mut padded = [0u8; 8 * SECTOR];
        padded[..chunk.len()].copy_from_slice(chunk);
        virtio_blk::write(at, &padded[..chunk.len().next_multiple_of(SECTOR)])?;
        at += chunk.len().div_ceil(SECTOR) as u64;
    }
    let zeros = [0u8; 2 * SECTOR];
    let marker = if at + 2 <= total { 2 } else { 1 };
    virtio_blk::write(at, &zeros[..marker * SECTOR])?;
    virtio_blk::write(end, &header(name, data.len() as u64, mode))?;
    Ok(())
}
