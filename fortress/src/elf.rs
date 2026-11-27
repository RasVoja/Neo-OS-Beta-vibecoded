//! Reads ELF64 executables for the machine the kernel runs on: just enough
//! to load a program.
//!
//! Only what the loader needs is checked and returned: the entry point and
//! the loadable segments. Everything comes from untrusted bytes, so every
//! offset and size is bounds-checked before use.

/// ELF machine number of this processor.
#[cfg(target_arch = "x86_64")]
const MACHINE: u16 = 62;
#[cfg(target_arch = "aarch64")]
const MACHINE: u16 = 183;

use alloc::vec::Vec;

pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2;
const PT_LOAD: u32 = 1;

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub vaddr: u64,
    pub mem_size: u64,
    /// The bytes to copy: `file_size` bytes from `offset` in the file.
    pub offset: u64,
    pub file_size: u64,
    pub flags: u32,
}

#[derive(Debug)]
pub struct Image {
    pub entry: u64,
    pub segments: Vec<Segment>,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

pub fn parse(b: &[u8]) -> Result<Image, &'static str> {
    if b.get(0..4) != Some(b"\x7FELF".as_slice()) {
        return Err("not an ELF file");
    }
    if b.get(4) != Some(&2) || b.get(5) != Some(&1) {
        return Err("not a 64-bit little-endian ELF file");
    }
    let e_type = u16_at(b, 16).ok_or("truncated header")?;
    let machine = u16_at(b, 18).ok_or("truncated header")?;
    if e_type != 2 {
        return Err("not an executable (ET_EXEC)");
    }
    if machine != MACHINE {
        return Err("not a program for this processor");
    }
    let entry = u64_at(b, 24).ok_or("truncated header")?;
    let phoff = u64_at(b, 32).ok_or("truncated header")? as usize;
    let phentsize = u16_at(b, 54).ok_or("truncated header")? as usize;
    let phnum = u16_at(b, 56).ok_or("truncated header")? as usize;
    if phentsize < 56 || phnum > 64 {
        return Err("bad program header table");
    }

    let mut segments = Vec::new();
    for i in 0..phnum {
        let at = phoff
            .checked_add(i.checked_mul(phentsize).ok_or("bad program header table")?)
            .ok_or("bad program header table")?;
        let p_type = u32_at(b, at).ok_or("truncated program header")?;
        if p_type != PT_LOAD {
            continue;
        }
        let seg = Segment {
            flags: u32_at(b, at + 4).ok_or("truncated program header")?,
            offset: u64_at(b, at + 8).ok_or("truncated program header")?,
            vaddr: u64_at(b, at + 16).ok_or("truncated program header")?,
            file_size: u64_at(b, at + 32).ok_or("truncated program header")?,
            mem_size: u64_at(b, at + 40).ok_or("truncated program header")?,
        };
        if seg.file_size > seg.mem_size {
            return Err("segment larger on disk than in memory");
        }
        let end = seg.offset.checked_add(seg.file_size).ok_or("segment outside the file")?;
        if end > b.len() as u64 {
            return Err("segment outside the file");
        }
        seg.vaddr.checked_add(seg.mem_size).ok_or("segment wraps around")?;
        segments.push(seg);
    }
    if segments.is_empty() {
        return Err("nothing to load");
    }
    Ok(Image { entry, segments })
}
