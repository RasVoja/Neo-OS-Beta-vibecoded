//! A small reader for the flattened device tree (DTB) that firmware and
//! QEMU hand to the kernel on ARM: just enough to find RAM, the reserved
//! areas, the boot image (initrd) and the kernel command line. No heap is needed.

const MAGIC: u32 = 0xd00d_feed;
const BEGIN_NODE: u32 = 1;
const END_NODE: u32 = 2;
const PROP: u32 = 3;
const NOP: u32 = 4;
const END: u32 = 9;

pub const MAX_REGIONS: usize = 8;

#[derive(Clone, Copy, Default)]
pub struct Region {
    pub addr: u64,
    pub size: u64,
}

pub struct DeviceTree {
    base: *const u8,
    size: usize,
}

/// What the kernel learns from the device tree.
pub struct BootInfo {
    pub memory: [Region; MAX_REGIONS],
    pub memory_count: usize,
    pub reserved: [Region; MAX_REGIONS],
    pub reserved_count: usize,
    pub cmdline: &'static str,
    /// The boot image (initrd) the loader put in memory, if any.
    pub initrd: Option<(u64, u64)>,
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn be64(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_be_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

/// Reads a number of `cells` 32-bit cells.
fn cells(b: &[u8], at: usize, cells: u32) -> Option<u64> {
    match cells {
        1 => be32(b, at).map(u64::from),
        2 => be64(b, at),
        _ => None,
    }
}

fn c_str(b: &[u8], at: usize) -> Option<&[u8]> {
    let rest = b.get(at..)?;
    let len = rest.iter().position(|&c| c == 0)?;
    Some(&rest[..len])
}

impl DeviceTree {
    /// # Safety
    /// `addr` must be readable memory if it holds a device tree.
    pub unsafe fn from_addr(addr: u64) -> Option<DeviceTree> {
        if addr == 0 || addr % 8 != 0 {
            return None;
        }
        let head = core::slice::from_raw_parts(addr as *const u8, 40);
        if be32(head, 0)? != MAGIC {
            return None;
        }
        let size = be32(head, 4)? as usize;
        if !(40..=(64 << 20)).contains(&size) {
            return None;
        }
        Some(DeviceTree { base: addr as *const u8, size })
    }

    pub fn range(&self) -> (u64, u64) {
        (self.base as u64, self.base as u64 + self.size as u64)
    }

    fn bytes(&self) -> &'static [u8] {
        unsafe { core::slice::from_raw_parts(self.base, self.size) }
    }

    /// Walks the tree once and collects what the kernel needs.
    pub fn boot_info(&self) -> Option<BootInfo> {
        let b = self.bytes();
        let structs = be32(b, 8)? as usize;
        let strings = be32(b, 12)? as usize;
        let rsvmap = be32(b, 16)? as usize;
        let mut info = BootInfo {
            memory: [Region::default(); MAX_REGIONS],
            memory_count: 0,
            reserved: [Region::default(); MAX_REGIONS],
            reserved_count: 0,
            cmdline: "",
            initrd: None,
        };
        let (mut initrd_start, mut initrd_end) = (None, None);

        // The memory reservation block: (address, size) pairs up to (0, 0).
        let mut at = rsvmap;
        loop {
            let (addr, size) = (be64(b, at)?, be64(b, at + 8)?);
            if size == 0 {
                break;
            }
            if info.reserved_count < MAX_REGIONS {
                info.reserved[info.reserved_count] = Region { addr, size };
                info.reserved_count += 1;
            }
            at += 16;
        }

        // The structure block. Only the root's cell sizes matter here,
        // since memory and chosen are children of the root.
        let (mut addr_cells, mut size_cells) = (2u32, 1u32);
        let mut depth = 0usize;
        let mut in_memory = false;
        let mut in_chosen = false;
        let mut at = structs;
        loop {
            let token = be32(b, at)?;
            at += 4;
            match token {
                BEGIN_NODE => {
                    let name = c_str(b, at)?;
                    at = (at + name.len() + 1).next_multiple_of(4);
                    depth += 1;
                    if depth == 2 {
                        in_memory = name == b"memory" || name.starts_with(b"memory@");
                        in_chosen = name == b"chosen";
                    }
                }
                END_NODE => {
                    if depth == 2 {
                        in_memory = false;
                        in_chosen = false;
                    }
                    depth = depth.checked_sub(1)?;
                }
                PROP => {
                    let len = be32(b, at)? as usize;
                    let name = c_str(b, strings + be32(b, at + 4)? as usize)?;
                    let value = b.get(at + 8..at + 8 + len)?;
                    at = (at + 8 + len).next_multiple_of(4);
                    if depth == 1 && name == b"#address-cells" {
                        addr_cells = be32(value, 0)?;
                    } else if depth == 1 && name == b"#size-cells" {
                        size_cells = be32(value, 0)?;
                    } else if depth == 2 && in_memory && name == b"reg" {
                        let entry = 4 * (addr_cells + size_cells) as usize;
                        for e in value.chunks_exact(entry) {
                            if info.memory_count < MAX_REGIONS {
                                let addr = cells(e, 0, addr_cells)?;
                                let size = cells(e, 4 * addr_cells as usize, size_cells)?;
                                info.memory[info.memory_count] = Region { addr, size };
                                info.memory_count += 1;
                            }
                        }
                    } else if depth == 2 && in_chosen && name == b"linux,initrd-start" {
                        initrd_start = cells(value, 0, (len / 4) as u32);
                    } else if depth == 2 && in_chosen && name == b"linux,initrd-end" {
                        initrd_end = cells(value, 0, (len / 4) as u32);
                    } else if depth == 2 && in_chosen && name == b"bootargs" {
                        let text = c_str(value, 0).unwrap_or(value);
                        info.cmdline = core::str::from_utf8(text).unwrap_or("");
                    }
                }
                NOP => {}
                END => break,
                _ => return None,
            }
        }
        if let (Some(s), Some(e)) = (initrd_start, initrd_end) {
            info.initrd = (e > s).then_some((s, e));
        }
        Some(info)
    }
}

impl BootInfo {
    pub fn memory(&self) -> &[Region] {
        &self.memory[..self.memory_count]
    }

    pub fn reserved(&self) -> &[Region] {
        &self.reserved[..self.reserved_count]
    }
}
