//! The PVH boot protocol's start info (Xen `hvm_start_info`, version 1).

pub const MAGIC: u32 = 0x336e_c578;
pub const E820_RAM: u32 = 1;

#[repr(C)]
pub struct StartInfo {
    magic: u32,
    version: u32,
    flags: u32,
    nr_modules: u32,
    modlist_paddr: u64,
    cmdline_paddr: u64,
    rsdp_paddr: u64,
    memmap_paddr: u64,
    memmap_entries: u32,
    reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemEntry {
    pub addr: u64,
    pub size: u64,
    pub kind: u32,
    reserved: u32,
}

impl MemEntry {
    pub fn kind_name(&self) -> &'static str {
        match self.kind {
            1 => "usable",
            2 => "reserved",
            3 => "ACPI reclaimable",
            4 => "ACPI NVS",
            5 => "bad",
            _ => "other",
        }
    }
}

/// One module the loader put in memory (QEMU: the -initrd file).
#[repr(C)]
struct ModEntry {
    paddr: u64,
    size: u64,
    cmdline_paddr: u64,
    reserved: u64,
}

pub struct BootInfo(&'static StartInfo);

impl BootInfo {
    /// # Safety
    /// `addr` must be what the PVH loader passed in %ebx, inside identity-mapped memory.
    pub unsafe fn from_addr(addr: usize) -> Option<Self> {
        if addr == 0 {
            return None;
        }
        let si = &*(addr as *const StartInfo);
        (si.magic == MAGIC).then_some(BootInfo(si))
    }

    pub fn memory_map(&self) -> &'static [MemEntry] {
        if self.0.version < 1 || self.0.memmap_paddr == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.0.memmap_paddr as *const MemEntry, self.0.memmap_entries as usize) }
    }

    /// The first module (the boot image), as a physical range.
    pub fn module(&self) -> Option<(u64, u64)> {
        if self.0.nr_modules == 0 || self.0.modlist_paddr == 0 {
            return None;
        }
        let m = unsafe { &*(self.0.modlist_paddr as *const ModEntry) };
        (m.paddr != 0 && m.size != 0).then_some((m.paddr, m.paddr + m.size))
    }

    /// Memory the loader's data occupies: start info, memory map, command
    /// line, module list and the first module. The frame allocator must not
    /// hand these out while they are in use. Missing ones are (0, 0).
    pub fn occupied(&self) -> [(u64, u64); 5] {
        let si = self.0 as *const StartInfo as u64;
        let map = self.0.memmap_paddr;
        let map_len = self.0.memmap_entries as u64 * core::mem::size_of::<MemEntry>() as u64;
        let cmd = self.0.cmdline_paddr;
        let mods = self.0.modlist_paddr;
        let mods_len = self.0.nr_modules as u64 * core::mem::size_of::<ModEntry>() as u64;
        [
            (si, si + core::mem::size_of::<StartInfo>() as u64),
            (map, map + map_len),
            (cmd, cmd + self.cmdline().len() as u64 + 1),
            (mods, mods + mods_len),
            self.module().unwrap_or((0, 0)),
        ]
    }

    pub fn cmdline(&self) -> &'static str {
        let p = self.0.cmdline_paddr as *const u8;
        if p.is_null() {
            return "";
        }
        unsafe {
            let mut len = 0;
            while *p.add(len) != 0 && len < 4096 {
                len += 1;
            }
            core::str::from_utf8(core::slice::from_raw_parts(p, len)).unwrap_or("")
        }
    }
}
