//! The same read-only file system as the boot image (a ustar archive), read
//! from the disk sector by sector instead of from memory. Only the headers
//! are read to list the disk; a file's bytes are read when it is opened, so
//! the disk can be much larger than memory.

use crate::bootfs::{c_field, checksum_ok, octal};
use crate::virtio_blk::{self, SECTOR};
use alloc::string::String;
use alloc::vec::Vec;

pub struct Entry {
    pub name: String,
    /// First sector of the file's bytes.
    pub sector: u64,
    pub size: u64,
}

/// Reads the archive's headers: every regular file with where it lies.
pub fn list() -> Result<Vec<Entry>, &'static str> {
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
            out.push(Entry { name, sector: body, size });
        }
        at = body + body_sectors;
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
